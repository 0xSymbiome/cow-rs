//! Compile-fail witness for the `wasm32` missing-transport typestate.
//!
//! Under `target_arch = "wasm32"`, the convenience default-transport
//! `SubgraphApiBuilder::build` impl (the one gated on
//! `#[cfg(not(target_arch = "wasm32"))]`) is absent. Reaching
//! `.build()` against `SubgraphApiBuilder<ChainIdSet, ApiKeySet,
//! TransportUnset>` on `wasm32` is therefore a compile error: the
//! caller must install a browser-runtime transport (for example
//! `FetchTransport` from `cow-sdk-transport-wasm`) via
//! [`SubgraphApiBuilder::transport`] before `.build()` becomes
//! reachable.
//!
//! This file is the pinned source shape of the compile failure. It is
//! intentionally located under `tests/ui/` so Cargo's default
//! test-target discovery (`tests/*.rs`, not `tests/*/*.rs`) does NOT
//! compile it as part of `cargo test --workspace`. The accompanying
//! `builder_wasm32_missing_transport.stderr` file records the expected
//! compiler diagnostic shape.
//!
//! The runtime assertion that these witness artifacts remain in place
//! lives at `crates/subgraph/tests/builder_ui.rs`.

#![cfg(target_arch = "wasm32")]

use cow_sdk_core::SupportedChainId;
use cow_sdk_subgraph::SubgraphApi;

fn main() {
    let _ = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("partner-graph-api-key")
        .build();
}
