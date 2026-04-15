//! Thin facade crate for the primary `CoW` Protocol Rust SDK surface.
//!
//! The root package is intentionally narrow:
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
//! - the full browser-runtime contract stays in `cow-sdk-browser-wallet`
//! - subgraph access lives in the separate `cow-sdk-subgraph` crate
//!
//! Top-level docs are trading-first and keep the facade aligned with its package role.
//! Optional browser-runtime support does not change the default facade identity.
//!
//! `cow-sdk-subgraph` is a separate crate surface and is not re-exported from
//! this root package.
//!
//! ```rust
//! use cow_sdk::{Address, SupportedChainId, TradingSdk};
//!
//! let _address = Address::new("0x1111111111111111111111111111111111111111").unwrap();
//! let _sdk = TradingSdk::builder()
//!     .with_chain_id(SupportedChainId::Sepolia)
//!     .with_app_code("cow-rs/rustdoc")
//!     .build()
//!     .unwrap();
//! ```
//!
//! ```compile_fail
//! use cow_sdk::subgraph;
//! ```
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Curated re-exports for the default `cow-sdk` facade.
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

/// Aggregate error type for the root facade crate.
#[derive(Debug, Error)]
pub enum SdkError {
    /// Shared types, validation, or configuration error.
    #[error("types error: {0}")]
    Types(#[from] cow_sdk_core::CowRsError),
    /// Signing or typed-data error.
    #[error("signing error: {0}")]
    Signing(#[from] cow_sdk_signing::SigningError),
    /// App-data generation, validation, or CID error.
    #[error("app-data error: {0}")]
    AppData(#[from] cow_sdk_app_data::AppDataError),
    /// Contract encoding, hashing, or provider interaction error.
    #[error("contracts error: {0}")]
    Contracts(#[from] cow_sdk_contracts::ContractsError),
    /// Orderbook transport, decoding, or request error.
    #[error("orderbook error: {0}")]
    Orderbook(#[from] cow_sdk_orderbook::OrderbookError),
    /// Trading workflow, quoting, or submission error.
    #[error("trading error: {0}")]
    Trading(#[from] cow_sdk_trading::TradingError),
    #[cfg(feature = "browser-wallet")]
    /// Browser-wallet transport or session error.
    #[error("browser wallet error: {0}")]
    BrowserWallet(#[from] cow_sdk_browser_wallet::BrowserWalletError),
}
