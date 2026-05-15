#![cfg_attr(doctest, doc = include_str!("../README.md"))]

//! Alloy-backed local-keystore `AsyncSigner` adapter for the `CoW` Protocol Rust SDK.
//!
//! [`LocalAlloyKeystoreSigner`] wraps an Alloy local private-key signer and
//! exposes message and typed-data signing through [`cow_sdk_core::AsyncSigner`].
//! The crate intentionally does not provide provider-backed transaction
//! methods: `sign_transaction`, `send_transaction`, and `estimate_gas` return
//! [`AsyncSignerError::ProviderRequired`] because a standalone signer cannot
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
pub use error::{AsyncSignerError, AsyncSignerErrorClass};
#[cfg(not(target_arch = "wasm32"))]
pub use signer::LocalAlloyKeystoreSigner;
