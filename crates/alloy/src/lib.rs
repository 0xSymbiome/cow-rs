#![cfg_attr(doctest, doc = include_str!("../README.md"))]

//! Native composed Alloy adapter for the `cow-sdk` crate family.
//!
//! This crate currently publishes the package boundary for the composed native
//! provider plus signer client. It re-exports the leaf package namespaces for
//! consumers that opt into the umbrella package.

#![warn(missing_docs)]

#[cfg(target_arch = "wasm32")]
compile_error!(
    "the alloy / alloy-provider / alloy-signer features on cow-sdk are for native targets only; cow-sdk-alloy is native-only, and wasm targets should use cow-sdk-browser-wallet for signing and consumer-supplied EIP-1193 providers for RPC reads."
);

/// Native Alloy provider leaf namespace.
pub use cow_sdk_alloy_provider as provider;
/// Native Alloy signer leaf namespace.
pub use cow_sdk_alloy_signer as signer;
