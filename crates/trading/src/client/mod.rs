//! The stateful, high-level `Trading` client, its construction builder, and the
//! fluent swap lifecycle.

use cow_sdk_core::{AddressPerChain, AppCode, CowEnv, SupportedChainId};

use crate::{PartialTraderParameters, TradingOptions};
mod builder;
mod helpers;
mod methods;
mod swap;

pub use self::builder::TradingBuilder;
pub use self::swap::{QuotedSwap, Set, SwapBuilder, Unset};
/// Typestate marker for a builder that has not yet been given a chain id.
#[derive(Debug, Clone, Copy)]
pub struct ChainIdUnset(());

/// Typestate marker for a builder that has been given a chain id.
#[derive(Debug, Clone, Copy)]
pub struct ChainIdSet(());

/// Typestate marker for a builder that has not yet been given an `appCode`.
#[derive(Debug, Clone, Copy)]
pub struct AppCodeUnset(());

/// Typestate marker for a builder that has been given an `appCode`.
#[derive(Debug, Clone, Copy)]
pub struct AppCodeSet(());

/// High-level trading facade that stores trader defaults plus optional injected services.
#[derive(Debug, Clone)]
pub struct Trading {
    trader_defaults: PartialTraderParameters,
    options: TradingOptions,
}

impl Trading {
    /// Returns a new [`TradingBuilder`] in the `<ChainIdUnset, AppCodeUnset>` typestate.
    #[must_use]
    pub fn builder() -> TradingBuilder<ChainIdUnset, AppCodeUnset> {
        TradingBuilder::new()
    }

    /// Returns the default chain id supplied at construction, if any.
    #[must_use]
    pub const fn chain_id(&self) -> Option<SupportedChainId> {
        self.trader_defaults.chain_id
    }

    /// Returns the default app code supplied at construction, if any.
    #[must_use]
    pub const fn app_code(&self) -> Option<&AppCode> {
        self.trader_defaults.app_code.as_ref()
    }

    /// Returns the default environment supplied at construction, if any.
    #[must_use]
    pub const fn env(&self) -> Option<CowEnv> {
        self.trader_defaults.env
    }

    /// Returns the default settlement-contract overrides supplied at construction, if any.
    #[must_use]
    pub const fn settlement_contract_override(&self) -> Option<&AddressPerChain> {
        self.trader_defaults.settlement_contract_override.as_ref()
    }

    /// Returns the default `EthFlow`-contract overrides supplied at construction, if any.
    #[must_use]
    pub const fn eth_flow_contract_override(&self) -> Option<&AddressPerChain> {
        self.trader_defaults.eth_flow_contract_override.as_ref()
    }

    /// Returns the stored SDK options.
    #[must_use]
    pub const fn options(&self) -> &TradingOptions {
        &self.options
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typestate_markers_are_sealed_against_external_construction() {
        // These constructors are visible only inside this module because the
        // tuple field is private; external callers cannot write `Marker(())`.
        let _ = ChainIdUnset(());
        let _ = ChainIdSet(());
        let _ = AppCodeUnset(());
        let _ = AppCodeSet(());
    }
}
