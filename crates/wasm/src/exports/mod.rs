//! wasm-bindgen exports for the TypeScript-callable WASM API.

pub mod callbacks;
pub mod cancel;
pub mod chains;
pub mod dto;
#[cfg(feature = "signing")]
pub mod eip1271;
pub mod envelope;
pub mod errors;
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
pub use {eip1271::*, signing::*};

use wasm_bindgen::prelude::*;

/// Initializes the wasm crate's panic hook once.
#[wasm_bindgen(js_name = "__cow_sdk_wasm_init")]
pub fn cow_sdk_wasm_init() {
    console_error_panic_hook::set_once();
}
