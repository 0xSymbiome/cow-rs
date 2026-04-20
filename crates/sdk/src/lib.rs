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
/// Transport-error classification shared across transport-capable crates.
///
/// Typed label that downstream telemetry and retry layers can use to
/// partition REST-transport failures without parsing error messages.
pub use cow_sdk_core::TransportErrorClass;
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

/// Coarse-grained classification surface for [`SdkError`].
///
/// Downstream telemetry layers partition failures through this class set
/// without pattern-matching every nested variant by hand. Retry policies
/// typically only retry [`ErrorClass::Transport`] and
/// [`ErrorClass::Remote`]; the other classes signal caller-side or
/// protocol-level conditions that benefit from different recovery paths.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorClass {
    /// Caller-side input failed a client-side validation boundary.
    Validation,
    /// A transport-layer failure occurred before a complete response was received.
    Transport,
    /// The remote endpoint returned a structured error response.
    Remote,
    /// A signing, provider, or cryptographic helper surfaced an error.
    Signing,
    /// A long-running operation was cancelled through a cooperative token.
    Cancelled,
    /// An internal invariant or helper contract was violated.
    Internal,
}

impl SdkError {
    /// Returns the coarse-grained class for this error.
    ///
    /// The classification is exhaustive: every supported variant resolves to
    /// one of the [`ErrorClass`] buckets without falling through to a
    /// default arm, so downstream telemetry layers can rely on the mapping
    /// staying stable across releases.
    #[must_use]
    pub const fn class(&self) -> ErrorClass {
        match self {
            Self::Types(error) => classify_core(error),
            Self::Signing(error) => classify_signing(error),
            Self::AppData(error) => classify_app_data(error),
            Self::Contracts(_) => ErrorClass::Signing,
            Self::Orderbook(error) => classify_orderbook(error),
            Self::Trading(error) => classify_trading(error),
            #[cfg(feature = "browser-wallet")]
            Self::BrowserWallet(error) => classify_browser_wallet(error),
        }
    }
}

impl From<cow_sdk_core::Cancelled> for SdkError {
    fn from(cancelled: cow_sdk_core::Cancelled) -> Self {
        Self::Types(cow_sdk_core::CoreError::from(cancelled))
    }
}

const fn classify_core(error: &cow_sdk_core::CoreError) -> ErrorClass {
    match error {
        cow_sdk_core::CoreError::Validation(_) | cow_sdk_core::CoreError::MissingBaseUrl { .. } => {
            ErrorClass::Validation
        }
        cow_sdk_core::CoreError::Cancelled => ErrorClass::Cancelled,
        // Serialization and transport-contract failures plus any future
        // additive variants signal invariant violations, so they are
        // classified as internal.
        _ => ErrorClass::Internal,
    }
}

const fn classify_app_data(error: &cow_sdk_app_data::AppDataError) -> ErrorClass {
    match error {
        cow_sdk_app_data::AppDataError::InvalidAppDataHex
        | cow_sdk_app_data::AppDataError::InvalidCid
        | cow_sdk_app_data::AppDataError::InvalidSchemaVersion(_)
        | cow_sdk_app_data::AppDataError::UnknownSchemaVersion(_)
        | cow_sdk_app_data::AppDataError::MissingSchemaVersion
        | cow_sdk_app_data::AppDataError::InvalidAppDataProvided { .. }
        | cow_sdk_app_data::AppDataError::MissingIpfsCredentials
        | cow_sdk_app_data::AppDataError::TooLarge { .. } => ErrorClass::Validation,
        cow_sdk_app_data::AppDataError::Transport { .. }
        | cow_sdk_app_data::AppDataError::Pinning { .. } => ErrorClass::Transport,
        // Json, Schema, Calculation failures plus any future additive
        // variants signal invariant violations and classify as internal.
        _ => ErrorClass::Internal,
    }
}

const fn classify_orderbook(error: &cow_sdk_orderbook::OrderbookError) -> ErrorClass {
    match error {
        cow_sdk_orderbook::OrderbookError::Core(core_error) => classify_core(core_error),
        cow_sdk_orderbook::OrderbookError::Api(_) => ErrorClass::Remote,
        cow_sdk_orderbook::OrderbookError::Transport { .. } => ErrorClass::Transport,
        cow_sdk_orderbook::OrderbookError::InvalidTradesQuery { .. }
        | cow_sdk_orderbook::OrderbookError::InvalidQuoteRequest { .. } => ErrorClass::Validation,
        cow_sdk_orderbook::OrderbookError::Cancelled => ErrorClass::Cancelled,
        // Serialization and transform failures plus any future additive
        // variants classify as internal.
        _ => ErrorClass::Internal,
    }
}

const fn classify_trading(error: &cow_sdk_trading::TradingError) -> ErrorClass {
    match error {
        cow_sdk_trading::TradingError::Core(core_error) => classify_core(core_error),
        cow_sdk_trading::TradingError::AppData(app_data_error) => classify_app_data(app_data_error),
        cow_sdk_trading::TradingError::Orderbook(orderbook_error) => {
            classify_orderbook(orderbook_error)
        }
        cow_sdk_trading::TradingError::Signing(signing_error) => classify_signing(signing_error),
        cow_sdk_trading::TradingError::Contracts(_)
        | cow_sdk_trading::TradingError::Signer { .. }
        | cow_sdk_trading::TradingError::Provider { .. } => ErrorClass::Signing,
        cow_sdk_trading::TradingError::Cancelled => ErrorClass::Cancelled,
        // Every remaining variant represents a caller-side input failure
        // (missing parameters, validity conflicts, owner mismatches, or
        // numeric input failures) and classifies as validation. Future
        // additive validation variants fall through the same arm.
        _ => ErrorClass::Validation,
    }
}

const fn classify_signing(error: &cow_sdk_signing::SigningError) -> ErrorClass {
    match error {
        cow_sdk_signing::SigningError::Core(core_error) => classify_core(core_error),
        cow_sdk_signing::SigningError::Cancelled => ErrorClass::Cancelled,
        // Contracts, Serialization, Signer, and UnsupportedSignerGeneratedScheme
        // failures plus any future additive variants classify as signing.
        _ => ErrorClass::Signing,
    }
}

#[cfg(feature = "browser-wallet")]
const fn classify_browser_wallet(error: &cow_sdk_browser_wallet::BrowserWalletError) -> ErrorClass {
    match error {
        cow_sdk_browser_wallet::BrowserWalletError::Core(core_error) => classify_core(core_error),
        cow_sdk_browser_wallet::BrowserWalletError::Cancelled => ErrorClass::Cancelled,
        // Every other typed wallet failure (user rejection, disconnected,
        // wrong chain, malformed response, JS interop, serialization, or
        // unclassified RPC payload) plus any future additive variants
        // classify as signing because they surface from the signing edge.
        _ => ErrorClass::Signing,
    }
}
