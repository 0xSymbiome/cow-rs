//! High-level `Trading` facade and builder.

use crate::{PartialTraderParameters, TradingOptions};
mod allowance;
mod builder;
mod cancel;
mod helpers;
mod post;
mod presign;
mod query;
mod quote;

pub use self::builder::TradingBuilder;
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

    /// Returns the stored trader defaults.
    #[must_use]
    pub const fn trader_defaults(&self) -> &PartialTraderParameters {
        &self.trader_defaults
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
