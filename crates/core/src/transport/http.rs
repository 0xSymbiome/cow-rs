use async_trait::async_trait;
use thiserror::Error;

use crate::validation::TransportErrorClass;

/// Typed error surface returned by every [`HttpTransport`] implementation.
///
/// Transport adapters funnel every failure into this enum so downstream
/// telemetry, retry, and backoff layers observe a uniform classification
/// without parsing free-form error strings. The
/// [`Transport`](Self::Transport) variant pairs a [`TransportErrorClass`] tag
/// with a redacted detail string; the [`Configuration`](Self::Configuration)
/// variant captures builder-time or input-validation failures that happen
/// before a network request is dispatched.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum TransportError {
    /// Network or request-execution failure observed by the transport layer.
    #[error("transport error ({class}): {detail}")]
    Transport {
        /// Categorical failure mode taken from the documented
        /// `is_timeout`, `is_connect`, `is_redirect`, `is_decode`, `is_body`,
        /// `is_builder`, `is_request`, `is_status`, fallthrough partition.
        class: TransportErrorClass,
        /// Redacted detail message with any URL stripped by the adapter before
        /// the wrap.
        detail: String,
    },
    /// Builder-time or input-validation failure that prevented a request from
    /// being dispatched.
    #[error("transport configuration error: {message}")]
    Configuration {
        /// Human-readable configuration-validation message.
        message: String,
    },
}

impl TransportError {
    /// Returns the [`TransportErrorClass`] for [`Transport`](Self::Transport)
    /// variants and [`None`] for configuration failures.
    #[must_use]
    pub const fn class(&self) -> Option<TransportErrorClass> {
        match self {
            Self::Transport { class, .. } => Some(*class),
            Self::Configuration { .. } => None,
        }
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
/// The trait uses [`macro@async_trait`] so downstream clients can hold the
/// transport behind `Arc<dyn HttpTransport>` without reaching for a
/// bespoke adapter trait. Implementations carry [`std::fmt::Debug`] so
/// trait objects render in derived `Debug` output of consumer-facing
/// clients without bespoke formatters. The returned futures are `!Send`
/// to keep the browser implementation viable; consumers that want to pin
/// a native transport onto a multi-threaded runtime keep the concrete
/// type or wrap it in `Arc<dyn HttpTransport + Send + Sync>` through
/// their own thin newtype.
#[async_trait(?Send)]
pub trait HttpTransport: std::fmt::Debug {
    /// Performs an HTTP `GET` against the supplied path.
    ///
    /// The semantics of `path` are adapter-defined: the native
    /// [`ReqwestTransport`](crate::transport::ReqwestTransport) resolves it
    /// against the configured base URL, while other adapters may interpret
    /// it as an absolute URL.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError::Transport`] when the underlying backend
    /// fails, with [`TransportError::class`] set to the categorical failure
    /// mode. Returns [`TransportError::Configuration`] when the adapter
    /// could not build the request from the supplied input.
    async fn get(&self, path: &str) -> Result<String, TransportError>;

    /// Performs an HTTP `POST` with a JSON-compatible body.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError::Transport`] when the underlying backend
    /// fails, with [`TransportError::class`] set to the categorical failure
    /// mode. Returns [`TransportError::Configuration`] when the adapter
    /// could not build the request from the supplied input.
    async fn post(&self, path: &str, body: &str) -> Result<String, TransportError>;

    /// Performs an HTTP `DELETE` with a JSON-compatible body.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError::Transport`] when the underlying backend
    /// fails, with [`TransportError::class`] set to the categorical failure
    /// mode. Returns [`TransportError::Configuration`] when the adapter
    /// could not build the request from the supplied input.
    async fn delete(&self, path: &str, body: &str) -> Result<String, TransportError>;
}
