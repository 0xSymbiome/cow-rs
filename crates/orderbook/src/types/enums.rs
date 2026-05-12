use serde::{Deserialize, Serialize};

/// Quote-quality mode accepted by the orderbook quote endpoint.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PriceQuality {
    /// Prefer the fastest available quote.
    Fast,
    /// Prefer the best available quote, allowing additional search.
    Optimal,
    /// Require the orderbook's verified quote mode.
    #[default]
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
