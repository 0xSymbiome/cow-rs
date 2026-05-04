use cow_sdk_core::{Address, Cancelled, CoreError, Redacted};
use thiserror::Error;

/// Errors returned by low-level `CoW` contract helpers.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ContractsError {
    /// Core validation failed for an input value.
    #[error("core validation error: {0}")]
    Core(#[from] CoreError),
    /// A long-running contracts operation was cancelled through a cooperative cancellation token.
    #[error("contracts operation was cancelled")]
    Cancelled,
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
        value: Redacted<String>,
    },
    /// A numeric value exceeded `uint256` bounds.
    #[error("numeric value for {field} exceeds uint256: {value}")]
    NumericOverflow {
        /// Field being encoded.
        field: &'static str,
        /// Original overflowing value.
        value: Redacted<String>,
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
        message: Redacted<String>,
    },
    /// The EIP-1271 call response could not be decoded.
    #[error("malformed EIP-1271 response: {response}")]
    MalformedEip1271Response {
        /// Raw response that failed decoding.
        response: Redacted<String>,
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
    /// A settlement trade referenced a token index outside the registered range.
    #[error("invalid trade token index {index}; only {registered} tokens are registered")]
    InvalidTokenIndex {
        /// Offending token index on the trade.
        index: usize,
        /// Number of registered tokens in the settlement registry.
        registered: usize,
    },
    /// A settlement interaction targeted a registry-paired forbidden contract.
    #[error("forbidden settlement interaction target: {target}")]
    ForbiddenInteractionTarget {
        /// Rejected interaction target address.
        target: Address,
    },
    /// Provider operation failed outside the EIP-1271 helpers.
    #[error("provider error during {operation}: {message}")]
    Provider {
        /// Failed provider operation.
        operation: &'static str,
        /// Provider error message.
        message: Redacted<String>,
    },
    /// ABI encoding or decoding failed through the `alloy-sol-types` surface.
    #[error("ABI error: {0}")]
    Abi(#[from] alloy_sol_types::Error),
    /// Hex decoding failed for a named field; the underlying
    /// [`hex::FromHexError`] is preserved in the error-source chain.
    #[error("hex decode error for field `{field}`: {source}")]
    DecodeHex {
        /// Public field or payload name that failed to decode.
        field: &'static str,
        /// Typed hex-decode error sourced from the decoder.
        #[source]
        source: hex::FromHexError,
    },
    /// A hexadecimal payload was not `0x`-prefixed.
    #[error("field `{field}` must be 0x-prefixed hexadecimal data")]
    InvalidHexPrefix {
        /// Public field or payload name that failed the prefix check.
        field: &'static str,
    },
    /// A hexadecimal payload decoded to an unexpected byte length.
    #[error(
        "field `{field}` must decode to {expected} bytes, got {actual} byte(s) after 0x prefix"
    )]
    InvalidDecodedLength {
        /// Public field or payload name that failed the length check.
        field: &'static str,
        /// Expected decoded byte length.
        expected: usize,
        /// Actual decoded byte length.
        actual: usize,
    },
    /// Serialization to JSON or ABI-adjacent payloads failed.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    /// Signature byte length is not the required 65.
    #[error("invalid signature length: expected 65 bytes, got {actual}")]
    InvalidSignatureLength {
        /// Observed byte length.
        actual: usize,
    },
    /// Recovery byte is outside the accepted {0, 1, 27, 28} set.
    #[error("invalid signature recovery byte: expected v in {{0, 1, 27, 28}}, got {value}")]
    InvalidSignatureRecoveryByte {
        /// Rejected recovery byte value.
        value: u8,
    },
    /// A non-ECDSA signature variant was passed to an ECDSA-only helper.
    #[error("signature scheme is not ECDSA")]
    SignatureSchemeNotEcdsa,
    /// ECDSA public-key recovery failed.
    #[error("ECDSA signature recovery failed: {message}")]
    SignatureRecovery {
        /// Sanitized recovery failure description from the backend.
        message: Redacted<String>,
    },
}

impl From<Cancelled> for ContractsError {
    fn from(_: Cancelled) -> Self {
        Self::Cancelled
    }
}
