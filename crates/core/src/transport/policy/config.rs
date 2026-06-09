//! Combined transport policy bundle.

use std::time::Duration;

use crate::{HttpClientPolicy, ValidationError};
use thiserror::Error;

use crate::transport::policy::{JitterStrategy, RequestRateLimiter, RetryPolicy};

const DEFAULT_JITTER_SEED: u64 = 0xC0DE_CAFE_5EED_0001;

/// Default orderbook user-agent string.
pub const DEFAULT_ORDERBOOK_USER_AGENT: &str =
    concat!("cow-sdk-orderbook", "/", env!("CARGO_PKG_VERSION"));
/// Default subgraph user-agent string.
pub const DEFAULT_SUBGRAPH_USER_AGENT: &str =
    concat!("cow-sdk-subgraph", "/", env!("CARGO_PKG_VERSION"));
/// Default trading user-agent string.
pub const DEFAULT_TRADING_USER_AGENT: &str =
    concat!("cow-sdk-trading", "/", env!("CARGO_PKG_VERSION"));
/// Default IPFS user-agent string.
pub const DEFAULT_IPFS_USER_AGENT: &str = concat!("cow-sdk-ipfs", "/", env!("CARGO_PKG_VERSION"));

/// Maximum response-body size buffered from the subgraph gateway, in bytes.
///
/// The subgraph is untrusted third-party infrastructure, but the SDK's
/// subgraph queries return small aggregate documents, so this generous
/// headroom never rejects a legitimate response while still bounding a
/// hostile or misbehaving gateway.
pub const SUBGRAPH_MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
/// Maximum response-body size buffered from an IPFS gateway, in bytes.
///
/// Sized at twice the protocol app-data document limit so encoding and
/// framing overhead never rejects a valid document, while bounding a hostile
/// gateway to a small read.
pub const IPFS_MAX_RESPONSE_BYTES: usize = 16 * 1024;

/// Combined HTTP client, retry, rate-limit, and tracing policy.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportPolicy {
    client: HttpClientPolicy,
    retry: RetryPolicy,
    rate_limit: RequestRateLimiter,
    tracing_enabled: bool,
}

impl Default for TransportPolicy {
    fn default() -> Self {
        Self::default_orderbook()
    }
}

impl TransportPolicy {
    /// Creates a policy from explicit component policies.
    #[must_use]
    pub const fn new(
        client: HttpClientPolicy,
        retry: RetryPolicy,
        rate_limit: RequestRateLimiter,
    ) -> Self {
        Self {
            client,
            retry,
            rate_limit,
            tracing_enabled: false,
        }
    }

    /// Returns the documented default orderbook transport policy.
    ///
    /// # Panics
    ///
    /// Panics only if the crate-owned default orderbook user-agent literal
    /// stops being encodable as an HTTP header value.
    #[must_use]
    pub fn default_orderbook() -> Self {
        Self {
            // SAFETY: this crate-owned user-agent literal is static and
            // validated by the HTTP header parser.
            client: HttpClientPolicy::new(DEFAULT_ORDERBOOK_USER_AGENT)
                .expect("static orderbook user-agent must remain valid"),
            retry: RetryPolicy::builder()
                .jitter(JitterStrategy::decorrelated_from_seed(DEFAULT_JITTER_SEED))
                .build(),
            rate_limit: RequestRateLimiter::default_orderbook(),
            tracing_enabled: false,
        }
    }

    /// Returns the documented default subgraph transport policy.
    ///
    /// # Panics
    ///
    /// Panics only if the crate-owned default subgraph user-agent literal
    /// stops being encodable as an HTTP header value.
    #[must_use]
    pub fn default_subgraph() -> Self {
        Self {
            // SAFETY: this crate-owned user-agent literal is static and
            // validated by the HTTP header parser.
            client: HttpClientPolicy::new(DEFAULT_SUBGRAPH_USER_AGENT)
                .expect("static subgraph user-agent must remain valid")
                .with_max_response_bytes(SUBGRAPH_MAX_RESPONSE_BYTES),
            retry: RetryPolicy::builder()
                .jitter(JitterStrategy::decorrelated_from_seed(DEFAULT_JITTER_SEED))
                .build(),
            rate_limit: RequestRateLimiter::default_subgraph(),
            tracing_enabled: false,
        }
    }

    /// Returns the documented default trading transport policy.
    ///
    /// Trading currently routes HTTP through the orderbook client, so this
    /// preserves the same retry and limiter behavior with a trading-specific
    /// client policy label.
    ///
    /// # Panics
    ///
    /// Panics only if the crate-owned default trading user-agent literal stops
    /// being encodable as an HTTP header value.
    #[must_use]
    pub fn default_trading() -> Self {
        Self {
            // SAFETY: this crate-owned user-agent literal is static and
            // validated by the HTTP header parser.
            client: HttpClientPolicy::new(DEFAULT_TRADING_USER_AGENT)
                .expect("static trading user-agent must remain valid"),
            retry: RetryPolicy::builder()
                .jitter(JitterStrategy::decorrelated_from_seed(DEFAULT_JITTER_SEED))
                .build(),
            rate_limit: RequestRateLimiter::default_orderbook(),
            tracing_enabled: false,
        }
    }

    /// Returns the documented default IPFS transport policy.
    ///
    /// IPFS reads historically performed one direct fetch with no SDK-owned
    /// retry, rate limiting, or default timeout, so the default policy keeps
    /// those behaviors disabled unless a caller opts in.
    ///
    /// # Panics
    ///
    /// Panics only if the crate-owned default IPFS user-agent literal stops
    /// being encodable as an HTTP header value.
    #[must_use]
    pub fn default_ipfs() -> Self {
        Self {
            // SAFETY: this crate-owned user-agent literal is static and
            // validated by the HTTP header parser.
            client: HttpClientPolicy::new(DEFAULT_IPFS_USER_AGENT)
                .expect("static IPFS user-agent must remain valid")
                .without_timeout()
                .with_max_response_bytes(IPFS_MAX_RESPONSE_BYTES),
            retry: RetryPolicy::no_retry(),
            rate_limit: RequestRateLimiter::unlimited(),
            tracing_enabled: false,
        }
    }

    /// Returns a builder seeded with orderbook defaults.
    #[must_use]
    pub fn builder() -> TransportPolicyBuilder {
        TransportPolicyBuilder::default()
    }

    /// Returns the shared HTTP client policy.
    #[must_use]
    pub const fn client_policy(&self) -> &HttpClientPolicy {
        &self.client
    }

    /// Returns the retry policy.
    #[must_use]
    pub const fn retry(&self) -> &RetryPolicy {
        &self.retry
    }

    /// Returns the request rate limiter.
    #[must_use]
    pub const fn rate_limit(&self) -> &RequestRateLimiter {
        &self.rate_limit
    }

    /// Returns whether tracing integration is enabled.
    #[must_use]
    pub const fn tracing_enabled(&self) -> bool {
        self.tracing_enabled
    }

    /// Returns the configured request timeout.
    #[must_use]
    pub const fn timeout(&self) -> Option<Duration> {
        self.client.timeout()
    }

    /// Returns the configured user-agent.
    #[must_use]
    pub fn user_agent(&self) -> &str {
        self.client.user_agent()
    }

    /// Returns a copy of this policy with a new HTTP client policy.
    #[must_use]
    pub fn with_client_policy(mut self, client: HttpClientPolicy) -> Self {
        self.client = client;
        self
    }

    /// Returns a copy of this policy with a new retry policy.
    #[must_use]
    pub const fn with_retry(mut self, retry: RetryPolicy) -> Self {
        self.retry = retry;
        self
    }

    /// Returns a copy of this policy with a new rate limiter.
    #[must_use]
    pub fn with_rate_limit(mut self, rate_limit: RequestRateLimiter) -> Self {
        self.rate_limit = rate_limit;
        self
    }

    /// Returns a copy of this policy with tracing enabled or disabled.
    #[must_use]
    pub const fn with_tracing_enabled(mut self, tracing_enabled: bool) -> Self {
        self.tracing_enabled = tracing_enabled;
        self
    }
}

/// Error returned when building a [`TransportPolicy`].
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum TransportPolicyBuildError {
    /// Shared HTTP client policy validation failed.
    #[error(transparent)]
    Client(#[from] ValidationError),
}

/// Builder for [`TransportPolicy`].
#[derive(Debug, Clone)]
pub struct TransportPolicyBuilder {
    client: Option<HttpClientPolicy>,
    retry: RetryPolicy,
    rate_limit: RequestRateLimiter,
    tracing_enabled: bool,
}

impl TransportPolicyBuilder {
    /// Creates a builder seeded with orderbook defaults.
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: None,
            retry: RetryPolicy::builder()
                .jitter(JitterStrategy::decorrelated_from_seed(DEFAULT_JITTER_SEED))
                .build(),
            rate_limit: RequestRateLimiter::default_orderbook(),
            tracing_enabled: false,
        }
    }

    /// Sets the shared HTTP client policy.
    #[must_use]
    pub fn client_policy(mut self, client: HttpClientPolicy) -> Self {
        self.client = Some(client);
        self
    }

    /// Sets the shared HTTP user-agent.
    ///
    /// # Errors
    ///
    /// Returns [`TransportPolicyBuildError`] if the user-agent is not a valid
    /// HTTP header value.
    pub fn user_agent(
        mut self,
        user_agent: impl Into<String>,
    ) -> Result<Self, TransportPolicyBuildError> {
        let existing_timeout = self
            .client
            .as_ref()
            .and_then(HttpClientPolicy::timeout)
            .unwrap_or(crate::DEFAULT_HTTP_TIMEOUT);
        self.client = Some(HttpClientPolicy::with_timeout_and_user_agent(
            existing_timeout,
            user_agent,
        )?);
        Ok(self)
    }

    /// Sets the shared HTTP timeout.
    ///
    /// # Errors
    ///
    /// Returns [`TransportPolicyBuildError`] if the existing user-agent is no
    /// longer a valid HTTP header value.
    pub fn timeout(mut self, timeout: Duration) -> Result<Self, TransportPolicyBuildError> {
        let user_agent = self
            .client
            .as_ref()
            .map_or(DEFAULT_ORDERBOOK_USER_AGENT, HttpClientPolicy::user_agent)
            .to_owned();
        self.client = Some(HttpClientPolicy::with_timeout_and_user_agent(
            timeout, user_agent,
        )?);
        Ok(self)
    }

    /// Sets the retry policy.
    #[must_use]
    pub const fn retry(mut self, retry: RetryPolicy) -> Self {
        self.retry = retry;
        self
    }

    /// Sets the request rate limiter.
    #[must_use]
    pub fn rate_limit(mut self, rate_limit: RequestRateLimiter) -> Self {
        self.rate_limit = rate_limit;
        self
    }

    /// Enables or disables tracing integration.
    #[must_use]
    pub const fn tracing_enabled(mut self, tracing_enabled: bool) -> Self {
        self.tracing_enabled = tracing_enabled;
        self
    }

    /// Builds the transport policy.
    ///
    /// # Errors
    ///
    /// Returns [`TransportPolicyBuildError`] if the default HTTP client policy
    /// cannot be constructed.
    pub fn build(self) -> Result<TransportPolicy, TransportPolicyBuildError> {
        Ok(TransportPolicy {
            client: match self.client {
                Some(client) => client,
                None => HttpClientPolicy::new(DEFAULT_ORDERBOOK_USER_AGENT)?,
            },
            retry: self.retry,
            rate_limit: self.rate_limit,
            tracing_enabled: self.tracing_enabled,
        })
    }
}

impl Default for TransportPolicyBuilder {
    fn default() -> Self {
        Self::new()
    }
}
