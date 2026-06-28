//! wasm-bindgen exports for the TypeScript-callable WASM API.

/// Typed JavaScript callback shapes for wallet, signer, and HTTP transport.
pub mod callbacks;
/// Per-call cancellation token and timeout option types.
pub mod cancel;
/// Supported-chain lookup helpers exposed to JavaScript.
pub mod chains;
/// TWAP conditional-order transaction builders.
#[cfg(feature = "composable")]
pub mod composable;
/// EIP-1271 signature-payload and smart-account signing exports.
#[cfg(feature = "signing")]
pub mod eip1271;
/// Versioned response envelope wrapping every exported result.
pub mod envelope;
/// Typed error surface and its JavaScript conversion.
pub mod errors;
#[cfg(feature = "signing")]
pub mod events;
/// IPFS app-data read-client exports.
#[cfg(feature = "ipfs")]
pub mod ipfs;
/// Orderbook client exports.
#[cfg(feature = "orderbook")]
pub mod orderbook;
#[cfg(any(
    feature = "orderbook",
    feature = "subgraph",
    feature = "ipfs",
    feature = "trading"
))]
mod registry;
/// Order and cancellation signing exports.
#[cfg(feature = "signing")]
pub mod signing;
/// Subgraph analytics client exports.
#[cfg(feature = "subgraph")]
pub mod subgraph;
/// Trading client exports.
#[cfg(feature = "trading")]
pub mod trading;
/// HTTP transport configuration and the JavaScript callback transport bridge.
#[cfg(any(
    feature = "orderbook",
    feature = "subgraph",
    feature = "ipfs",
    feature = "trading"
))]
pub mod transport;

#[cfg(feature = "composable")]
pub use composable::*;
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
pub use {crate::dto::*, cancel::*, chains::*, envelope::*, errors::*};
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

/// Converts a JavaScript `number` into a native `i64`, rejecting non-integral or
/// out-of-range values so a lossy float cannot cross the ABI boundary. Used for
/// the database integers carried as `number` on the TypeScript surface — quote
/// ids and auction ids — which are non-negative and well within the JavaScript
/// safe-integer range.
#[cfg(any(feature = "orderbook", feature = "trading"))]
#[allow(
    clippy::cast_possible_truncation,
    reason = "value is validated as a non-negative integer at most 2^53-1 before the cast, so the i64 conversion is exact"
)]
pub(crate) fn js_safe_integer_to_i64(
    value: f64,
    field: &'static str,
) -> Result<i64, errors::WasmError> {
    /// Largest integer a JavaScript `number` represents exactly (2^53 - 1).
    const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;
    if value.is_finite() && value.fract() == 0.0 && (0.0..=MAX_SAFE_INTEGER).contains(&value) {
        Ok(value as i64)
    } else {
        Err(errors::WasmError::invalid(
            field,
            format!(
                "{field} must be a non-negative integer within the JavaScript safe-integer range"
            ),
        ))
    }
}

/// Initializes the wasm crate's panic hook once.
#[wasm_bindgen(js_name = "__cow_sdk_wasm_init")]
pub fn cow_sdk_wasm_init() {
    console_error_panic_hook::set_once();
}
