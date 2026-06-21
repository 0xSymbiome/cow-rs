use cow_sdk_core::Redacted;
use thiserror::Error;

/// Typed COW Shed helper errors.
///
/// These cover the provider-free signing orchestration in
/// [`crate::cow_shed::CowShedHooks`]. On-chain revert conditions are not
/// mirrored here: the `sol!` interfaces in [`crate::cow_shed::bindings`]
/// already emit typed, decodable error types for every deployed custom error
/// (`COWShed::COWShedErrors`, `COWShedFactory::COWShedFactoryErrors`).
///
/// The signer-authored message in [`CowShedError::OwnerResolution`] and
/// [`CowShedError::Signing`] is captured behind [`Redacted`] so a custom
/// `Signer` implementation cannot leak credential-bearing text through
/// `Display` or `Debug`, matching [`crate::ContractsError::Eip1271Provider`].
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CowShedError {
    /// The signer could not resolve the owner address.
    #[error("cow-shed: resolve owner address: {0}")]
    OwnerResolution(Redacted<String>),
    /// Signing the `ExecuteHooks` typed-data payload failed.
    #[error("cow-shed: sign ExecuteHooks payload: {0}")]
    Signing(Redacted<String>),
    /// The signer returned a value that is not a canonical 65-byte
    /// recoverable signature.
    #[error("cow-shed: parse signature")]
    SignatureParse(#[source] crate::ContractsError),
}
