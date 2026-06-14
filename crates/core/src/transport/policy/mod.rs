//! Shared HTTP transport policy for `CoW` Protocol SDK clients.
//!
//! This module carries the retry, jitter, rate-limit, `Retry-After`, and
//! transport-error classification contracts used by the orderbook, subgraph,
//! and IPFS clients, and the shared [`run_with_retry`] driver that runs every
//! attempt through one retry loop so that behavior is defined once rather than
//! per client. The API is target-neutral: native builds use `futures-timer`
//! for sleeps and the standard wall clock, while browser builds use
//! `gloo-timers` and read the wall clock through [`system_now`] so the retry
//! path never aborts a wasm runtime.
//!
//! The module is gated behind the `transport-policy` feature so a `cow-sdk-core`
//! consumer that needs only the primitive types does not pull the retry-timer
//! dependencies.
//!
//! ```
//! use std::time::Duration;
//!
//! use cow_sdk_core::transport::policy::{JitterStrategy, RetryPolicy, TransportPolicy};
//!
//! let retry = RetryPolicy::builder()
//!     .max_attempts(4)
//!     .base_delay(Duration::from_millis(100))
//!     .jitter(JitterStrategy::decorrelated_from_seed(7))
//!     .build();
//!
//! let policy = TransportPolicy::default_orderbook().with_retry(retry);
//!
//! assert_eq!(policy.retry().max_attempts(), 4);
//! ```

pub mod classify;
pub mod config;
pub mod jitter;
pub mod rate_limit;
pub mod retry;
pub mod retry_after;
pub mod runner;
pub mod status;
pub mod time;

pub use classify::NetworkErrorKind;
pub use config::{
    DEFAULT_IPFS_USER_AGENT, DEFAULT_ORDERBOOK_USER_AGENT, DEFAULT_SUBGRAPH_USER_AGENT,
    DEFAULT_TRADING_USER_AGENT, IPFS_MAX_RESPONSE_BYTES, SUBGRAPH_MAX_RESPONSE_BYTES,
    TransportPolicy, TransportPolicyBuilder,
};
pub use jitter::JitterStrategy;
pub use rate_limit::{DEFAULT_INTERVAL_LABEL, DEFAULT_TOKENS_PER_INTERVAL};
pub use rate_limit::{LimiterScope, RequestRateLimiter, RequestRateLimiterBuilder};
pub use retry::{DEFAULT_MAX_ATTEMPTS, RetryPolicy, RetryPolicyBuilder, is_retryable_network};
pub use retry_after::{RetryAfter, parse_retry_after, retry_after_from_headers};
pub use runner::{AttemptOutcome, LimiterKey, RetrySignal, run_with_retry};
pub use status::{RETRYABLE_STATUSES, is_retryable_status};
pub use time::{sleep, system_now};
