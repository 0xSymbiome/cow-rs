#![forbid(unsafe_code)]
#![cfg_attr(doctest, doc = include_str!("../README.md"))]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! Shared HTTP transport policy for `CoW` Protocol SDK clients.
//!
//! This crate carries the retry, jitter, rate-limit, `Retry-After`, and
//! transport-error classification contracts used by the orderbook,
//! subgraph, and IPFS clients, and the shared [`run_with_retry`] driver that
//! runs every attempt through one retry loop so that behavior is defined once
//! rather than per client. The API is target-neutral: native builds use
//! `futures-timer` for sleeps and the standard wall clock, while browser
//! builds use `gloo-timers` and read the wall clock through
//! [`system_now`] so the retry path never aborts a wasm runtime.

pub mod classify;
pub mod jitter;
pub mod policy;
pub mod rate_limit;
pub mod retry;
pub mod retry_after;
pub mod runner;
pub mod status;
pub mod time;

#[cfg(all(feature = "reqwest-classifier", not(target_arch = "wasm32")))]
#[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "reqwest-classifier", not(target_arch = "wasm32"))))
)]
pub use classify::ReqwestErrorClassifier;
pub use classify::{ErrorClassifier, NetworkErrorKind};
pub use jitter::JitterStrategy;
pub use policy::{
    DEFAULT_IPFS_USER_AGENT, DEFAULT_ORDERBOOK_USER_AGENT, DEFAULT_SUBGRAPH_USER_AGENT,
    DEFAULT_TRADING_USER_AGENT, IPFS_MAX_RESPONSE_BYTES, SUBGRAPH_MAX_RESPONSE_BYTES,
    TransportPolicy, TransportPolicyBuildError, TransportPolicyBuilder,
};
pub use rate_limit::{DEFAULT_INTERVAL_LABEL, DEFAULT_TOKENS_PER_INTERVAL};
pub use rate_limit::{LimiterScope, RequestRateLimiter, RequestRateLimiterBuilder};
pub use retry::{DEFAULT_MAX_ATTEMPTS, RetryPolicy, RetryPolicyBuilder};
pub use retry_after::{RetryAfter, parse_retry_after};
pub use runner::{AttemptOutcome, LimiterKey, RetrySignal, run_with_retry};
pub use status::{
    BAD_GATEWAY, GATEWAY_TIMEOUT, INTERNAL_SERVER_ERROR, REQUEST_TIMEOUT, RETRYABLE_STATUSES,
    SERVICE_UNAVAILABLE, TOO_EARLY, TOO_MANY_REQUESTS, is_retryable_status,
};
pub use time::{sleep, system_now};
