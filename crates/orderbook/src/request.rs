use std::{future::Future, sync::Arc, time::Duration};

use async_lock::Mutex;
use cow_sdk_core::{HttpClientPolicy, HttpTransport, TransportError};
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap};
use serde::de::DeserializeOwned;
use serde_json::Value;
use thiserror::Error;

use crate::error::OrderbookError;

#[cfg(not(target_arch = "wasm32"))]
use futures_timer::Delay;
#[cfg(target_arch = "wasm32")]
use gloo_timers::future::TimeoutFuture;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

/// HTTP `408 Request Timeout`.
pub const REQUEST_TIMEOUT: u16 = 408;
/// HTTP `425 Too Early`.
pub const TOO_EARLY: u16 = 425;
/// HTTP `429 Too Many Requests`.
pub const TOO_MANY_REQUESTS: u16 = 429;
/// HTTP `500 Internal Server Error`.
pub const INTERNAL_SERVER_ERROR: u16 = 500;
/// HTTP `502 Bad Gateway`.
pub const BAD_GATEWAY: u16 = 502;
/// HTTP `503 Service Unavailable`.
pub const SERVICE_UNAVAILABLE: u16 = 503;
/// HTTP `504 Gateway Timeout`.
pub const GATEWAY_TIMEOUT: u16 = 504;
/// Status codes treated as retryable by the default orderbook request policy.
pub const RETRYABLE_STATUS_CODES: [u16; 7] = [
    REQUEST_TIMEOUT,
    TOO_EARLY,
    TOO_MANY_REQUESTS,
    INTERNAL_SERVER_ERROR,
    BAD_GATEWAY,
    SERVICE_UNAVAILABLE,
    GATEWAY_TIMEOUT,
];
/// Default maximum number of request attempts, including the first try.
pub const DEFAULT_MAX_ATTEMPTS: usize = 10;
/// Default request budget granted per limiter interval.
pub const DEFAULT_TOKENS_PER_INTERVAL: u32 = 5;
/// Human-readable label for the default limiter interval.
pub const DEFAULT_INTERVAL_LABEL: &str = "second";
/// Default orderbook user-agent string embedded in [`OrderBookTransportPolicy`].
pub const DEFAULT_ORDERBOOK_USER_AGENT: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Shared dyn-compatible [`HttpTransport`] handle threaded through orderbook
/// request helpers.
pub(crate) type SharedTransport = Arc<dyn HttpTransport + Send + Sync>;

/// HTTP methods used by the orderbook transport helpers.
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

/// Decoded response body preserved on [`OrderBookApiError`].
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
pub struct OrderBookApiError {
    /// HTTP status code.
    pub status: u16,
    /// HTTP status text.
    pub status_text: String,
    /// Decoded response body captured from the error response.
    pub body: ResponseBody,
    message: String,
}

impl OrderBookApiError {
    /// Creates a typed API error from status metadata and a decoded body.
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
            status_text,
            body,
            message,
        }
    }
}

/// Token-bucket settings for the shared request limiter.
///
/// Closed internally so the SDK can add limiter knobs additively; external
/// callers should construct through [`new`](Self::new).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimitSettings {
    /// Number of requests allowed per limiter interval.
    pub tokens_per_interval: u32,
    /// Duration of the limiter window.
    pub interval: Duration,
    /// Human-readable label for the limiter interval used in docs and tests.
    pub interval_label: &'static str,
}

impl RateLimitSettings {
    /// Creates token-bucket settings from an explicit budget and interval.
    #[must_use]
    pub const fn new(
        tokens_per_interval: u32,
        interval: Duration,
        interval_label: &'static str,
    ) -> Self {
        Self {
            tokens_per_interval,
            interval,
            interval_label,
        }
    }
}

impl Default for RateLimitSettings {
    fn default() -> Self {
        Self::new(
            DEFAULT_TOKENS_PER_INTERVAL,
            Duration::from_secs(1),
            DEFAULT_INTERVAL_LABEL,
        )
    }
}

/// Retry and rate-limit policy for orderbook HTTP requests.
///
/// Closed internally so the SDK can add policy fields additively; external
/// callers should construct through [`new`](Self::new).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestPolicy {
    /// Maximum number of attempts before surfacing an error.
    pub max_attempts: usize,
    /// Shared limiter settings applied before every attempt.
    pub rate_limit: RateLimitSettings,
}

impl Default for RequestPolicy {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_ATTEMPTS, RateLimitSettings::default())
    }
}

impl RequestPolicy {
    /// Creates a retry policy from explicit attempt and rate-limit settings.
    #[must_use]
    pub const fn new(max_attempts: usize, rate_limit: RateLimitSettings) -> Self {
        Self {
            max_attempts,
            rate_limit,
        }
    }

    /// Returns `true` when `status` should be retried under this policy.
    #[must_use]
    pub fn should_retry_status(&self, status: u16) -> bool {
        RETRYABLE_STATUS_CODES.contains(&status)
    }

    /// Returns the exponential backoff delay for `attempt_index`.
    ///
    /// # Panics
    ///
    /// Panics only if the internally clamped retry exponent no longer fits into `u32`.
    /// The implementation clamps it to a `u32`-safe range before conversion.
    #[must_use]
    pub fn backoff_delay(&self, attempt_index: usize) -> Duration {
        let exponent = u32::try_from(attempt_index.saturating_sub(1).min(6))
            .expect("backoff exponent is clamped to a `u32`-safe range");
        Duration::from_millis(50 * (1u64 << exponent))
    }
}

/// Combined client-policy and request-policy surface for the orderbook client.
///
/// Closed internally so the SDK can add transport knobs additively while
/// external callers use [`new`](Self::new) and the `with_*` setters.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderBookTransportPolicy {
    http_policy: HttpClientPolicy,
    request: RequestPolicy,
}

impl Default for OrderBookTransportPolicy {
    fn default() -> Self {
        Self {
            http_policy: HttpClientPolicy::new(DEFAULT_ORDERBOOK_USER_AGENT)
                .expect("static orderbook user-agent must remain valid"),
            request: RequestPolicy::default(),
        }
    }
}

impl OrderBookTransportPolicy {
    /// Creates a transport policy from explicit shared-client and request policies.
    #[must_use]
    pub const fn new(client: HttpClientPolicy, request: RequestPolicy) -> Self {
        Self {
            http_policy: client,
            request,
        }
    }

    /// Returns the shared HTTP client policy.
    #[must_use]
    pub const fn client_policy(&self) -> &HttpClientPolicy {
        &self.http_policy
    }

    /// Returns the request retry and limiter policy.
    #[must_use]
    pub const fn request_policy(&self) -> &RequestPolicy {
        &self.request
    }

    /// Returns a copy of this transport policy with a new HTTP client policy.
    #[must_use]
    pub fn with_client_policy(mut self, client: HttpClientPolicy) -> Self {
        self.http_policy = client;
        self
    }

    /// Returns a copy of this transport policy with a new request policy.
    #[must_use]
    pub const fn with_request_policy(mut self, request: RequestPolicy) -> Self {
        self.request = request;
        self
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

struct RequestExecution<'a> {
    transport: &'a SharedTransport,
    base_url: &'a str,
    params: &'a FetchParams,
    timeout: Option<Duration>,
    additional_headers: Option<HeaderMap>,
}

/// Shared token-bucket limiter used by orderbook request helpers.
#[derive(Debug, Clone)]
pub struct RequestRateLimiter {
    settings: RateLimitSettings,
    state: Arc<Mutex<LimiterState>>,
}

#[derive(Debug, Clone)]
struct LimiterState {
    window_started_at: Instant,
    remaining_tokens: u32,
}

impl RequestRateLimiter {
    /// Creates a new limiter with the provided settings.
    #[must_use]
    pub fn new(settings: RateLimitSettings) -> Self {
        Self {
            settings,
            state: Arc::new(Mutex::new(LimiterState {
                window_started_at: Instant::now(),
                remaining_tokens: settings.tokens_per_interval,
            })),
        }
    }

    #[allow(
        clippy::significant_drop_tightening,
        reason = "the async mutex guard is already scoped to the inner block and is released before awaiting the timer"
    )]
    async fn acquire(&self) {
        loop {
            let wait_for = {
                let mut state = self.state.lock().await;
                let elapsed = state.window_started_at.elapsed();

                if elapsed >= self.settings.interval {
                    state.window_started_at = Instant::now();
                    state.remaining_tokens = self.settings.tokens_per_interval;
                }

                if state.remaining_tokens > 0 {
                    state.remaining_tokens -= 1;
                    None
                } else {
                    Some(self.settings.interval.saturating_sub(elapsed))
                }
            };

            match wait_for {
                Some(duration) if !duration.is_zero() => delay_for(duration).await,
                _ => return,
            }
        }
    }
}

/// Executes a JSON request without overriding the shared-client timeout.
///
/// # Errors
///
/// Returns [`OrderbookError`] when request execution fails, the API returns a
/// non-success response, or the success body cannot be decoded as JSON.
pub async fn request_json<T>(
    transport: &SharedTransport,
    base_url: &str,
    params: &FetchParams,
    policy: &RequestPolicy,
    rate_limiter: &RequestRateLimiter,
    additional_headers: Option<HeaderMap>,
) -> Result<T, OrderbookError>
where
    T: DeserializeOwned,
{
    request_json_with_timeout(
        transport,
        base_url,
        params,
        policy,
        rate_limiter,
        None,
        additional_headers,
    )
    .await
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
    policy: &RequestPolicy,
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

/// Executes a text request without overriding the shared-client timeout.
///
/// # Errors
///
/// Returns [`OrderbookError`] when request execution fails, the API returns a
/// non-success response, or the success body cannot be decoded as UTF-8 text.
pub async fn request_text(
    transport: &SharedTransport,
    base_url: &str,
    params: &FetchParams,
    policy: &RequestPolicy,
    rate_limiter: &RequestRateLimiter,
    additional_headers: Option<HeaderMap>,
) -> Result<String, OrderbookError> {
    request_text_with_timeout(
        transport,
        base_url,
        params,
        policy,
        rate_limiter,
        None,
        additional_headers,
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
    policy: &RequestPolicy,
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

/// Executes a request that expects an empty success body.
///
/// # Errors
///
/// Returns [`OrderbookError`] when request execution fails or the API returns a
/// non-success response.
pub async fn request_empty(
    transport: &SharedTransport,
    base_url: &str,
    params: &FetchParams,
    policy: &RequestPolicy,
    rate_limiter: &RequestRateLimiter,
    additional_headers: Option<HeaderMap>,
) -> Result<(), OrderbookError> {
    request_empty_with_timeout(
        transport,
        base_url,
        params,
        policy,
        rate_limiter,
        None,
        additional_headers,
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
    policy: &RequestPolicy,
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
    policy: &RequestPolicy,
    rate_limiter: &RequestRateLimiter,
    attempt: F,
) -> Result<T, OrderbookError>
where
    T: DeserializeOwned,
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<ResponseEnvelope, (cow_sdk_core::TransportErrorClass, String)>>,
{
    execute_with(policy, rate_limiter, attempt, decode_success_body::<T>).await
}

/// Executes an abstract text-producing attempt with retry and rate-limit policy.
///
/// # Errors
///
/// Returns [`OrderbookError`] when all attempts fail, the API returns a
/// non-success response, or the success body cannot be decoded as text.
pub async fn execute_text_with<F, Fut>(
    policy: &RequestPolicy,
    rate_limiter: &RequestRateLimiter,
    attempt: F,
) -> Result<String, OrderbookError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<ResponseEnvelope, (cow_sdk_core::TransportErrorClass, String)>>,
{
    execute_with(policy, rate_limiter, attempt, decode_text_body).await
}

/// Executes an abstract empty-body attempt with retry and rate-limit policy.
///
/// # Errors
///
/// Returns [`OrderbookError`] when all attempts fail or the API returns a
/// non-success response.
pub async fn execute_empty_with<F, Fut>(
    policy: &RequestPolicy,
    rate_limiter: &RequestRateLimiter,
    attempt: F,
) -> Result<(), OrderbookError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<ResponseEnvelope, (cow_sdk_core::TransportErrorClass, String)>>,
{
    execute_with(policy, rate_limiter, attempt, |_| Ok(())).await
}

async fn request_with<T, D>(
    request: RequestExecution<'_>,
    policy: &RequestPolicy,
    rate_limiter: &RequestRateLimiter,
    response_kind: ResponseKind,
    decode_success: D,
) -> Result<T, OrderbookError>
where
    D: Fn(&ResponseEnvelope) -> Result<T, OrderbookError>,
{
    let url = format!("{}{}", request.base_url, request.params.path);
    let transport = Arc::clone(request.transport);
    let params = request.params.clone();
    let timeout = request.timeout;
    let additional_headers = request.additional_headers;

    execute_with(
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
) -> Result<ResponseEnvelope, (cow_sdk_core::TransportErrorClass, String)> {
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
        Ok(body) => Ok(ResponseEnvelope {
            status: 200,
            status_text: canonical_status_text(200),
            content_type: None,
            body: body.into_bytes(),
        }),
        Err(TransportError::HttpStatus { status, body }) => Ok(ResponseEnvelope {
            status,
            status_text: canonical_status_text(status),
            content_type: None,
            body: body.into_bytes(),
        }),
        Err(TransportError::Transport { class, detail }) => Err((class, detail)),
        Err(TransportError::Configuration { message }) => {
            Err((cow_sdk_core::TransportErrorClass::Builder, message))
        }
        Err(other) => Err((cow_sdk_core::TransportErrorClass::Other, other.to_string())),
    }
}

fn append_query_string(url: &str, query: &[(String, String)]) -> Result<String, String> {
    if query.is_empty() {
        return Ok(url.to_owned());
    }
    reqwest::Url::parse_with_params(
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
    policy: &RequestPolicy,
    rate_limiter: &RequestRateLimiter,
    mut attempt: F,
    decode_success: D,
) -> Result<T, OrderbookError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<ResponseEnvelope, (cow_sdk_core::TransportErrorClass, String)>>,
    D: Fn(&ResponseEnvelope) -> Result<T, OrderbookError>,
{
    let mut last_transport_error = None;

    for attempt_index in 1..=policy.max_attempts {
        rate_limiter.acquire().await;

        match attempt().await {
            Ok(response) if (200..300).contains(&response.status) => {
                return decode_success(&response);
            }
            Ok(response) => {
                let body = response.decoded_body();
                let error = OrderBookApiError::new(response.status, response.status_text, body);
                let should_retry =
                    policy.should_retry_status(error.status) && attempt_index < policy.max_attempts;

                if should_retry {
                    delay_for(policy.backoff_delay(attempt_index)).await;
                    continue;
                }

                return Err(error.into());
            }
            Err(error) => {
                last_transport_error = Some(error);

                if attempt_index < policy.max_attempts {
                    delay_for(policy.backoff_delay(attempt_index)).await;
                }
            }
        }
    }

    let (class, detail) = last_transport_error.unwrap_or_else(|| {
        (
            cow_sdk_core::TransportErrorClass::Other,
            "request attempts exhausted".to_owned(),
        )
    });
    Err(OrderbookError::Transport { class, detail })
}

async fn delay_for(duration: Duration) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Delay::new(duration).await;
    }

    #[cfg(target_arch = "wasm32")]
    {
        let millis = u32::try_from(duration.as_millis().min(u128::from(u32::MAX)))
            .expect("millisecond delay is clamped to `u32::MAX`");
        if millis > 0 {
            TimeoutFuture::new(millis).await;
        }
    }
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
        detail: error.to_string(),
    })
}

fn canonical_status_text(status: u16) -> String {
    reqwest::StatusCode::from_u16(status)
        .ok()
        .and_then(|status| status.canonical_reason().map(ToOwned::to_owned))
        .unwrap_or_else(|| "Unknown Status".to_owned())
}
