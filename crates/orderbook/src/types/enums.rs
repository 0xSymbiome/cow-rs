use serde::{Deserialize, Serialize};
use thiserror::Error;

use cow_sdk_contracts::SigningScheme as ContractsSigningScheme;

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

impl From<ContractsSigningScheme> for SigningScheme {
    /// Bridges a [`ContractsSigningScheme`] protocol tag onto the orderbook
    /// wire-form enum.
    ///
    /// # Panics
    ///
    /// Panics only if a future [`ContractsSigningScheme`] variant is added
    /// upstream without a corresponding orderbook variant landing in this
    /// match. The non-exhaustive wildcard arm exists solely to satisfy the
    /// compiler across crate boundaries; the variant-by-variant bridge parity
    /// test prevents drift, so any new variant must land here in the same
    /// patch.
    fn from(scheme: ContractsSigningScheme) -> Self {
        match scheme {
            ContractsSigningScheme::Eip712 => Self::Eip712,
            ContractsSigningScheme::EthSign => Self::EthSign,
            ContractsSigningScheme::Eip1271 => Self::Eip1271,
            ContractsSigningScheme::PreSign => Self::PreSign,
            // SAFETY: cow_sdk_contracts::SigningScheme and cow_sdk_orderbook::SigningScheme
            // share the four variants Eip712, EthSign, Eip1271, PreSign per ADR 0052; the
            // variant-by-variant parity test in tests/signing_scheme_bridge_contract.rs
            // prevents drift, and any new variant added upstream must land here in the
            // same patch.
            _ => unreachable!(
                "cow_sdk_contracts::SigningScheme variant added without updating the orderbook bridge"
            ),
        }
    }
}

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

impl From<SigningScheme> for ContractsSigningScheme {
    fn from(scheme: SigningScheme) -> Self {
        match scheme {
            SigningScheme::Eip712 => Self::Eip712,
            SigningScheme::EthSign => Self::EthSign,
            SigningScheme::Eip1271 => Self::Eip1271,
            SigningScheme::PreSign => Self::PreSign,
        }
    }
}

impl From<EcdsaSigningScheme> for ContractsSigningScheme {
    fn from(scheme: EcdsaSigningScheme) -> Self {
        match scheme {
            EcdsaSigningScheme::Eip712 => Self::Eip712,
            EcdsaSigningScheme::EthSign => Self::EthSign,
        }
    }
}

/// Order class surfaced by the orderbook API.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
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
}
