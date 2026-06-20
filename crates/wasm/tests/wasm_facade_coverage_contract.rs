#![cfg(not(target_arch = "wasm32"))]

//! Facade coverage contract: every public symbol in the raw wasm-bindgen bindings
//! (`snapshots/raw/`) must be exposed by the hand-written TypeScript facade
//! (`snapshots/facade/`), or listed in an allowlist below with a reason.
//!
//! The byte-locked snapshots and the two snapshot-surface tests pin *what each
//! layer contains*; this test pins the *relationship* between them. It exists
//! because the facade is curated by hand: a raw export with no facade counterpart
//! — a client method, a free function, or a DTO type — is otherwise invisible to
//! every gate and ships unusable, as the native wrap/unwrap builders did before
//! they were wired through the facade.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;

/// Raw free functions the facade intentionally does not re-export.
const FUNC_ALLOWLIST: &[&str] = &[
    "__cow_sdk_wasm_init", // wasm-bindgen module-init export, replaced by `initialize`
];

/// Raw exported types the facade intentionally does not surface, with the reason.
const TYPE_ALLOWLIST: &[&str] = &[
    // Opaque wasm-bindgen newtypes — the DTO surface represents these as `string`.
    "Address",
    "AppDataHash",
    "Hash32",
    "HexData",
    "OrderUid",
    // wasm-bindgen init plumbing.
    "InitInput",
    "InitOutput",
    "SyncInitInput",
    // serde_json passthrough escape hatch.
    "Value",
    // Deliberate facade renames (surfaced under a cleaner name).
    "WasmError",                     // surfaced as `CowError`
    "OrderBookRejectionCategoryDto", // surfaced as `OrderBookRejectionCategory`
    // Fetch HTTP-method union owned by the hand-written options module; the facade
    // surfaces the options-side fetch types, not this raw tsify one.
    "CowFetchMethod",
];

/// Class members that are not part of the public method surface.
const METHOD_SKIP: &[&str] = &["free", "constructor"];

#[derive(Default)]
struct Surface {
    funcs: BTreeSet<String>,
    types: BTreeSet<String>,
    methods: BTreeMap<String, BTreeSet<String>>,
}

#[test]
fn facade_covers_raw_public_surface() {
    let mut gaps = Vec::new();
    for flavour in flavours() {
        let raw = parse(&read(&snapshot("raw", &flavour)));
        let facade = parse(&read(&snapshot("facade", &flavour)));

        for name in raw.funcs.difference(&facade.funcs) {
            if !FUNC_ALLOWLIST.contains(&name.as_str()) {
                gaps.push(format!("{flavour}: free function `{name}`"));
            }
        }
        for (class, methods) in &raw.methods {
            let Some(facade_methods) = facade.methods.get(class) else {
                continue; // a raw class missing from the facade is caught as a missing type
            };
            for name in methods.difference(facade_methods) {
                gaps.push(format!("{flavour}: method `{class}.{name}`"));
            }
        }
        for name in raw.types.difference(&facade.types) {
            if !TYPE_ALLOWLIST.contains(&name.as_str()) {
                gaps.push(format!("{flavour}: type `{name}`"));
            }
        }
    }

    assert!(
        gaps.is_empty(),
        "the hand-written facade is missing raw surface. Expose each in \
         `crates/wasm/npm/src/<flavour>.ts` (a wrapper method/function or an \
         `export type` re-export), or add it to an allowlist in this test with a \
         reason:\n  {}",
        gaps.join("\n  "),
    );
}

fn parse(text: &str) -> Surface {
    let mut surface = Surface::default();
    let mut current_class: Option<String> = None;
    for line in text.lines() {
        if let Some(name) = export_decl(line, "function ") {
            surface.funcs.insert(name);
        } else if let Some(name) = class_decl(line) {
            surface.types.insert(name.clone());
            surface.methods.entry(name.clone()).or_default();
            current_class = Some(name);
        } else if let Some(names) = reexport_decl(line) {
            surface.types.extend(names);
        } else if let Some(name) = type_decl(line) {
            surface.types.insert(name);
        } else if let Some(class) = &current_class {
            if line.trim_end() == "}" {
                current_class = None;
            } else if let Some(name) = method_decl(line) {
                surface
                    .methods
                    .get_mut(class)
                    .expect("open class scope")
                    .insert(name);
            }
        }
    }
    surface
}

/// `export [declare] <keyword><ident>` -> ident.
fn export_decl(line: &str, keyword: &str) -> Option<String> {
    non_empty(take_ident(after_export(line)?.strip_prefix(keyword)?))
}

fn class_decl(line: &str) -> Option<String> {
    let rest = after_export(line)?;
    let rest = rest.strip_prefix("abstract ").unwrap_or(rest);
    non_empty(take_ident(rest.strip_prefix("class ")?))
}

fn type_decl(line: &str) -> Option<String> {
    let rest = after_export(line)?;
    for keyword in ["interface ", "type ", "enum ", "const "] {
        if let Some(rest) = rest.strip_prefix(keyword) {
            if rest.starts_with('{') {
                return None; // an `export type { … }` re-export, handled separately
            }
            return non_empty(take_ident(rest));
        }
    }
    None
}

/// `export [type] { A, B as C, … } from "…"` -> [A, C, …].
fn reexport_decl(line: &str) -> Option<Vec<String>> {
    if !line.contains(" from ") {
        return None;
    }
    let rest = line.strip_prefix("export ")?;
    let rest = rest.strip_prefix("type ").unwrap_or(rest);
    let inner = rest.strip_prefix('{')?.split('}').next()?;
    let names: Vec<String> = inner
        .split(',')
        .filter_map(|entry| {
            let entry = entry.trim();
            non_empty(
                entry
                    .rsplit(" as ")
                    .next()
                    .unwrap_or(entry)
                    .trim()
                    .to_owned(),
            )
        })
        .collect();
    (!names.is_empty()).then_some(names)
}

/// A four-space-indented `name(` member of an open class body.
fn method_decl(line: &str) -> Option<String> {
    let rest = line.strip_prefix("    ")?;
    if rest.starts_with(' ') {
        return None; // deeper nesting or doc continuation, not a direct member
    }
    let name = take_ident(rest);
    if name.is_empty()
        || !rest[name.len()..].starts_with('(')
        || METHOD_SKIP.contains(&name.as_str())
    {
        return None;
    }
    Some(name)
}

fn after_export(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("export ")?;
    Some(rest.strip_prefix("declare ").unwrap_or(rest))
}

fn take_ident(s: &str) -> String {
    s.chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

fn non_empty(s: String) -> Option<String> {
    (!s.is_empty()).then_some(s)
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
