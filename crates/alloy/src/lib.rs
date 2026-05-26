#![cfg_attr(doctest, doc = include_str!("../README.md"))]

//! Native composed Alloy adapter for the `CoW` Protocol Rust SDK.
//!
//! [`AlloyClient`] combines an Alloy HTTP provider with an Alloy local private
//! key signer through Alloy's wallet-filler stack. It implements
//! [`cow_sdk_core::AsyncProvider`] and
//! [`cow_sdk_core::AsyncSigningProvider`], while the handle returned by
//! `create_signer` implements [`cow_sdk_core::AsyncSigner`].
//!
//! ```rust,no_run
//! use cow_sdk_alloy::AlloyClient;
//! use cow_sdk_core::SupportedChainId;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = AlloyClient::builder()
//!     .http("https://example.invalid/rpc")?
//!     .private_key("0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d")?
//!     .chain_id(SupportedChainId::Sepolia)
//!     .build()
//!     .await?;
//! # let _ = client;
//! # Ok(())
//! # }
//! ```
//!
//! `send_transaction` submits through the wallet-filler provider and returns
//! [`cow_sdk_core::TransactionBroadcast`] with the broadcast transaction hash
//! from Alloy's pending transaction handle. It does not wait for receipt
//! observation. Use [`cow_sdk_core::AsyncProvider::get_transaction_receipt`]
//! on the client when mined status, block, gas, sender, or recipient fields are
//! needed. Raw `sign_transaction` is intentionally not exposed by this release
//! because Alloy's provider method routes to the remote JSON-RPC peer rather
//! than producing a local signed payload.

#![warn(missing_docs)]
#![allow(
    clippy::redundant_pub_crate,
    reason = "helper modules keep crate-private visibility explicit"
)]

#[cfg(not(target_arch = "wasm32"))]
mod builder;
#[cfg(not(target_arch = "wasm32"))]
mod client;
#[cfg(not(target_arch = "wasm32"))]
mod conversion;
#[cfg(not(target_arch = "wasm32"))]
mod error;
#[cfg(not(target_arch = "wasm32"))]
mod handle;

#[cfg(not(target_arch = "wasm32"))]
pub use builder::{
    AlloyClientBuilder, AlloyClientBuilderError, ChainSet, ChainState, ChainUnset, HttpTransport,
    KeySourceState, KeySourceUnset, PrivateKeySource, TransportState, TransportUnset,
};
#[cfg(not(target_arch = "wasm32"))]
pub use client::AlloyClient;
#[cfg(not(target_arch = "wasm32"))]
pub use error::{AlloyClientError, AlloyClientErrorClass};
#[cfg(not(target_arch = "wasm32"))]
pub use handle::AlloyClientSignerHandle;

/// Native Alloy provider leaf namespace.
#[cfg(not(target_arch = "wasm32"))]
pub use cow_sdk_alloy_provider as provider;
/// Native Alloy signer leaf namespace.
#[cfg(not(target_arch = "wasm32"))]
pub use cow_sdk_alloy_signer as signer;
