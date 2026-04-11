use serde::{Deserialize, Serialize};

use cow_sdk_core::{Address, Amount, HexData};

/// Fully normalized settlement interaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Interaction {
    /// Target contract address.
    pub target: Address,
    /// Native value transferred with the call.
    pub value: Amount,
    /// Encoded calldata.
    pub call_data: HexData,
}

/// Partially specified interaction accepted by higher-level encoders.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractionLike {
    /// Target contract address.
    pub target: Address,
    /// Optional native value. Missing values normalize to zero.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Amount>,
    /// Optional calldata. Missing values normalize to `0x`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_data: Option<HexData>,
}

/// Normalizes an interaction by filling default value and calldata fields.
#[must_use]
pub fn normalize_interaction(interaction: &InteractionLike) -> Interaction {
    Interaction {
        target: interaction.target.clone(),
        value: interaction.value.clone().unwrap_or_else(Amount::zero),
        call_data: interaction.call_data.clone().unwrap_or_else(HexData::empty),
    }
}

/// Normalizes a slice of interaction-like values.
#[must_use]
pub fn normalize_interactions(interactions: &[InteractionLike]) -> Vec<Interaction> {
    interactions.iter().map(normalize_interaction).collect()
}
