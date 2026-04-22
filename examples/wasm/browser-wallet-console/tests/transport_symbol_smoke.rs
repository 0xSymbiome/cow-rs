//! Compile-time smoke that names the fetch-backed transport symbols
//! the browser wallet console depends on.
//!
//! On the host target this file is empty: the gated module below is
//! excluded by its `cfg(target_arch = "wasm32")` attribute so the
//! host-target test lane continues to build the example as an `rlib`.
//! On `wasm32-unknown-unknown` the compiler resolves both imported
//! symbols from the transport crate root or fails to build — giving
//! a narrow build-time signal if the transport crate ever renames
//! or hides them.

#![allow(dead_code)]

#[cfg(target_arch = "wasm32")]
mod wasm32_only {
    use cow_sdk_transport_wasm::{FetchTransport, FetchTransportConfig};

    const _: usize = core::mem::size_of::<FetchTransport>();
    const _: usize = core::mem::size_of::<FetchTransportConfig>();
}
