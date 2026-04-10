use serde::{Deserialize, Serialize};

use cow_sdk_core::{Address, Amount, HexData};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Interaction {
    pub target: Address,
    pub value: Amount,
    pub call_data: HexData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractionLike {
    pub target: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Amount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_data: Option<HexData>,
}

pub fn normalize_interaction(interaction: &InteractionLike) -> Interaction {
    Interaction {
        target: interaction.target.clone(),
        value: interaction.value.clone().unwrap_or_else(Amount::zero),
        call_data: interaction.call_data.clone().unwrap_or_else(HexData::empty),
    }
}

pub fn normalize_interactions(interactions: &[InteractionLike]) -> Vec<Interaction> {
    interactions.iter().map(normalize_interaction).collect()
}
