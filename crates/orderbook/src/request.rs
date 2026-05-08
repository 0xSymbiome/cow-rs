use std::{
    fmt,
    future::Future,
    sync::{Arc, Mutex as StdMutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use async_lock::Mutex;
use cow_sdk_core::{HttpClientPolicy, HttpTransport, Redacted, TransportError};
use http::header::{ACCEPT, CONTENT_TYPE, HeaderMap};
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
const RETRY_AFTER_HEADER: &str = "retry-after";
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
const DEFAULT_JITTER_WINDOW_DIVISOR: u128 = 2;

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
pub struct OrderBookApiError {
    /// HTTP status code.
    pub status: u16,
    /// HTTP status text.
    pub status_text: Redacted<String>,
    /// Decoded response body captured from the error response.
    pub body: Redacted<ResponseBody>,
    message: Redacted<String>,
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
            status_text: Redacted::new(status_text),
            body: Redacted::new(body),
            message: Redacted::new(message),
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

/// Jitter policy applied to retry backoff delays.
///
/// The default decorrelated strategy seeds its first offset from the operating
/// system random source and then advances an internal sequence so cloned retry
/// policies do not schedule identical waits for parallel retrying calls.
#[derive(Clone)]
pub struct JitterStrategy {
    kind: JitterStrategyKind,
    sequence: Arc<StdMutex<u64>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JitterStrategyKind {
    None,
    Decorrelated { seed: u64 },
}

impl JitterStrategy {
    /// Returns a strategy that leaves retry delays unchanged.
    #[must_use]
    pub fn none() -> Self {
        Self::from_kind(JitterStrategyKind::None)
    }

    /// Returns the default decorrelated retry jitter strategy.
    ///
    /// The seed is read from `getrandom` when available. If the platform
    /// entropy source is temporarily unavailable, the strategy falls back to a
    /// time-derived seed because retry jitter is operational desynchronization,
    /// not cryptographic randomness.
    #[must_use]
    pub fn decorrelated() -> Self {
        Self::decorrelated_from_seed(random_jitter_seed())
    }

    /// Returns a decorrelated strategy with a caller-supplied seed.
    ///
    /// This is primarily useful for deterministic tests and controlled
    /// deployments that need reproducible retry schedules.
    #[must_use]
    pub fn decorrelated_from_seed(seed: u64) -> Self {
        Self::from_kind(JitterStrategyKind::Decorrelated { seed })
    }

    fn from_kind(kind: JitterStrategyKind) -> Self {
        Self {
            kind,
            sequence: Arc::new(StdMutex::new(0)),
        }
    }

    /// Returns the jitter offset for one retry base delay.
    ///
    /// # Panics
    ///
    /// Panics only if the explicitly capped jitter window no longer fits into
    /// `u64`.
    fn delay_for(&self, base: Duration) -> Duration {
        let JitterStrategyKind::Decorrelated { seed } = self.kind else {
            return Duration::ZERO;
        };

        let window_ms = base.as_millis() / DEFAULT_JITTER_WINDOW_DIVISOR;
        if window_ms == 0 {
            return Duration::ZERO;
        }

        let sequence = self.next_sequence();
        let window = window_ms.saturating_add(1).min(u128::from(u64::MAX));
        let offset = (u128::from(splitmix64(seed)) + u128::from(sequence)) % window;
        // SAFETY: window is capped to u64::MAX before the modulo operation, so
        // offset is representable as u64.
        let offset = u64::try_from(offset).expect("jitter offset is capped to `u64::MAX`");
        Duration::from_millis(offset)
    }

    fn next_sequence(&self) -> u64 {
        let mut sequence = self
            .sequence
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let current = *sequence;
        *sequence = current.wrapping_add(1);
        current
    }
}

impl Default for JitterStrategy {
    fn default() -> Self {
        Self::decorrelated()
    }
}

impl fmt::Debug for JitterStrategy {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("JitterStrategy")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl PartialEq for JitterStrategy {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Eq for JitterStrategy {}

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
    /// Retry jitter strategy applied to exponential backoff delays.
    pub jitter: JitterStrategy,
}

impl Default for RequestPolicy {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_ATTEMPTS, RateLimitSettings::default())
    }
}

impl RequestPolicy {
    /// Creates a retry policy from explicit attempt and rate-limit settings.
    #[must_use]
    pub fn new(max_attempts: usize, rate_limit: RateLimitSettings) -> Self {
        Self {
            max_attempts,
            rate_limit,
            jitter: JitterStrategy::default(),
        }
    }

    /// Returns this policy with an explicit retry jitter strategy.
    #[must_use]
    pub fn with_jitter(mut self, jitter: JitterStrategy) -> Self {
        self.jitter = jitter;
        self
    }

    /// Returns `true` when `status` should be retried under this policy.
    #[must_use]
    pub fn should_retry_status(&self, status: u16) -> bool {
        RETRYABLE_STATUS_CODES.contains(&status)
    }

    /// Returns the jittered exponential backoff delay for `attempt_index`.
    ///
    /// # Panics
    ///
    /// Panics only if the internally clamped retry exponent no longer fits into `u32`.
    /// The implementation clamps it to a `u32`-safe range before conversion.
    #[must_use]
    pub fn backoff_delay(&self, attempt_index: usize) -> Duration {
        let base = Self::base_backoff_delay(attempt_index);
        base.saturating_add(self.jitter.delay_for(base))
    }

    /// Returns the unclamped exponential backoff base for an attempt.
    ///
    /// # Panics
    ///
    /// Panics only if the locally capped retry exponent no longer fits into
    /// `u32`.
    fn base_backoff_delay(attempt_index: usize) -> Duration {
        // SAFETY: the exponent is capped to six before conversion, which is
        // always representable as u32.
        let exponent = u32::try_from(attempt_index.saturating_sub(1).min(6))
            .expect("backoff exponent is clamped to a `u32`-safe range");
        Duration::from_millis(50 * (1u64 << exponent))
    }

    fn retry_delay(
        &self,
        attempt_index: usize,
        status: u16,
        headers: &[(String, String)],
        now: SystemTime,
    ) -> Duration {
        let backoff = self.backoff_delay(attempt_index);
        if !matches!(status, TOO_MANY_REQUESTS | SERVICE_UNAVAILABLE) {
            return backoff;
        }

        retry_after_delay(headers, now).map_or(backoff, |retry_after| backoff.max(retry_after))
    }
}

fn random_jitter_seed() -> u64 {
    let mut seed = [0_u8; 8];
    if getrandom::fill(&mut seed).is_ok() {
        return u64::from_le_bytes(seed);
    }

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0x9E37_79B9_7F4A_7C15, |duration| {
            duration.as_secs().rotate_left(32) ^ u64::from(duration.subsec_nanos())
        })
}

const fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
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
    /// Creates the default orderbook transport policy.
    ///
    /// # Panics
    ///
    /// Panics only if the crate-owned default orderbook user-agent literal
    /// stops being encodable as an HTTP header value.
    fn default() -> Self {
        Self {
            // SAFETY: DEFAULT_ORDERBOOK_USER_AGENT is a crate-owned static
            // literal validated by the shared HTTP policy constructor.
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
    pub fn with_request_policy(mut self, request: RequestPolicy) -> Self {
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
    mut attempt: F,
) -> Result<T, OrderbookError>
where
    T: DeserializeOwned,
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<ResponseEnvelope, (cow_sdk_core::TransportErrorClass, String)>>,
{
    execute_with(
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

/// Executes an abstract text-producing attempt with retry and rate-limit policy.
///
/// # Errors
///
/// Returns [`OrderbookError`] when all attempts fail, the API returns a
/// non-success response, or the success body cannot be decoded as text.
pub async fn execute_text_with<F, Fut>(
    policy: &RequestPolicy,
    rate_limiter: &RequestRateLimiter,
    mut attempt: F,
) -> Result<String, OrderbookError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<ResponseEnvelope, (cow_sdk_core::TransportErrorClass, String)>>,
{
    execute_with(
        policy,
        rate_limiter,
        move || {
            let future = attempt();
            async move { future.await.map(AttemptOutcome::Response) }
        },
        decode_text_body,
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
    policy: &RequestPolicy,
    rate_limiter: &RequestRateLimiter,
    mut attempt: F,
) -> Result<(), OrderbookError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<ResponseEnvelope, (cow_sdk_core::TransportErrorClass, String)>>,
{
    execute_with(
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
        Ok(body) => Ok(AttemptOutcome::Response(ResponseEnvelope {
            status: 200,
            status_text: canonical_status_text(200),
            content_type: None,
            body: body.into_bytes(),
        })),
        Err(TransportError::HttpStatus {
            status,
            headers,
            body,
        }) => Ok(AttemptOutcome::HttpError {
            response: ResponseEnvelope {
                status,
                status_text: canonical_status_text(status),
                content_type: None,
                body: body.into_inner().into_bytes(),
            },
            headers: headers
                .into_iter()
                .map(|(name, value)| (name, value.into_inner()))
                .collect(),
        }),
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
    policy: &RequestPolicy,
    rate_limiter: &RequestRateLimiter,
    mut attempt: F,
    decode_success: D,
) -> Result<T, OrderbookError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<AttemptOutcome, (cow_sdk_core::TransportErrorClass, String)>>,
    D: Fn(&ResponseEnvelope) -> Result<T, OrderbookError>,
{
    let mut last_transport_error = None;

    for attempt_index in 1..=policy.max_attempts {
        rate_limiter.acquire().await;
        #[cfg(feature = "tracing")]
        record_span_attempts(attempt_index);

        match attempt().await {
            Ok(AttemptOutcome::Response(response)) if (200..300).contains(&response.status) => {
                #[cfg(feature = "tracing")]
                record_span_status(response.status);
                return decode_success(&response);
            }
            Ok(outcome) => {
                let (response, headers) = match outcome {
                    AttemptOutcome::Response(response) => (response, None),
                    AttemptOutcome::HttpError { response, headers } => (response, Some(headers)),
                };
                #[cfg(feature = "tracing")]
                record_span_status(response.status);
                let body = response.decoded_body();
                let error = OrderBookApiError::new(response.status, response.status_text, body);
                let should_retry =
                    policy.should_retry_status(error.status) && attempt_index < policy.max_attempts;

                if should_retry {
                    let delay = headers.as_ref().map_or_else(
                        || policy.backoff_delay(attempt_index),
                        |headers| {
                            policy.retry_delay(
                                attempt_index,
                                error.status,
                                headers,
                                SystemTime::now(),
                            )
                        },
                    );
                    #[cfg(feature = "tracing")]
                    emit_retry_status_event(attempt_index, error.status, delay);
                    delay_for(delay).await;
                    continue;
                }

                #[cfg(feature = "tracing")]
                if policy.should_retry_status(error.status) {
                    emit_final_status_event(attempt_index, error.status);
                }
                return Err(error.into());
            }
            Err(error) => {
                #[cfg(feature = "tracing")]
                let error_class = error.0;
                last_transport_error = Some(error);

                if attempt_index < policy.max_attempts {
                    let delay = policy.backoff_delay(attempt_index);
                    #[cfg(feature = "tracing")]
                    emit_retry_transport_event(attempt_index, error_class, delay);
                    delay_for(delay).await;
                } else {
                    #[cfg(feature = "tracing")]
                    emit_final_transport_event(attempt_index, error_class);
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
    Err(OrderbookError::Transport {
        class,
        detail: Redacted::new(detail),
    })
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

#[cfg(feature = "tracing")]
fn emit_retry_status_event(attempt_index: usize, status: u16, delay: Duration) {
    tracing::debug!(
        attempt_index = u64::try_from(attempt_index).unwrap_or(u64::MAX),
        status = u64::from(status),
        backoff_ms = duration_millis(delay),
        "orderbook retry scheduled after status response"
    );
}

#[cfg(feature = "tracing")]
fn emit_final_status_event(attempt_index: usize, status: u16) {
    tracing::warn!(
        attempt_index = u64::try_from(attempt_index).unwrap_or(u64::MAX),
        status = u64::from(status),
        backoff_ms = 0_u64,
        "orderbook retry attempts exhausted after status response"
    );
}

#[cfg(feature = "tracing")]
fn emit_retry_transport_event(
    attempt_index: usize,
    class: cow_sdk_core::TransportErrorClass,
    delay: Duration,
) {
    tracing::debug!(
        attempt_index = u64::try_from(attempt_index).unwrap_or(u64::MAX),
        transport_error_class = class.as_str(),
        backoff_ms = duration_millis(delay),
        "orderbook retry scheduled after transport error"
    );
}

#[cfg(feature = "tracing")]
fn emit_final_transport_event(attempt_index: usize, class: cow_sdk_core::TransportErrorClass) {
    tracing::warn!(
        attempt_index = u64::try_from(attempt_index).unwrap_or(u64::MAX),
        transport_error_class = class.as_str(),
        backoff_ms = 0_u64,
        "orderbook retry attempts exhausted after transport error"
    );
}

#[cfg(feature = "tracing")]
fn duration_millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

/// Sleeps for the supplied retry delay on the active target.
///
/// # Panics
///
/// Panics only on `wasm32` if the millisecond duration no longer fits into
/// `u32` after explicit clamping.
async fn delay_for(duration: Duration) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Delay::new(duration).await;
    }

    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the duration is clamped to u32::MAX before conversion for
        // the wasm timer API.
        let millis = u32::try_from(duration.as_millis().min(u128::from(u32::MAX)))
            .expect("millisecond delay is clamped to `u32::MAX`");
        // TimeoutFuture::new(0) yields on wasm32, matching native zero-delay semantics.
        TimeoutFuture::new(millis).await;
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
        detail: Redacted::new(error.to_string()),
    })
}

fn canonical_status_text(status: u16) -> String {
    http::StatusCode::from_u16(status)
        .ok()
        .and_then(|status| status.canonical_reason().map(ToOwned::to_owned))
        .unwrap_or_else(|| "Unknown Status".to_owned())
}

fn retry_after_delay(headers: &[(String, String)], now: SystemTime) -> Option<Duration> {
    headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(RETRY_AFTER_HEADER))
        .and_then(|(_, value)| parse_retry_after(value, now))
}

fn parse_retry_after(value: &str, now: SystemTime) -> Option<Duration> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.chars().all(|character| character.is_ascii_digit()) {
        return trimmed.parse::<u64>().ok().map(Duration::from_secs);
    }

    let retry_at = parse_http_date(trimmed)?;
    let now = unix_timestamp(now)?;
    if retry_at <= now {
        return Some(Duration::from_secs(0));
    }
    Some(Duration::from_secs((retry_at - now).cast_unsigned()))
}

fn parse_http_date(value: &str) -> Option<i64> {
    let mut parts = value.split_ascii_whitespace();
    let weekday = parts.next()?;
    let day = parts.next()?.parse::<u32>().ok()?;
    let month = parse_http_month(parts.next()?)?;
    let year = parts.next()?.parse::<i32>().ok()?;
    let (hour, minute, second) = parse_http_time(parts.next()?)?;
    let timezone = parts.next()?;

    if !weekday.ends_with(',') || timezone != "GMT" || parts.next().is_some() {
        return None;
    }

    let days = days_from_civil(year, month, day)?;
    let seconds = i64::from(hour) * 3_600 + i64::from(minute) * 60 + i64::from(second);
    days.checked_mul(86_400)?.checked_add(seconds)
}

fn parse_http_month(value: &str) -> Option<u32> {
    match value {
        "Jan" => Some(1),
        "Feb" => Some(2),
        "Mar" => Some(3),
        "Apr" => Some(4),
        "May" => Some(5),
        "Jun" => Some(6),
        "Jul" => Some(7),
        "Aug" => Some(8),
        "Sep" => Some(9),
        "Oct" => Some(10),
        "Nov" => Some(11),
        "Dec" => Some(12),
        _ => None,
    }
}

fn parse_http_time(value: &str) -> Option<(u32, u32, u32)> {
    let mut parts = value.split(':');
    let hour = parts.next()?.parse::<u32>().ok()?;
    let minute = parts.next()?.parse::<u32>().ok()?;
    let second = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some() || hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    Some((hour, minute, second))
}

fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i64> {
    if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
        return None;
    }

    let adjusted_year = year - i32::from(month <= 2);
    let era = if adjusted_year >= 0 {
        adjusted_year
    } else {
        adjusted_year - 399
    } / 400;
    let year_of_era = adjusted_year - era * 400;
    let month_prime = month.cast_signed() + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day.cast_signed() - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

    Some(i64::from(era * 146_097 + day_of_era - 719_468))
}

const fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

const fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn unix_timestamp(time: SystemTime) -> Option<i64> {
    time.duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs()
        .try_into()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::{
        INTERNAL_SERVER_ERROR, JitterStrategy, REQUEST_TIMEOUT, RateLimitSettings, RequestPolicy,
        SERVICE_UNAVAILABLE, TOO_MANY_REQUESTS, parse_retry_after,
    };
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn parse_retry_after_covers_documented_boundary_matrix() {
        assert_eq!(
            parse_retry_after("0", UNIX_EPOCH),
            Some(Duration::from_secs(0))
        );
        assert_eq!(
            parse_retry_after("1", UNIX_EPOCH),
            Some(Duration::from_secs(1))
        );
        assert_eq!(
            parse_retry_after("60", UNIX_EPOCH),
            Some(Duration::from_secs(60))
        );
        assert_eq!(
            parse_retry_after("3600", UNIX_EPOCH),
            Some(Duration::from_secs(3_600))
        );
        assert_eq!(
            parse_retry_after("Thu, 01 Jan 1970 00:00:10 GMT", UNIX_EPOCH),
            Some(Duration::from_secs(10))
        );
        assert_eq!(
            parse_retry_after(
                "Thu, 01 Jan 1970 00:00:00 GMT",
                UNIX_EPOCH + Duration::from_secs(1),
            ),
            Some(Duration::from_secs(0))
        );
        assert_eq!(parse_retry_after("not-a-date", UNIX_EPOCH), None);
        assert_eq!(parse_retry_after("", UNIX_EPOCH), None);
    }

    #[test]
    fn retry_delay_uses_retry_after_only_for_429_and_503_and_picks_the_larger_wait() {
        let policy = RequestPolicy::new(10, RateLimitSettings::default())
            .with_jitter(JitterStrategy::none());
        let long_retry_after = vec![("Retry-After".to_owned(), "5".to_owned())];
        let short_retry_after = vec![("Retry-After".to_owned(), "1".to_owned())];
        let now = UNIX_EPOCH;

        assert_eq!(
            policy.retry_delay(1, TOO_MANY_REQUESTS, &long_retry_after, now),
            Duration::from_secs(5)
        );
        assert_eq!(
            policy.retry_delay(1, SERVICE_UNAVAILABLE, &long_retry_after, now),
            Duration::from_secs(5)
        );
        assert_eq!(
            policy.retry_delay(7, TOO_MANY_REQUESTS, &short_retry_after, now),
            Duration::from_millis(3_200)
        );
        assert_eq!(
            policy.retry_delay(1, INTERNAL_SERVER_ERROR, &long_retry_after, now),
            Duration::from_millis(50)
        );
        assert_eq!(
            policy.retry_delay(1, REQUEST_TIMEOUT, &long_retry_after, now),
            Duration::from_millis(50)
        );
    }
}
