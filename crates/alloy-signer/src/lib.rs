#![cfg_attr(doctest, doc = include_str!("../README.md"))]

//! Native Alloy-backed signing adapter for the `cow-sdk` crate family.
//!
//! This crate currently publishes the native local-signer package boundary and
//! its dependency graph. Its surface stays opt-in so the default `cow-sdk`
//! facade does not pull native Alloy signer dependencies.

#![warn(missing_docs)]

#[cfg(target_arch = "wasm32")]
compile_error!(
    "the alloy / alloy-provider / alloy-signer features on cow-sdk are for native targets only; cow-sdk-alloy-signer is native-only, and wasm targets should use cow-sdk-browser-wallet for signing and consumer-supplied EIP-1193 providers for RPC reads."
);
