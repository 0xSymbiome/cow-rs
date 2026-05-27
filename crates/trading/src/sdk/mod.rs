//! High-level `TradingSdk` facade and builder.

use crate::{PartialTraderParameters, TradingSdkOptions};
mod allowance;
mod builder;
mod cancel;
mod helper_only;
mod helpers;
mod post;
mod presign;
mod query;
mod quote;

pub use self::builder::TradingSdkBuilder;
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
pub struct TradingSdk {
    trader_defaults: PartialTraderParameters,
    options: TradingSdkOptions,
}

/// Helper-only trading facade for chain-bound helper workflows.
///
/// `HelperOnlySdk` intentionally exposes only allowance, approval, pre-sign,
/// and on-chain cancellation helpers. Quote, post, order lookup, and off-chain
/// cancellation methods exist only on [`TradingSdk`].
#[derive(Debug, Clone)]
pub struct HelperOnlySdk {
    trader_defaults: PartialTraderParameters,
    options: TradingSdkOptions,
}

impl TradingSdk {
    /// Returns a new [`TradingSdkBuilder`] in the `<ChainIdUnset, AppCodeUnset>` typestate.
    #[must_use]
    pub fn builder() -> TradingSdkBuilder<ChainIdUnset, AppCodeUnset> {
        TradingSdkBuilder::new()
    }

    /// Returns the stored trader defaults.
    #[must_use]
    pub const fn trader_defaults(&self) -> &PartialTraderParameters {
        &self.trader_defaults
    }

    /// Returns the stored SDK options.
    #[must_use]
    pub const fn options(&self) -> &TradingSdkOptions {
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
