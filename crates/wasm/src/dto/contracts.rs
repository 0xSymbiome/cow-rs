#[cfg(feature = "trading")]
use serde::{Deserialize, Serialize};

/// Native-currency sell transaction bundle.
#[cfg(feature = "trading")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "trading"),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "trading"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "this wasm-boundary projection derives only PartialEq for test equality and omits Eq to match the boundary DTO derive set"
)]
pub struct BuiltSellNativeCurrencyTx {
    /// Deterministic order UID.
    #[serde(rename = "orderUid")]
    pub order_uid: String,
    /// Transaction request to submit.
    pub transaction: cow_sdk_core::TransactionRequest,
    /// Unsigned order encoded by the transaction.
    pub order_to_sign: cow_sdk_core::OrderData,
    /// Effective order owner.
    pub from: String,
}
