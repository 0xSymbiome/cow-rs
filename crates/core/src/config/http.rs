use std::time::Duration;

use http::HeaderValue;

use crate::errors::ValidationError;

use super::{DEFAULT_HTTP_TIMEOUT, DEFAULT_MAX_RESPONSE_BYTES};

/// Shared HTTP client policy used by transport-owning crates.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpClientPolicy {
    timeout: Option<Duration>,
    user_agent: String,
    max_response_bytes: usize,
}

impl HttpClientPolicy {
    /// Creates a policy with the default timeout and a validated user agent.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError`] if the user agent is empty or cannot be
    /// encoded as an HTTP header value.
    pub fn new(user_agent: impl Into<String>) -> Result<Self, ValidationError> {
        Self::with_timeout_and_user_agent(DEFAULT_HTTP_TIMEOUT, user_agent)
    }

    /// Creates a policy with an explicit timeout and validated user agent.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError`] if the user agent is empty or cannot be
    /// encoded as an HTTP header value.
    pub fn with_timeout_and_user_agent(
        timeout: Duration,
        user_agent: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        let user_agent = validate_user_agent(user_agent.into())?;

        Ok(Self {
            timeout: Some(timeout),
            user_agent,
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
        })
    }

    /// Returns a copy of this policy with timeouts disabled.
    #[must_use]
    pub const fn without_timeout(mut self) -> Self {
        self.timeout = None;
        self
    }

    /// Returns a copy of this policy with the supplied timeout.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Returns a copy of this policy with the supplied maximum response-body
    /// size, in bytes. The HTTP transport refuses to buffer a response whose
    /// decoded body would exceed this many bytes.
    #[must_use]
    pub const fn with_max_response_bytes(mut self, max_response_bytes: usize) -> Self {
        self.max_response_bytes = max_response_bytes;
        self
    }

    /// Returns a copy of this policy with a newly validated user agent.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError`] if the user agent is empty or cannot be
    /// encoded as an HTTP header value.
    pub fn try_with_user_agent(
        mut self,
        user_agent: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        self.user_agent = validate_user_agent(user_agent.into())?;
        Ok(self)
    }

    /// Returns the configured timeout, if one is enabled.
    #[must_use]
    pub const fn timeout(&self) -> Option<Duration> {
        self.timeout
    }

    /// Returns the configured maximum response-body size, in bytes.
    #[must_use]
    pub const fn max_response_bytes(&self) -> usize {
        self.max_response_bytes
    }

    /// Returns the configured user-agent header value.
    #[must_use]
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }
}

pub(super) fn validate_user_agent(user_agent: String) -> Result<String, ValidationError> {
    if user_agent.trim().is_empty() {
        return Err(ValidationError::EmptyField {
            field: "user_agent",
        });
    }

    validate_header_value(&user_agent, "user_agent")?;

    Ok(user_agent)
}

pub(super) fn validate_header_value(
    value: &str,
    field: &'static str,
) -> Result<(), ValidationError> {
    HeaderValue::from_str(value).map_err(|_| ValidationError::InvalidHttpHeaderValue { field })?;
    Ok(())
}
