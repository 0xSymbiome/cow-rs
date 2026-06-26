#![cfg_attr(any(doctest, docsrs), doc = include_str!("../README.md"))]
#![forbid(unsafe_code)]
//! TypeScript-callable wasm-bindgen leaf for the `CoW` Protocol Rust SDK.
//!
//! This crate exposes deterministic `CoW` Protocol helpers — order
//! signing, EIP-1271 envelope construction, app-data hashing, on-chain
//! event-log decoding, orderbook, subgraph, IPFS, trading — to JavaScript
//! and TypeScript consumers through typed DTOs and explicit JavaScript
//! callbacks for wallet, signer, and HTTP transport.
//!
//! The crate is appropriate for specialized cases: deterministic Rust
//! signing parity, single-source-of-truth Rust + TypeScript embedding,
//! edge runtimes such as Cloudflare Workers, Deno, and Vercel Edge
//! (size-compatible at the time of measurement; full Workers support
//! pending release-bundle and startup validation), and embeddable
//! signing helpers. For most browser dapps and standard
//! TypeScript applications, the upstream `@cowprotocol/cow-sdk`
//! TypeScript SDK is the recommended choice because of its smaller
//! bundle size at equivalent feature subsets. The npm package README
//! covers the consumer routing matrix.
//!
//! The crate separates runtime-neutral helpers from the JavaScript binding
//! surface:
//!
//! - [`helpers`] holds host-safe protocol helpers. Those modules compile for
//!   both native and `wasm32-unknown-unknown` targets and contain no
//!   `wasm-bindgen` derives, no `tsify` derives, and no `JsValue` references.
//! - [`dto`] holds the boundary shapes the surface accepts and returns that have
//!   no native crate counterpart. They compile for both targets; their
//!   TypeScript declaration derive is gated to the wasm-bindgen target, so the
//!   host build links the plain shapes.
//! - `exports` (visible only on `wasm32-unknown-unknown`) holds the
//!   `wasm-bindgen` surface, the `tsify`-derived DTOs, the four
//!   typed wallet callback shapes, the JS callback HTTP transport,
//!   and the fetch-callback registry.
//!
//! The split is enforced by a host gate: building the crate for the
//! native target with `cargo check -p cow-sdk-wasm
//! --no-default-features` succeeds only when no wasm-bindgen or
//! tsify derive leaks into target-agnostic dependencies.

#![warn(missing_docs)]

pub mod dto;
pub mod helpers;

#[cfg(target_arch = "wasm32")]
pub mod exports;
