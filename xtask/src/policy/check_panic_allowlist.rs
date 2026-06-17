use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};
use proc_macro2::{LineColumn, Span};
use serde::Deserialize;
use syn::{Attribute, Expr, ExprLit, Item, Lit, spanned::Spanned};

use crate::policy::{
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

pub fn run_default() -> anyhow::Result<()> {
    run(&Args {
        repo_root: PathBuf::from("."),
        allowlist: None,
    })
}

pub fn run(args: &Args) -> anyhow::Result<()> {
    let allowlist_path = args
        .allowlist
        .clone()
        .unwrap_or_else(|| args.repo_root.join(".github/config/panic-allowlist.yaml"));
    let allowlist: PanicAllowlist = fixtures::load_yaml(&allowlist_path)
        .with_context(|| format!("failed to load {}", allowlist_path.display()))?;
    let calls = workspace::collect_panic_calls(&args.repo_root)?;
    let mut errors = validate_allowlist(&allowlist, &calls);
    errors.extend(validate_allowlist_artifacts(&args.repo_root, &allowlist));

    if errors.is_empty() {
        println!(
            "panic allowlist covers {} panic-bearing call(s)",
            calls.len()
        );
        return Ok(());
    }

    for error in &errors {
        eprintln!("error: {error}");
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
        let key = (
            workspace::normalize_manifest_path(&entry.file),
            entry.item.clone(),
        );
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

        let relative_path = workspace::normalize_manifest_path(&entry.file);
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

    let body = item.body.map_or("", |span| span_source(span, src));
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

struct LocatedItem<'a> {
    attrs: &'a [Attribute],
    body: Option<Span>,
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
                if workspace::item_path(modules, &function.sig.ident.to_string()) == target {
                    return Some(LocatedItem {
                        attrs: &function.attrs,
                        body: Some(function.block.span()),
                    });
                }
            }
            Item::Impl(implementation) => {
                let impl_path = workspace::item_path(
                    modules,
                    &workspace::impl_type_name(&implementation.self_ty),
                );
                for item in &implementation.items {
                    if let syn::ImplItem::Fn(function) = item
                        && format!("{impl_path}::{}", function.sig.ident) == target
                    {
                        return Some(LocatedItem {
                            attrs: &function.attrs,
                            body: Some(function.block.span()),
                        });
                    }
                }
            }
            Item::Mod(module) => {
                if workspace::is_cfg_test(&module.attrs) {
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
                let trait_path = workspace::item_path(modules, &trait_item.ident.to_string());
                for item in &trait_item.items {
                    if let syn::TraitItem::Fn(function) = item
                        && format!("{trait_path}::{}", function.sig.ident) == target
                    {
                        return Some(LocatedItem {
                            attrs: &function.attrs,
                            body: function.default.as_ref().map(Spanned::span),
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

/// Returns the source slice spanned by a syn node (here, a function body
/// block). A `// SAFETY:` comment is not part of the token stream, so it is
/// searched for in the original source between the body's spanned braces.
fn span_source(span: Span, src: &str) -> &str {
    let start = line_col_to_offset(src, span.start());
    let end = line_col_to_offset(src, span.end());
    src.get(start..end).unwrap_or_default()
}

/// Maps a proc-macro2 `LineColumn` (1-based line, 0-based character column) to
/// a byte offset in `src`.
fn line_col_to_offset(src: &str, location: LineColumn) -> usize {
    let mut offset = 0;
    for (index, line) in src.split_inclusive('\n').enumerate() {
        if index + 1 == location.line {
            return offset
                + line
                    .char_indices()
                    .nth(location.column)
                    .map_or(line.len(), |(byte, _)| byte);
        }
        offset += line.len();
    }
    offset
}
