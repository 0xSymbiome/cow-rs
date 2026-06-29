#[cfg(all(target_arch = "wasm32", feature = "signing"))]
use cow_sdk_core::TypedDataPayload;
use cow_sdk_core::{OrderData, TypedDataEnvelope};
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[cfg(all(target_arch = "wasm32", feature = "signing"))]
use wasm_bindgen::{JsValue, prelude::*};

#[cfg(all(target_arch = "wasm32", feature = "signing"))]
use super::to_js_value;
#[cfg(all(target_arch = "wasm32", feature = "signing"))]
use crate::exports::errors::WasmError;

/// Projects the signer-facing typed-data payload into the boundary envelope.
///
/// The payload carries its message as canonical JSON; the boundary envelope
/// carries it as a parsed value so JavaScript wallets receive a structured
/// object. The domain and type map are already the boundary types, so only the
/// message is reshaped.
#[cfg(all(target_arch = "wasm32", feature = "signing"))]
pub(crate) fn payload_to_envelope(
    payload: &TypedDataPayload,
) -> Result<TypedDataEnvelope<Value>, WasmError> {
    let message: Value = serde_json::from_str(payload.message_json())?;
    Ok(payload.clone().with_message(message))
}

/// Serializes a typed-data envelope into the EIP-1193 `eth_signTypedData_v4`
/// object passed to a wallet callback.
#[cfg(all(target_arch = "wasm32", feature = "signing"))]
pub(crate) fn envelope_callback_value(
    envelope: &TypedDataEnvelope<Value>,
) -> Result<JsValue, JsValue> {
    let value = serde_json::json!({
        "domain": envelope.domain,
        "types": envelope.types,
        "primaryType": envelope.primary_type,
        "message": envelope.message,
    });
    to_js_value(&value)
}

/// Signed order returned by wallet callback exports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "the `typed_data` field carries a serde_json::Value message, which is not Eq, so the struct cannot derive Eq"
)]
pub struct SignedOrder {
    /// Compact order UID.
    #[serde(rename = "orderUid")]
    pub order_uid: String,
    /// Signature payload submitted to the orderbook.
    pub signature: String,
    /// Signing scheme.
    pub signing_scheme: String,
    /// Effective owner submitted as `from`.
    pub from: String,
    /// Underlying order digest.
    pub order_digest: String,
    /// Typed-data envelope used for signing.
    pub typed_data: TypedDataEnvelope<Value>,
    /// Optional quote id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<i64>,
}

/// Custom EIP-1271 callback request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "the `typed_data` field carries a serde_json::Value message, which is not Eq, so the struct cannot derive Eq"
)]
pub struct CowEip1271SignRequest {
    /// Unsigned order being signed.
    pub order: OrderData,
    /// Typed-data envelope.
    pub typed_data: TypedDataEnvelope<Value>,
    /// Owner or smart-account address.
    pub owner: String,
    /// Numeric chain id.
    pub chain_id: u32,
}

/// Signed order-cancellation DTO.
#[cfg(any(feature = "cancellation", feature = "orderbook"))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    all(
        target_arch = "wasm32",
        target_os = "unknown",
        any(feature = "cancellation", feature = "orderbook")
    ),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(
        target_arch = "wasm32",
        target_os = "unknown",
        any(feature = "cancellation", feature = "orderbook")
    ),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct SignedCancellations {
    /// Order UIDs to cancel.
    #[serde(rename = "orderUids")]
    pub order_uids: Vec<String>,
    /// Cancellation signature.
    pub signature: String,
    /// ECDSA signing scheme.
    pub signing_scheme: String,
}
