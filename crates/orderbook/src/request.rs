use std::{future::Future, sync::Arc, time::Duration};

use cow_sdk_core::transport::policy::{
    AttemptOutcome as RetryOutcome, LimiterKey, RequestRateLimiter, RetryPolicy, RetrySignal,
    retry_after_from_headers, run_with_retry,
};
use cow_sdk_core::{HttpTransport, Redacted, TransportError};
use http::header::{ACCEPT, CONTENT_TYPE, HeaderMap};
use serde::de::DeserializeOwned;
use serde_json::Value;
use thiserror::Error;

use crate::error::OrderbookError;

/// Shared dyn-compatible [`HttpTransport`] handle threaded through orderbook
/// request helpers.
pub(crate) type SharedTransport = Arc<dyn HttpTransport + Send + Sync>;

/// HTTP verb used by the typed orderbook transport.
///
/// Internal helper; the SDK chooses the method per request shape. Classified
/// as `sdk-local-state` in the workspace enum policy manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    /// `GET`.
    Get,
    /// `POST`.
    Post,
    /// `DELETE`.
    Delete,
    /// `PUT`.
    Put,
}

/// Decoded transport response body.
///
/// Internal wrapper around payloads decoded by the typed transport. Classified
/// as `sdk-local-state` in the workspace enum policy manifest.
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "the `Json(serde_json::Value)` variant cannot implement `Eq` because `serde_json::Value` does not implement `Eq`"
)]
#[derive(Debug, Clone, PartialEq)]
pub enum ResponseBody {
    /// JSON payload.
    Json(Value),
    /// Plain-text payload.
    Text(String),
    /// Empty response body.
    Empty,
}

/// Structured non-2xx error returned by the orderbook transport layer.
#[derive(Debug, Clone, PartialEq, Error)]
#[error("{message}")]
pub struct OrderbookApiError {
    /// HTTP status code.
    pub status: u16,
    /// HTTP status text.
    pub status_text: Redacted<String>,
    /// Decoded response body captured from the error response.
    pub body: Redacted<ResponseBody>,
    message: Redacted<String>,
    /// Server-suggested backoff parsed from the `Retry-After` response header,
    /// when one was present on the failing response.
    retry_after: Option<Duration>,
}

impl OrderbookApiError {
    /// Creates a typed API error from status metadata and a decoded body.
    ///
    /// The resulting error carries no `Retry-After` hint; attach one parsed
    /// from the response headers with [`OrderbookApiError::with_retry_after`].
    #[must_use]
    pub fn new(status: u16, status_text: impl Into<String>, body: ResponseBody) -> Self {
        let status_text = status_text.into();
        let message = match &body {
            ResponseBody::Json(Value::Object(map)) => map
                .get("description")
                .or_else(|| map.get("error"))
                .and_then(Value::as_str)
                .map_or_else(|| status_text.clone(), ToOwned::to_owned),
            ResponseBody::Json(Value::String(text)) => text.clone(),
            ResponseBody::Text(text) if !text.is_empty() => text.clone(),
            _ => status_text.clone(),
        };

        Self {
            status,
            status_text: Redacted::new(status_text),
            body: Redacted::new(body),
            message: Redacted::new(message),
            retry_after: None,
        }
    }

    /// Returns this error annotated with a parsed `Retry-After` backoff hint.
    ///
    /// `retry_after` is the resolved delay from the failing response's
    /// `Retry-After` header (see
    /// [`cow_sdk_core::transport::policy::retry_after_from_headers`]), or [`None`]
    /// when the server sent no hint.
    #[must_use]
    pub const fn with_retry_after(mut self, retry_after: Option<Duration>) -> Self {
        self.retry_after = retry_after;
        self
    }

    /// Returns the server-suggested backoff parsed from the `Retry-After`
    /// response header, when one was present on the failing response.
    #[must_use]
    pub const fn retry_after(&self) -> Option<Duration> {
        self.retry_after
    }
}

/// Low-level request description used by the transport helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchParams {
    /// Relative API path appended to the resolved base URL.
    pub path: String,
    /// HTTP method used for the request.
    pub method: HttpMethod,
    /// Query pairs encoded onto the request URL.
    pub query: Vec<(String, String)>,
    /// Optional JSON request body.
    pub body: Option<Value>,
}

impl FetchParams {
    /// Creates a request descriptor from a path and method.
    #[must_use]
    pub fn new(path: impl Into<String>, method: HttpMethod) -> Self {
        Self {
            path: path.into(),
            method,
            query: Vec::new(),
            body: None,
        }
    }

    /// Returns a copy of this descriptor with an additional query parameter.
    #[must_use]
    pub fn with_query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.push((key.into(), value.into()));
        self
    }

    /// Returns a copy of this descriptor with a JSON request body.
    #[must_use]
    pub fn with_body(mut self, body: Value) -> Self {
        self.body = Some(body);
        self
    }
}

/// Fully decoded HTTP response captured by low-level transport helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseEnvelope {
    /// HTTP status code.
    pub status: u16,
    /// HTTP status text.
    pub status_text: String,
    /// Response content type, when present.
    pub content_type: Option<String>,
    /// Raw response bytes.
    pub body: Vec<u8>,
}

impl ResponseEnvelope {
    /// Creates a JSON response envelope.
    ///
    /// # Panics
    ///
    /// Panics only if serializing an in-memory [`Value`] into a `Vec<u8>`
    /// unexpectedly fails.
    #[must_use]
    pub fn json(status: u16, value: &Value) -> Self {
        Self {
            status,
            status_text: canonical_status_text(status),
            content_type: Some("application/json".to_owned()),
            // SAFETY: serde_json::Value serializes to JSON bytes without
            // relying on caller-controlled type implementations.
            body: serde_json::to_vec(value).expect("test JSON serialization must succeed"),
        }
    }

    /// Creates a plain-text response envelope.
    #[must_use]
    pub fn text(status: u16, body: impl Into<String>) -> Self {
        Self {
            status,
            status_text: canonical_status_text(status),
            content_type: Some("text/plain".to_owned()),
            body: body.into().into_bytes(),
        }
    }

    /// Creates an empty response envelope.
    #[must_use]
    pub fn empty(status: u16) -> Self {
        Self {
            status,
            status_text: canonical_status_text(status),
            content_type: None,
            body: Vec::new(),
        }
    }

    fn decoded_body(&self) -> ResponseBody {
        if self.status == 204 || self.body.is_empty() {
            return ResponseBody::Empty;
        }

        let prefer_json = self.content_type.as_deref().is_none_or(|content_type| {
            content_type
                .to_ascii_lowercase()
                .starts_with("application/json")
        });

        if prefer_json && let Ok(value) = serde_json::from_slice::<Value>(&self.body) {
            return ResponseBody::Json(value);
        }

        ResponseBody::Text(String::from_utf8_lossy(&self.body).into_owned())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResponseKind {
    Json,
    Text,
    Empty,
}

impl ResponseKind {
    const fn accept_header(self) -> &'static str {
        match self {
            Self::Text => "text/plain, application/json",
            Self::Json | Self::Empty => "application/json",
        }
    }
}

enum AttemptOutcome {
    Response(ResponseEnvelope),
    HttpError {
        response: ResponseEnvelope,
        headers: Vec<(String, String)>,
    },
}

struct RequestExecution<'a> {
    transport: &'a SharedTransport,
    base_url: &'a str,
    params: &'a FetchParams,
    timeout: Option<Duration>,
    additional_headers: Option<HeaderMap>,
}

/// Executes a JSON request with an optional per-request timeout override.
///
/// # Errors
///
/// Returns [`OrderbookError`] when request execution fails, the API returns a
/// non-success response, or the success body cannot be decoded as JSON.
pub async fn request_json_with_timeout<T>(
    transport: &SharedTransport,
    base_url: &str,
    params: &FetchParams,
    policy: &RetryPolicy,
    rate_limiter: &RequestRateLimiter,
    timeout: Option<Duration>,
    additional_headers: Option<HeaderMap>,
) -> Result<T, OrderbookError>
where
    T: DeserializeOwned,
{
    request_with(
        RequestExecution {
            transport,
            base_url,
            params,
            timeout,
            additional_headers,
        },
        policy,
        rate_limiter,
        ResponseKind::Json,
        decode_success_body::<T>,
    )
    .await
}

/// Executes a text request with an optional per-request timeout override.
///
/// # Errors
///
/// Returns [`OrderbookError`] when request execution fails, the API returns a
/// non-success response, or the success body cannot be decoded as UTF-8 text.
pub async fn request_text_with_timeout(
    transport: &SharedTransport,
    base_url: &str,
    params: &FetchParams,
    policy: &RetryPolicy,
    rate_limiter: &RequestRateLimiter,
    timeout: Option<Duration>,
    additional_headers: Option<HeaderMap>,
) -> Result<String, OrderbookError> {
    request_with(
        RequestExecution {
            transport,
            base_url,
            params,
            timeout,
            additional_headers,
        },
        policy,
        rate_limiter,
        ResponseKind::Text,
        decode_text_body,
    )
    .await
}

/// Executes an empty-body request with an optional per-request timeout override.
///
/// # Errors
///
/// Returns [`OrderbookError`] when request execution fails or the API returns a
/// non-success response.
pub async fn request_empty_with_timeout(
    transport: &SharedTransport,
    base_url: &str,
    params: &FetchParams,
    policy: &RetryPolicy,
    rate_limiter: &RequestRateLimiter,
    timeout: Option<Duration>,
    additional_headers: Option<HeaderMap>,
) -> Result<(), OrderbookError> {
    request_with(
        RequestExecution {
            transport,
            base_url,
            params,
            timeout,
            additional_headers,
        },
        policy,
        rate_limiter,
        ResponseKind::Empty,
        |_| Ok(()),
    )
    .await
}

/// Executes an abstract JSON-producing attempt with retry and rate-limit policy.
///
/// # Errors
///
/// Returns [`OrderbookError`] when all attempts fail, the API returns a
/// non-success response, or the success body cannot be decoded as JSON.
pub async fn execute_json_with<T, F, Fut>(
    policy: &RetryPolicy,
    rate_limiter: &RequestRateLimiter,
    mut attempt: F,
) -> Result<T, OrderbookError>
where
    T: DeserializeOwned,
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<ResponseEnvelope, (cow_sdk_core::TransportErrorClass, String)>>,
{
    execute_with(
        None,
        policy,
        rate_limiter,
        move || {
            let future = attempt();
            async move { future.await.map(AttemptOutcome::Response) }
        },
        decode_success_body::<T>,
    )
    .await
}

/// Executes an abstract empty-body attempt with retry and rate-limit policy.
///
/// # Errors
///
/// Returns [`OrderbookError`] when all attempts fail or the API returns a
/// non-success response.
pub async fn execute_empty_with<F, Fut>(
    policy: &RetryPolicy,
    rate_limiter: &RequestRateLimiter,
    mut attempt: F,
) -> Result<(), OrderbookError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<ResponseEnvelope, (cow_sdk_core::TransportErrorClass, String)>>,
{
    execute_with(
        None,
        policy,
        rate_limiter,
        move || {
            let future = attempt();
            async move { future.await.map(AttemptOutcome::Response) }
        },
        |_| Ok(()),
    )
    .await
}

async fn request_with<T, D>(
    request: RequestExecution<'_>,
    policy: &RetryPolicy,
    rate_limiter: &RequestRateLimiter,
    response_kind: ResponseKind,
    decode_success: D,
) -> Result<T, OrderbookError>
where
    D: Fn(&ResponseEnvelope) -> Result<T, OrderbookError>,
{
    let url = format!("{}{}", request.base_url, request.params.path);
    let limiter_url = url::Url::parse(&url).map_err(|error| OrderbookError::Transport {
        class: cow_sdk_core::TransportErrorClass::Builder,
        detail: Redacted::new(format!(
            "could not parse request URL for rate limiting: {error}"
        )),
    })?;
    let transport = Arc::clone(request.transport);
    let params = request.params.clone();
    let timeout = request.timeout;
    let additional_headers = request.additional_headers;

    execute_with(
        Some(&limiter_url),
        policy,
        rate_limiter,
        || {
            send_request(
                Arc::clone(&transport),
                url.clone(),
                params.clone(),
                timeout,
                response_kind,
                additional_headers.clone(),
            )
        },
        decode_success,
    )
    .await
}

async fn send_request(
    transport: SharedTransport,
    url: String,
    params: FetchParams,
    timeout: Option<Duration>,
    response_kind: ResponseKind,
    additional_headers: Option<HeaderMap>,
) -> Result<AttemptOutcome, (cow_sdk_core::TransportErrorClass, String)> {
    let full_url = match append_query_string(&url, &params.query) {
        Ok(url) => url,
        Err(message) => {
            return Err((cow_sdk_core::TransportErrorClass::Builder, message));
        }
    };

    let body_string = match params.body.as_ref() {
        Some(value) => match serde_json::to_string(value) {
            Ok(body) => body,
            Err(error) => {
                return Err((
                    cow_sdk_core::TransportErrorClass::Builder,
                    format!("could not serialize request body: {error}"),
                ));
            }
        },
        None => String::new(),
    };

    let header_pairs = request_header_pairs(response_kind, additional_headers);

    let result = match params.method {
        HttpMethod::Get => transport.get(&full_url, &header_pairs, timeout).await,
        HttpMethod::Post => {
            transport
                .post(&full_url, &body_string, &header_pairs, timeout)
                .await
        }
        HttpMethod::Put => {
            transport
                .put(&full_url, &body_string, &header_pairs, timeout)
                .await
        }
        HttpMethod::Delete => {
            transport
                .delete(&full_url, &body_string, &header_pairs, timeout)
                .await
        }
    };

    match result {
        Ok(response) => {
            let status = response.status();
            let content_type = response
                .header(CONTENT_TYPE.as_str())
                .map(ToOwned::to_owned);
            let success = (200..300).contains(&status);
            let headers: Vec<(String, String)> = if success {
                Vec::new()
            } else {
                response
                    .headers()
                    .iter()
                    .map(|(name, value)| (name.clone(), value.as_inner().clone()))
                    .collect()
            };
            let envelope = ResponseEnvelope {
                status,
                status_text: canonical_status_text(status),
                content_type,
                body: response.into_body().into_bytes(),
            };
            if success {
                Ok(AttemptOutcome::Response(envelope))
            } else {
                // A conforming transport never returns `Ok` for a non-2xx
                // status; normalize a misbehaving custom transport onto the
                // HTTP-error outcome so retry classification and
                // `Retry-After` handling stay uniform.
                Ok(AttemptOutcome::HttpError {
                    response: envelope,
                    headers,
                })
            }
        }
        Err(TransportError::HttpStatus {
            status,
            headers,
            body,
        }) => {
            let content_type = headers
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(CONTENT_TYPE.as_str()))
                .map(|(_, value)| value.as_inner().clone());
            Ok(AttemptOutcome::HttpError {
                response: ResponseEnvelope {
                    status,
                    status_text: canonical_status_text(status),
                    content_type,
                    body: body.into_inner().into_bytes(),
                },
                headers: headers
                    .into_iter()
                    .map(|(name, value)| (name, value.into_inner()))
                    .collect(),
            })
        }
        Err(TransportError::Transport { class, detail }) => Err((class, detail.into_inner())),
        Err(TransportError::Configuration { message }) => Err((
            cow_sdk_core::TransportErrorClass::Builder,
            message.into_inner(),
        )),
        Err(other) => Err((cow_sdk_core::TransportErrorClass::Other, other.to_string())),
    }
}

fn append_query_string(url: &str, query: &[(String, String)]) -> Result<String, String> {
    if query.is_empty() {
        return Ok(url.to_owned());
    }
    url::Url::parse_with_params(
        url,
        query
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str())),
    )
    .map(String::from)
    .map_err(|error| format!("could not encode query parameters: {error}"))
}

fn request_header_pairs(
    response_kind: ResponseKind,
    additional_headers: Option<HeaderMap>,
) -> Vec<(String, String)> {
    let mut pairs = Vec::with_capacity(2 + additional_headers.as_ref().map_or(0, HeaderMap::len));
    pairs.push((ACCEPT.to_string(), response_kind.accept_header().to_owned()));
    pairs.push((CONTENT_TYPE.to_string(), "application/json".to_owned()));
    if let Some(extra) = additional_headers {
        for (name, value) in &extra {
            let Ok(value_str) = value.to_str() else {
                continue;
            };
            pairs.push((name.as_str().to_owned(), value_str.to_owned()));
        }
    }
    pairs
}

async fn execute_with<T, F, Fut, D>(
    limiter_url: Option<&url::Url>,
    policy: &RetryPolicy,
    rate_limiter: &RequestRateLimiter,
    mut attempt: F,
    decode_success: D,
) -> Result<T, OrderbookError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<AttemptOutcome, (cow_sdk_core::TransportErrorClass, String)>>,
    D: Fn(&ResponseEnvelope) -> Result<T, OrderbookError>,
{
    // The shared driver in `cow_sdk_core::transport::policy` owns the retry loop,
    // rate-limit acquisition, backoff, `Retry-After` clock, and retry telemetry.
    // The closure performs one dispatch and classifies the result into the
    // unified outcome space; the success envelope is decoded after the driver
    // returns so a decode failure stays terminal and is never retried.
    let limiter_key = limiter_url.map_or(LimiterKey::Global, LimiterKey::PerUrl);
    let response = run_with_retry::<ResponseEnvelope, OrderbookError, _, _>(
        policy,
        rate_limiter,
        limiter_key,
        |attempt_index| {
            let future = attempt();
            async move {
                #[cfg(feature = "tracing")]
                record_span_attempts(attempt_index);
                #[cfg(not(feature = "tracing"))]
                let _ = attempt_index;

                match future.await {
                    Ok(AttemptOutcome::Response(response))
                        if (200..300).contains(&response.status) =>
                    {
                        #[cfg(feature = "tracing")]
                        record_span_status(response.status);
                        RetryOutcome::Success(response)
                    }
                    Ok(outcome) => {
                        let (response, headers) = match outcome {
                            AttemptOutcome::Response(response) => (response, Vec::new()),
                            AttemptOutcome::HttpError { response, headers } => (response, headers),
                        };
                        #[cfg(feature = "tracing")]
                        record_span_status(response.status);
                        let status = response.status;
                        let body = response.decoded_body();
                        // Resolve the server `Retry-After` hint while the
                        // response headers are still in scope, then attach it to
                        // the terminal error so a caller can read the suggested
                        // backoff after the retry budget is exhausted. The retry
                        // driver computes its own clock-injected backoff and does
                        // not depend on this value.
                        let retry_after = retry_after_from_headers(&headers);
                        let error = OrderbookApiError::new(status, response.status_text, body)
                            .with_retry_after(retry_after);
                        RetryOutcome::Failure {
                            error: error.into(),
                            signal: RetrySignal::HttpStatus { status, headers },
                        }
                    }
                    Err((class, detail)) => RetryOutcome::Failure {
                        error: OrderbookError::Transport {
                            class,
                            detail: Redacted::new(detail),
                        },
                        signal: RetrySignal::Transport { class },
                    },
                }
            }
        },
    )
    .await?;

    decode_success(&response)
}

#[cfg(feature = "tracing")]
fn record_span_attempts(attempt_index: usize) {
    let attempts = u64::try_from(attempt_index).unwrap_or(u64::MAX);
    tracing::Span::current().record("attempts", attempts);
}

#[cfg(feature = "tracing")]
fn record_span_status(status: u16) {
    tracing::Span::current().record("status", u64::from(status));
}

fn decode_success_body<T>(response: &ResponseEnvelope) -> Result<T, OrderbookError>
where
    T: DeserializeOwned,
{
    serde_json::from_slice::<T>(&response.body).map_err(OrderbookError::from)
}

fn decode_text_body(response: &ResponseEnvelope) -> Result<String, OrderbookError> {
    String::from_utf8(response.body.clone()).map_err(|error| OrderbookError::Transport {
        class: cow_sdk_core::TransportErrorClass::Decode,
        detail: Redacted::new(error.to_string()),
    })
}

fn canonical_status_text(status: u16) -> String {
    http::StatusCode::from_u16(status)
        .ok()
        .and_then(|status| status.canonical_reason().map(ToOwned::to_owned))
        .unwrap_or_else(|| "Unknown Status".to_owned())
}
