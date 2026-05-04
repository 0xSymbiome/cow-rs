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
//! numeric status code, response headers, and raw body so the calling layer
//! receives the response through the typed error channel instead of through
//! `Ok(String)`.
//! Per-call headers merge with any constructor-configured defaults, and the
//! optional per-call timeout overrides the transport's default timeout when
//! supplied.
//!
//! URL-bearing configuration is held in the [`Redacted`] newtype so the base
//! URL never appears in debug, display, or serialized output. Callers that
//! need to observe the configured URL for audit or telemetry purposes unwrap
//! it explicitly through [`Redacted::as_inner`].

use std::{borrow::Cow, time::Duration};

use ::reqwest::{
    Client, RequestBuilder,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use async_trait::async_trait;

use crate::{
    config::{DEFAULT_TCP_KEEPALIVE, DEFAULT_USER_AGENT},
    redaction::Redacted,
    transport::{
        CUSTOM_OVERRIDE_ROUTE_IDENTITY, error::TransportError, http::HttpTransport,
        sanitize_public_base_url,
    },
    validation::TransportErrorClass,
};

/// Configuration bundle for [`ReqwestTransport`].
///
/// The base URL is wrapped in [`Redacted`] so it is never emitted through
/// debug, display, or serde representations of the configuration value.
#[derive(Debug, Clone)]
pub struct ReqwestTransportConfig {
    base_url: Redacted<String>,
    user_agent: String,
    tcp_keepalive: Duration,
    timeout: Option<Duration>,
}

impl ReqwestTransportConfig {
    /// Creates a configuration with the supplied base URL and default
    /// transport policy.
    ///
    /// The default policy applies [`DEFAULT_USER_AGENT`],
    /// [`DEFAULT_TCP_KEEPALIVE`], and no explicit request timeout.
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: Redacted::new(base_url.into()),
            user_agent: DEFAULT_USER_AGENT.to_owned(),
            tcp_keepalive: DEFAULT_TCP_KEEPALIVE,
            timeout: None,
        }
    }

    /// Returns a copy of the configuration with a validated user-agent.
    #[must_use]
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Returns a copy of the configuration with an explicit TCP keepalive.
    #[must_use]
    pub const fn with_tcp_keepalive(mut self, tcp_keepalive: Duration) -> Self {
        self.tcp_keepalive = tcp_keepalive;
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

    /// Returns the configured user-agent header value.
    #[must_use]
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// Returns the configured TCP keepalive duration.
    #[must_use]
    pub const fn tcp_keepalive(&self) -> Duration {
        self.tcp_keepalive
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

        let mut builder = Client::builder()
            .user_agent(config.user_agent)
            .tcp_keepalive(config.tcp_keepalive);
        if let Some(timeout) = config.timeout {
            builder = builder.timeout(timeout);
        }

        let client = builder
            .build()
            .map_err(|error| TransportError::Configuration {
                message: Redacted::new(format!("could not build reqwest client: {error}")),
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

    async fn dispatch(
        &self,
        builder: RequestBuilder,
        method: &str,
        endpoint: &str,
        bytes_sent: usize,
    ) -> Result<String, TransportError> {
        #[cfg(feature = "tracing")]
        {
            use tracing::Instrument as _;

            let span = tracing::info_span!(
                "transport.dispatch",
                method = method,
                endpoint = endpoint,
                bytes_sent = bytes_sent as u64,
                bytes_received = tracing::field::Empty,
            );
            let recorder = span.clone();
            async move {
                let result = dispatch_request(builder).await;
                if let Some(bytes_received) = bytes_received(&result) {
                    recorder.record("bytes_received", bytes_received as u64);
                }
                result
            }
            .instrument(span)
            .await
        }

        #[cfg(not(feature = "tracing"))]
        {
            let _ = (method, endpoint, bytes_sent);
            dispatch_request(builder).await
        }
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
        let endpoint = span_endpoint(path);
        self.dispatch(builder, "GET", endpoint.as_ref(), 0).await
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
        let endpoint = span_endpoint(path);
        self.dispatch(builder, "POST", endpoint.as_ref(), body.len())
            .await
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
        let endpoint = span_endpoint(path);
        self.dispatch(builder, "PUT", endpoint.as_ref(), body.len())
            .await
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
        let endpoint = span_endpoint(path);
        self.dispatch(builder, "DELETE", endpoint.as_ref(), body.len())
            .await
    }
}

async fn dispatch_request(builder: RequestBuilder) -> Result<String, TransportError> {
    let response = builder.send().await.map_err(map_reqwest_error)?;
    let status = response.status();
    if status.is_success() {
        return response.text().await.map_err(map_reqwest_error);
    }

    let status_code = status.as_u16();
    let headers = response_headers(&response);
    let body = response
        .text()
        .await
        .unwrap_or_else(|error| format!("<body unavailable: {error}>"));
    Err(TransportError::HttpStatus {
        status: status_code,
        headers,
        body: Redacted::new(body),
    })
}

#[cfg(feature = "tracing")]
const fn bytes_received(result: &Result<String, TransportError>) -> Option<usize> {
    match result {
        Ok(body) => Some(body.len()),
        Err(TransportError::HttpStatus { body, .. }) => Some(body.as_inner().len()),
        Err(_) => None,
    }
}

fn span_endpoint(path: &str) -> Cow<'_, str> {
    let has_authority = path.contains("://");
    if has_authority && sanitize_public_base_url(path) == CUSTOM_OVERRIDE_ROUTE_IDENTITY {
        return Cow::Borrowed("/");
    }

    let endpoint = path.find("://").map_or(path, |scheme_end| {
        let after_authority = &path[scheme_end + 3..];
        after_authority
            .find('/')
            .map_or("/", |path_start| &after_authority[path_start..])
    });
    let end = endpoint.find(['?', '#']).unwrap_or(endpoint.len());
    let endpoint = &endpoint[..end];
    if endpoint.is_empty() && has_authority {
        Cow::Borrowed("/")
    } else {
        Cow::Borrowed(endpoint)
    }
}

fn build_header_map(headers: &[(String, String)]) -> Result<HeaderMap, TransportError> {
    let mut header_map = HeaderMap::with_capacity(headers.len());
    for (name, value) in headers {
        let header_name = HeaderName::from_bytes(name.as_bytes()).map_err(|error| {
            TransportError::Configuration {
                message: Redacted::new(format!("invalid header name: {error}")),
            }
        })?;
        let header_value =
            HeaderValue::from_str(value).map_err(|error| TransportError::Configuration {
                message: Redacted::new(format!("invalid header value: {error}")),
            })?;
        header_map.append(header_name, header_value);
    }
    Ok(header_map)
}

fn response_headers(response: &::reqwest::Response) -> Vec<(String, Redacted<String>)> {
    response
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                name.as_str().to_owned(),
                Redacted::new(String::from_utf8_lossy(value.as_bytes()).into_owned()),
            )
        })
        .collect()
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
        detail: Redacted::new(sanitized.to_string()),
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
