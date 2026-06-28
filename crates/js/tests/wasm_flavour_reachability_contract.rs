#![cfg(not(target_arch = "wasm32"))]

//! Flavour reachability contract: every DTO emitted into a flavour's raw bindings
//! must be reachable from a public export in that flavour, or be an allowlisted
//! standalone type. `tsify` emits a `.d.ts` type for every *compiled*
//! `#[derive(Tsify)]` struct, so a DTO whose `#[cfg]` is broader than the union of
//! its consumers' gates leaks into leaner flavours — emitted but unusable.
//!
//! The facade-coverage contract catches *asymmetric* leaks (raw has it, facade does
//! not). It cannot catch a *symmetric* leak — both layers carry an under-gated DTO —
//! which is how `TransactionRequestDto`, `PaginationOptions`, and others rode into the
//! `signing` flavour. This contract closes that gap by checking the raw surface
//! directly: a declared DTO that no export signature reaches (transitively) is a
//! gating leak unless it is an intentional standalone type.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;

/// DTOs that are legitimately not reachable from an export *signature*: the value
/// surface uses them only through a callback the consumer implements, as a thrown
/// error, or as `string` (the opaque wasm-bindgen newtypes). They are part of the
/// public contract; they are simply not parameters or returns.
const STANDALONE_ALLOWLIST: &[&str] = &[
    // Opaque wasm-bindgen newtypes — the DTO surface represents these as `string`.
    "Address",
    "Amount",
    "AppDataHash",
    "Hash32",
    "HexData",
    "OrderUid",
    // Error types — thrown across the boundary, not returned in a signature.
    "WasmError",
    "CowError",
    "OrderBookRejectionCategoryDto",
    // Callback request shapes — reached through the callback type a consumer
    // implements, not through a direct export signature.
    "CowEip1271SignRequest",
    "ContractCall",
    "CowFetchCallback",
    "CowFetchRequest",
    "CowFetchResponse",
    "CowFetchMethod",
];

const METHOD_SKIP: &[&str] = &["free", "constructor"];

#[test]
fn every_raw_dto_is_reachable_or_allowlisted() {
    let mut leaks = Vec::new();
    for flavour in flavours() {
        let text = read(&snapshot("raw", &flavour));
        let (dtos, reachable) = analyze(&text);
        for name in dtos.difference(&reachable) {
            if !STANDALONE_ALLOWLIST.contains(&name.as_str()) {
                leaks.push(format!("{flavour}: DTO `{name}`"));
            }
        }
    }
    assert!(
        leaks.is_empty(),
        "raw DTO(s) emitted into a flavour but unreachable from any export — a \
         feature-gating leak. Gate the struct + its `dto/mod.rs` re-export to the union \
         of its consumers' features, or add it to STANDALONE_ALLOWLIST with a reason:\n  {}",
        leaks.join("\n  "),
    );
}

/// Parse a raw `.d.ts` and return (declared DTO names, reachable type names). A type
/// is reachable if it appears in an export/method signature (a root) or in a field of
/// a reachable type (transitive).
fn analyze(text: &str) -> (BTreeSet<String>, BTreeSet<String>) {
    let universe: BTreeSet<String> = text.lines().filter_map(decl).map(|(_, n)| n).collect();
    let mut is_class: BTreeMap<String, bool> = BTreeMap::new();
    let mut edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut roots: BTreeSet<String> = BTreeSet::new();
    let mut current: Option<String> = None;

    for line in text.lines() {
        if let Some((kind, name)) = decl(line) {
            is_class.insert(name.clone(), kind == "class");
            if kind == "class" {
                roots.insert(name.clone()); // consumers instantiate client classes
            }
            // decl-line refs: type-alias RHS, or interface/class `extends`/generics.
            let tail = match line.split_once('=') {
                Some((_, rhs)) if kind == "type" => rhs,
                _ => line.split_once(&name).map_or("", |(_, rest)| rest),
            };
            edges
                .entry(name.clone())
                .or_default()
                .extend(type_names(tail, &universe));
            current = Some(name);
            continue;
        }
        if let Some(class) = current.clone() {
            if line.trim_end() == "}" {
                current = None;
            } else if is_method(line) {
                roots.extend(type_names(line, &universe)); // entry-point signature
            } else {
                edges
                    .entry(class)
                    .or_default()
                    .extend(type_names(line, &universe)); // field
            }
            continue;
        }
        if line.starts_with("export ") && line.contains("function ") {
            roots.extend(type_names(line, &universe));
        }
    }

    let mut reachable = BTreeSet::new();
    let mut stack: Vec<String> = roots.into_iter().filter(|r| universe.contains(r)).collect();
    while let Some(t) = stack.pop() {
        if !reachable.insert(t.clone()) {
            continue;
        }
        if let Some(next) = edges.get(&t) {
            stack.extend(next.iter().cloned());
        }
    }

    let dtos = universe.iter().filter(|t| !is_class[*t]).cloned().collect();
    (dtos, reachable)
}

/// `export [declare] [abstract] interface|type|class|enum NAME` -> (kind, NAME).
fn decl(line: &str) -> Option<(&'static str, String)> {
    let rest = line.strip_prefix("export ")?;
    let rest = rest.strip_prefix("declare ").unwrap_or(rest);
    let rest = rest.strip_prefix("abstract ").unwrap_or(rest);
    for (kw, kind) in [
        ("interface ", "interface"),
        ("type ", "type"),
        ("class ", "class"),
        ("enum ", "enum"),
    ] {
        if let Some(rest) = rest.strip_prefix(kw) {
            let name = take_ident(rest);
            if !name.is_empty() {
                return Some((kind, name));
            }
        }
    }
    None
}

/// A four-space-indented method or constructor signature inside a class body.
fn is_method(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("    ") else {
        return false;
    };
    if rest.starts_with(' ') {
        return false;
    }
    let name = take_ident(rest);
    !name.is_empty() && rest[name.len()..].starts_with('(') && !METHOD_SKIP.contains(&name.as_str())
}

fn type_names(s: &str, universe: &BTreeSet<String>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut token = String::new();
    for ch in s.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            token.push(ch);
        } else {
            if token.chars().next().is_some_and(char::is_uppercase) && universe.contains(&token) {
                out.insert(token.clone());
            }
            token.clear();
        }
    }
    if token.chars().next().is_some_and(char::is_uppercase) && universe.contains(&token) {
        out.insert(token);
    }
    out
}

fn take_ident(s: &str) -> String {
    s.chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

fn flavours() -> Vec<String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("npm")
        .join("flavours.json");
    let descriptor: Value =
        serde_json::from_str(&fs::read_to_string(path).expect("flavours.json must be readable"))
            .expect("flavours.json must be valid JSON");
    descriptor["flavours"]
        .as_array()
        .expect("flavours must be an array")
        .iter()
        .map(|flavour| flavour["name"].as_str().expect("flavour name").to_owned())
        .collect()
}

fn snapshot(kind: &str, flavour: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("snapshots")
        .join(kind)
        .join(format!("{flavour}.d.ts"))
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("snapshot {} must be readable", path.display()))
}
