use bytes::Bytes;
use serde::{Deserialize, Serialize};

use cow_sdk_core::{Address, Amount};

/// Fully normalized settlement interaction.
///
/// The calldata payload is stored as [`bytes::Bytes`] so encoder pipelines that
/// fan the same payload across multiple settlement candidates share a single
/// backing allocation through reference-counted clones. The JSON wire form
/// remains the `0x`-prefixed hexadecimal string accepted by downstream
/// consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Interaction {
    /// Target contract address.
    pub target: Address,
    /// Native value transferred with the call.
    pub value: Amount,
    /// Encoded calldata.
    #[serde(with = "crate::bytes_serde::hex_bytes")]
    pub call_data: Bytes,
}

/// Partially specified interaction accepted by higher-level encoders.
///
/// Optional calldata is carried as [`Option`] over [`bytes::Bytes`] so callers
/// can build interaction proposals without materializing empty-buffer
/// placeholders and without losing the cheap-clone property during encoding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractionLike {
    /// Target contract address.
    pub target: Address,
    /// Optional native value. Missing values normalize to zero.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Amount>,
    /// Optional calldata. Missing values normalize to an empty buffer.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "crate::bytes_serde::option_hex_bytes"
    )]
    pub call_data: Option<Bytes>,
}

/// Normalizes an interaction by filling default value and calldata fields.
#[must_use]
pub fn normalize_interaction(interaction: &InteractionLike) -> Interaction {
    Interaction {
        target: interaction.target.clone(),
        value: interaction.value.clone().unwrap_or_else(Amount::zero),
        call_data: interaction.call_data.clone().unwrap_or_default(),
    }
}

/// Normalizes a slice of interaction-like values.
#[must_use]
pub fn normalize_interactions(interactions: &[InteractionLike]) -> Vec<Interaction> {
    interactions.iter().map(normalize_interaction).collect()
}
