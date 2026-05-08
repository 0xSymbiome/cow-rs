#![forbid(unsafe_code)]
//! TypeScript-callable wasm-bindgen leaf for the `CoW` Protocol Rust SDK.
//!
//! This crate exposes deterministic `CoW` Protocol helpers — order
//! signing, EIP-1271 envelope construction, app-data hashing,
//! orderbook, subgraph, IPFS, trading — to JavaScript and TypeScript
//! consumers through typed DTOs and explicit JavaScript callbacks for
//! wallet, signer, and HTTP transport.
//!
//! The crate is split into two module trees:
//!
//! - [`pure`] holds host-safe protocol helpers. Modules here compile
//!   for both native and `wasm32-unknown-unknown` targets. They
//!   contain no `wasm-bindgen` derives, no `tsify` derives, and no
//!   `JsValue` references, which keeps the host build cleanly
//!   separated from the JavaScript binding surface.
//! - `exports` (visible only on `wasm32-unknown-unknown`) holds the
//!   `wasm-bindgen` surface, the `tsify`-derived DTOs, the four
//!   typed wallet callback shapes, the JS callback HTTP transport,
//!   and the fetch-callback registry.
//!
//! The split is enforced by a host gate: building the crate for the
//! native target with `cargo check -p cow-sdk-wasm
//! --no-default-features` succeeds only when no wasm-bindgen or
//! tsify derive leaks into the `pure` module tree.

pub mod pure;

#[cfg(target_arch = "wasm32")]
pub mod exports;
