use std::time::Duration;

use async_trait::async_trait;

use crate::redaction::Redacted;
use crate::transport::error::TransportError;

/// Successful HTTP response captured by an [`HttpTransport`] implementation.
///
/// Carries the numeric status code, the response headers, and the response
/// body of a 2xx dispatch — the same field set
/// [`TransportError::HttpStatus`] already carries for non-2xx responses, so
/// one representation spans the success and failure channels. Accessor names
/// mirror `http::Response` (`status`, `headers`, `into_body`) so a later
/// migration onto `http` types is a mechanical rename rather than a
/// redesign. Fields stay private so the representation can evolve behind the
/// accessors.
///
/// Header values are wrapped in [`Redacted`]: response header sections can
/// carry `Set-Cookie` or gateway-injected credentials, so values never
/// render through `Debug`. The body is the payload the caller requested and
/// is exposed raw through [`TransportResponse::body`]; the [`std::fmt::Debug`]
/// implementation prints only its byte length.
///
/// Implementations construct a value only for 2xx responses; non-2xx
/// responses keep flowing through [`TransportError::HttpStatus`]. On browser
/// targets, cross-origin header visibility is bounded by CORS exposure:
/// `Content-Type` and the other safelisted names are always readable, while
/// anything else requires the server to opt in through
/// `Access-Control-Expose-Headers`.
#[derive(Clone, PartialEq, Eq)]
pub struct TransportResponse {
    status: u16,
    headers: Vec<(String, Redacted<String>)>,
    body: String,
}

impl TransportResponse {
    /// Creates a response from its status code, headers, and body.
    #[must_use]
    pub fn new(
        status: u16,
        headers: Vec<(String, Redacted<String>)>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            status,
            headers,
            body: body.into(),
        }
    }

    /// Returns the numeric HTTP status code.
    #[must_use]
    pub const fn status(&self) -> u16 {
        self.status
    }

    /// Returns the response headers as name/value pairs in wire order.
    #[must_use]
    pub fn headers(&self) -> &[(String, Redacted<String>)] {
        &self.headers
    }

    /// Returns the first value of the named header, matching the name
    /// ASCII-case-insensitively.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_inner().as_str())
    }

    /// Returns the response body.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Consumes the response and returns the body.
    #[must_use]
    pub fn into_body(self) -> String {
        self.body
    }
}

impl std::fmt::Debug for TransportResponse {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TransportResponse")
            .field("status", &self.status)
            .field("headers", &self.headers)
            .field("body_bytes", &self.body.len())
            .finish()
    }
}

/// Production injection point for HTTPS REST transport.
///
/// Implementations dispatch REST requests without committing the calling
/// crate to any specific backend. The native default implementation is
/// [`ReqwestTransport`](crate::transport::ReqwestTransport); the browser
/// default implementation lives in `cow-sdk-transport-wasm` and bridges the
/// same async signature through `JsFuture`.
///
/// Most consumers never implement this trait. The orderbook and subgraph
/// builders install the per-target default automatically, so the zero-config
/// `.build()` path serves native and browser callers alike. Common tuning does
/// not require a custom transport either: reuse a pre-configured
/// `reqwest::Client` (proxy, custom TLS, connection pool) through the native
/// builder's `.client(..)` seam, supply credentials through `.api_key(..)` and
/// the per-call header set, and shape retry, rate limiting, timeout, and
/// user-agent through `TransportPolicy`. Implementing this trait is the
/// deliberate escape hatch for three cases: a JavaScript host supplying its own
/// `fetch` or callback (see `cow_sdk_wasm::exports::JsCallbackHttpTransport`),
/// test doubles that record or replay requests, and wrapping an inner transport
/// to add caching or other middleware. The `Arc<dyn HttpTransport>` seam is what
/// keeps those injectable at runtime.
///
/// This trait does not retry. Retry, jitter, rate limiting, and
/// `Retry-After` handling are applied at the orderbook layer via
/// `cow_sdk_core::transport::policy::TransportPolicy`. See `docs/transport.md`.
///
/// Every method carries the per-call header set and an optional per-call
/// timeout alongside the URL and body so downstream crates compose typed
/// clients without holding a parallel `reqwest::Client` for header or
/// deadline overrides. Implementations merge per-call headers with any
/// constructor-configured defaults, honor the per-call timeout when `Some`,
/// and map non-2xx responses into
/// [`TransportError::HttpStatus`]
/// so the calling layer receives the numeric status, response headers, and
/// raw body through the typed error channel. The success channel carries the
/// same fidelity: `Ok` returns a [`TransportResponse`] with the 2xx status
/// code, the response headers, and the body, so calling layers never have to
/// fabricate response metadata.
///
/// The trait uses [`macro@async_trait`] so downstream clients can hold the
/// transport behind `Arc<dyn HttpTransport + Send + Sync>` without reaching for a
/// bespoke adapter trait. Implementations carry [`std::fmt::Debug`] so
/// trait objects render in derived `Debug` output of consumer-facing
/// clients without bespoke formatters. On native targets the returned
/// futures are `Send` so downstream crates compose them onto
/// multi-threaded runtimes; on `wasm32` targets the futures drop the
/// `Send` bound so the browser adapter remains viable.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait HttpTransport: std::fmt::Debug {
    /// Performs an HTTP `GET` against the supplied path.
    ///
    /// Implementations merge `headers` with any constructor-configured
    /// defaults and apply `timeout` when `Some`, otherwise honor the
    /// transport's default timeout. The semantics of `path` are
    /// adapter-defined: the native
    /// [`ReqwestTransport`](crate::transport::ReqwestTransport) resolves it
    /// against the configured base URL, while other adapters may interpret
    /// it as an absolute URL.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError::Transport`] when the underlying backend
    /// fails, with [`TransportError::class`] set to the categorical failure
    /// mode. Returns [`TransportError::Configuration`] when the adapter
    /// could not build the request from the supplied input. Returns
    /// [`TransportError::HttpStatus`] when the remote endpoint responded
    /// with a non-2xx status code.
    async fn get(
        &self,
        path: &str,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError>;

    /// Performs an HTTP `POST` with a JSON-compatible body.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError::Transport`] when the underlying backend
    /// fails, with [`TransportError::class`] set to the categorical failure
    /// mode. Returns [`TransportError::Configuration`] when the adapter
    /// could not build the request from the supplied input. Returns
    /// [`TransportError::HttpStatus`] when the remote endpoint responded
    /// with a non-2xx status code.
    async fn post(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError>;

    /// Performs an HTTP `PUT` with a JSON-compatible body.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError::Transport`] when the underlying backend
    /// fails, with [`TransportError::class`] set to the categorical failure
    /// mode. Returns [`TransportError::Configuration`] when the adapter
    /// could not build the request from the supplied input. Returns
    /// [`TransportError::HttpStatus`] when the remote endpoint responded
    /// with a non-2xx status code.
    async fn put(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError>;

    /// Performs an HTTP `DELETE` with a JSON-compatible body.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError::Transport`] when the underlying backend
    /// fails, with [`TransportError::class`] set to the categorical failure
    /// mode. Returns [`TransportError::Configuration`] when the adapter
    /// could not build the request from the supplied input. Returns
    /// [`TransportError::HttpStatus`] when the remote endpoint responded
    /// with a non-2xx status code.
    async fn delete(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError>;
}
