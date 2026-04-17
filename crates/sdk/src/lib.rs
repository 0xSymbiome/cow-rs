//! Primary Rust SDK facade for `CoW` Protocol.
//!
//! This crate re-exports the main public surface for:
//!
//! - shared core and config types
//! - signing helpers
//! - contracts helpers
//! - orderbook client types
//! - app-data helpers
//! - trading orchestration
//!
//! Top-level docs are trading-first and keep the facade aligned with its package role.
//! Optional browser-runtime support does not change the default facade identity.
//! Browser-wallet support is additive behind the `browser-wallet` feature,
//! and the full browser-runtime contract stays in `cow-sdk-browser-wallet`.
//!
//! Read-only subgraph access is a separate crate surface that lives in
//! `cow-sdk-subgraph` and is not re-exported from this root package.
//!
//! ```rust
//! use cow_sdk::{Address, SupportedChainId, TradingSdk};
//!
//! let _address = Address::new("0x1111111111111111111111111111111111111111").unwrap();
//! let _sdk = TradingSdk::builder()
//!     .with_chain_id(SupportedChainId::Sepolia)
//!     .with_app_code("your-app-code")
//!     .build()
//!     .unwrap();
//! ```
//!
//! The subgraph module is intentionally not re-exported, so attempting to
//! reach it through the root facade fails to compile:
//!
//! ```compile_fail
//! use cow_sdk::subgraph;
//! ```
//!
//! The typed `SubgraphApi` entry point is likewise not reachable from the
//! facade and must be imported from `cow-sdk-subgraph` directly:
//!
//! ```compile_fail
//! use cow_sdk::SubgraphApi;
//! ```
#![warn(missing_docs)]
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
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SdkError {
    /// Shared types, validation, or configuration error.
    #[error("types error: {0}")]
    Types(#[from] cow_sdk_core::CoreError),
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
