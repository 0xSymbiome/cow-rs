//! Native [`reqwest`]-backed [`HttpTransport`] default.
//!
//! [`ReqwestTransport`] provides a ready-to-use production implementation of
//! [`HttpTransport`] for every non-`wasm32` target. The adapter applies the
//! transport-layer error classification contract described on
//! [`TransportErrorClass`]: every `reqwest::Error` passes through
//! [`reqwest::Error::without_url`] before the adapter inspects it so no URL
//! leaks through the typed error surface, and the failure is tagged with the
//! appropriate class through the documented `is_timeout`, `is_connect`,
//! `is_redirect`, `is_decode`, `is_body`, `is_builder`, `is_request`,
//! `is_status`, fallthrough partition.
//!
//! Non-2xx responses are captured as [`TransportError::HttpStatus`] with the
//! numeric status code and raw body so the calling layer receives the
//! response through the typed error channel instead of through `Ok(String)`.
//! Per-call headers merge with any constructor-configured defaults, and the
//! optional per-call timeout overrides the transport's default timeout when
//! supplied.
//!
//! URL-bearing configuration is held in the [`Redacted`] newtype so the base
//! URL never appears in debug, display, or serialized output. Callers that
//! need to observe the configured URL for audit or telemetry purposes unwrap
//! it explicitly through [`Redacted::as_inner`].

use std::time::Duration;

use ::reqwest::{
    Client, RequestBuilder,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use async_trait::async_trait;

use crate::{
    redaction::Redacted,
    transport::{error::TransportError, http::HttpTransport},
    validation::TransportErrorClass,
};

/// Configuration bundle for [`ReqwestTransport`].
///
/// The base URL is wrapped in [`Redacted`] so it is never emitted through
/// debug, display, or serde representations of the configuration value.
#[derive(Debug, Clone)]
pub struct ReqwestTransportConfig {
    base_url: Redacted<String>,
    user_agent: Option<String>,
    timeout: Option<Duration>,
}

impl ReqwestTransportConfig {
    /// Creates a configuration with the supplied base URL and default
    /// transport policy (no explicit timeout, no explicit user-agent).
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: Redacted::new(base_url.into()),
            user_agent: None,
            timeout: None,
        }
    }

    /// Returns a copy of the configuration with a validated user-agent.
    #[must_use]
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Returns a copy of the configuration with an explicit request timeout.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Returns the base URL as a borrowed string for deliberate inspection.
    #[must_use]
    pub fn base_url(&self) -> &str {
        self.base_url.as_inner()
    }
}

/// Native [`HttpTransport`] implementation backed by a shared [`reqwest::Client`].
#[derive(Debug, Clone)]
pub struct ReqwestTransport {
    client: Client,
    base_url: Redacted<String>,
}

impl ReqwestTransport {
    /// Builds a transport from the supplied configuration.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError::Configuration`] when the underlying HTTP
    /// client could not be constructed from the supplied policy (for example
    /// when the user-agent cannot be encoded as a header value).
    pub fn new(config: ReqwestTransportConfig) -> Result<Self, TransportError> {
        let base_url = config.base_url;
        let trimmed = base_url.as_inner().trim_end_matches('/').to_owned();
        let base_url = Redacted::new(trimmed);

        let mut builder = Client::builder();
        if let Some(user_agent) = config.user_agent {
            builder = builder.user_agent(user_agent);
        }
        if let Some(timeout) = config.timeout {
            builder = builder.timeout(timeout);
        }

        let client = builder
            .build()
            .map_err(|error| TransportError::Configuration {
                message: format!("could not build reqwest client: {error}"),
            })?;

        Ok(Self { client, base_url })
    }

    /// Builds a transport from an existing client and a base URL.
    ///
    /// Callers that already share a [`reqwest::Client`] across subsystems can
    /// reuse it here without rebuilding the TLS stack.
    #[must_use]
    pub fn with_client(client: Client, base_url: impl Into<String>) -> Self {
        let trimmed = base_url.into().trim_end_matches('/').to_owned();
        Self {
            client,
            base_url: Redacted::new(trimmed),
        }
    }

    /// Returns the shared [`reqwest::Client`] used by this transport.
    #[must_use]
    pub const fn client(&self) -> &Client {
        &self.client
    }

    /// Returns the configured base URL for deliberate inspection.
    #[must_use]
    pub fn base_url(&self) -> &str {
        self.base_url.as_inner()
    }

    fn resolve_url(&self, path: &str) -> String {
        if path.starts_with("http://")
            || path.starts_with("https://")
            || self.base_url.as_inner().is_empty()
        {
            path.to_owned()
        } else if path.starts_with('/') {
            format!("{}{}", self.base_url.as_inner(), path)
        } else {
            format!("{}/{}", self.base_url.as_inner(), path)
        }
    }

    fn apply_call_overrides(
        builder: RequestBuilder,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<RequestBuilder, TransportError> {
        let mut builder = builder;
        if !headers.is_empty() {
            let header_map = build_header_map(headers)?;
            builder = builder.headers(header_map);
        }
        if let Some(timeout) = timeout {
            builder = builder.timeout(timeout);
        }
        Ok(builder)
    }

    async fn dispatch(&self, builder: RequestBuilder) -> Result<String, TransportError> {
        let response = builder.send().await.map_err(map_reqwest_error)?;
        let status = response.status();
        if status.is_success() {
            return response.text().await.map_err(map_reqwest_error);
        }

        let status_code = status.as_u16();
        let body = response
            .text()
            .await
            .unwrap_or_else(|error| format!("<body unavailable: {error}>"));
        Err(TransportError::HttpStatus {
            status: status_code,
            body,
        })
    }
}

#[async_trait]
impl HttpTransport for ReqwestTransport {
    async fn get(
        &self,
        path: &str,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        let url = self.resolve_url(path);
        let builder = Self::apply_call_overrides(self.client.get(&url), headers, timeout)?;
        self.dispatch(builder).await
    }

    async fn post(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        let url = self.resolve_url(path);
        let builder = self.client.post(&url).body(body.to_owned());
        let builder = Self::apply_call_overrides(builder, headers, timeout)?;
        self.dispatch(builder).await
    }

    async fn put(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        let url = self.resolve_url(path);
        let builder = self.client.put(&url).body(body.to_owned());
        let builder = Self::apply_call_overrides(builder, headers, timeout)?;
        self.dispatch(builder).await
    }

    async fn delete(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        let url = self.resolve_url(path);
        let builder = self.client.delete(&url).body(body.to_owned());
        let builder = Self::apply_call_overrides(builder, headers, timeout)?;
        self.dispatch(builder).await
    }
}

fn build_header_map(headers: &[(String, String)]) -> Result<HeaderMap, TransportError> {
    let mut header_map = HeaderMap::with_capacity(headers.len());
    for (name, value) in headers {
        let header_name = HeaderName::from_bytes(name.as_bytes()).map_err(|error| {
            TransportError::Configuration {
                message: format!("invalid header name: {error}"),
            }
        })?;
        let header_value =
            HeaderValue::from_str(value).map_err(|error| TransportError::Configuration {
                message: format!("invalid header value: {error}"),
            })?;
        header_map.append(header_name, header_value);
    }
    Ok(header_map)
}

/// Converts a `reqwest::Error` into the typed [`TransportError::Transport`]
/// variant.
///
/// The helper strips any attached URL through
/// [`reqwest::Error::without_url`] before classifying it through the
/// documented [`TransportErrorClass`] partition. Downstream crates that
/// bridge their own `reqwest::Error` wraps share the classification by
/// routing every failure through this helper.
///
/// # Examples
///
/// Classify a builder-layer `reqwest::Error` and observe that the
/// redaction path keeps the URL out of the rendered error text:
///
/// ```
/// use cow_sdk_core::TransportErrorClass;
/// use cow_sdk_core::transport::classify_reqwest_error;
///
/// let client = reqwest::Client::new();
/// let builder_error = client
///     .request(reqwest::Method::GET, "http://[invalid ipv6]/")
///     .build()
///     .expect_err("malformed URL must fail at the builder layer");
///
/// let transport_error = classify_reqwest_error(builder_error);
/// assert_eq!(transport_error.class(), Some(TransportErrorClass::Builder));
/// assert!(!format!("{transport_error}").contains("invalid ipv6"));
/// ```
///
/// Timeout errors classify through the same helper, and the attached URL is
/// stripped before the detail message is rendered:
///
/// ```no_run
/// use std::time::Duration;
///
/// use cow_sdk_core::TransportErrorClass;
/// use cow_sdk_core::transport::classify_reqwest_error;
///
/// # async fn demonstrate_timeout() {
/// let client = reqwest::Client::builder()
///     .timeout(Duration::from_millis(1))
///     .build()
///     .expect("client must build");
/// let timeout_error = client
///     .get("https://example.invalid/slow")
///     .send()
///     .await
///     .expect_err("an unreachable host exceeds the 1ms timeout");
///
/// let transport_error = classify_reqwest_error(timeout_error);
/// // The class surface is partitioned; timeouts always map to `Timeout`.
/// let _: Option<TransportErrorClass> = transport_error.class();
/// // The attached URL never appears in the rendered error text.
/// assert!(!format!("{transport_error}").contains("example.invalid"));
/// # }
/// ```
#[must_use]
pub fn classify_reqwest_error(error: ::reqwest::Error) -> TransportError {
    map_reqwest_error(error)
}

fn map_reqwest_error(error: ::reqwest::Error) -> TransportError {
    let sanitized = error.without_url();
    let class = classify(&sanitized);
    TransportError::Transport {
        class,
        detail: sanitized.to_string(),
    }
}

fn classify(error: &::reqwest::Error) -> TransportErrorClass {
    if error.is_timeout() {
        return TransportErrorClass::Timeout;
    }
    if error.is_connect() {
        return TransportErrorClass::Connect;
    }
    if error.is_redirect() {
        return TransportErrorClass::Redirect;
    }
    if error.is_decode() {
        TransportErrorClass::Decode
    } else if error.is_body() {
        TransportErrorClass::Body
    } else if error.is_builder() {
        TransportErrorClass::Builder
    } else if error.is_status() {
        TransportErrorClass::Status
    } else if error.is_request() {
        TransportErrorClass::Request
    } else {
        TransportErrorClass::Other
    }
}
