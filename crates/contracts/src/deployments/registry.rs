//! Chain-keyed registry of canonical `CoW` Protocol contract deployments.
//!
//! [`Registry`] is the single in-crate authority for resolving a deployed
//! contract address from the `(ContractId, SupportedChainId, CowEnv)` key
//! triple. [`Registry::default`] loads the manifest committed at
//! `crates/contracts/registry.toml` (embedded into the crate binary at
//! compile time through `include_str!`) and [`Registry::address`] is the
//! primary lookup API.
//!
//! The manifest is validated twice: once by `build.rs` as a compile-time
//! gate so malformed rows produce a precise fix target, and once again by
//! this module's runtime parser so downstream consumers can pipe arbitrary
//! TOML into [`Registry::from_toml_str`] without inviting panics. The
//! runtime parser surfaces every failure mode as a typed [`RegistryError`].

use std::cmp::Ordering;
use std::collections::BTreeMap;

use cow_sdk_core::{Address, CowEnv, SupportedChainId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::ContractId;

/// Reviewed TOML-schema version carried at the head of every manifest.
const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RegistryKey {
    contract_id: ContractId,
    chain_id: SupportedChainId,
    env: CowEnv,
}

impl Ord for RegistryKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.contract_id
            .cmp(&other.contract_id)
            .then_with(|| self.chain_id.cmp(&other.chain_id))
            .then_with(|| self.env.as_str().cmp(other.env.as_str()))
    }
}

impl PartialOrd for RegistryKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Canonical registry data embedded from `crates/contracts/registry.toml`.
const EMBEDDED_REGISTRY_TOML: &str = include_str!("../../registry.toml");

/// Chain-keyed lookup table of deployed `CoW` Protocol contracts.
///
/// The backing storage is a [`BTreeMap`] so iteration order is
/// deterministic across audits and so CI diffs remain stable. Keys are the
/// `(ContractId, SupportedChainId, CowEnv)` triple; values are typed
/// [`Address`] handles that have already passed the validator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registry {
    entries: BTreeMap<RegistryKey, Address>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::from_toml_str(EMBEDDED_REGISTRY_TOML)
            .expect("embedded registry manifest must be valid — build.rs gates the shape")
    }
}

impl Registry {
    /// Returns the deployed address registered for the supplied identifier
    /// tuple, or [`None`] when no matching entry is present.
    #[must_use]
    pub fn address(
        &self,
        contract_id: ContractId,
        chain_id: SupportedChainId,
        env: CowEnv,
    ) -> Option<Address> {
        self.entries
            .get(&RegistryKey {
                contract_id,
                chain_id,
                env,
            })
            .cloned()
    }

    /// Returns a sorted view of every `(ContractId, SupportedChainId, CowEnv)`
    /// tuple registered in this registry, paired with its resolved
    /// [`Address`]. Useful for audit diffs and validation suites that walk
    /// the complete manifest.
    pub fn entries(
        &self,
    ) -> impl Iterator<Item = (ContractId, SupportedChainId, CowEnv, &Address)> + '_ {
        self.entries
            .iter()
            .map(|(key, address)| (key.contract_id, key.chain_id, key.env, address))
    }

    /// Number of registered entries in this registry.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` when the registry carries no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns a new registry with the supplied
    /// `(ContractId, SupportedChainId, CowEnv)` entry replaced by `address`.
    ///
    /// Consumers that need to point a single lookup at a non-default
    /// deployment (for example, a local-dev settlement contract) layer the
    /// override on top of [`Registry::default`] and keep resolving through
    /// the typed [`Registry::address`] surface.
    #[must_use]
    pub fn with_override(
        mut self,
        contract_id: ContractId,
        chain_id: SupportedChainId,
        env: CowEnv,
        address: Address,
    ) -> Self {
        self.entries.insert(
            RegistryKey {
                contract_id,
                chain_id,
                env,
            },
            address,
        );
        self
    }

    /// Parses a TOML manifest string into a typed registry, applying the
    /// same validation rules the compile-time gate enforces.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] when the manifest schema version drifts,
    /// when any row uses an unknown contract identifier, when any row pins a
    /// chain id outside the supported set, when any row's address fails the
    /// 20-byte hex shape, or when the manifest contains a duplicate
    /// `(ContractId, SupportedChainId, CowEnv)` key.
    pub fn from_toml_str(raw: &str) -> Result<Self, RegistryError> {
        let manifest: ManifestSchema =
            toml::from_str(raw).map_err(|source| RegistryError::Parse {
                source: Box::new(source),
            })?;
        if manifest.schema_version != SCHEMA_VERSION {
            return Err(RegistryError::UnsupportedSchemaVersion {
                expected: SCHEMA_VERSION,
                actual: manifest.schema_version,
            });
        }

        let mut entries: BTreeMap<RegistryKey, Address> = BTreeMap::new();
        for row in manifest.entries {
            let chain_id = SupportedChainId::try_from(row.chain_id).map_err(|_| {
                RegistryError::UnsupportedChainId {
                    contract_id: row.contract_id,
                    chain_id: row.chain_id,
                }
            })?;
            let address =
                Address::new(&row.address).map_err(|source| RegistryError::InvalidAddress {
                    contract_id: row.contract_id,
                    chain_id: row.chain_id,
                    env: row.env,
                    address: row.address.clone(),
                    message: source.to_string(),
                })?;
            let key = RegistryKey {
                contract_id: row.contract_id,
                chain_id,
                env: row.env,
            };
            if entries.insert(key, address).is_some() {
                return Err(RegistryError::DuplicateEntry {
                    contract_id: row.contract_id,
                    chain_id: row.chain_id,
                    env: row.env,
                });
            }
        }

        Ok(Self { entries })
    }
}

/// Typed error surface returned by the registry runtime parser.
///
/// Mirrors the set of failure modes that `build.rs` rejects at compile
/// time, so downstream consumers that load their own manifest through
/// [`Registry::from_toml_str`] see the same diagnostic taxonomy the
/// shipped manifest must satisfy.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum RegistryError {
    /// The TOML manifest could not be parsed; the underlying
    /// [`toml::de::Error`] is preserved through the error-source chain.
    #[error("failed to parse registry manifest: {source}")]
    Parse {
        /// Typed TOML-deserialization error sourced from the parser.
        #[source]
        source: Box<toml::de::Error>,
    },
    /// The manifest declared a `schema_version` the loader does not support.
    #[error("unsupported registry schema version: expected {expected}, got {actual}")]
    UnsupportedSchemaVersion {
        /// Schema version the loader was built against.
        expected: u32,
        /// Schema version the manifest declared.
        actual: u32,
    },
    /// A manifest row referenced a chain id outside the supported set.
    #[error("unsupported chain id {chain_id} on `{contract_id}` entry")]
    UnsupportedChainId {
        /// Contract identifier on the offending row.
        contract_id: ContractId,
        /// Raw chain id value that failed the `SupportedChainId` check.
        chain_id: u64,
    },
    /// A manifest row carried a malformed deployment address.
    #[error(
        "invalid address `{address}` on `{contract_id}` / chain {chain_id} / {env:?}: {message}"
    )]
    InvalidAddress {
        /// Contract identifier on the offending row.
        contract_id: ContractId,
        /// Raw chain id value copied from the manifest row.
        chain_id: u64,
        /// Environment value copied from the manifest row.
        env: CowEnv,
        /// Raw address literal that failed the 20-byte hex check.
        address: String,
        /// Redacted detail from the address validator.
        message: String,
    },
    /// Two manifest rows shared the same `(ContractId, SupportedChainId, CowEnv)` key.
    #[error("duplicate registry entry for `{contract_id}` / chain {chain_id} / {env:?}")]
    DuplicateEntry {
        /// Contract identifier on the duplicated rows.
        contract_id: ContractId,
        /// Raw chain id value shared by the duplicated rows.
        chain_id: u64,
        /// Environment value shared by the duplicated rows.
        env: CowEnv,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestSchema {
    schema_version: u32,
    #[serde(default)]
    entries: Vec<ManifestEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ManifestEntry {
    contract_id: ContractId,
    chain_id: u64,
    env: CowEnv,
    address: String,
}
