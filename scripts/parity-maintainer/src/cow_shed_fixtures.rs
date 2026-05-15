//! Vendor commands for the COW Shed parity fixture set.
//!
//! The COW Shed parity fixtures live under `parity/fixtures/cow_shed/` and
//! cover proxy addresses, execute-hooks digests, calldata snapshots, domain
//! separators, and EOA signature byte order. Each fixture is byte-exact
//! against an upstream COW Shed test vector; this module owns the catalog
//! that ties each shipped fixture file to its upstream provenance row.

use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

/// Fixture entry pairing a shipped fixture path with its upstream provenance
/// references. The provenance rows are not currently consumed by the binary;
/// they exist so a future `vendor-cow-shed-fixtures` subcommand can resolve
/// the upstream artifacts when regenerating the byte-identity vectors.
#[derive(Debug, Clone)]
pub(crate) struct CowShedFixtureEntry {
    /// Repository-relative path to the shipped fixture.
    pub fixture_path: PathBuf,
    /// One or more upstream provenance entries (repo id and source path).
    #[allow(dead_code)]
    pub upstream_provenance: Vec<UpstreamProvenance>,
}

/// One upstream provenance row.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct UpstreamProvenance {
    /// Source-lock repository id (e.g. `cow-shed`, `cow-sdk`).
    pub repo: &'static str,
    /// Repository-relative path inside the pinned upstream tree.
    pub path: &'static str,
}

/// The shipped COW Shed parity fixture catalog. Each entry must remain
/// byte-exact against its upstream provenance row.
pub(crate) fn catalog() -> Vec<CowShedFixtureEntry> {
    vec![
        CowShedFixtureEntry {
            fixture_path: PathBuf::from("parity/fixtures/cow_shed/proxy_addresses.json"),
            upstream_provenance: vec![
                UpstreamProvenance {
                    repo: "cow-shed",
                    path: "src/COWShedFactory.sol",
                },
                UpstreamProvenance {
                    repo: "cow-sdk",
                    path: "packages/cow-shed/src/const.ts",
                },
            ],
        },
        CowShedFixtureEntry {
            fixture_path: PathBuf::from("parity/fixtures/cow_shed/execute_hooks_digest.json"),
            upstream_provenance: vec![UpstreamProvenance {
                repo: "cow-shed",
                path: "src/LibAuthenticatedHooks.sol",
            }],
        },
        CowShedFixtureEntry {
            fixture_path: PathBuf::from("parity/fixtures/cow_shed/execute_hooks_calldata.json"),
            upstream_provenance: vec![UpstreamProvenance {
                repo: "cow-shed",
                path: "src/COWShed.sol",
            }],
        },
        CowShedFixtureEntry {
            fixture_path: PathBuf::from("parity/fixtures/cow_shed/domain_separator.json"),
            upstream_provenance: vec![UpstreamProvenance {
                repo: "cow-shed",
                path: "src/COWShed.sol",
            }],
        },
        CowShedFixtureEntry {
            fixture_path: PathBuf::from("parity/fixtures/cow_shed/eoa_signature_byte_order.json"),
            upstream_provenance: vec![UpstreamProvenance {
                repo: "cow-shed",
                path: "src/LibAuthenticatedHooks.sol",
            }],
        },
    ]
}

/// Validate that every catalog fixture file exists on disk under `repo_root`.
pub(crate) fn validate_catalog_files_exist(repo_root: &Path) -> Result<usize> {
    let entries = catalog();
    for entry in &entries {
        let abs = repo_root.join(&entry.fixture_path);
        if !abs.exists() {
            bail!(
                "COW Shed fixture file `{}` is missing under repo root `{}`",
                entry.fixture_path.display(),
                repo_root.display()
            );
        }
    }
    Ok(entries.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_five_fixtures() {
        assert_eq!(catalog().len(), 5);
    }

    #[test]
    fn every_entry_has_at_least_one_upstream_row() {
        for entry in catalog() {
            assert!(
                !entry.upstream_provenance.is_empty(),
                "fixture {} has no upstream provenance",
                entry.fixture_path.display()
            );
        }
    }
}
