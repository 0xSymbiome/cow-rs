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
//! The facade is trading-first: the high-level trading flow is the primary surface.
//! Optional browser-runtime support does not change the default facade identity.
//! Browser-wallet support is additive behind the `browser-wallet` feature,
//! and the full browser-runtime contract stays in `cow-sdk-browser-wallet`.
//!
//! Read-only subgraph analytics are available behind the off-by-default
//! `subgraph` feature as `cow_sdk::subgraph`; the full subgraph contract stays
//! in `cow-sdk-subgraph`.
//!
//! Native/default ready-state setup:
//!
//! ```rust
//! use cow_sdk::core::{Address, SupportedChainId, address};
//! use cow_sdk::trading::Trading;
//!
//! // Compile-time validated address literal — no runtime parse, no unwrap.
//! // The literal is the lowercase wire form; mixed-case literals reject at
//! // build time because EIP-55 checksums cannot be verified in const eval.
//! const SETTLEMENT: Address = address!("0x9008d19f58aabd9ed0d60971565aa8510560ab41");
//!
//! let _trading = Trading::builder()
//!     .chain_id(SupportedChainId::Sepolia)
//!     .app_code("your-app-code")
//!     .build()
//!     .unwrap();
//! ```
//!
//! Once constructed, a single call quotes, signs, and posts a swap. The order
//! owner defaults to the signer's address:
//!
//! ```rust,no_run
//! # use std::error::Error;
//! use cow_sdk::core::{Address, Amount, OrderKind, SupportedChainId, address};
//! use cow_sdk::trading::{TradeParams, Trading};
//!
//! // Sell 0.1 WETH for COW on Sepolia.
//! const WETH: Address = address!("0xfff9976782d46cc05630d1f6ebab18b2324d6b14");
//! const COW: Address = address!("0x0625afb445c3b6b7b929342a04a22599fd5dbb59");
//! #
//! # async fn run<S>(signer: &S) -> Result<(), Box<dyn Error>>
//! # where
//! #     S: cow_sdk::core::Signer,
//! #     S::Error: std::fmt::Display + cow_sdk::core::SignerError,
//! # {
//! let trading = Trading::builder()
//!     .chain_id(SupportedChainId::Sepolia)
//!     .app_code("your-app-code")
//!     .build()?;
//!
//! let params = TradeParams::new(
//!     OrderKind::Sell,
//!     WETH,
//!     COW,
//!     Amount::from(100_000_000_000_000_000u128),
//! );
//!
//! // One call quotes, signs with `signer`, and posts to the orderbook.
//! let posted = trading.post_swap_order(params, signer, None).await?;
//! println!("posted order: {}", posted.order_id);
//! # Ok(())
//! # }
//! ```
//!
//! For allowance, approval, pre-sign, or on-chain cancellation that does not
//! need an app code, call the crate's free functions directly
//! (`cow_protocol_allowance`, `approval_transaction`,
//! `pre_sign_transaction`, `onchain_cancel_order`) without constructing a
//! trading client.
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

// The `cow-sdk` crate root is a thin, module-organised facade: each leaf crate
// is re-exported as a named module (`core`, `trading`, `orderbook`, `signing`,
// `contracts`, `app_data`, …), and every workflow and identity type is reached
// on its module path (`cow_sdk::core::Address`, `cow_sdk::trading::Trading`),
// matching `alloy`, `reqwest`, and `tower`. The crate root itself carries only
// the cross-cutting aggregate error (`CowError` / `ErrorClass`, below) and the
// typed transport, registry, and EIP-1271 cache leaf surfaces consumers match
// against. There is no facade prelude; the workspace's identity prelude is the
// opt-in `cow_sdk::core::prelude` (the cow primitive newtypes, ADR 0052).

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
/// Opt-in COW Shed account-abstraction hook helpers (proxy derivation,
/// EIP-712 signing, factory calldata, and the [`cow_shed::CowShedHooks`]
/// orchestrator). Behind the off-by-default `cow-shed` feature, so the default
/// `cow-sdk` surface stays trading-first; enable it with
/// `cow-sdk = { features = ["cow-shed"] }`.
#[cfg(feature = "cow-shed")]
#[cfg_attr(docsrs, doc(cfg(feature = "cow-shed")))]
pub use cow_sdk_contracts::cow_shed;
pub use cow_sdk_core as core;
/// Shared HTTP retry, rate-limit, and classification policy.
pub mod http {
    pub use cow_sdk_core::transport::policy::{
        ErrorClassifier, JitterStrategy, LimiterScope, NetworkErrorKind, RequestRateLimiter,
        RequestRateLimiterBuilder, RetryAfter, RetryPolicy, RetryPolicyBuilder, TransportPolicy,
        TransportPolicyBuildError, TransportPolicyBuilder, is_retryable_status, parse_retry_after,
    };

    /// Production HTTP transport seam and its typed failure surface.
    ///
    /// [`HttpTransport`] is the async injection point downstream clients
    /// consume; [`TransportError`] is its typed failure surface, and
    /// [`TransportErrorClass`] is the label telemetry and retry layers use to
    /// partition REST-transport failures without parsing error messages. The
    /// native default implementation is [`ReqwestTransport`]; the browser
    /// default lives in `cow-sdk-transport-wasm`.
    pub use cow_sdk_core::{HttpTransport, TransportError, TransportErrorClass};
    /// Native default HTTP transport implementation and its configuration.
    #[cfg(not(target_arch = "wasm32"))]
    pub use cow_sdk_core::{ReqwestTransport, ReqwestTransportConfig};

    #[cfg(all(feature = "http-classifier", not(target_arch = "wasm32")))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(feature = "http-classifier", not(target_arch = "wasm32"))))
    )]
    pub use cow_sdk_core::transport::policy::ReqwestErrorClassifier;
}
pub use cow_sdk_orderbook as orderbook;
pub use cow_sdk_signing as signing;
/// Optional read-only subgraph analytics (protocol totals, daily and hourly
/// volume, and a typed raw-GraphQL escape hatch). Behind the off-by-default
/// `subgraph` feature so the default facade stays trading-first; enable it with
/// `cow-sdk = { features = ["subgraph"] }`. The full subgraph contract stays in
/// `cow-sdk-subgraph`.
///
/// ```
/// # #[cfg(not(target_arch = "wasm32"))]
/// # {
/// use cow_sdk::core::SupportedChainId;
/// use cow_sdk::subgraph::SubgraphApi;
///
/// let _subgraph = SubgraphApi::builder()
///     .chain(SupportedChainId::Mainnet)
///     .api_key("your-subgraph-api-key")
///     .build()
///     .expect("subgraph client builds with canonical defaults");
/// # }
/// ```
#[cfg(feature = "subgraph")]
#[cfg_attr(docsrs, doc(cfg(feature = "subgraph")))]
pub use cow_sdk_subgraph as subgraph;
/// In-memory test doubles for the SDK public trait seams, for use from a
/// consumer's `[dev-dependencies]`. Enabled by the opt-in `testing` feature and
/// off by default, so the doubles never enter a production dependency graph
/// (ADR 0063).
#[cfg(feature = "testing")]
#[cfg_attr(docsrs, doc(cfg(feature = "testing")))]
pub use cow_sdk_test as testing;
pub use cow_sdk_trading as trading;
/// Browser-native HTTP transport surface — the `wasm32` sibling of the native
/// `ReqwestTransport` default. [`FetchTransport`] is the browser default
/// implementation of [`HttpTransport`](crate::http::HttpTransport); compose it
/// into typed clients as `Arc<dyn cow_sdk_core::HttpTransport + Send + Sync>`
/// exactly like the native transport.
#[cfg(target_arch = "wasm32")]
#[cfg_attr(docsrs, doc(cfg(target_arch = "wasm32")))]
pub use cow_sdk_transport_wasm::{FetchTransport, FetchTransportConfig};

#[cfg(feature = "wasm")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm")))]
/// TypeScript-callable WASM surface plus the host-safe protocol helpers.
///
/// `helpers` is reachable here on both targets so a single
/// `cow_sdk::wasm::helpers` path works whether the crate is built for
/// the host or for `wasm32`. The `wasm32`-only JavaScript ABI lives under
/// `cow_sdk::wasm::exports`.
pub mod wasm {
    /// JavaScript ABI surface, available only on `wasm32` targets.
    #[cfg(target_arch = "wasm32")]
    pub use cow_sdk_wasm::exports;
    /// Host-safe protocol helper modules shared with the WASM crate.
    pub use cow_sdk_wasm::helpers;
}

use thiserror::Error;

/// Aggregate error type for the root facade crate.
///
/// `CowError` is the convenience aggregate for consumers that `?`-propagate
/// every SDK call into one type; each leaf error converts in through `#[from]`.
/// A consumer with its own error type, or that needs rejection-specific
/// handling, can match the leaf error directly — every leaf exposes the same
/// [`ErrorClass`] through `class()` (and the orderbook and trading errors also
/// expose `is_retryable()` / `backoff_hint()`), so the verdict is identical
/// whether a caller holds the facade error or a bare leaf.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CowError {
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
    #[cfg(feature = "subgraph")]
    /// Subgraph transport, GraphQL, or decoding error.
    #[error("subgraph error: {0}")]
    Subgraph(#[from] cow_sdk_subgraph::SubgraphError),
}

/// Coarse-grained failure classification, re-exported from `cow-sdk-core`.
///
/// Every public error type the facade aggregates exposes a matching
/// `class()` accessor, so the classification is consistent whether a caller
/// holds the facade [`CowError`] or a bare leaf error.
pub use cow_sdk_core::ErrorClass;

impl CowError {
    /// Returns the coarse-grained class for this error.
    ///
    /// The classification is exhaustive: every supported variant resolves to
    /// one of the [`ErrorClass`] buckets without falling through to a
    /// default arm, so downstream telemetry layers can rely on the mapping
    /// staying stable across releases.
    #[must_use]
    pub const fn class(&self) -> ErrorClass {
        match self {
            Self::Types(error) => error.class(),
            Self::Signing(error) => error.class(),
            Self::AppData(error) => error.class(),
            Self::Contracts(error) => error.class(),
            Self::Orderbook(error) => error.class(),
            Self::Trading(error) => error.class(),
            #[cfg(feature = "browser-wallet")]
            Self::BrowserWallet(error) => error.class(),
            #[cfg(feature = "subgraph")]
            Self::Subgraph(error) => error.class(),
        }
    }

    /// Returns `true` when retrying the same request may succeed.
    ///
    /// The orderbook and trading errors carry the HTTP retry classification, so
    /// the verdict delegates to their `is_retryable()` accessors; every other
    /// facade variant is never retryable. Pair it with
    /// [`CowError::backoff_hint`] for the suggested wait before the next
    /// attempt.
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        match self {
            Self::Orderbook(error) => error.is_retryable(),
            Self::Trading(error) => error.is_retryable(),
            _ => false,
        }
    }

    /// Returns the server-suggested backoff before the next attempt, when the
    /// failing orderbook response carried a `Retry-After` header.
    ///
    /// Delegates to the orderbook and trading errors; returns [`None`] for
    /// every other facade variant and for responses without a `Retry-After`
    /// header.
    #[must_use]
    pub fn backoff_hint(&self) -> Option<std::time::Duration> {
        match self {
            Self::Orderbook(error) => error.backoff_hint(),
            Self::Trading(error) => error.backoff_hint(),
            _ => None,
        }
    }
}

impl From<cow_sdk_core::Cancelled> for CowError {
    fn from(cancelled: cow_sdk_core::Cancelled) -> Self {
        Self::Types(cow_sdk_core::CoreError::from(cancelled))
    }
}

// The per-variant classification now lives on each leaf error type's
// `class()` accessor; `CowError::class()` above delegates to them.
