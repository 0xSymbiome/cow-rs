#[cfg(feature = "trading")]
use serde::{Deserialize, Serialize};

#[cfg(all(target_arch = "wasm32", target_os = "unknown", feature = "trading"))]
use cow_sdk_core::TransactionRequest;
#[cfg(all(target_arch = "wasm32", target_os = "unknown", feature = "trading"))]
use cow_sdk_trading::{
    OrderPlacement as NativeOrderPlacement, SafeActivation as NativeSafeActivation,
};

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

/// On-chain activation a smart-contract wallet runs to authorize a posted
/// pre-sign order (ADR 0073).
///
/// Carries the ordered approve-then-set-pre-signature pair as transaction
/// requests for one smart-account batch. The first call grants the vault relayer
/// the sell-token allowance the order needs at fill time; the second flips the
/// settlement `setPreSignature` flag that makes the order fillable. The bundle is
/// transport-neutral: a single-owner Safe sends the calls directly, while a
/// higher-threshold Safe proposes them to its transaction service for the owners
/// to co-sign.
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
pub struct SafeActivation {
    /// Ordered `[approve, setPreSignature]` calls for one smart-account batch.
    pub calls: Vec<cow_sdk_core::TransactionRequest>,
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown", feature = "trading"))]
impl SafeActivation {
    /// Projects the native [`SafeActivation`](cow_sdk_trading::SafeActivation),
    /// converting each gas-free `UnsignedTransaction` leg into its
    /// [`TransactionRequest`] wire shape.
    pub(crate) fn from_native(activation: NativeSafeActivation) -> Self {
        Self {
            calls: activation
                .calls
                .into_iter()
                .map(TransactionRequest::from)
                .collect(),
        }
    }
}

/// Typed placement result returned by `placeSwap` and `placeLimit` (ADR 0073).
///
/// The authorization mode statically selects the arm: an ECDSA or EIP-1271 order
/// is `live` at post, while a pre-sign order is `pendingActivation` and carries
/// the [`SafeActivation`] the owner must send or propose from the smart account.
/// The `status` discriminator distinguishes the variants, and the `orderId` is
/// reachable only by matching the arm, so the on-chain obligation of a pre-sign
/// order cannot be dropped.
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
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "this wasm-boundary projection derives only PartialEq for test equality and omits Eq to match the boundary DTO derive set"
)]
pub enum OrderPlacement {
    /// The order is live at post — produced by an ECDSA or EIP-1271
    /// authorization.
    Live {
        /// Final order UID.
        order_id: String,
    },
    /// The order is posted but not yet authorized on-chain — produced by a
    /// pre-sign authorization. The owner must send or propose `activation` from
    /// the smart account to make the order fillable.
    PendingActivation {
        /// Final order UID.
        order_id: String,
        /// On-chain approve-then-set-pre-signature bundle.
        activation: SafeActivation,
    },
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown", feature = "trading"))]
impl OrderPlacement {
    /// Projects the native [`OrderPlacement`](cow_sdk_trading::OrderPlacement)
    /// sum, rendering the order UID as a `0x`-prefixed hex string and the
    /// pre-sign activation as its boundary projection.
    pub(crate) fn from_native(placement: NativeOrderPlacement) -> Self {
        match placement {
            NativeOrderPlacement::Live { order_uid } => Self::Live {
                order_id: order_uid.to_hex_string(),
            },
            NativeOrderPlacement::PendingActivation {
                order_uid,
                activation,
            } => Self::PendingActivation {
                order_id: order_uid.to_hex_string(),
                activation: SafeActivation::from_native(activation),
            },
            // The native sum is `#[non_exhaustive]`; a future arm this wasm
            // build does not model is rendered as a live placement carrying only
            // the order UID rather than silently dropping it.
            other => Self::Live {
                order_id: other.order_uid().to_hex_string(),
            },
        }
    }
}
