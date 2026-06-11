//! wasm-bindgen exports for the TypeScript-callable WASM API.

pub mod callbacks;
pub mod cancel;
pub mod chains;
pub mod dto;
#[cfg(feature = "signing")]
pub mod eip1271;
pub mod envelope;
pub mod errors;
#[cfg(feature = "signing")]
pub mod events;
#[cfg(feature = "ipfs")]
pub mod ipfs;
#[cfg(feature = "orderbook")]
pub mod orderbook;
#[cfg(any(
    feature = "orderbook",
    feature = "subgraph",
    feature = "ipfs",
    feature = "trading"
))]
mod registry;
#[cfg(feature = "signing")]
pub mod signing;
#[cfg(feature = "subgraph")]
pub mod subgraph;
#[cfg(feature = "trading")]
pub mod trading;
#[cfg(any(
    feature = "orderbook",
    feature = "subgraph",
    feature = "ipfs",
    feature = "trading"
))]
pub mod transport;

#[cfg(feature = "ipfs")]
pub use ipfs::*;
#[cfg(feature = "orderbook")]
pub use orderbook::*;
#[cfg(feature = "subgraph")]
pub use subgraph::*;
#[cfg(feature = "trading")]
pub use trading::*;
#[cfg(any(
    feature = "orderbook",
    feature = "subgraph",
    feature = "ipfs",
    feature = "trading"
))]
pub use transport::*;
pub use {cancel::*, chains::*, dto::*, envelope::*, errors::*};
#[cfg(feature = "signing")]
pub use {eip1271::*, events::*, signing::*};

use wasm_bindgen::prelude::*;

/// Runs an export future inside a telemetry span carrying its stable
/// `wasm.<area>.<method>` endpoint label.
///
/// The span is attached here, in the export body, rather than through a
/// `#[tracing::instrument]` attribute on the `#[wasm_bindgen]` export.
/// Instrumenting an exported function leaves a comparison op in the
/// wasm-bindgen describe shim, and the descriptor interpreter only evaluates
/// add/sub/and, so it would reject the module at bindgen time
/// (`invalid binary op`).
#[cfg(feature = "tracing")]
pub(crate) fn traced<F>(
    endpoint: &'static str,
    future: F,
) -> impl core::future::Future<Output = F::Output>
where
    F: core::future::Future,
{
    use tracing::Instrument as _;
    future.instrument(tracing::info_span!("wasm_export", endpoint))
}

/// Endpoint-labelled passthrough used when the `tracing` feature is disabled.
/// Returns the future unchanged, so a non-tracing build pays nothing — no span,
/// no wrapper future, and the endpoint literal is dead-code-eliminated.
#[cfg(not(feature = "tracing"))]
pub(crate) fn traced<F>(_endpoint: &'static str, future: F) -> F
where
    F: core::future::Future,
{
    future
}

/// Initializes the wasm crate's panic hook once.
#[wasm_bindgen(js_name = "__cow_sdk_wasm_init")]
pub fn cow_sdk_wasm_init() {
    console_error_panic_hook::set_once();
}
