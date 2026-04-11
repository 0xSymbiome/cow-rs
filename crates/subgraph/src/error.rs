use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubgraphGraphQlError {
    pub message: String,
    #[serde(default)]
    pub locations: Vec<SubgraphGraphQlErrorLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubgraphGraphQlErrorLocation {
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubgraphRequestErrorContext {
    pub api: String,
    pub document: String,
    pub operation_name: Option<String>,
    pub variables: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SubgraphError {
    #[error("Unsupported Network. The subgraph API is not available in the Network {chain_id}")]
    UnsupportedNetwork { chain_id: u64 },
    #[error("No totals found")]
    NoTotalsFound,
    #[error("subgraph transport error for {}: {details}", context.api)]
    Transport {
        context: Box<SubgraphRequestErrorContext>,
        details: String,
    },
    #[error("subgraph http status error for {}: {status}", context.api)]
    HttpStatus {
        context: Box<SubgraphRequestErrorContext>,
        status: u16,
        body: String,
    },
    #[error("subgraph serialization error for {}: {details}", context.api)]
    Serialization {
        context: Box<SubgraphRequestErrorContext>,
        body: String,
        details: String,
    },
    #[error("subgraph graphql error response for {}", context.api)]
    GraphQl {
        context: Box<SubgraphRequestErrorContext>,
        errors: Vec<SubgraphGraphQlError>,
    },
    #[error("subgraph response missing data for {}", context.api)]
    MissingData {
        context: Box<SubgraphRequestErrorContext>,
    },
}
