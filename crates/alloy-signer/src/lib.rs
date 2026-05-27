#![cfg_attr(doctest, doc = include_str!("../README.md"))]

//! Alloy-backed local-keystore `Signer` adapter for the `CoW` Protocol Rust SDK.
//!
//! [`LocalAlloyKeystoreSigner`] wraps an Alloy local private-key signer and
//! exposes message and typed-data signing through [`cow_sdk_core::Signer`].
//! The crate intentionally does not provide provider-backed transaction
//! methods: `sign_transaction`, `send_transaction`, and `estimate_gas` return
//! [`SignerError::ProviderRequired`] because a standalone signer cannot
//! fill nonce, fee, chain, or transaction-type fields.
//!
//! ```rust,no_run
//! use cow_sdk_alloy_signer::LocalAlloyKeystoreSigner;
//! use cow_sdk_core::SupportedChainId;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let signer = LocalAlloyKeystoreSigner::builder()
//!     .private_key("0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d")?
//!     .chain_id(SupportedChainId::Sepolia)
//!     .build()?;
//! # let _ = signer;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![allow(
    clippy::redundant_pub_crate,
    reason = "helper modules keep crate-private visibility explicit"
)]

#[cfg(not(target_arch = "wasm32"))]
mod builder;
#[cfg(not(target_arch = "wasm32"))]
mod conversion;
#[cfg(not(target_arch = "wasm32"))]
mod error;
#[cfg(not(target_arch = "wasm32"))]
mod signer;

#[cfg(not(target_arch = "wasm32"))]
pub use builder::{
    ChainSet, ChainState, ChainUnset, KeySourceState, KeySourceUnset,
    LocalAlloyKeystoreSignerBuilder, LocalAlloyKeystoreSignerBuilderError, PrivateKeySource,
};
#[cfg(not(target_arch = "wasm32"))]
pub use error::{SignerError, SignerErrorClass};
#[cfg(not(target_arch = "wasm32"))]
pub use signer::LocalAlloyKeystoreSigner;

/// Inter-crate seam for sibling `CoW` Protocol Alloy adapter crates.
///
/// This module is not a stable consumer API. Anything exported here may
/// change without notice across minor releases; it exists so sibling
/// adapter crates can reuse the reviewed EIP-712 typed-data conversion
/// and signature normalization helpers without duplicating them.
///
/// See the Stability section of
/// `docs/adr/0036-alloy-signer-adapter.md` for the semver posture.
#[cfg(not(target_arch = "wasm32"))]
#[doc(hidden)]
pub mod __seam {
    pub use crate::conversion::{
        alloy_signature_to_hex, cow_flat_to_alloy_typed_data, cow_typed_data_payload_to_alloy,
    };
}
