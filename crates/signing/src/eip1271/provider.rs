use async_trait::async_trait;
use cow_sdk_core::OrderData;

use super::Eip1271SignatureError;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
/// Custom EIP-1271 signature provider used during order submission.
///
/// # Implementing
///
/// Native implementors write one plain attribute, using the macro re-exported
/// from this crate, so no direct `async-trait` dependency is needed:
///
/// ```
/// use cow_sdk_core::OrderData;
/// use cow_sdk_signing::{Eip1271SignatureError, Eip1271Signer, async_trait};
///
/// struct SmartAccountSigner;
///
/// #[async_trait]
/// impl Eip1271Signer for SmartAccountSigner {
///     async fn sign(&self, _order: &OrderData) -> Result<String, Eip1271SignatureError> {
///         Ok("0x7e57c0de".to_owned())
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
pub trait Eip1271Signer: Send + Sync {
    /// Produces an order signature payload for the provided unsigned order.
    ///
    /// # Errors
    ///
    /// Returns [`Eip1271SignatureError`] when signing fails.
    async fn sign(&self, order_to_sign: &OrderData) -> Result<String, Eip1271SignatureError>;
}
