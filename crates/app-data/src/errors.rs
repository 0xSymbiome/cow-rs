use cow_sdk_core::{Cancelled, Redacted, TransportErrorClass, ValidationReason};
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use thiserror::Error;

use crate::types::SchemaVersion;

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
    #[error("AppData version {0} is not a valid version")]
    InvalidSchemaVersion(Redacted<String>),
    /// The requested schema version was not embedded in the crate.
    #[error("AppData version {0} doesn't exist")]
    UnknownSchemaVersion(SchemaVersion),
    /// The app-data document did not contain a string `version` field.
    #[error("AppData document is missing string field `version`")]
    MissingSchemaVersion,
    /// JSON serialization or parsing failed.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    /// JSON schema validation or schema construction failed; the path-prefixed
    /// validator message is exposed through `Display` and the typed underlying
    /// [`jsonschema::ValidationError`] is preserved through the error-source
    /// chain.
    #[error("schema error: {message}")]
    Schema {
        /// Path-prefixed validator message rendered for human inspection;
        /// includes the failing JSON instance path when available so the
        /// `Display` rendering identifies the offending field.
        message: Redacted<String>,
        /// Owned schema-validator error returned by the underlying
        /// [`jsonschema`] crate.
        #[source]
        source: Box<jsonschema::ValidationError<'static>>,
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
    #[error("appDataHex calculation failed: {source}")]
    Calculation {
        /// Typed source error returned by the underlying hashing or CID
        /// crate. Boxed as a trait object so the variant can carry either
        /// a [`cid`]-crate or a [`multihash`]-crate failure without
        /// widening the enum surface.
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
    /// Upload helpers were called without the required credentials.
    #[error("You need to pass IPFS api credentials.")]
    MissingIpfsCredentials,
    /// Pinning or upload failed.
    #[error("pinning error (status {status:?}): {message}")]
    Pinning {
        /// HTTP status code returned by the pinning service, when known.
        status: Option<u16>,
        /// Redacted and bounded detail message sourced from the pinning response.
        message: Redacted<String>,
    },
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
            Self::UnknownSchemaVersion(version) => {
                map.serialize_entry("type", "UnknownSchemaVersion")?;
                map.serialize_entry("version", version)?;
            }
            Self::MissingSchemaVersion => {
                map.serialize_entry("type", "MissingSchemaVersion")?;
            }
            Self::Json(error) => {
                map.serialize_entry("type", "Json")?;
                map.serialize_entry("message", &error.to_string())?;
            }
            Self::Schema { message, .. } => {
                map.serialize_entry("type", "Schema")?;
                map.serialize_entry("message", message)?;
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
            Self::Calculation { source } => {
                map.serialize_entry("type", "Calculation")?;
                map.serialize_entry("message", &source.to_string())?;
            }
            Self::Transport { class, detail } => {
                map.serialize_entry("type", "Transport")?;
                map.serialize_entry("class", &class.to_string())?;
                map.serialize_entry("detail", detail)?;
            }
            Self::Cancelled => {
                map.serialize_entry("type", "Cancelled")?;
            }
            Self::MissingIpfsCredentials => {
                map.serialize_entry("type", "MissingIpfsCredentials")?;
            }
            Self::Pinning { status, message } => {
                map.serialize_entry("type", "Pinning")?;
                map.serialize_entry("status", status)?;
                map.serialize_entry("message", message)?;
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
