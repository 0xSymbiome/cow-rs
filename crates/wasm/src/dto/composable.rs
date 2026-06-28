//! Composable (TWAP conditional-order) boundary DTOs.

#[cfg(feature = "composable")]
use serde::{Deserialize, Serialize};

/// Inputs for a TWAP authorization transaction.
///
/// `sellAmount` / `buyAmount` are totals across all parts, as decimal atom
/// strings; the handler receives the per-part division. The owner is the
/// submitting Safe.
#[cfg(feature = "composable")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "composable"),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "composable"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct TwapCreateParams {
    /// Token sold across the whole TWAP (`0x` address).
    pub sell_token: String,
    /// Token bought across the whole TWAP (`0x` address).
    pub buy_token: String,
    /// Total sell amount across all parts, in atoms.
    pub sell_amount: String,
    /// Total minimum buy amount across all parts, in atoms.
    pub buy_amount: String,
    /// Number of parts; must be greater than one.
    pub number_of_parts: u32,
    /// Seconds between parts; non-zero and at most 365 days.
    pub time_between_parts: u32,
    /// 32-byte salt (`0x`) distinguishing this order from an identical one.
    pub salt: String,
    /// App-data hash (`0x`) applied to every part.
    pub app_data: String,
    /// Receiver of the bought token; omit to pay the owner.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiver: Option<String>,
    /// Explicit start epoch (unix seconds); omit to start at mining time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_epoch: Option<u32>,
    /// Per-part validity window in seconds; omit for the whole interval.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit_duration: Option<u32>,
}

/// A TWAP authorization transaction and its conditional-order id.
#[cfg(feature = "composable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "composable"),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "composable"),
    tsify(into_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct TwapCreateResult {
    /// The `ComposableCoW` transaction to submit from the owner Safe.
    pub transaction: cow_sdk_core::TransactionRequest,
    /// The conditional-order id, for tracking the discrete parts (`0x`).
    pub order_id: String,
}
