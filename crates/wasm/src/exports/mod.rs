//! wasm-bindgen exports for the TypeScript-callable WASM API.

pub mod callbacks;
pub mod cancel;
pub mod chains;
pub mod dto;
pub mod eip1271;
pub mod envelope;
pub mod errors;
pub mod ipfs;
pub mod orderbook;
mod registry;
pub mod signing;
pub mod subgraph;
pub mod trading;
pub mod transport;

pub use {
    cancel::*, chains::*, dto::*, eip1271::*, envelope::*, errors::*, ipfs::*, orderbook::*,
    signing::*, subgraph::*, trading::*, transport::*,
};

use wasm_bindgen::prelude::*;

/// Initializes the wasm crate's panic hook once.
#[wasm_bindgen(js_name = "__cow_sdk_wasm_init")]
pub fn cow_sdk_wasm_init() {
    console_error_panic_hook::set_once();
}
