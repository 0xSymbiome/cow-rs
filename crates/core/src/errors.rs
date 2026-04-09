use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValidationError {
    #[error("{field} must not be empty")]
    EmptyField { field: &'static str },
    #[error("{field} must be 0x-prefixed hexadecimal data")]
    InvalidHexPrefix { field: &'static str },
    #[error("{field} must contain exactly {expected} hex characters")]
    InvalidHexLength {
        field: &'static str,
        expected: usize,
    },
    #[error("{field} contains non-hex characters")]
    InvalidHexCharacters { field: &'static str },
    #[error("unsupported chain id {chain_id}")]
    UnsupportedChain { chain_id: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CoreError {
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),
    #[error(
        "missing API base URL for chain id {chain_id} in {env} environment (partner_api={partner_api})"
    )]
    MissingBaseUrl {
        chain_id: u64,
        env: String,
        partner_api: bool,
    },
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("transport contract violation: {0}")]
    TransportContract(String),
}

pub type CowRsError = CoreError;
