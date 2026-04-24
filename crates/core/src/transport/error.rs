//! Typed error surface returned by every [`HttpTransport`] implementation.
//!
//! The [`TransportError`] enum partitions transport-layer failures into a
//! categorical `Transport` variant, a builder-time `Configuration` variant,
//! and a structured `HttpStatus` variant. Downstream telemetry, retry, and
//! backoff layers observe a uniform classification without parsing
//! free-form error strings.
//!
//! [`HttpTransport`]: super::HttpTransport

use thiserror::Error;

use crate::validation::TransportErrorClass;

/// Typed error surface returned by every
/// [`HttpTransport`](super::HttpTransport) implementation.
///
/// Transport adapters funnel every failure into this enum so downstream
/// telemetry, retry, and backoff layers observe a uniform classification
/// without parsing free-form error strings. The
/// [`Transport`](Self::Transport) variant pairs a [`TransportErrorClass`] tag
/// with a redacted detail string; the [`Configuration`](Self::Configuration)
/// variant captures builder-time or input-validation failures that happen
/// before a network request is dispatched; the
/// [`HttpStatus`](Self::HttpStatus) variant captures a non-2xx response so
/// the orderbook and subgraph layers receive the numeric status, response
/// headers, and body together without rebuilding them from free-form error
/// strings.
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
    /// Non-2xx HTTP response returned by the remote endpoint.
    ///
    /// Adapters surface the numeric status code and the raw response body
    /// together with the response headers through this variant so downstream
    /// orchestration can classify the outcome without re-reading the
    /// transport's rendered error text.
    #[error("http status error ({status}): {body}")]
    HttpStatus {
        /// Numeric HTTP status code returned by the remote endpoint.
        status: u16,
        /// Response headers returned alongside the non-success status code.
        headers: Vec<(String, String)>,
        /// Raw response body returned alongside the status code.
        body: String,
    },
}

impl TransportError {
    /// Returns the [`TransportErrorClass`] for [`Transport`](Self::Transport)
    /// variants and [`None`] for configuration and HTTP-status failures.
    #[must_use]
    pub const fn class(&self) -> Option<TransportErrorClass> {
        match self {
            Self::Transport { class, .. } => Some(*class),
            Self::Configuration { .. } | Self::HttpStatus { .. } => None,
        }
    }
}
