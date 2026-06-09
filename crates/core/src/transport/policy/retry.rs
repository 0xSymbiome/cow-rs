//! Retry policy and backoff decisions.

use std::time::{Duration, SystemTime};

use crate::transport::policy::{
    JitterStrategy, NetworkErrorKind, RetryAfter, parse_retry_after,
    status::{SERVICE_UNAVAILABLE, TOO_MANY_REQUESTS, is_retryable_status},
};

/// Default maximum number of attempts, including the first try.
pub const DEFAULT_MAX_ATTEMPTS: usize = 10;
/// Default base delay for exponential retry backoff.
pub const DEFAULT_BASE_DELAY: Duration = Duration::from_millis(50);
/// Default maximum delay for retry backoff.
pub const DEFAULT_MAX_DELAY: Duration = Duration::from_millis(3_200);

/// Retry policy for SDK HTTP requests.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    max_attempts: usize,
    base_delay: Duration,
    max_delay: Duration,
    jitter: JitterStrategy,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl RetryPolicy {
    /// Creates a retry policy with the supplied maximum attempt count.
    #[must_use]
    pub const fn new(max_attempts: usize) -> Self {
        Self::builder().max_attempts(max_attempts).build()
    }

    /// Returns a retry policy that performs only the first attempt.
    #[must_use]
    pub const fn no_retry() -> Self {
        Self::builder()
            .max_attempts(1)
            .jitter(JitterStrategy::none())
            .build()
    }

    /// Returns a builder seeded with documented defaults.
    #[must_use]
    pub const fn builder() -> RetryPolicyBuilder {
        RetryPolicyBuilder::new()
    }

    /// Returns the maximum number of attempts, including the first try.
    #[must_use]
    pub const fn max_attempts(&self) -> usize {
        self.max_attempts
    }

    /// Returns the base delay used for exponential backoff.
    #[must_use]
    pub const fn base_delay(&self) -> Duration {
        self.base_delay
    }

    /// Returns the maximum delay used for exponential backoff.
    #[must_use]
    pub const fn max_delay(&self) -> Duration {
        self.max_delay
    }

    /// Returns the retry jitter strategy.
    #[must_use]
    pub const fn jitter(&self) -> JitterStrategy {
        self.jitter
    }

    /// Returns this policy with an explicit retry jitter strategy.
    #[must_use]
    pub const fn with_jitter(mut self, jitter: JitterStrategy) -> Self {
        self.jitter = jitter;
        self
    }

    /// Returns `true` when `status` should be retried under this policy.
    #[must_use]
    pub const fn should_retry_status(&self, status: u16) -> bool {
        is_retryable_status(status)
    }

    /// Returns `true` when `kind` should be retried under this policy.
    #[must_use]
    pub const fn should_retry_network(&self, kind: NetworkErrorKind) -> bool {
        matches!(
            kind,
            NetworkErrorKind::Timeout
                | NetworkErrorKind::Connect
                | NetworkErrorKind::Request
                | NetworkErrorKind::Other
        )
    }

    /// Returns the jittered exponential backoff delay for `attempt_index`.
    #[must_use]
    pub fn delay_for_attempt(&self, attempt_index: usize) -> Duration {
        let base = self.base_backoff_delay(attempt_index);
        self.jitter
            .delay_for_attempt(base, self.max_delay, attempt_index)
    }

    /// Returns the retry delay for a status response and optional headers.
    #[must_use]
    pub fn delay_for_status(
        &self,
        attempt_index: usize,
        status: u16,
        headers: &[(String, String)],
        now: SystemTime,
    ) -> Duration {
        let backoff = self.delay_for_attempt(attempt_index);
        if !matches!(status, TOO_MANY_REQUESTS | SERVICE_UNAVAILABLE) {
            return backoff;
        }

        retry_after(headers, now).map_or(backoff, |retry_after| backoff.max(retry_after.delay()))
    }

    /// Returns the uncluttered exponential retry delay before jitter.
    ///
    /// # Panics
    ///
    /// Panics only if the bounded retry exponent cannot be represented as
    /// `u32`.
    fn base_backoff_delay(&self, attempt_index: usize) -> Duration {
        // SAFETY: the exponent is clamped to at most 6 before conversion.
        let exponent = u32::try_from(attempt_index.saturating_sub(1).min(6))
            .expect("backoff exponent is clamped to a u32-safe range");
        self.base_delay
            .saturating_mul(1_u32.checked_shl(exponent).unwrap_or(u32::MAX))
            .min(self.max_delay)
    }
}

fn retry_after(headers: &[(String, String)], now: SystemTime) -> Option<RetryAfter> {
    headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("retry-after"))
        .and_then(|(_, value)| parse_retry_after(value, now))
}

/// Builder for [`RetryPolicy`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicyBuilder {
    max_attempts: usize,
    base_delay: Duration,
    max_delay: Duration,
    jitter: JitterStrategy,
}

impl RetryPolicyBuilder {
    /// Creates a builder seeded with documented retry defaults.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            base_delay: DEFAULT_BASE_DELAY,
            max_delay: DEFAULT_MAX_DELAY,
            jitter: JitterStrategy::None,
        }
    }

    /// Sets the maximum number of attempts, including the first try.
    #[must_use]
    pub const fn max_attempts(mut self, max_attempts: usize) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Sets the base delay used for exponential backoff.
    #[must_use]
    pub const fn base_delay(mut self, base_delay: Duration) -> Self {
        self.base_delay = base_delay;
        self
    }

    /// Sets the maximum delay used for exponential backoff.
    #[must_use]
    pub const fn max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }

    /// Sets the retry jitter strategy.
    #[must_use]
    pub const fn jitter(mut self, jitter: JitterStrategy) -> Self {
        self.jitter = jitter;
        self
    }

    /// Builds the retry policy.
    #[must_use]
    pub const fn build(self) -> RetryPolicy {
        RetryPolicy {
            max_attempts: if self.max_attempts == 0 {
                1
            } else {
                self.max_attempts
            },
            base_delay: self.base_delay,
            max_delay: self.max_delay,
            jitter: self.jitter,
        }
    }
}

impl Default for RetryPolicyBuilder {
    fn default() -> Self {
        Self::new()
    }
}
