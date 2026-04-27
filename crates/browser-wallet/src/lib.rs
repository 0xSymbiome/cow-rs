#![cfg_attr(doctest, doc = include_str!("../README.md"))]

//! Browser wallet integration for WASM consumers using typed EIP-1193 provider,
//! signer, discovery, and session contracts.
//!
//! This crate is the browser-runtime leaf of the SDK package family.
//!
//! It exposes three support layers:
//!
//! - deterministic proof mode through [`MockEip1193Transport`] for tests and review flows
//! - typed browser-wallet runtime flows through [`BrowserWallet`], [`Eip1193Provider`], and
//!   [`Eip1193Signer`]
//! - environment-sensitive injected-wallet discovery for `wasm32` consumers
//!
//! The public contract stays typed and Rust-native. Raw JavaScript payloads remain local to the
//! crate, and this package does not add a generic wallet-RPC passthrough beyond the typed EIP-1193
//! transport seam it owns.
//! Typed chain-management helpers confirm the refreshed wallet session chain before they report
//! switch success.
//!
//! Injected-wallet behavior remains environment-sensitive. Authorization prompts, provider
//! inventory, extension timing, and vendor-specific support are controlled by the browser runtime
//! and wallet extension rather than normalized into universal SDK guarantees.
//!
//! # Dependency Posture
//!
//! The typed EIP-1193 contract-call bridge inside [`provider`] uses the
//! `alloy-primitives`, `alloy-dyn-abi`, and `alloy-json-abi` family for ABI encoding
//! and decoding. That dependency choice, including the reviewed advisories the alloy
//! toolchain transits, is tracked publicly in
//! [docs/audit/browser-wallet-alloy-dependency-audit.md](https://github.com/cowdao-grants/cow-rs/blob/main/docs/audit/browser-wallet-alloy-dependency-audit.md).
//! No `alloy_*` type appears in any `pub fn` signature across the workspace.

#![warn(missing_docs)]

/// Browser-wallet error and RPC failure types.
pub mod error;
/// Session state, event-log types, and provider-driven session synchronization.
pub mod events;
/// Browser-runtime discovery and injected-provider transport bindings.
pub mod js;
/// Deterministic mock transport used for tests, examples, and proof-oriented verification.
pub mod mock;
/// Typed EIP-1193 provider transport and `AsyncProvider` bridge.
pub mod provider;
/// Typed EIP-1193 signer and typed-data signing helpers.
pub mod signer;
/// Browser-wallet discovery, session, and typed chain-management entrypoints.
pub mod wallet;

pub use error::{BrowserWalletError, RpcErrorPayload};
pub use events::{EventLog, WalletEvent, WalletSession};
pub use mock::{MockEip1193Transport, MockRequestRecord};
pub use provider::{Eip1193Provider, Eip1193ProviderBuilder, Eip1193Transport, Origin};
pub use signer::Eip1193Signer;
pub use wallet::{
    BrowserWallet, InjectedWalletDetectionOptions, InjectedWalletDiscovery,
    InjectedWalletDiscoverySource, InjectedWalletInfo, WalletChainChange, WalletChainChangeKind,
    WalletChainParameters, WalletNativeCurrency,
};

pub use cow_sdk_core::{AsyncProvider, AsyncSigner, AsyncSigningProvider};
