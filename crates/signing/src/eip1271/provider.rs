use async_trait::async_trait;
use cow_sdk_core::OrderData;

use super::Eip1271SignatureError;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
/// Custom EIP-1271 signature provider used during order submission.
pub trait Eip1271SignatureProvider: Send + Sync {
    /// Produces an order signature payload for the provided unsigned order.
    ///
    /// # Errors
    ///
    /// Returns [`Eip1271SignatureError`] when signing fails.
    async fn sign(&self, order_to_sign: &OrderData) -> Result<String, Eip1271SignatureError>;
}
