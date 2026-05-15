//! Audit refresh-map validator.
//!
//! Confirms `.github/config/audit-refresh-map.yml` has a non-zero version and
//! at least one entry. Future expansion may check that every audit slug has a
//! matching `docs/audit/<slug>.md` artifact; the current contract is the
//! minimum required by the readiness gate.

use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AuditRefreshManifest {
    version: u32,
    #[serde(default)]
    entries: Vec<AuditRefreshRow>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AuditRefreshRow {
    #[serde(default)]
    audit: Option<String>,
    #[serde(default)]
    owning_surface: Option<String>,
}

/// Load and validate the audit refresh map. Returns `Ok(())` when version is
/// non-zero and the entry list is non-empty.
pub(crate) fn run(path: &Path) -> Result<()> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read audit refresh map {}", path.display()))?;
    check_yaml(&raw)?;
    println!("validated audit refresh map");
    Ok(())
}

fn check_yaml(raw: &str) -> Result<()> {
    let manifest: AuditRefreshManifest =
        serde_yaml::from_str(raw).context("failed to parse audit refresh map")?;
    if manifest.version == 0 {
        bail!("audit refresh map version must be non-zero");
    }
    if manifest.entries.is_empty() {
        bail!("audit refresh map must contain at least one entry");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_manifest_passes() {
        let raw = "version: 1\nentries:\n  - audit: foo\n    owning_surface: bar\n";
        assert!(check_yaml(raw).is_ok());
    }

    #[test]
    fn zero_version_fails() {
        let raw = "version: 0\nentries:\n  - audit: foo\n";
        assert!(check_yaml(raw).is_err());
    }

    #[test]
    fn empty_entries_fails() {
        let raw = "version: 1\nentries: []\n";
        assert!(check_yaml(raw).is_err());
    }
}
