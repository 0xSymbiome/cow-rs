use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AppDataError {
    #[error("invalid app data hex")]
    InvalidAppDataHex,
    #[error("invalid cid format")]
    InvalidCid,
    #[error("AppData version {0} is not a valid version")]
    InvalidSchemaVersion(String),
    #[error("AppData version {0} doesn't exist")]
    UnknownSchemaVersion(String),
    #[error("AppData document is missing string field `version`")]
    MissingSchemaVersion,
    #[error("json error: {0}")]
    Json(String),
    #[error("schema error: {0}")]
    Schema(String),
    #[error("Invalid appData provided: {0}")]
    InvalidAppDataProvided(String),
    #[error("Failed to calculate appDataHex: {0}")]
    Calculation(String),
    #[error("transport error: {0}")]
    Transport(String),
    #[error("You need to pass IPFS api credentials.")]
    MissingIpfsCredentials,
    #[error("pinning error: {0}")]
    Pinning(String),
}
