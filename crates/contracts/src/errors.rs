use cow_sdk_core::{Address, CoreError};
use thiserror::Error;

/// Errors returned by low-level `CoW` contract helpers.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContractsError {
    /// Core validation failed for an input value.
    #[error("core validation error: {0}")]
    Core(#[from] CoreError),
    /// A chain id is outside the supported `CoW` deployment set.
    #[error("unsupported chain id: {0}")]
    UnsupportedChain(u64),
    /// An order UID had the wrong encoded byte length.
    #[error("invalid order UID length: expected 56 bytes, got {actual}")]
    InvalidOrderUidLength {
        /// Actual decoded byte length.
        actual: usize,
    },
    /// A numeric value could not be parsed for ABI encoding.
    #[error("invalid numeric value for {field}: {value}")]
    InvalidNumeric {
        /// Field being encoded.
        field: &'static str,
        /// Original invalid value.
        value: String,
    },
    /// A numeric value exceeded `uint256` bounds.
    #[error("numeric value for {field} exceeds uint256: {value}")]
    NumericOverflow {
        /// Field being encoded.
        field: &'static str,
        /// Original overflowing value.
        value: String,
    },
    /// Encoded settlement or trade flags used unsupported bits.
    #[error("invalid encoded flag bits: {0:#010b}")]
    InvalidFlags(u8),
    /// A signing-scheme discriminator was not recognized.
    #[error("unsupported signing scheme value: {0}")]
    UnsupportedSigningScheme(u8),
    /// An encoded EIP-1271 signature payload was malformed.
    #[error("invalid EIP-1271 signature payload")]
    InvalidEip1271SignatureData,
    /// A verifier address did not have contract bytecode.
    #[error("EIP-1271 verifier {verifier} does not expose contract code")]
    UnsupportedEip1271Verifier {
        /// Verifier address that lacked contract code.
        verifier: Address,
    },
    /// The provider failed during an EIP-1271 operation.
    #[error("provider error during EIP-1271 {operation}: {message}")]
    Eip1271Provider {
        /// Operation being performed.
        operation: &'static str,
        /// Provider error message.
        message: String,
    },
    /// The EIP-1271 call response could not be decoded.
    #[error("malformed EIP-1271 response: {response}")]
    MalformedEip1271Response {
        /// Raw response that failed decoding.
        response: String,
    },
    /// The verifier returned an unexpected EIP-1271 magic value.
    #[error(
        "unexpected EIP-1271 magic value: expected 0x{}, got 0x{}",
        hex::encode(expected),
        hex::encode(actual)
    )]
    Eip1271MagicValueMismatch {
        /// Expected 4-byte magic value.
        expected: [u8; 4],
        /// Actual 4-byte magic value returned by the verifier.
        actual: [u8; 4],
    },
    /// A clearing price was missing for a token used in a settlement.
    #[error("missing clearing price for token {token}")]
    MissingClearingPrice {
        /// Token address whose clearing price was missing.
        token: Address,
    },
    /// Partially fillable trade encoding requires an executed amount.
    #[error("missing executed amount for partially fillable trade")]
    MissingExecutedAmount,
    /// Swap encoding requires a trade to have been encoded first.
    #[error("trade not encoded")]
    MissingTrade,
    /// Contract orders cannot use the zero address as a receiver.
    #[error("receiver cannot be address(0)")]
    ZeroReceiver,
    /// Provider operation failed outside the EIP-1271 helpers.
    #[error("provider error: {0}")]
    Provider(String),
    /// ABI encoding failed.
    #[error("ABI encoding error: {0}")]
    Abi(String),
    /// Hex, JSON, or response decoding failed.
    #[error("decode error: {0}")]
    Decode(String),
    /// Serialization to JSON or ABI-adjacent payloads failed.
    #[error("serialization error: {0}")]
    Serialization(String),
}
