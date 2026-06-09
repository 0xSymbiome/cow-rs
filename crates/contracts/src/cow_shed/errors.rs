use std::error::Error;

use alloy_primitives::Bytes;
use crate::DeploymentChainId;
use thiserror::Error;

/// Signature path that failed COW Shed validation.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SigSource {
    /// Externally owned account signature path.
    Eoa,
    /// EIP-1271 contract signature path.
    Eip1271,
    /// On-chain pre-signature path.
    PreSigned,
}

/// Typed COW Shed helper errors.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CowShedError {
    /// The supplied signature did not authorize the hook payload.
    #[error("invalid COW Shed signature from {0:?}")]
    InvalidSignature(SigSource),
    /// The hook payload has not been pre-signed.
    #[error("hook payload is not pre-signed")]
    NotPreSigned,
    /// The nonce was already consumed.
    #[error("nonce was already used")]
    NonceReplayed,
    /// The execution deadline has elapsed.
    #[error("deadline has expired")]
    DeadlineExpired,
    /// The proxy has no pre-sign storage contract configured.
    #[error("pre-sign storage is not configured")]
    PreSignStorageMissing,
    /// The operation requires the proxy admin.
    #[error("operation is restricted to the admin")]
    OnlyAdmin,
    /// The operation requires the admin or trusted executor role.
    #[error("operation is restricted to a trusted role")]
    OnlyTrustedRole,
    /// The operation can only be invoked by the proxy itself.
    #[error("operation is restricted to the COW Shed proxy itself")]
    OnlySelf,
    /// An EIP-1271 validation path returned a hash mismatch.
    #[error("EIP-1271 hash mismatch")]
    Erc1271HashMismatch,
    /// The operation can only be invoked by a COW Shed proxy.
    #[error("operation is restricted to COW Shed")]
    OnlyCowShed,
    /// The proxy was already initialized.
    #[error("proxy is already initialized")]
    AlreadyInitialized,
    /// The proxy has not been initialized.
    #[error("proxy is not initialized")]
    ProxyNotInitialized,
    /// The factory could not construct a proxy.
    #[error("factory construction failed: {reason}")]
    FactoryConstruction {
        /// Human-readable failure reason.
        reason: String,
    },
    /// A hook call reverted.
    #[error("hook call {index} reverted")]
    HookCallReverted {
        /// Zero-based hook index.
        index: usize,
        /// Revert data returned by the hook target.
        data: Bytes,
    },
    /// ECDSA signature recovery failed.
    #[error("signature recovery failed")]
    SignatureRecoverFailed,
    /// ENS record setup failed.
    #[error("setting ENS records failed")]
    SettingEnsRecordsFailed,
    /// The composable forwarder binding was targeted at a non-Gnosis chain.
    #[error("COWShedForComposableCoW is only deployed on Gnosis Chain, got {chain:?}")]
    COWShedForComposableCoWGnosisOnly {
        /// Chain targeted by the caller.
        chain: DeploymentChainId,
    },
    /// Forward-compatible catch-all for externally sourced errors.
    #[error(transparent)]
    Other(#[from] Box<dyn Error + Send + Sync>),
}
