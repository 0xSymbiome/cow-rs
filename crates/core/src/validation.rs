//! Shared validation-failure and transport-classification enums used across
//! the `cow-sdk` crate family.
//!
//! [`ValidationReason`] describes canonical validation-failure modes on the
//! public input boundary. Downstream crates that surface structured
//! `{ field, reason: ValidationReason }` error variants route through this
//! enum so callers can pattern-match on the reason without parsing free-form
//! strings.
//!
//! [`TransportErrorClass`] classifies REST-transport failure categories
//! produced by `reqwest` error states. Downstream crates that expose typed
//! `Transport { class, detail }` error variants use this shared enum so
//! telemetry and retry policies can partition transport outcomes uniformly.

use thiserror::Error;

/// Canonical validation-failure modes emitted by structured error variants.
///
/// Variants carry `&'static str` detail slots where applicable so consumers
/// get actionable context without paying the cost of a heap allocation.
/// `Missing` names the field itself as the whole reason; structured call
/// sites still carry a `field: &'static str` alongside this enum to make the
/// combination self-describing.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValidationReason {
    /// The field was required but was not supplied.
    #[error("missing required value")]
    Missing,
    /// The field was out of the documented acceptable range.
    #[error("out of range: {details}")]
    OutOfRange {
        /// Narrow explanation of the out-of-range condition.
        details: &'static str,
    },
    /// The field did not match the documented shape or format.
    #[error("bad shape: {details}")]
    BadShape {
        /// Narrow explanation of the shape violation.
        details: &'static str,
    },
    /// The field violated a documented precondition for the operation.
    #[error("precondition violated: {details}")]
    Precondition {
        /// Narrow explanation of the violated precondition.
        details: &'static str,
    },
}

/// REST-transport error classification shared across transport-capable crates.
///
/// Built by classifying a [`reqwest::Error`] through the documented partition
/// (`is_timeout`, `is_connect`, `is_redirect`, `is_decode`, `is_body`,
/// `is_builder`, `is_request`, `is_status`, fallthrough). The
/// [`ResponseTooLarge`](Self::ResponseTooLarge) class is the exception: it is
/// produced by the transport's own response-size guard rather than by
/// `reqwest` classification. Downstream error surfaces pair this enum with a
/// redacted detail string to produce the public `Transport { class, detail }`
/// variant shape.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportErrorClass {
    /// The request timed out before a response was received.
    Timeout,
    /// The request could not establish a connection to the remote host.
    Connect,
    /// The request hit a redirect-handling failure.
    Redirect,
    /// Response body decoding failed.
    Decode,
    /// Request or response body-stream handling failed.
    Body,
    /// The request could not be built locally.
    Builder,
    /// The request failed at the HTTP request layer without a structured status.
    Request,
    /// The server returned a non-success status code without a structured body.
    Status,
    /// HTTP `101 Switching Protocols` upgrade (e.g., WebSocket); reserved
    /// for future streaming response API. Not currently produced by any
    /// in-tree transport implementation.
    Upgrade,
    /// The response body exceeded the configured maximum size, so the
    /// transport refused to buffer it. Produced by the SDK's response-size
    /// guard, not by `reqwest` classification.
    ResponseTooLarge,
    /// The transport failure does not match any of the named categories.
    Other,
}

impl TransportErrorClass {
    /// Returns the canonical lowercase label for this class, suitable for
    /// telemetry labels and structured log fields.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::Connect => "connect",
            Self::Redirect => "redirect",
            Self::Decode => "decode",
            Self::Body => "body",
            Self::Builder => "builder",
            Self::Request => "request",
            Self::Status => "status",
            Self::Upgrade => "upgrade",
            Self::ResponseTooLarge => "response_too_large",
            Self::Other => "other",
        }
    }
}

impl std::fmt::Display for TransportErrorClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
