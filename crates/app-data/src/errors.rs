use thiserror::Error;

/// Errors returned by app-data generation, validation, transport, and CID helpers.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
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
    Json(String),
    /// JSON schema validation or schema construction failed.
    #[error("schema error: {0}")]
    Schema(String),
    /// The supplied app-data document failed semantic validation.
    #[error("Invalid appData provided: {0}")]
    InvalidAppDataProvided(String),
    /// CID or digest calculation failed.
    #[error("Failed to calculate appDataHex: {0}")]
    Calculation(String),
    /// Fetch-transport configuration or execution failed.
    #[error("transport error: {0}")]
    Transport(String),
    /// Upload helpers were called without the required credentials.
    #[error("You need to pass IPFS api credentials.")]
    MissingIpfsCredentials,
    /// Pinning or upload failed.
    #[error("pinning error: {0}")]
    Pinning(String),
    /// The stringified app-data document exceeded the configured size ceiling.
    #[error("app-data document is {actual_bytes} bytes which exceeds the {max_bytes}-byte limit")]
    TooLarge {
        /// Size of the stringified document in bytes.
        actual_bytes: usize,
        /// Configured size ceiling in bytes.
        max_bytes: usize,
    },
}
