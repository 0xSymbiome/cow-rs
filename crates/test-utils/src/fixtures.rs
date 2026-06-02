//! Shared parity-fixture loaders for the workspace test suites.
//!
//! Because `include_str!` needs a *literal* path, a shared loader must read at
//! runtime. This crate lives at `<workspace_root>/crates/test-utils`, so the
//! workspace root is two parents up from `CARGO_MANIFEST_DIR`.

use std::path::{Path, PathBuf};

use serde_json::Value;

/// Resolves the workspace root from this crate's manifest directory.
///
/// # Panics
/// Panics if the crate is not located at `<root>/crates/test-utils`.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("cow-sdk-test-utils must live at <workspace_root>/crates/test-utils")
        .to_path_buf()
}

/// Loads `parity/fixtures/<name>.json` (Form A — the canonical
/// `{ "cases": [ { "id": .. } ] }` registry shape).
///
/// # Panics
/// Panics if the file cannot be read or is not valid JSON.
#[must_use]
pub fn fixture(name: &str) -> Value {
    let path = workspace_root()
        .join("parity/fixtures")
        .join(format!("{name}.json"));
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|error| panic!("fixture {} must be valid JSON: {error}", path.display()))
}

/// Returns the case with the given `id` from the `cases` array of
/// `parity/fixtures/<name>.json`.
///
/// # Panics
/// Panics if the fixture lacks a `cases` array or the `id` is absent.
#[must_use]
pub fn case(name: &str, id: &str) -> Value {
    fixture(name)["cases"]
        .as_array()
        .unwrap_or_else(|| panic!("fixture {name} must have a `cases` array"))
        .iter()
        .find(|case| case["id"] == id)
        .cloned()
        .unwrap_or_else(|| panic!("missing fixture case {id} in {name}"))
}

/// Loads a fixture relative to a *consuming crate's* manifest directory
/// (Form B — the contracts selector fixtures under `tests/fixtures/`). The
/// caller passes its own `env!("CARGO_MANIFEST_DIR")`.
///
/// # Panics
/// Panics if the file cannot be read or is not valid JSON.
#[must_use]
pub fn manifest_fixture(manifest_dir: &str, rel_path: &str) -> Value {
    let path = Path::new(manifest_dir).join(rel_path);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|error| panic!("fixture {} must be valid JSON: {error}", path.display()))
}

/// Finds the row whose `name` field equals `name` within `fixture[array_key]`.
///
/// # Panics
/// Panics if `array_key` is not an array or no row matches.
#[must_use]
pub fn row_by_name(fixture: &Value, array_key: &str, name: &str) -> Value {
    fixture[array_key]
        .as_array()
        .unwrap_or_else(|| panic!("fixture key `{array_key}` must be an array"))
        .iter()
        .find(|row| row["name"] == name)
        .cloned()
        .unwrap_or_else(|| panic!("missing `{name}` under `{array_key}`"))
}
