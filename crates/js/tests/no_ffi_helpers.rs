//! The runtime-neutral `helpers` module must stay free of JavaScript FFI
//! bindings so its deterministic protocol logic compiles and runs on the host
//! without a wasm runtime. The wasm-bindgen surface lives in `src/exports`,
//! which legitimately carries these tokens and is intentionally not scanned.

use std::{fs, path::Path};

const FORBIDDEN_TOKENS: &[&str] = &[
    "wasm-bindgen",
    "wasm_bindgen",
    "tsify",
    "Tsify",
    "JsValue",
    "js-sys",
    "js_sys",
    "web-sys",
    "web_sys",
    "serde-wasm-bindgen",
    "serde_wasm_bindgen",
    "wasm-bindgen-futures",
    "wasm_bindgen_futures",
];

#[test]
fn helpers_module_does_not_import_ffi_bindings() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut hits = Vec::new();

    scan_dir(&manifest_dir.join("src").join("helpers"), &mut hits);

    assert!(
        hits.is_empty(),
        "the helpers module must remain FFI-neutral:\n{}",
        hits.join("\n")
    );
}

fn scan_dir(dir: &Path, hits: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", dir.display());
    }) {
        let entry = entry.unwrap_or_else(|error| {
            panic!("failed to read entry in {}: {error}", dir.display());
        });
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, hits);
        } else {
            scan_file(&path, hits);
        }
    }
}

fn scan_file(path: &Path, hits: &mut Vec<String>) {
    let content = fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    });
    for token in FORBIDDEN_TOKENS {
        if content.contains(token) {
            hits.push(format!("{} contains `{token}`", path.display()));
        }
    }
}
