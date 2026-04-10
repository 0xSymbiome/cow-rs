//! Thin facade crate for the primary CoW Protocol Rust SDK surface.
//!
//! The root package stays intentionally narrow:
//!
//! - shared core and config types
//! - signing helpers
//! - contracts helpers
//! - orderbook client types
//! - app-data helpers
//! - trading orchestration
//!
//! Supported surface split:
//!
//! - native and server-side consumers use the default facade
//! - wasm consumers can use the same default facade for pure SDK flows
//! - browser wallet integration is additive behind the `browser-wallet` feature
//! - subgraph access remains in the separate `cow-sdk-subgraph` crate
//!
//! Top-level docs stay trading-first, matching the pinned upstream `packages/sdk`
//! documentation entrypoint.
//!
//! `cow-sdk-subgraph` remains a separate crate surface and is not re-exported from
//! this root package.
//!
//! ```rust
//! use cow_sdk::{Address, PartialTraderParameters, TradingSdk, TradingSdkOptions};
//!
//! let _address = Address::new("0x1111111111111111111111111111111111111111").unwrap();
//! let _sdk = TradingSdk::new(PartialTraderParameters::default(), TradingSdkOptions::default());
//! ```
//!
//! ```compile_fail
//! use cow_sdk::subgraph;
//! ```
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod prelude;

pub use prelude::*;

pub use cow_sdk_app_data as app_data;
#[cfg(feature = "browser-wallet")]
#[cfg_attr(docsrs, doc(cfg(feature = "browser-wallet")))]
pub use cow_sdk_browser_wallet as browser_wallet;
pub use cow_sdk_contracts as contracts;
pub use cow_sdk_core as core;
pub use cow_sdk_orderbook as orderbook;
pub use cow_sdk_signing as signing;
pub use cow_sdk_trading as trading;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SdkError {
    #[error("types error: {0}")]
    Types(#[from] cow_sdk_core::CowRsError),
    #[error("signing error: {0}")]
    Signing(#[from] cow_sdk_signing::SigningError),
    #[error("app-data error: {0}")]
    AppData(#[from] cow_sdk_app_data::AppDataError),
    #[error("contracts error: {0}")]
    Contracts(#[from] cow_sdk_contracts::ContractsError),
    #[error("orderbook error: {0}")]
    Orderbook(#[from] cow_sdk_orderbook::OrderbookError),
    #[error("trading error: {0}")]
    Trading(#[from] cow_sdk_trading::TradingError),
    #[cfg(feature = "browser-wallet")]
    #[error("browser wallet error: {0}")]
    BrowserWallet(#[from] cow_sdk_browser_wallet::BrowserWalletError),
}
