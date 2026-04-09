use cow_sdk_contracts::{ContractsError, SigningScheme};
use cow_sdk_core::CoreError;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SigningError {
    #[error("core error: {0}")]
    Core(#[from] CoreError),
    #[error("contracts error: {0}")]
    Contracts(#[from] ContractsError),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("signer {operation} failed: {message}")]
    Signer {
        operation: &'static str,
        message: String,
    },
    #[error(
        "local signer-generated signatures only support EIP712 and ETHSIGN; received {scheme:?}"
    )]
    UnsupportedSignerGeneratedScheme { scheme: SigningScheme },
}
