use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SubgraphError {
    #[error("Unsupported Network. The subgraph API is not available in the Network {chain_id}")]
    UnsupportedNetwork { chain_id: u64 },
    #[error("No totals found")]
    NoTotalsFound,
    #[error("transport error: {details}")]
    Transport { details: String },
    #[error("serialization error: {details}")]
    Serialization { details: String },
    #[error(
        "Error running query: {query}. Variables: {variables}. API: {api}. Inner Error: {inner_error}"
    )]
    QueryFailed {
        query: String,
        variables: String,
        api: String,
        inner_error: String,
    },
}
