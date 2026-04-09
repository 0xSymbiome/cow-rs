use serde::{Deserialize, Serialize};

use cow_sdk_core::Address;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Interaction {
    pub target: Address,
    pub value: String,
    pub call_data: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractionLike {
    pub target: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_data: Option<String>,
}

pub fn normalize_interaction(interaction: &InteractionLike) -> Interaction {
    Interaction {
        target: interaction.target.clone(),
        value: interaction.value.clone().unwrap_or_else(|| "0".to_owned()),
        call_data: interaction
            .call_data
            .clone()
            .unwrap_or_else(|| "0x".to_owned()),
    }
}

pub fn normalize_interactions(interactions: &[InteractionLike]) -> Vec<Interaction> {
    interactions.iter().map(normalize_interaction).collect()
}
