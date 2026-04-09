use cow_sdk_core::CoreError;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContractsError {
    #[error("core validation error: {0}")]
    Core(#[from] CoreError),
    #[error("unsupported chain id: {0}")]
    UnsupportedChain(u64),
    #[error("invalid order UID length: expected 56 bytes, got {actual}")]
    InvalidOrderUidLength { actual: usize },
    #[error("invalid numeric value for {field}: {value}")]
    InvalidNumeric { field: &'static str, value: String },
    #[error("numeric value for {field} exceeds uint256: {value}")]
    NumericOverflow { field: &'static str, value: String },
    #[error("invalid encoded flag bits: {0:#010b}")]
    InvalidFlags(u8),
    #[error("unsupported signing scheme value: {0}")]
    UnsupportedSigningScheme(u8),
    #[error("invalid EIP-1271 signature payload")]
    InvalidEip1271SignatureData,
    #[error("missing clearing price for token {0}")]
    MissingClearingPrice(String),
    #[error("missing executed amount for partially fillable trade")]
    MissingExecutedAmount,
    #[error("trade not encoded")]
    MissingTrade,
    #[error("receiver cannot be address(0)")]
    ZeroReceiver,
    #[error("provider error: {0}")]
    Provider(String),
    #[error("ABI encoding error: {0}")]
    Abi(String),
    #[error("decode error: {0}")]
    Decode(String),
    #[error("serialization error: {0}")]
    Serialization(String),
}
