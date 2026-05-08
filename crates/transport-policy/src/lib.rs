#![forbid(unsafe_code)]
#![cfg_attr(doctest, doc = include_str!("../README.md"))]

//! Shared HTTP transport policy for `CoW` Protocol SDK clients.
//!
//! This crate carries the retry, jitter, rate-limit, `Retry-After`, and
//! transport-error classification contracts used by the orderbook and
//! subgraph clients. The API is target-neutral: native builds use
//! `futures-timer` for sleeps, while browser builds use `gloo-timers`.

pub mod classify;
pub mod jitter;
pub mod policy;
pub mod rate_limit;
pub mod retry;
pub mod retry_after;
pub mod status;
pub mod time;

#[cfg(feature = "reqwest-classifier")]
#[cfg_attr(docsrs, doc(cfg(feature = "reqwest-classifier")))]
pub use classify::ReqwestErrorClassifier;
pub use classify::{ErrorClassifier, NetworkErrorKind};
pub use jitter::JitterStrategy;
pub use policy::{
    DEFAULT_ORDERBOOK_USER_AGENT, DEFAULT_SUBGRAPH_USER_AGENT, TransportPolicy,
    TransportPolicyBuildError, TransportPolicyBuilder,
};
pub use rate_limit::{DEFAULT_INTERVAL_LABEL, DEFAULT_TOKENS_PER_INTERVAL};
pub use rate_limit::{LimiterScope, RequestRateLimiter, RequestRateLimiterBuilder};
pub use retry::{DEFAULT_MAX_ATTEMPTS, RetryPolicy, RetryPolicyBuilder};
pub use retry_after::{RetryAfter, parse_retry_after};
pub use status::{
    BAD_GATEWAY, GATEWAY_TIMEOUT, INTERNAL_SERVER_ERROR, REQUEST_TIMEOUT, RETRYABLE_STATUSES,
    SERVICE_UNAVAILABLE, TOO_EARLY, TOO_MANY_REQUESTS, is_retryable_status,
};
pub use time::sleep;
