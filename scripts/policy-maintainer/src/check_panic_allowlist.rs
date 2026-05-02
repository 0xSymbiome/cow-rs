use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};
use serde::Deserialize;
use syn::{Attribute, Expr, ExprLit, Item, Lit};

use crate::{
    diagnostics::{Diagnostic, OutputMode},
    fixtures,
    workspace::{self, PanicCall},
};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
    /// Override allowlist path.
    #[arg(long)]
    pub allowlist: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct PanicAllowlist {
    pub version: u32,
    pub allowed: Vec<PanicAllowlistEntry>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PanicAllowlistEntry {
    pub file: String,
    pub item: String,
    pub reason: String,
    #[serde(default)]
    pub documented: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PanicGateError {
    ItemNotFound { file: String, item: String },
    MissingPanicsRustdoc { file: String, item: String },
    MissingSafetyComment { file: String, item: String },
}

impl fmt::Display for PanicGateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ItemNotFound { file, item } => {
                write!(f, "panic allowlist entry {file}::{item} names no Rust item")
            }
            Self::MissingPanicsRustdoc { file, item } => write!(
                f,
                "panic allowlist entry {file}::{item} is missing a # Panics rustdoc section"
            ),
            Self::MissingSafetyComment { file, item } => write!(
                f,
                "panic allowlist entry {file}::{item} is missing a // SAFETY: comment in the item body"
            ),
        }
    }
}

pub fn run(args: Args, output_mode: OutputMode) -> anyhow::Result<()> {
    let mut stdout = io::stdout().lock();
    run_with_writer(args, output_mode, &mut stdout)
}

pub fn run_with_writer(
    args: Args,
    output_mode: OutputMode,
    writer: &mut impl Write,
) -> anyhow::Result<()> {
    let allowlist_path = args
        .allowlist
        .unwrap_or_else(|| args.repo_root.join(".github/config/panic-allowlist.yaml"));
    let allowlist: PanicAllowlist = fixtures::load_yaml(&allowlist_path)
        .with_context(|| format!("failed to load {}", allowlist_path.display()))?;
    let calls = workspace::collect_panic_calls(&args.repo_root)?;
    let mut errors = validate_allowlist(&allowlist, &calls);
    errors.extend(validate_allowlist_artifacts(&args.repo_root, &allowlist));

    if errors.is_empty() {
        Diagnostic::info(
            "PM3000",
            format!(
                "panic allowlist covers {} panic-bearing call(s)",
                calls.len()
            ),
        )
        .emit(output_mode, writer)?;
        return Ok(());
    }

    for error in &errors {
        Diagnostic::error("PM3001", error).emit(output_mode, writer)?;
    }
    bail!("panic allowlist has {} error(s)", errors.len())
}

pub fn validate_allowlist(allowlist: &PanicAllowlist, calls: &[PanicCall]) -> Vec<String> {
    let mut errors = Vec::new();
    if allowlist.version != 1 {
        errors.push(format!(
            "panic-allowlist.yaml version must be 1, got {}",
            allowlist.version
        ));
    }

    let mut allowed = BTreeMap::new();
    for entry in &allowlist.allowed {
        if entry.reason.trim().is_empty() {
            errors.push(format!(
                "{}::{} has an empty panic allowlist rationale",
                entry.file, entry.item
            ));
        }
        let key = (normalize_manifest_path(&entry.file), entry.item.clone());
        if allowed.insert(key.clone(), entry).is_some() {
            errors.push(format!(
                "duplicate panic allowlist entry for {}::{}",
                key.0, key.1
            ));
        }
    }

    let mut matched = BTreeSet::new();
    for call in calls {
        let key = (call.file.clone(), call.item.clone());
        if allowed.contains_key(&key) {
            matched.insert(key);
        } else {
            errors.push(format!(
                "panic call `{}` in {}::{} is not allowlisted",
                call.kind, call.file, call.item
            ));
        }
    }

    for key in allowed.keys() {
        if !matched.contains(key) {
            errors.push(format!(
                "panic allowlist entry {}::{} has no matching panic-bearing call",
                key.0, key.1
            ));
        }
    }

    errors
}

pub fn validate_allowlist_artifacts(repo_root: &Path, allowlist: &PanicAllowlist) -> Vec<String> {
    let mut errors = Vec::new();

    for entry in &allowlist.allowed {
        if !entry.documented.unwrap_or(true) {
            continue;
        }

        let relative_path = normalize_manifest_path(&entry.file);
        let source_path = repo_root.join(&relative_path);
        let source = match fs::read_to_string(&source_path) {
            Ok(source) => source,
            Err(error) => {
                errors.push(format!(
                    "failed to read panic allowlist source {}: {error}",
                    source_path.display()
                ));
                continue;
            }
        };
        let syntax = match syn::parse_file(&source) {
            Ok(syntax) => syntax,
            Err(error) => {
                errors.push(format!(
                    "failed to parse panic allowlist source {}: {error}",
                    source_path.display()
                ));
                continue;
            }
        };

        if let Err(entry_errors) = check_entry_artifacts(entry, &syntax, &source) {
            errors.extend(entry_errors.into_iter().map(|error| error.to_string()));
        }
    }

    errors
}

pub fn check_entry_artifacts(
    entry: &PanicAllowlistEntry,
    file: &syn::File,
    src: &str,
) -> Result<(), Vec<PanicGateError>> {
    if !entry.documented.unwrap_or(true) {
        return Ok(());
    }

    let Some(item) = locate_item(file, &entry.item) else {
        return Err(vec![PanicGateError::ItemNotFound {
            file: entry.file.clone(),
            item: entry.item.clone(),
        }]);
    };

    let mut errors = Vec::new();
    let docs = collect_rustdoc(item.attrs);
    if !has_panics_section(&docs) {
        errors.push(PanicGateError::MissingPanicsRustdoc {
            file: entry.file.clone(),
            item: entry.item.clone(),
        });
    }

    let body = item_body_source_range(&entry.item, src).unwrap_or_default();
    if !body.contains("// SAFETY:") {
        errors.push(PanicGateError::MissingSafetyComment {
            file: entry.file.clone(),
            item: entry.item.clone(),
        });
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn normalize_manifest_path(path: &str) -> String {
    path.replace('\\', "/")
}

struct LocatedItem<'a> {
    attrs: &'a [Attribute],
}

fn locate_item<'a>(file: &'a syn::File, target: &str) -> Option<LocatedItem<'a>> {
    locate_in_items(&file.items, &mut Vec::new(), target)
}

fn locate_in_items<'a>(
    items: &'a [Item],
    modules: &mut Vec<String>,
    target: &str,
) -> Option<LocatedItem<'a>> {
    for item in items {
        match item {
            Item::Fn(function) => {
                if item_path(modules, &function.sig.ident.to_string()) == target {
                    return Some(LocatedItem {
                        attrs: &function.attrs,
                    });
                }
            }
            Item::Impl(implementation) => {
                let impl_path = item_path(modules, &impl_type_name(&implementation.self_ty));
                for item in &implementation.items {
                    if let syn::ImplItem::Fn(function) = item
                        && format!("{impl_path}::{}", function.sig.ident) == target
                    {
                        return Some(LocatedItem {
                            attrs: &function.attrs,
                        });
                    }
                }
            }
            Item::Mod(module) => {
                if is_cfg_test(&module.attrs) {
                    continue;
                }
                if let Some((_, items)) = &module.content {
                    modules.push(module.ident.to_string());
                    let found = locate_in_items(items, modules, target);
                    modules.pop();
                    if found.is_some() {
                        return found;
                    }
                }
            }
            Item::Trait(trait_item) => {
                let trait_path = item_path(modules, &trait_item.ident.to_string());
                for item in &trait_item.items {
                    if let syn::TraitItem::Fn(function) = item
                        && format!("{trait_path}::{}", function.sig.ident) == target
                    {
                        return Some(LocatedItem {
                            attrs: &function.attrs,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    None
}

fn collect_rustdoc(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|attr| {
            if !attr.path().is_ident("doc") {
                return None;
            }
            match &attr.meta {
                syn::Meta::NameValue(meta) => match &meta.value {
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(value),
                        ..
                    }) => Some(value.value()),
                    _ => None,
                },
                _ => None,
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn has_panics_section(docs: &str) -> bool {
    docs.lines()
        .any(|line| line.trim_start().starts_with("# Panics"))
}

fn item_body_source_range<'a>(item: &str, src: &'a str) -> Option<&'a str> {
    let mut segments = item.rsplit("::");
    let name = segments.next()?;
    if let Some(owner) = segments.next()
        && let Some(body) = find_impl_function_body(owner, name, src)
    {
        return Some(body);
    }
    find_function_body(name, src)
}

fn find_impl_function_body<'a>(owner: &str, name: &str, src: &'a str) -> Option<&'a str> {
    let mut offset = 0;
    while offset < src.len() {
        let found = src[offset..].find("impl")?;
        let impl_start = offset + found;
        let before = src[..impl_start].chars().next_back();
        if before.is_some_and(is_ident_char) {
            offset = impl_start + 4;
            continue;
        }
        let after = src[impl_start + 4..].chars().next();
        if after.is_some_and(is_ident_char) {
            offset = impl_start + 4;
            continue;
        }

        let Some(open) = find_body_open_brace(src, impl_start + 4) else {
            offset = impl_start + 4;
            continue;
        };
        let Some(close) = find_matching_brace(src, open) else {
            offset = impl_start + 4;
            continue;
        };
        let header = &src[impl_start..open];
        let body = &src[open..=close];
        if header_mentions_ident(header, owner)
            && let Some(function_body) = find_function_body(name, body)
        {
            return Some(function_body);
        }
        offset = close + 1;
    }

    None
}

fn find_function_body<'a>(name: &str, src: &'a str) -> Option<&'a str> {
    let mut offset = 0;
    while offset < src.len() {
        let found = src[offset..].find("fn")?;
        let fn_start = offset + found;
        let before = src[..fn_start].chars().next_back();
        if before.is_some_and(is_ident_char) {
            offset = fn_start + 2;
            continue;
        }

        let mut cursor = fn_start + 2;
        cursor = skip_ws(src, cursor);
        if src[cursor..].starts_with("r#") {
            cursor += 2;
        }
        let ident_start = cursor;
        while cursor < src.len() && is_ident_char(src.as_bytes()[cursor] as char) {
            cursor += 1;
        }
        if &src[ident_start..cursor] != name {
            offset = fn_start + 2;
            continue;
        }
        let after = src[cursor..].chars().next();
        if after.is_some_and(is_ident_char) {
            offset = fn_start + 2;
            continue;
        }
        let open = find_body_open_brace(src, cursor)?;
        let close = find_matching_brace(src, open)?;
        return Some(&src[open..=close]);
    }

    None
}

fn skip_ws(src: &str, mut cursor: usize) -> usize {
    while cursor < src.len() && src.as_bytes()[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    cursor
}

fn find_body_open_brace(src: &str, mut cursor: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'(' => paren_depth += 1,
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b'[' => bracket_depth += 1,
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b'{' if paren_depth == 0 && bracket_depth == 0 => return Some(cursor),
            b';' if paren_depth == 0 && bracket_depth == 0 => return None,
            _ => {}
        }
        cursor += 1;
    }
    None
}

fn find_matching_brace(src: &str, open: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut cursor = open;
    let mut depth = 0usize;
    let mut state = LexerState::Code;

    while cursor < bytes.len() {
        match state {
            LexerState::Code => match bytes[cursor] {
                b'{' => depth += 1,
                b'}' => {
                    depth = depth.checked_sub(1)?;
                    if depth == 0 {
                        return Some(cursor);
                    }
                }
                b'"' => state = LexerState::String,
                b'\'' => state = LexerState::Char,
                b'/' if bytes.get(cursor + 1) == Some(&b'/') => {
                    state = LexerState::LineComment;
                    cursor += 1;
                }
                b'/' if bytes.get(cursor + 1) == Some(&b'*') => {
                    state = LexerState::BlockComment;
                    cursor += 1;
                }
                _ => {}
            },
            LexerState::String => match bytes[cursor] {
                b'\\' => cursor += 1,
                b'"' => state = LexerState::Code,
                _ => {}
            },
            LexerState::Char => match bytes[cursor] {
                b'\\' => cursor += 1,
                b'\'' => state = LexerState::Code,
                _ => {}
            },
            LexerState::LineComment => {
                if bytes[cursor] == b'\n' {
                    state = LexerState::Code;
                }
            }
            LexerState::BlockComment => {
                if bytes[cursor] == b'*' && bytes.get(cursor + 1) == Some(&b'/') {
                    state = LexerState::Code;
                    cursor += 1;
                }
            }
        }
        cursor += 1;
    }

    None
}

#[derive(Clone, Copy)]
enum LexerState {
    Code,
    String,
    Char,
    LineComment,
    BlockComment,
}

fn is_ident_char(value: char) -> bool {
    value == '_' || value.is_ascii_alphanumeric()
}

fn header_mentions_ident(header: &str, ident: &str) -> bool {
    let mut cursor = 0;
    while cursor < header.len() {
        if header.as_bytes()[cursor].is_ascii_alphabetic() || header.as_bytes()[cursor] == b'_' {
            let ident_start = cursor;
            while cursor < header.len() && is_ident_char(header.as_bytes()[cursor] as char) {
                cursor += 1;
            }
            if &header[ident_start..cursor] == ident {
                return true;
            }
        } else {
            cursor += 1;
        }
    }
    false
}

fn is_cfg_test(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|attr| attr.path().is_ident("cfg") && format!("{:?}", attr.meta).contains("test"))
}

fn item_path(modules: &[String], name: &str) -> String {
    if modules.is_empty() {
        name.to_owned()
    } else {
        format!("{}::{name}", modules.join("::"))
    }
}

fn impl_type_name(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(path) => path
            .path
            .segments
            .last()
            .map_or_else(|| "impl".to_owned(), |segment| segment.ident.to_string()),
        _ => "impl".to_owned(),
    }
}
