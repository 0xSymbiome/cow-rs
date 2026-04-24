use std::time::Duration;

use async_trait::async_trait;

use crate::transport::error::TransportError;

/// Production injection point for HTTPS REST transport.
///
/// Implementations dispatch REST requests without committing the calling
/// crate to any specific backend. The native default implementation is
/// [`ReqwestTransport`](crate::transport::ReqwestTransport); the browser
/// default implementation lives in `cow-sdk-transport-wasm` and bridges the
/// same async signature through `JsFuture`.
///
/// Every method carries the per-call header set and an optional per-call
/// timeout alongside the URL and body so downstream crates compose typed
/// clients without holding a parallel `reqwest::Client` for header or
/// deadline overrides. Implementations merge per-call headers with any
/// constructor-configured defaults, honor the per-call timeout when `Some`,
/// and map non-2xx responses into
/// [`TransportError::HttpStatus`](crate::transport::TransportError::HttpStatus)
/// so the calling layer receives the numeric status, response headers, and
/// raw body through the typed error channel instead of through `Ok(String)`.
///
/// The trait uses [`macro@async_trait`] so downstream clients can hold the
/// transport behind `Arc<dyn HttpTransport>` without reaching for a
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
    ) -> Result<String, TransportError>;

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
    ) -> Result<String, TransportError>;

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
    ) -> Result<String, TransportError>;

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
    ) -> Result<String, TransportError>;
}
