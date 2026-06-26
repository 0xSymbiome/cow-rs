use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Quote-quality mode accepted by the orderbook quote endpoint.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PriceQuality {
    /// Fastest available estimate, intended for previews. The orderbook
    /// returns an already-elapsed expiration for this mode, so a `Fast` quote
    /// is not suitable for submission.
    Fast,
    /// Best available quote within the quoting window.
    ///
    /// This is the default and the mode used for a quote that will be signed
    /// and submitted: the orderbook returns a quote identifier for order
    /// placement alongside an optimal price estimate.
    #[default]
    Optimal,
    /// `Optimal` plus on-chain simulation of the quoted amounts. The response
    /// `verified` flag reports whether that simulation succeeded.
    Verified,
}

/// Signature scheme encoded in orderbook wire DTOs.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "ts-bindings"),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "ts-bindings"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "lowercase")]
pub enum SigningScheme {
    /// EIP-712 typed-data signature.
    #[default]
    Eip712,
    /// `eth_sign` / personal-sign style signature.
    EthSign,
    /// EIP-1271 smart-account signature.
    Eip1271,
    /// Pre-signed order recorded on-chain.
    PreSign,
}

/// ECDSA signing schemes accepted by order-cancellation payloads.
///
/// Closed internally so SDK matches remain exhaustive; open externally so
/// future signing schemes land additively.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EcdsaSigningScheme {
    /// EIP-712 typed-data signature.
    #[default]
    Eip712,
    /// `eth_sign` / personal-sign style signature.
    EthSign,
}

/// Error returned when an orderbook [`SigningScheme`] cannot be narrowed to
/// the cancellation-only [`EcdsaSigningScheme`] subset.
///
/// `PreSign` and `Eip1271` are accepted by the order-creation surface but are
/// not valid signing schemes for ECDSA cancellation payloads, so the typed
/// fallible bridge surfaces the rejection instead of silently dropping data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("signing scheme {0:?} is not an ECDSA cancellation scheme")]
pub struct SigningSchemeNotEcdsa(pub SigningScheme);

impl From<EcdsaSigningScheme> for SigningScheme {
    fn from(scheme: EcdsaSigningScheme) -> Self {
        match scheme {
            EcdsaSigningScheme::Eip712 => Self::Eip712,
            EcdsaSigningScheme::EthSign => Self::EthSign,
        }
    }
}

impl TryFrom<SigningScheme> for EcdsaSigningScheme {
    type Error = SigningSchemeNotEcdsa;

    fn try_from(scheme: SigningScheme) -> Result<Self, Self::Error> {
        match scheme {
            SigningScheme::Eip712 => Ok(Self::Eip712),
            SigningScheme::EthSign => Ok(Self::EthSign),
            other @ (SigningScheme::Eip1271 | SigningScheme::PreSign) => {
                Err(SigningSchemeNotEcdsa(other))
            }
        }
    }
}

/// Order class surfaced by the orderbook API.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "ts-bindings"),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "ts-bindings"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "lowercase")]
pub enum OrderClass {
    /// Market order.
    #[default]
    Market,
    /// Limit order.
    Limit,
    /// Liquidity order.
    Liquidity,
}

/// Order lifecycle status returned by the orderbook API.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "ts-bindings"),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "ts-bindings"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub enum OrderStatus {
    /// Waiting for a pre-signature to become valid.
    PresignaturePending,
    /// Open and fillable.
    #[default]
    Open,
    /// Fully or terminally fulfilled.
    Fulfilled,
    /// Cancelled by the owner or protocol.
    Cancelled,
    /// Expired because `valid_to` has passed.
    Expired,
}

impl OrderStatus {
    /// Returns whether the order has reached a terminal lifecycle state, where
    /// no further fills or transitions are possible (`Fulfilled`, `Cancelled`,
    /// or `Expired`).
    ///
    /// Prefer this predicate over a hand-rolled match. `OrderStatus` is
    /// `#[non_exhaustive]` because the orderbook owns the wire tags and may add
    /// variants, so an exhaustive caller-side match risks silently
    /// misclassifying a future terminal status; this accessor is updated in the
    /// same crate that adds the variant.
    #[inline]
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Fulfilled | Self::Cancelled | Self::Expired)
    }

    /// Returns whether the order is still live — open and fillable, or awaiting a
    /// pre-signature (`Open` or `PresignaturePending`) — and therefore not yet
    /// terminal.
    ///
    /// The complement of [`OrderStatus::is_terminal`] over the variants known
    /// today. Both predicates partition the current surface, but because the
    /// enum is `#[non_exhaustive]` callers should treat them as the SDK-owned
    /// classification rather than re-deriving either side.
    #[inline]
    #[must_use]
    pub const fn is_open(self) -> bool {
        matches!(self, Self::Open | Self::PresignaturePending)
    }

    /// Returns whether the order settled with a fill (`Fulfilled`).
    ///
    /// The precise predicate within the terminal set: a terminal order may have
    /// filled, been cancelled, or expired, and a fill is the outcome a consumer
    /// usually acts on. Prefer this over a caller-side `Fulfilled` match, which
    /// the `#[non_exhaustive]` enum cannot keep exhaustive; like the other
    /// predicates it is owned by the crate that owns the wire tags.
    #[inline]
    #[must_use]
    pub const fn is_fulfilled(self) -> bool {
        matches!(self, Self::Fulfilled)
    }
}
