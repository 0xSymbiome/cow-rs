use std::path::Path;

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;

/// Loads and deserializes a YAML fixture, attaching the path to read and parse
/// failures.
pub fn load_yaml<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    serde_norway::from_str(&content).with_context(|| format!("failed to parse {}", path.display()))
}
