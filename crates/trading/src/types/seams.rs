use async_trait::async_trait;

use cow_sdk_core::{OrderDigest, OrderUid};

use super::{SlippageToleranceRequest, SlippageToleranceResponse};
use crate::TradingError;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
/// External slippage-suggestion provider used by advanced swap settings.
///
/// # Implementing
///
/// Native implementors write one plain attribute, using the macro re-exported
/// from this crate, so no direct `async-trait` dependency is needed:
///
/// ```
/// use cow_sdk_trading::{
///     SlippageSuggester, SlippageToleranceRequest, SlippageToleranceResponse, TradingError,
///     async_trait,
/// };
///
/// struct FixedSlippage;
///
/// #[async_trait]
/// impl SlippageSuggester for FixedSlippage {
///     async fn slippage_suggestion(
///         &self,
///         _request: SlippageToleranceRequest,
///     ) -> Result<SlippageToleranceResponse, TradingError> {
///         Ok(SlippageToleranceResponse::new().with_slippage_bps(100))
///     }
/// }
/// ```
///
/// Libraries that compile for both native and wasm32 targets replace the
/// plain attribute with the target-gated pair, because browser futures are
/// not `Send` — the same shape alloy's `Signer` trait uses:
///
/// ```text
/// #[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
/// #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
/// ```
pub trait SlippageSuggester: Send + Sync {
    /// Returns an optional slippage suggestion for the supplied request.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when the provider cannot compute a suggestion.
    async fn slippage_suggestion(
        &self,
        request: SlippageToleranceRequest,
    ) -> Result<SlippageToleranceResponse, TradingError>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
/// External existence checker used during `EthFlow` unique-order-id generation.
///
/// # Implementing
///
/// Native implementors write one plain attribute, using the macro re-exported
/// from this crate, so no direct `async-trait` dependency is needed:
///
/// ```
/// use cow_sdk_core::{OrderDigest, OrderUid};
/// use cow_sdk_trading::{EthFlowOrderExistsChecker, TradingError, async_trait};
///
/// struct NeverCollides;
///
/// #[async_trait]
/// impl EthFlowOrderExistsChecker for NeverCollides {
///     async fn order_exists(
///         &self,
///         _order_id: &OrderUid,
///         _order_digest: &OrderDigest,
///     ) -> Result<bool, TradingError> {
///         Ok(false)
///     }
/// }
/// ```
///
/// Libraries that compile for both native and wasm32 targets replace the
/// plain attribute with the target-gated pair, because browser futures are
/// not `Send` — the same shape alloy's `Signer` trait uses:
///
/// ```text
/// #[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
/// #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
/// ```
pub trait EthFlowOrderExistsChecker: Send + Sync {
    /// Returns `true` when the generated `EthFlow` order id already exists.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when the existence check fails.
    async fn order_exists(
        &self,
        order_id: &OrderUid,
        order_digest: &OrderDigest,
    ) -> Result<bool, TradingError>;
}
