use cow_sdk_core::{Cancelled, ErrorClass, Redacted, TransportErrorClass, ValidationReason};
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use thiserror::Error;

/// Errors returned by app-data generation, validation, transport, and CID helpers.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum AppDataError {
    /// The supplied app-data hash was not valid `0x`-prefixed 32-byte hex.
    #[error("invalid app data hex")]
    InvalidAppDataHex,
    /// The supplied CID was malformed or unsupported.
    #[error("invalid cid format")]
    InvalidCid,
    /// The supplied schema version did not match the expected `major.minor.patch` format.
    #[error("app-data version {0} is not a valid version")]
    InvalidSchemaVersion(Redacted<String>),
    /// The app-data document did not contain a string `version` field.
    #[error("app-data document is missing string field `version`")]
    MissingSchemaVersion,
    /// JSON serialization or decoding failed.
    ///
    /// Only the serde failure category and the structural position are
    /// surfaced. The raw `serde_json::Error` rendering can echo bytes from a
    /// decoded document or response body, so the conversion drops it
    /// (ADR 0025); the `category`/`line`/`column` triple is the safe structural
    /// diagnostic, mirroring `cow_sdk_orderbook::OrderbookError::Serialization`.
    #[error("json error ({category}) at line {line} column {column}")]
    Json {
        /// serde failure category: `"syntax"`, `"data"`, `"eof"`, or `"io"`.
        category: &'static str,
        /// 1-based line where decoding failed, or `0` when the position is unknown.
        line: usize,
        /// 1-based column where decoding failed, or `0` when the position is unknown.
        column: usize,
    },
    /// The supplied app-data document failed semantic validation.
    #[error("invalid appData field `{field}`: {reason}")]
    InvalidAppDataProvided {
        /// Public field name that failed validation.
        field: &'static str,
        /// Canonical validation-failure mode.
        reason: ValidationReason,
    },
    /// A partner-fee policy failed semantic validation against the
    /// documented basis-point bounds or recipient preconditions.
    #[error("invalid partner-fee field `{field}`: {reason}")]
    InvalidPartnerFee {
        /// Public field name that failed validation.
        field: &'static str,
        /// Canonical validation-failure mode.
        reason: ValidationReason,
    },
    /// A flash-loan hint failed semantic validation against the documented
    /// bounds for `amount` or the non-zero-address preconditions on
    /// `liquidityProvider`, `protocolAdapter`, `receiver`, and `token`.
    #[error("invalid flashloan-hints field `{field}`: {reason}")]
    InvalidFlashloanHints {
        /// Public field name that failed validation, spelled as the
        /// camelCase wire key for stable error observability.
        field: &'static str,
        /// Canonical validation-failure mode.
        reason: ValidationReason,
    },
    /// CID or digest calculation failed with a typed underlying error
    /// preserved through the error-source chain.
    ///
    /// The boxed source is intentionally not rendered into `Display` or
    /// `Serialize`: a future hashing or CID backend could embed
    /// caller-derived bytes in its message, so only the stable operation
    /// label is surfaced (ADR 0025). Callers that need the precise failure
    /// walk [`std::error::Error::source`].
    #[error("appDataHex calculation failed")]
    Calculation {
        /// Typed source error returned by the underlying hashing or CID
        /// crate. Boxed as a trait object so the variant can carry either
        /// a [`cid`]-crate or a [`multihash`]-crate failure without
        /// widening the enum surface. Reachable through
        /// [`std::error::Error::source`] for callers that deliberately cross
        /// the redaction boundary.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
    /// Fetch-transport configuration or execution failed.
    #[error("transport error ({class}): {detail}")]
    Transport {
        /// Classification of the underlying REST-transport failure.
        class: TransportErrorClass,
        /// Redacted detail message sourced from the transport layer.
        detail: Redacted<String>,
    },
    /// A long-running app-data operation was cancelled through a cooperative cancellation token.
    #[error("app-data operation was cancelled")]
    Cancelled,
    /// The stringified app-data document exceeded the configured size ceiling.
    #[error("app-data document is {actual_bytes} bytes which exceeds the {max_bytes}-byte limit")]
    TooLarge {
        /// Size of the stringified document in bytes.
        actual_bytes: usize,
        /// Configured size ceiling in bytes.
        max_bytes: usize,
    },
}

impl From<Cancelled> for AppDataError {
    fn from(_: Cancelled) -> Self {
        Self::Cancelled
    }
}

impl From<serde_json::Error> for AppDataError {
    /// Captures only the serde failure category and structural position.
    ///
    /// The raw `serde_json::Error` rendering can echo bytes from a decoded
    /// document or response body, so it is intentionally dropped here
    /// (ADR 0025); only the `category`/`line`/`column` triple is retained.
    fn from(error: serde_json::Error) -> Self {
        Self::Json {
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

impl AppDataError {
    /// Returns the coarse-grained [`ErrorClass`] for this error.
    #[must_use]
    pub const fn class(&self) -> ErrorClass {
        match self {
            Self::InvalidAppDataHex
            | Self::InvalidCid
            | Self::InvalidSchemaVersion(_)
            | Self::MissingSchemaVersion
            | Self::InvalidAppDataProvided { .. }
            | Self::TooLarge { .. } => ErrorClass::Validation,
            Self::Transport { .. } => ErrorClass::Transport,
            Self::Cancelled => ErrorClass::Cancelled,
            // Json, Calculation, and partner-fee / flashloan validation failures
            // plus any future additive variants classify as internal.
            _ => ErrorClass::Internal,
        }
    }
}

impl Serialize for AppDataError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;

        match self {
            Self::InvalidAppDataHex => {
                map.serialize_entry("type", "InvalidAppDataHex")?;
            }
            Self::InvalidCid => {
                map.serialize_entry("type", "InvalidCid")?;
            }
            Self::InvalidSchemaVersion(version) => {
                map.serialize_entry("type", "InvalidSchemaVersion")?;
                map.serialize_entry("version", version)?;
            }
            Self::MissingSchemaVersion => {
                map.serialize_entry("type", "MissingSchemaVersion")?;
            }
            Self::Json {
                category,
                line,
                column,
            } => {
                map.serialize_entry("type", "Json")?;
                map.serialize_entry("category", category)?;
                map.serialize_entry("line", line)?;
                map.serialize_entry("column", column)?;
            }
            Self::InvalidAppDataProvided { field, reason } => {
                map.serialize_entry("type", "InvalidAppDataProvided")?;
                map.serialize_entry("field", field)?;
                map.serialize_entry("reason", &reason.to_string())?;
            }
            Self::InvalidPartnerFee { field, reason } => {
                map.serialize_entry("type", "InvalidPartnerFee")?;
                map.serialize_entry("field", field)?;
                map.serialize_entry("reason", &reason.to_string())?;
            }
            Self::InvalidFlashloanHints { field, reason } => {
                map.serialize_entry("type", "InvalidFlashloanHints")?;
                map.serialize_entry("field", field)?;
                map.serialize_entry("reason", &reason.to_string())?;
            }
            Self::Calculation { .. } => {
                map.serialize_entry("type", "Calculation")?;
                // The boxed source is not serialized: it could embed
                // caller-derived bytes. Only the stable label is emitted
                // (ADR 0025); callers walk `Error::source` for the detail.
                map.serialize_entry("message", "appDataHex calculation failed")?;
            }
            Self::Transport { class, detail } => {
                map.serialize_entry("type", "Transport")?;
                map.serialize_entry("class", &class.to_string())?;
                map.serialize_entry("detail", detail)?;
            }
            Self::Cancelled => {
                map.serialize_entry("type", "Cancelled")?;
            }
            Self::TooLarge {
                actual_bytes,
                max_bytes,
            } => {
                map.serialize_entry("type", "TooLarge")?;
                map.serialize_entry("actualBytes", actual_bytes)?;
                map.serialize_entry("maxBytes", max_bytes)?;
            }
        }

        map.end()
    }
}
