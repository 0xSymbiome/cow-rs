use serde::{Deserialize, Serialize};
use serde_json::Value;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

/// Explicit raw GraphQL query input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct SubgraphQueryInput {
    /// Raw GraphQL document.
    pub query: String,
    /// Optional GraphQL variables.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<Value>,
    /// Optional operation name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_name: Option<String>,
}
