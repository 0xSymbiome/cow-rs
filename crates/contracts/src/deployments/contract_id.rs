//! Logical identifier for the canonical `CoW` Protocol contracts the SDK resolves.
//!
//! The enum types the key space of [`Registry`](crate::deployments::Registry)
//! so downstream callers cannot accidentally pair an arbitrary string with a
//! chain id and a deployment environment.

use serde::{Deserialize, Serialize};

/// Canonical contract identifiers resolved by
/// [`Registry`](crate::deployments::Registry).
///
/// Every variant is a CREATE2 singleton that deploys to the same address on
/// every supported chain; the eth-flow periphery additionally carries a
/// distinct production and staging deployment. Variants follow the
/// **Pascal-case** convention.
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
    /// Returns the canonical Pascal-case name for this identifier.
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
