use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, bail};
use proc_macro2::{Delimiter, TokenStream, TokenTree};
use serde::Deserialize;
use syn::{Attribute, Item, Visibility, parse::Parser, visit::Visit};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PublicEnum {
    pub file: String,
    pub item: String,
    pub name: String,
    pub is_non_exhaustive: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DenyUnknownFields {
    pub file: String,
    pub item: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PanicCall {
    pub file: String,
    pub item: String,
    pub kind: String,
}

pub fn normalize_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            Component::CurDir => None,
            other => Some(other.as_os_str().to_string_lossy().into_owned()),
        })
        .collect::<Vec<_>>()
        .join("/")
}

pub fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map_or_else(|_| normalize_path(path), normalize_path)
}

pub fn rust_source_files(repo_root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let crates_root = repo_root.join("crates");
    let mut files = Vec::new();
    for entry in fs::read_dir(&crates_root)
        .with_context(|| format!("failed to read {}", crates_root.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let crate_dir = entry.path();
        if is_unpublished_crate(&crate_dir) {
            continue;
        }
        collect_rs_files(&crate_dir.join("src"), &mut files)?;
    }
    files.sort();
    Ok(files)
}

/// Returns `true` when a crate manifest opts out of publication with
/// `publish = false`. The published-surface policy gates (public enums,
/// wire-type field policy, and the panic-free surface) govern only crates
/// that ship to consumers, so an unpublished dev-only crate such as the
/// shared test-support crate stays outside their scope (see ADR 0062).
fn is_unpublished_crate(crate_dir: &Path) -> bool {
    let manifest = crate_dir.join("Cargo.toml");
    let Ok(contents) = fs::read_to_string(&manifest) else {
        return false;
    };
    contents.lines().any(|line| {
        let normalized: String = line
            .chars()
            .filter(|value| !value.is_whitespace())
            .collect();
        normalized == "publish=false"
    })
}

pub fn parse_rust_file(path: &Path) -> anyhow::Result<syn::File> {
    let source =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    syn::parse_file(&source).with_context(|| format!("failed to parse {}", path.display()))
}

pub fn collect_public_enums(repo_root: &Path) -> anyhow::Result<Vec<PublicEnum>> {
    let mut output = Vec::new();
    for path in rust_source_files(repo_root)? {
        let file = relative_path(repo_root, &path);
        let syntax = parse_rust_file(&path)?;
        let mut visitor = PublicEnumVisitor {
            file,
            modules: Vec::new(),
            output: &mut output,
        };
        visitor.visit_file(&syntax);
    }
    output.sort();
    Ok(output)
}

pub fn collect_deny_unknown_fields(repo_root: &Path) -> anyhow::Result<Vec<DenyUnknownFields>> {
    let mut output = Vec::new();
    for path in rust_source_files(repo_root)? {
        let file = relative_path(repo_root, &path);
        let syntax = parse_rust_file(&path)?;
        let mut visitor = DenyUnknownFieldsVisitor {
            file,
            modules: Vec::new(),
            output: &mut output,
        };
        visitor.visit_file(&syntax);
    }
    output.sort();
    Ok(output)
}

pub fn collect_panic_calls(repo_root: &Path) -> anyhow::Result<Vec<PanicCall>> {
    let mut output = Vec::new();
    for path in rust_source_files(repo_root)? {
        let file = relative_path(repo_root, &path);
        let syntax = parse_rust_file(&path)?;
        let mut visitor = PanicVisitor {
            file,
            modules: Vec::new(),
            items: Vec::new(),
            output: &mut output,
        };
        visitor.visit_file(&syntax);
    }
    output.sort();
    Ok(output)
}

pub fn has_attr(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(name))
}

pub fn has_serde_deny_unknown_fields(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("serde"))
        .any(|attr| format!("{:?}", attr.meta).contains("deny_unknown_fields"))
}

pub fn is_test_attr(attr: &Attribute) -> bool {
    attr.path().is_ident("test")
        || attr.path().is_ident("wasm_bindgen_test")
        || attr
            .path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "test")
        || format!("{:?}", attr.meta).contains("wasm_bindgen_test")
}

pub fn is_test_function(item: &syn::ItemFn) -> bool {
    item.attrs.iter().any(is_test_attr)
}

pub fn test_functions(path: &Path) -> anyhow::Result<Vec<String>> {
    let syntax = parse_rust_file(path)?;
    let mut visitor = TestFunctionVisitor { output: Vec::new() };
    visitor.visit_file(&syntax);
    visitor.output.sort();
    Ok(visitor.output)
}

pub fn read_to_string(path: &Path) -> anyhow::Result<String> {
    fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))
}

pub fn ensure_file_exists(path: &Path) -> anyhow::Result<()> {
    if path.is_file() {
        Ok(())
    } else {
        bail!("expected file does not exist: {}", path.display())
    }
}

#[derive(Deserialize)]
struct RootManifest {
    workspace: WorkspaceTable,
}

#[derive(Deserialize)]
struct WorkspaceTable {
    members: Vec<String>,
}

#[derive(Deserialize)]
struct MemberManifest {
    package: Option<MemberPackage>,
}

#[derive(Deserialize)]
struct MemberPackage {
    name: String,
    publish: Option<PublishFlag>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum PublishFlag {
    Toggle(bool),
    Registries(Vec<String>),
}

/// Sorted package names of every publishable workspace crate, read from the
/// workspace manifest.
///
/// The shipped-surface dependency gates target exactly this set, so deriving it
/// from `Cargo.toml` keeps coverage correct as crates are added or removed
/// rather than tracking a hand-maintained roster. A member is publishable
/// unless its manifest disables publication with `publish = false` or an empty
/// `publish = []` registry list.
pub fn shipped_crates(repo_root: &Path) -> anyhow::Result<Vec<String>> {
    let root: RootManifest = toml::from_str(&read_to_string(&repo_root.join("Cargo.toml"))?)
        .context("failed to parse workspace Cargo.toml")?;
    let mut names = Vec::new();
    for member in &root.workspace.members {
        let manifest = repo_root.join(member).join("Cargo.toml");
        let parsed: MemberManifest = toml::from_str(&read_to_string(&manifest)?)
            .with_context(|| format!("failed to parse {}", manifest.display()))?;
        let Some(package) = parsed.package else {
            continue;
        };
        let publishable = match package.publish {
            None => true,
            Some(PublishFlag::Toggle(flag)) => flag,
            Some(PublishFlag::Registries(registries)) => !registries.is_empty(),
        };
        if publishable {
            names.push(package.name);
        }
    }
    names.sort();
    Ok(names)
}

/// Reads a dotted-path string value from a manifest's TOML text, returning
/// `None` when any segment is missing or the leaf is not a string.
pub fn manifest_string(toml_text: &str, dotted_key: &str) -> Option<String> {
    let value: toml::Value = toml::from_str(toml_text).ok()?;
    let mut current = &value;
    for segment in dotted_key.split('.') {
        current = current.get(segment)?;
    }
    current.as_str().map(ToOwned::to_owned)
}

fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_rs_files(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

const fn is_public(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
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

struct PublicEnumVisitor<'a> {
    file: String,
    modules: Vec<String>,
    output: &'a mut Vec<PublicEnum>,
}

impl<'ast> Visit<'ast> for PublicEnumVisitor<'_> {
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        if is_cfg_test(&item.attrs) {
            return;
        }
        if let Some((_, items)) = &item.content {
            self.modules.push(item.ident.to_string());
            for item in items {
                self.visit_item(item);
            }
            self.modules.pop();
        }
    }

    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        if is_cfg_test(&item.attrs) || !is_public(&item.vis) {
            return;
        }
        let name = item.ident.to_string();
        self.output.push(PublicEnum {
            file: self.file.clone(),
            item: item_path(&self.modules, &name),
            name,
            is_non_exhaustive: has_attr(&item.attrs, "non_exhaustive"),
        });
    }
}

struct DenyUnknownFieldsVisitor<'a> {
    file: String,
    modules: Vec<String>,
    output: &'a mut Vec<DenyUnknownFields>,
}

impl<'ast> Visit<'ast> for DenyUnknownFieldsVisitor<'_> {
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        if is_cfg_test(&item.attrs) {
            return;
        }
        if let Some((_, items)) = &item.content {
            self.modules.push(item.ident.to_string());
            for item in items {
                self.visit_item(item);
            }
            self.modules.pop();
        }
    }

    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        if is_cfg_test(&item.attrs) || !has_serde_deny_unknown_fields(&item.attrs) {
            return;
        }
        self.output.push(DenyUnknownFields {
            file: self.file.clone(),
            item: item_path(&self.modules, &item.ident.to_string()),
        });
    }

    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        if is_cfg_test(&item.attrs) || !has_serde_deny_unknown_fields(&item.attrs) {
            return;
        }
        self.output.push(DenyUnknownFields {
            file: self.file.clone(),
            item: item_path(&self.modules, &item.ident.to_string()),
        });
    }
}

struct PanicVisitor<'a> {
    file: String,
    modules: Vec<String>,
    items: Vec<String>,
    output: &'a mut Vec<PanicCall>,
}

impl PanicVisitor<'_> {
    fn current_item(&self) -> String {
        if self.items.is_empty() {
            "<module>".to_owned()
        } else {
            self.items.join("::")
        }
    }

    fn push_call(&mut self, kind: &str) {
        self.output.push(PanicCall {
            file: self.file.clone(),
            item: self.current_item(),
            kind: kind.to_owned(),
        });
    }
}

impl<'ast> Visit<'ast> for PanicVisitor<'_> {
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        if is_cfg_test(&item.attrs) {
            return;
        }
        if let Some((_, items)) = &item.content {
            self.modules.push(item.ident.to_string());
            for item in items {
                self.visit_item(item);
            }
            self.modules.pop();
        }
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        if is_cfg_test(&item.attrs) {
            return;
        }
        self.items
            .push(item_path(&self.modules, &item.sig.ident.to_string()));
        syn::visit::visit_item_fn(self, item);
        self.items.pop();
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        if is_cfg_test(&item.attrs) {
            return;
        }
        self.items.push(item.sig.ident.to_string());
        syn::visit::visit_impl_item_fn(self, item);
        self.items.pop();
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        if is_cfg_test(&item.attrs) {
            return;
        }
        let type_name = impl_type_name(&item.self_ty);
        self.items.push(item_path(&self.modules, &type_name));
        syn::visit::visit_item_impl(self, item);
        self.items.pop();
    }

    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        if is_cfg_test(&item.attrs) {
            return;
        }
        self.items.push(item.sig.ident.to_string());
        syn::visit::visit_trait_item_fn(self, item);
        self.items.pop();
    }

    fn visit_expr_method_call(&mut self, expr: &'ast syn::ExprMethodCall) {
        let method = expr.method.to_string();
        if matches!(method.as_str(), "unwrap" | "expect") {
            self.push_call(&method);
        }
        syn::visit::visit_expr_method_call(self, expr);
    }

    fn visit_expr_macro(&mut self, expr: &'ast syn::ExprMacro) {
        if let Some(segment) = expr.mac.path.segments.last() {
            let name = segment.ident.to_string();
            if matches!(
                name.as_str(),
                "panic" | "unreachable" | "todo" | "unimplemented"
            ) {
                self.push_call(&format!("{name}!"));
            }
        }
        let parser = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
        if let Ok(arguments) = parser.parse2(expr.mac.tokens.clone()) {
            for argument in &arguments {
                self.visit_expr(argument);
            }
        }
        syn::visit::visit_expr_macro(self, expr);
    }
}

struct TestFunctionVisitor {
    output: Vec<String>,
}

impl<'ast> Visit<'ast> for TestFunctionVisitor {
    fn visit_item(&mut self, item: &'ast Item) {
        match item {
            Item::Fn(function) if is_test_function(function) => {
                self.output.push(function.sig.ident.to_string());
            }
            Item::Macro(item_macro)
                if item_macro
                    .mac
                    .path
                    .segments
                    .last()
                    .is_some_and(|segment| segment.ident == "proptest") =>
            {
                self.output
                    .extend(extract_proptest_test_names(&item_macro.mac.tokens));
            }
            _ => syn::visit::visit_item(self, item),
        }
    }
}

/// Extracts `#[test] fn NAME(...)` function identifiers from a `proptest!`
/// macro body. The macro wraps test functions in a proptest-specific
/// parameter syntax (`arg in strategy`) that `syn` cannot parse as ordinary
/// items, so the function walks the raw token tree looking for the
/// `# [test] fn NAME (` pattern.
fn extract_proptest_test_names(tokens: &TokenStream) -> Vec<String> {
    let trees: Vec<TokenTree> = tokens.clone().into_iter().collect();
    let mut names = Vec::new();
    let mut index = 0usize;
    while index + 3 < trees.len() {
        if let (
            TokenTree::Punct(hash),
            TokenTree::Group(bracket),
            TokenTree::Ident(keyword),
            TokenTree::Ident(name),
        ) = (
            &trees[index],
            &trees[index + 1],
            &trees[index + 2],
            &trees[index + 3],
        ) && hash.as_char() == '#'
            && bracket.delimiter() == Delimiter::Bracket
            && bracket.stream().to_string().trim() == "test"
            && keyword == "fn"
        {
            names.push(name.to_string());
            index += 4;
            continue;
        }
        index += 1;
    }
    names
}
