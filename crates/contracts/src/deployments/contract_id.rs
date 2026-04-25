//! Logical identifier for every contract tracked by the deployments registry.
//!
//! The enum types the key space of
//! [`cow_sdk_contracts::deployments::Registry`] so downstream callers
//! cannot accidentally pair an arbitrary string with a chain id and a
//! deployment environment.

use serde::{Deserialize, Serialize};

/// Canonical contract identifiers.
///
/// Variants follow the **Pascal-case** convention. Variants without a
/// version suffix track the currently deployed canonical version of the
/// underlying contract; new versions land as new variants with an explicit
/// version suffix (for example, `EthFlowV2`), and this convention is fixed
/// before composable or cow-shed registry expansion; do not introduce
/// kebab-case or numbered prefix styles for new variants.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ContractId {
    /// `GPv2Settlement` — the `CoW` Protocol settlement entry point.
    Settlement,
    /// `GPv2VaultRelayer` — relays balance operations into the Balancer vault.
    VaultRelayer,
    /// `CoWSwapEthFlow` — wraps the native asset into orders on behalf of traders.
    EthFlow,
}

impl ContractId {
    /// Returns the canonical Pascal-case name for this identifier, matching
    /// the TOML manifest spelling used by the registry's on-disk encoding.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Settlement => "Settlement",
            Self::VaultRelayer => "VaultRelayer",
            Self::EthFlow => "EthFlow",
        }
    }
}

impl std::fmt::Display for ContractId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
