//! Deployment coverage records for intentionally absent contracts.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::ContractId;

/// Coverage status for a contract/chain pair without a registry deployment row.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentCoverageStatus {
    /// No deployment exists for the chain.
    NotDeployed,
    /// The chain is not supported for this deployment family.
    NotSupported,
    /// The chain is intentionally outside the deployment scope.
    OutOfScope,
}

impl DeploymentCoverageStatus {
    /// Returns the manifest spelling for this status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotDeployed => "not_deployed",
            Self::NotSupported => "not_supported",
            Self::OutOfScope => "out_of_scope",
        }
    }
}

impl std::fmt::Display for DeploymentCoverageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CoverageKey {
    contract_id: ContractId,
    chain_id: u64,
}

impl Ord for CoverageKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.contract_id
            .cmp(&other.contract_id)
            .then_with(|| self.chain_id.cmp(&other.chain_id))
    }
}

impl PartialOrd for CoverageKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Embedded deployment coverage matrix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeploymentCoverage {
    records: BTreeMap<CoverageKey, DeploymentCoverageRecord>,
}

impl Default for DeploymentCoverage {
    fn default() -> Self {
        Self::from_yaml_str(include_str!("../../deployment-coverage.yaml"))
            .expect("embedded deployment coverage must be valid - build.rs gates the shape")
    }
}

impl DeploymentCoverage {
    /// Returns the coverage status for a contract/chain pair when present.
    #[must_use]
    pub fn status(
        &self,
        contract_id: ContractId,
        chain_id: impl Into<u64>,
    ) -> Option<DeploymentCoverageStatus> {
        self.records
            .get(&CoverageKey {
                contract_id,
                chain_id: chain_id.into(),
            })
            .map(|record| record.status)
    }

    /// Returns the evidence note for a contract/chain pair when present.
    #[must_use]
    pub fn evidence(&self, contract_id: ContractId, chain_id: impl Into<u64>) -> Option<&str> {
        self.records
            .get(&CoverageKey {
                contract_id,
                chain_id: chain_id.into(),
            })
            .map(|record| record.evidence.as_str())
    }

    /// Returns every coverage record in deterministic order.
    pub fn records(
        &self,
    ) -> impl Iterator<Item = (ContractId, u64, DeploymentCoverageStatus, &str)> + '_ {
        self.records.iter().map(|(key, record)| {
            (
                key.contract_id,
                key.chain_id,
                record.status,
                record.evidence.as_str(),
            )
        })
    }

    /// Number of coverage records.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns `true` when the coverage matrix is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Parses a coverage YAML manifest.
    ///
    /// # Errors
    ///
    /// Returns [`DeploymentCoverageError`] when the schema version is not
    /// supported or duplicate records are present.
    pub fn from_yaml_str(raw: &str) -> Result<Self, DeploymentCoverageError> {
        let manifest: CoverageManifest =
            serde_yaml::from_str(raw).map_err(DeploymentCoverageError::Parse)?;
        if manifest.schema_version != 2 {
            return Err(DeploymentCoverageError::UnsupportedSchemaVersion {
                expected: 2,
                actual: manifest.schema_version,
            });
        }

        let mut records = BTreeMap::new();
        for row in manifest.coverage {
            let key = CoverageKey {
                contract_id: row.contract_id,
                chain_id: row.chain_id,
            };
            if records.insert(key, row).is_some() {
                return Err(DeploymentCoverageError::DuplicateRecord {
                    contract_id: key.contract_id,
                    chain_id: key.chain_id,
                });
            }
        }

        Ok(Self { records })
    }
}

/// Error returned by the deployment coverage parser.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum DeploymentCoverageError {
    /// YAML parsing failed.
    #[error("failed to parse deployment coverage manifest: {0}")]
    Parse(#[source] serde_yaml::Error),
    /// The coverage manifest declared an unsupported schema version.
    #[error("unsupported deployment coverage schema version: expected {expected}, got {actual}")]
    UnsupportedSchemaVersion {
        /// Schema version the loader was built against.
        expected: u32,
        /// Schema version the manifest declared.
        actual: u32,
    },
    /// Two rows shared the same `(ContractId, DeploymentChainId)` key.
    #[error("duplicate deployment coverage record for `{contract_id}` / chain {chain_id}")]
    DuplicateRecord {
        /// Contract identifier on the duplicated row.
        contract_id: ContractId,
        /// Chain identifier on the duplicated row.
        chain_id: u64,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CoverageManifest {
    schema_version: u32,
    #[serde(default)]
    coverage: Vec<DeploymentCoverageRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeploymentCoverageRecord {
    contract_id: ContractId,
    chain_id: u64,
    status: DeploymentCoverageStatus,
    evidence: String,
}
