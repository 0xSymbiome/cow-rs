//! Vendor commands for the composable parity fixture set.
//!
//! The composable parity fixtures live under `parity/fixtures/composable/`
//! and cover selectors, params hashes, multiplexer merkle leaves, the two
//! EIP-1271 signature blob shapes, conditional-order params decoders, poll
//! result selectors, TWAP static input encoding, TWAP order id derivation,
//! TWAP merkle leaf computation, per-handler revert reasons, perpetual
//! stable swap overflow probes, and poll-result classification rows. This
//! module owns the catalog that ties each shipped fixture file to its
//! upstream provenance row.

use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use crate::cow_shed_fixtures::UpstreamProvenance;

/// Fixture entry pairing a shipped fixture path with its upstream provenance.
#[derive(Debug, Clone)]
pub(crate) struct ComposableFixtureEntry {
    pub fixture_path: PathBuf,
    #[allow(dead_code)]
    pub upstream_provenance: Vec<UpstreamProvenance>,
}

/// The shipped composable parity fixture catalog.
pub(crate) fn catalog() -> Vec<ComposableFixtureEntry> {
    vec![
        ComposableFixtureEntry {
            fixture_path: PathBuf::from("parity/fixtures/composable/selectors.json"),
            upstream_provenance: vec![UpstreamProvenance {
                repo: "composable-cow",
                path: "src/ComposableCoW.sol",
            }],
        },
        ComposableFixtureEntry {
            fixture_path: PathBuf::from("parity/fixtures/composable/params_hash.json"),
            upstream_provenance: vec![UpstreamProvenance {
                repo: "composable-cow",
                path: "src/ComposableCoW.sol",
            }],
        },
        ComposableFixtureEntry {
            fixture_path: PathBuf::from("parity/fixtures/composable/multiplexer_leaf.json"),
            upstream_provenance: vec![UpstreamProvenance {
                repo: "composable-cow",
                path: "src/ComposableCoW.sol",
            }],
        },
        ComposableFixtureEntry {
            fixture_path: PathBuf::from(
                "parity/fixtures/composable/safe_muxer_signature_blob.json",
            ),
            upstream_provenance: vec![UpstreamProvenance {
                repo: "composable-cow",
                path: "src/ERC1271Forwarder.sol",
            }],
        },
        ComposableFixtureEntry {
            fixture_path: PathBuf::from("parity/fixtures/composable/forwarder_signature_blob.json"),
            upstream_provenance: vec![UpstreamProvenance {
                repo: "composable-cow",
                path: "src/ERC1271Forwarder.sol",
            }],
        },
        ComposableFixtureEntry {
            fixture_path: PathBuf::from(
                "parity/fixtures/composable/conditional_order_params_decode.json",
            ),
            upstream_provenance: vec![UpstreamProvenance {
                repo: "composable-cow",
                path: "src/interfaces/IConditionalOrder.sol",
            }],
        },
        ComposableFixtureEntry {
            fixture_path: PathBuf::from("parity/fixtures/composable/poll_result_selectors.json"),
            upstream_provenance: vec![UpstreamProvenance {
                repo: "composable-cow",
                path: "src/interfaces/IConditionalOrder.sol",
            }],
        },
        ComposableFixtureEntry {
            fixture_path: PathBuf::from("parity/fixtures/composable/twap_static_input.json"),
            upstream_provenance: vec![UpstreamProvenance {
                repo: "composable-cow",
                path: "src/types/twap/TWAP.sol",
            }],
        },
        ComposableFixtureEntry {
            fixture_path: PathBuf::from("parity/fixtures/composable/twap_order_id.json"),
            upstream_provenance: vec![UpstreamProvenance {
                repo: "composable-cow",
                path: "src/types/twap/TWAP.sol",
            }],
        },
        ComposableFixtureEntry {
            fixture_path: PathBuf::from("parity/fixtures/composable/twap_merkle_leaf.json"),
            upstream_provenance: vec![UpstreamProvenance {
                repo: "composable-cow",
                path: "src/types/twap/TWAP.sol",
            }],
        },
        ComposableFixtureEntry {
            fixture_path: PathBuf::from(
                "parity/fixtures/composable/good_after_time_revert_sites.json",
            ),
            upstream_provenance: vec![UpstreamProvenance {
                repo: "composable-cow",
                path: "src/types/GoodAfterTime.sol",
            }],
        },
        ComposableFixtureEntry {
            fixture_path: PathBuf::from("parity/fixtures/composable/stop_loss_revert_sites.json"),
            upstream_provenance: vec![UpstreamProvenance {
                repo: "composable-cow",
                path: "src/types/StopLoss.sol",
            }],
        },
        ComposableFixtureEntry {
            fixture_path: PathBuf::from(
                "parity/fixtures/composable/trade_above_threshold_revert_sites.json",
            ),
            upstream_provenance: vec![UpstreamProvenance {
                repo: "composable-cow",
                path: "src/types/TradeAboveThreshold.sol",
            }],
        },
        ComposableFixtureEntry {
            fixture_path: PathBuf::from(
                "parity/fixtures/composable/perpetual_stable_swap_revert_sites.json",
            ),
            upstream_provenance: vec![UpstreamProvenance {
                repo: "composable-cow",
                path: "src/types/PerpetualStableSwap.sol",
            }],
        },
        ComposableFixtureEntry {
            fixture_path: PathBuf::from(
                "parity/fixtures/composable/perpetual_stable_swap_overflow.json",
            ),
            upstream_provenance: vec![UpstreamProvenance {
                repo: "composable-cow",
                path: "src/types/PerpetualStableSwap.sol",
            }],
        },
        ComposableFixtureEntry {
            fixture_path: PathBuf::from(
                "parity/fixtures/composable/poll_result_classification.json",
            ),
            upstream_provenance: vec![UpstreamProvenance {
                repo: "composable-cow",
                path: "src/interfaces/IConditionalOrder.sol",
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
                "composable fixture file `{}` is missing under repo root `{}`",
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
    fn catalog_has_sixteen_fixtures() {
        assert_eq!(catalog().len(), 16);
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
