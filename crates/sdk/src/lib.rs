#![cfg_attr(any(doctest, docsrs), doc = include_str!("../README.md"))]

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
//! Native/default ready-state setup:
//!
//! ```rust
//! use cow_sdk::{Address, SupportedChainId, Trading};
//!
//! let _address = Address::new("0x1111111111111111111111111111111111111111").unwrap();
//! let _sdk = Trading::builder()
//!     .with_chain_id(SupportedChainId::Sepolia)
//!     .with_app_code("your-app-code")
//!     .build_ready()
//!     .unwrap();
//! ```
//!
//! For allowance, approval, pre-sign, or on-chain cancellation helpers that do
//! not need quote or submission flows, construct a helper-only SDK:
//!
//! ```rust
//! use cow_sdk::{SupportedChainId, Trading};
//!
//! let _sdk = Trading::builder()
//!     .with_chain_id(SupportedChainId::Sepolia)
//!     .build_helper_only()
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

#[cfg(all(
    target_arch = "wasm32",
    any(
        feature = "alloy",
        feature = "alloy-provider",
        feature = "alloy-signer"
    )
))]
compile_error!(
    "the alloy / alloy-provider / alloy-signer features on cow-sdk are for native targets only"
);

/// Curated re-exports for the default `cow-sdk` facade.
pub mod prelude;

pub use prelude::*;

#[cfg(all(feature = "alloy", not(target_arch = "wasm32")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloy")))]
pub use cow_sdk_alloy as alloy;
#[cfg(all(feature = "alloy-provider", not(target_arch = "wasm32")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloy-provider")))]
pub use cow_sdk_alloy_provider as alloy_provider;
#[cfg(all(feature = "alloy-signer", not(target_arch = "wasm32")))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloy-signer")))]
pub use cow_sdk_alloy_signer as alloy_signer;
pub use cow_sdk_app_data as app_data;
#[cfg(feature = "browser-wallet")]
#[cfg_attr(docsrs, doc(cfg(feature = "browser-wallet")))]
pub use cow_sdk_browser_wallet as browser_wallet;
pub use cow_sdk_contracts as contracts;
/// Typed [`RegistryError`] surface produced by the runtime registry
/// loader, re-exported on the facade so downstream consumers can match
/// against every failure mode without reaching into the contracts crate
/// directly.
pub use cow_sdk_contracts::RegistryError;
pub use cow_sdk_core as core;
/// Shared HTTP retry, rate-limit, and classification policy.
pub mod http {
    pub use cow_sdk_transport_policy::{
        ErrorClassifier, JitterStrategy, LimiterScope, NetworkErrorKind, RequestRateLimiter,
        RequestRateLimiterBuilder, RetryAfter, RetryPolicy, RetryPolicyBuilder, TransportPolicy,
        TransportPolicyBuildError, TransportPolicyBuilder, is_retryable_status, parse_retry_after,
    };

    #[cfg(all(feature = "http-classifier", not(target_arch = "wasm32")))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(feature = "http-classifier", not(target_arch = "wasm32"))))
    )]
    pub use cow_sdk_transport_policy::ReqwestErrorClassifier;
}
/// Transport-error classification shared across transport-capable crates.
///
/// Typed label that downstream telemetry and retry layers can use to
/// partition REST-transport failures without parsing error messages.
pub use cow_sdk_core::TransportErrorClass;
/// Production HTTP transport surface shared across `cow-sdk` crates.
///
/// [`HttpTransport`] is the async injection point downstream clients
/// consume; [`TransportError`] is its typed failure surface. The native
/// default implementation is `ReqwestTransport`; the browser default
/// lives in `cow-sdk-transport-wasm`.
pub use cow_sdk_core::{HttpTransport, TransportError};
#[cfg(not(target_arch = "wasm32"))]
pub use cow_sdk_core::{ReqwestTransport, ReqwestTransportConfig};
pub use cow_sdk_orderbook as orderbook;
pub use cow_sdk_signing as signing;
#[cfg(feature = "in-memory-cache")]
#[cfg_attr(docsrs, doc(cfg(feature = "in-memory-cache")))]
pub use cow_sdk_signing::InMemoryEip1271VerificationCache;
/// Optional caching seam for EIP-1271 signature verification.
///
/// [`Eip1271VerificationCache`] is the trait consumed by
/// [`cow_sdk_contracts::verify_eip1271_signature_cached`].
/// [`NoopEip1271VerificationCache`] is the always-available zero-sized
/// default for callers that do not want caching. The TTL-respecting,
/// capacity-bounded `InMemoryEip1271VerificationCache` is re-exported only
/// when the opt-in `in-memory-cache` feature is enabled.
pub use cow_sdk_signing::{Eip1271VerificationCache, NoopEip1271VerificationCache};
pub use cow_sdk_trading as trading;
/// Browser-native HTTP transport surface — the `wasm32` sibling of the native
/// `ReqwestTransport` default. [`FetchTransport`] is the browser default
/// implementation of [`HttpTransport`]; compose it into typed clients as
/// `Arc<dyn HttpTransport + Send + Sync>` exactly like the native transport.
#[cfg(target_arch = "wasm32")]
#[cfg_attr(docsrs, doc(cfg(target_arch = "wasm32")))]
pub use cow_sdk_transport_wasm::{FetchTransport, FetchTransportConfig};

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm")))]
/// TypeScript-callable WASM surface plus the host-safe protocol helpers.
///
/// `pure_helpers` is reachable here on both targets so a single
/// `cow_sdk::wasm::pure_helpers` path works whether the crate is built for
/// the host or for `wasm32`.
pub mod wasm {
    /// Host-safe protocol helper modules shared with the WASM crate.
    pub use cow_sdk_pure_helpers as pure_helpers;
    pub use cow_sdk_wasm::*;
}

#[cfg(all(feature = "wasm", not(target_arch = "wasm32")))]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm")))]
/// Host-safe subset of the TypeScript-callable WASM crate.
pub mod wasm {
    /// Host-safe protocol helper modules shared with the WASM crate.
    pub use cow_sdk_pure_helpers as pure_helpers;
}

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
    /// The remote endpoint signalled rate limiting (HTTP 429) and the
    /// transport layer's retry budget was exhausted before it cleared.
    ///
    /// Transport retries already honor `Retry-After`, so reaching this
    /// class means the throttle outlived the retry policy rather than a
    /// transient spike the client absorbed.
    RateLimited,
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
            Self::Contracts(error) => classify_contracts(error),
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
        cow_sdk_app_data::AppDataError::Cancelled => ErrorClass::Cancelled,
        // Json, Schema, Calculation failures plus any future additive
        // variants signal invariant violations and classify as internal.
        _ => ErrorClass::Internal,
    }
}

const fn classify_orderbook(error: &cow_sdk_orderbook::OrderbookError) -> ErrorClass {
    match error {
        cow_sdk_orderbook::OrderbookError::Core(core_error) => classify_core(core_error),
        cow_sdk_orderbook::OrderbookError::Rejected { status, .. } if status.as_u16() == 429 => {
            ErrorClass::RateLimited
        }
        cow_sdk_orderbook::OrderbookError::Api(error) if error.status == 429 => {
            ErrorClass::RateLimited
        }
        cow_sdk_orderbook::OrderbookError::Api(_)
        | cow_sdk_orderbook::OrderbookError::Rejected { .. } => ErrorClass::Remote,
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
        cow_sdk_trading::TradingError::Contracts(contracts_error) => {
            classify_contracts(contracts_error)
        }
        cow_sdk_trading::TradingError::Signer { .. }
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

const fn classify_contracts(error: &cow_sdk_contracts::ContractsError) -> ErrorClass {
    match error {
        cow_sdk_contracts::ContractsError::Core(core_error) => classify_core(core_error),
        cow_sdk_contracts::ContractsError::Cancelled => ErrorClass::Cancelled,
        // Contract encoding, ABI, provider, signature, and EIP-1271 failures
        // plus future additive variants classify as signing-edge failures.
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
