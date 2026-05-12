use async_trait::async_trait;

use cow_sdk_core::{OrderDigest, OrderUid, UnsignedOrder};

use super::{SlippageToleranceRequest, SlippageToleranceResponse};
use crate::TradingError;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
/// External slippage-suggestion provider used by advanced swap settings.
pub trait SlippageSuggestionProvider: Send + Sync {
    /// Returns an optional slippage suggestion for the supplied request.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when the provider cannot compute a suggestion.
    async fn get_slippage_suggestion(
        &self,
        request: SlippageToleranceRequest,
    ) -> Result<SlippageToleranceResponse, TradingError>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
/// External existence checker used during `EthFlow` unique-order-id generation.
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

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
/// Custom EIP-1271 signature provider used during order submission.
pub trait Eip1271SignatureProvider: Send + Sync {
    /// Produces an order signature payload for the provided unsigned order.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when signing fails.
    async fn sign(&self, order_to_sign: &UnsignedOrder) -> Result<String, TradingError>;
}
