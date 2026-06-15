//! The stateful, high-level `Trading` client, its construction builder, and the
//! fluent swap lifecycle.

use std::{fmt, sync::Arc};

use cow_sdk_core::{AddressPerChain, AppCode, CowEnv, SupportedChainId};

use crate::{OrderbookClient, PartialTraderParams};

/// Generates the optional setters shared by the fluent order-placement builders
/// ([`SwapBuilder`](swap::SwapBuilder) and [`LimitBuilder`](limit::LimitBuilder)),
/// mirroring `impl_common_trade_setters!` on the parameter structs. The required
/// token and amount setters and the async terminals stay builder-specific; only
/// the marker-preserving optional setters are shared, so a future common setter
/// lands on both builders at once.
macro_rules! impl_common_order_builder_setters {
    ($builder:ident <$life:lifetime, $($marker:ident),+>) => {
        impl<$life, $($marker),+> $builder<$life, $($marker),+> {
            /// Sets an explicit owner. When omitted, the signer address resolves
            /// the owner at the terminal.
            #[must_use]
            pub const fn owner(mut self, owner: Address) -> Self {
                self.owner = Some(owner);
                self
            }

            /// Sets an explicit receiver address.
            #[must_use]
            pub const fn receiver(mut self, receiver: Address) -> Self {
                self.receiver = Some(receiver);
                self
            }

            /// Sets a relative validity window in seconds.
            #[must_use]
            pub const fn valid_for(mut self, valid_for: u32) -> Self {
                self.valid_for = Some(valid_for);
                self
            }

            /// Sets an absolute expiry timestamp.
            #[must_use]
            pub const fn valid_to(mut self, valid_to: u32) -> Self {
                self.valid_to = Some(valid_to);
                self
            }

            /// Allows partial fills.
            #[must_use]
            pub const fn partially_fillable(mut self, partially_fillable: bool) -> Self {
                self.partially_fillable = partially_fillable;
                self
            }
        }
    };
}

mod builder;
mod helpers;
mod limit;
mod methods;
mod swap;

pub use self::builder::TradingBuilder;
pub use self::limit::LimitBuilder;
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

/// High-level trading facade that stores trader defaults plus an optional
/// injected orderbook client.
#[derive(Clone)]
pub struct Trading {
    trader_defaults: PartialTraderParams,
    orderbook: Option<Arc<dyn OrderbookClient>>,
}

impl fmt::Debug for Trading {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Trading")
            .field("trader_defaults", &self.trader_defaults)
            .field("orderbook", &self.orderbook.is_some())
            .finish()
    }
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
