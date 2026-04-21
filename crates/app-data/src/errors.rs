use cow_sdk_core::{TransportErrorClass, ValidationReason};
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
    #[error("AppData version {0} is not a valid version")]
    InvalidSchemaVersion(String),
    /// The requested schema version was not embedded in the crate.
    #[error("AppData version {0} doesn't exist")]
    UnknownSchemaVersion(String),
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
        message: String,
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
        detail: String,
    },
    /// Upload helpers were called without the required credentials.
    #[error("You need to pass IPFS api credentials.")]
    MissingIpfsCredentials,
    /// Pinning or upload failed.
    #[error("pinning error (status {status:?}): {message}")]
    Pinning {
        /// HTTP status code returned by the pinning service, when known.
        status: Option<u16>,
        /// Redacted detail message sourced from the pinning response.
        message: String,
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
