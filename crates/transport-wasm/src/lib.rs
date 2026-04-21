//! Browser-native `HttpTransport` implementation for the `cow-sdk` family.
//!
//! This crate ships `FetchTransport`, a `wasm32`-only implementation of
//! `cow_sdk_core::HttpTransport` backed by `web-sys::fetch` and
//! `wasm-bindgen-futures`. It is the browser sibling of the native-only
//! `cow_sdk_core::ReqwestTransport` default and ships alongside
//! `cow-sdk-browser-wallet` as a single-responsibility HTTP-transport leaf
//! crate. Wallet and signer surfaces stay in `cow-sdk-browser-wallet`; this
//! crate owns HTTP transport only.
//!
//! # Scope
//!
//! The crate root is gated with `#![cfg(target_arch = "wasm32")]` so every
//! non-wasm32 target sees an empty compilation unit. Consumers compose the
//! transport into typed clients without opting into the native `reqwest`
//! stack; publishing an `Arc<dyn HttpTransport>` built from
//! `FetchTransport` keeps the surface runtime-neutral.
//!
//! # Feature flags
//!
//! - `default` — no features enabled.
//! - `tracing` — emits span-level tracing events around each request when
//!   the downstream application wires the [`tracing`](https://docs.rs/tracing)
//!   subscriber; unchanged public surface otherwise.
//!
//! # Minimal usage
//!
//! ```
//! # #[cfg(target_arch = "wasm32")]
//! # mod wasm_only {
//! use cow_sdk_core::HttpTransport;
//! use cow_sdk_transport_wasm::{FetchTransport, FetchTransportConfig};
//!
//! pub async fn example(
//!     transport: &FetchTransport,
//! ) -> Result<String, cow_sdk_core::TransportError> {
//!     transport.get("/api/v1/version").await
//! }
//!
//! pub fn build_transport() -> FetchTransport {
//!     FetchTransport::new(&FetchTransportConfig::new("https://api.cow.fi"))
//! }
//! # }
//! ```
//!
//! `FetchTransport` surfaces every browser-`fetch` failure through the
//! shared `cow_sdk_core::TransportError` enum with the same
//! `cow_sdk_core::TransportErrorClass` taxonomy the native adapter uses
//! (`Timeout`, `Connect`, `Redirect`, `Decode`, `Body`, `Status`, fallthrough).

#![cfg(target_arch = "wasm32")]
#![warn(missing_docs)]

/// Browser fetch-backed [`HttpTransport`](cow_sdk_core::HttpTransport)
/// implementation and its configuration bundle.
pub mod fetch;

pub use fetch::{FetchTransport, FetchTransportConfig};
