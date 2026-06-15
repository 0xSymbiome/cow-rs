use cow_sdk_core::{Address, Cancelled, CoreError, ErrorClass, Redacted};
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
    /// A signing-scheme discriminator was not recognized.
    #[error("unsupported signing scheme value: {0}")]
    UnsupportedSigningScheme(u8),
    /// An order-kind or token-balance marker decoded from an on-chain
    /// `OrderPlacement` event did not match any canonical `GPv2` label hash.
    #[error("unrecognized GPv2 order marker: {0}")]
    UnknownOrderMarker(alloy_primitives::B256),
    /// An on-chain event log carried a topic set that did not match the
    /// expected event signature hash or indexed-parameter arity.
    #[error("unexpected event log topics for {event}")]
    UnexpectedEventTopics {
        /// Event whose topic set failed validation.
        event: &'static str,
    },
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
        alloy_primitives::hex::encode(expected),
        alloy_primitives::hex::encode(actual)
    )]
    Eip1271MagicValueMismatch {
        /// Expected 4-byte magic value.
        expected: [u8; 4],
        /// Actual 4-byte magic value returned by the verifier.
        actual: [u8; 4],
    },
    /// Contract orders cannot use the zero address as a receiver.
    #[error("receiver cannot be address(0)")]
    ZeroReceiver,
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
    /// [`alloy_primitives::hex::FromHexError`] is preserved in the error-source chain.
    #[error("hex decode error for field `{field}`: {source}")]
    DecodeHex {
        /// Public field or payload name that failed to decode.
        field: &'static str,
        /// Typed hex-decode error sourced from the decoder.
        #[source]
        source: alloy_primitives::hex::FromHexError,
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
    /// A hexadecimal payload exceeded the maximum decoded byte length
    /// permitted for its field.
    #[error("field `{field}` exceeds the maximum of {max_bytes} decoded bytes")]
    FieldTooLarge {
        /// Public field or payload name that exceeded the limit.
        field: &'static str,
        /// Maximum decoded byte length permitted for the field.
        max_bytes: usize,
    },
    /// JSON serialization or decoding failed.
    ///
    /// Only the serde failure category and the structural position are
    /// surfaced. The raw `serde_json::Error` rendering can echo bytes from a
    /// decoded payload, so the conversion drops it (ADR 0025); the
    /// `category`/`line`/`column` triple is the safe structural diagnostic,
    /// mirroring [`cow_sdk_orderbook::OrderbookError::Serialization`].
    ///
    /// [`cow_sdk_orderbook::OrderbookError::Serialization`]: https://docs.rs/cow-sdk-orderbook
    #[error("serialization error ({category}) at line {line} column {column}")]
    Serialization {
        /// serde failure category: `"syntax"`, `"data"`, `"eof"`, or `"io"`.
        category: &'static str,
        /// 1-based line where decoding failed, or `0` when the position is unknown.
        line: usize,
        /// 1-based column where decoding failed, or `0` when the position is unknown.
        column: usize,
    },
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

impl ContractsError {
    /// Returns the coarse-grained [`ErrorClass`] for this error.
    #[must_use]
    pub const fn class(&self) -> ErrorClass {
        match self {
            Self::Core(error) => error.class(),
            Self::Cancelled => ErrorClass::Cancelled,
            // Caller-supplied input that failed a client-side shape or range
            // check classifies as validation.
            Self::UnsupportedChain(_)
            | Self::InvalidOrderUidLength { .. }
            | Self::InvalidHexPrefix { .. }
            | Self::InvalidDecodedLength { .. }
            | Self::FieldTooLarge { .. }
            | Self::InvalidSignatureLength { .. }
            | Self::InvalidSignatureRecoveryByte { .. }
            | Self::ZeroReceiver => ErrorClass::Validation,
            // Serialization, ABI, hex-decode, and on-chain event/marker decode
            // failures are data round-trip invariants, matching the
            // `CoreError` serialization classification.
            Self::Serialization { .. }
            | Self::Abi(_)
            | Self::DecodeHex { .. }
            | Self::UnknownOrderMarker(_)
            | Self::UnexpectedEventTopics { .. } => ErrorClass::Internal,
            // EIP-1271 verification, provider interaction, ECDSA recovery,
            // signing-scheme classification, and any future additive variant
            // classify as the contracts crate's signing-edge bucket.
            _ => ErrorClass::Signing,
        }
    }
}

impl From<serde_json::Error> for ContractsError {
    /// Captures only the serde failure category and structural position.
    ///
    /// The raw `serde_json::Error` rendering can echo bytes from a decoded
    /// payload, so it is intentionally dropped here (ADR 0025); only the
    /// `category`/`line`/`column` triple is retained.
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization {
            category: serialization_error_category(&error),
            line: error.line(),
            column: error.column(),
        }
    }
}

/// Maps a `serde_json` failure to its stable category tag.
fn serialization_error_category(error: &serde_json::Error) -> &'static str {
    match error.classify() {
        serde_json::error::Category::Io => "io",
        serde_json::error::Category::Syntax => "syntax",
        serde_json::error::Category::Data => "data",
        serde_json::error::Category::Eof => "eof",
    }
}
