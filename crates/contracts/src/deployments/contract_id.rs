//! Logical identifier for every contract tracked by the deployments registry.
//!
//! The enum types the key space of
//! [`Registry`](crate::deployments::Registry) so downstream callers
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
    /// `ComposableCoW` — conditional-order dispatcher for composable order flows.
    ComposableCow,
    /// `ExtensibleFallbackHandler` — Safe fallback handler used by composable orders.
    ExtensibleFallbackHandler,
    /// `CurrentBlockTimestampFactory` — handler factory for current-block timestamp conditions.
    CurrentBlockTimestampFactory,
    /// `TwapHandler` — time-weighted average price conditional-order handler.
    TwapHandler,
    /// `GoodAfterTimeHandler` — good-after-time conditional-order handler.
    GoodAfterTimeHandler,
    /// `StopLossHandler` — stop-loss conditional-order handler.
    StopLossHandler,
    /// `TradeAboveThresholdHandler` — threshold conditional-order handler.
    TradeAboveThresholdHandler,
    /// `PerpetualStableSwapHandler` — perpetual stable-swap conditional-order handler.
    PerpetualStableSwapHandler,
    /// `COWShed` implementation contract.
    CowShedImplementation,
    /// `COWShedFactory` deployment contract.
    CowShedFactory,
    /// `COWShedForComposableCoW` deployment contract.
    CowShedForComposableCow,
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
            Self::ComposableCow => "ComposableCow",
            Self::ExtensibleFallbackHandler => "ExtensibleFallbackHandler",
            Self::CurrentBlockTimestampFactory => "CurrentBlockTimestampFactory",
            Self::TwapHandler => "TwapHandler",
            Self::GoodAfterTimeHandler => "GoodAfterTimeHandler",
            Self::StopLossHandler => "StopLossHandler",
            Self::TradeAboveThresholdHandler => "TradeAboveThresholdHandler",
            Self::PerpetualStableSwapHandler => "PerpetualStableSwapHandler",
            Self::CowShedImplementation => "CowShedImplementation",
            Self::CowShedFactory => "CowShedFactory",
            Self::CowShedForComposableCow => "CowShedForComposableCow",
        }
    }

    /// Returns `true` when the contract is keyed independently from prod/staging.
    #[must_use]
    pub const fn is_environment_agnostic(self) -> bool {
        matches!(
            self,
            Self::ComposableCow
                | Self::ExtensibleFallbackHandler
                | Self::CurrentBlockTimestampFactory
                | Self::TwapHandler
                | Self::GoodAfterTimeHandler
                | Self::StopLossHandler
                | Self::TradeAboveThresholdHandler
                | Self::PerpetualStableSwapHandler
                | Self::CowShedImplementation
                | Self::CowShedFactory
                | Self::CowShedForComposableCow
        )
    }
}

/// Capability contracts that are `EnvironmentAgnostic` per schema v2.
///
/// The slice and the `is_environment_agnostic` predicate are the crate's
/// runtime source of truth for which contract identifiers may carry
/// `env = environment_agnostic`; the predicate consults the same set via a
/// `matches!` expression, and the unit test below proves the two stay in
/// lockstep. An in-file compile-time assertion pins the slice length at 11.
/// The `build.rs` manifest validator enforces the same environment-agnostic
/// scope on `registry.toml` rows through its own independent check.
pub const ENVIRONMENT_AGNOSTIC_CONTRACTS: &[ContractId] = &[
    ContractId::ComposableCow,
    ContractId::ExtensibleFallbackHandler,
    ContractId::CurrentBlockTimestampFactory,
    ContractId::TwapHandler,
    ContractId::GoodAfterTimeHandler,
    ContractId::StopLossHandler,
    ContractId::TradeAboveThresholdHandler,
    ContractId::PerpetualStableSwapHandler,
    ContractId::CowShedImplementation,
    ContractId::CowShedFactory,
    ContractId::CowShedForComposableCow,
];

impl std::fmt::Display for ContractId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Compile-time assertion: the public `ENVIRONMENT_AGNOSTIC_CONTRACTS`
/// slice carries exactly 11 entries (the schema-v2 capability contract
/// set covering the composable conditional-order framework and the COW
/// Shed account-abstraction proxy). The runtime `contract_id_variants`
/// test reinforces the invariant; this const-evaluated path catches
/// drift even before the test runs.
const _: () = {
    assert!(
        ENVIRONMENT_AGNOSTIC_CONTRACTS.len() == 11,
        "ENVIRONMENT_AGNOSTIC_CONTRACTS must list exactly 11 capability ContractId variants"
    );
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_agnostic_slice_matches_is_environment_agnostic_predicate() {
        for variant in ENVIRONMENT_AGNOSTIC_CONTRACTS {
            assert!(
                variant.is_environment_agnostic(),
                "{variant:?} is in ENVIRONMENT_AGNOSTIC_CONTRACTS but is_environment_agnostic() returned false",
            );
        }
        for variant in [
            ContractId::Settlement,
            ContractId::VaultRelayer,
            ContractId::EthFlow,
        ] {
            assert!(
                !variant.is_environment_agnostic(),
                "{variant:?} is a GPv2 contract but is_environment_agnostic() returned true",
            );
        }
    }
}
