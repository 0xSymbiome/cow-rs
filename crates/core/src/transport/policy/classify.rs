//! Transport-error classification helpers.

use crate::TransportErrorClass;

/// Retry-oriented network failure categories used by transport policies.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetworkErrorKind {
    /// The request timed out before a complete response was received.
    Timeout,
    /// The client could not establish a connection to the remote host.
    Connect,
    /// Response body decoding failed.
    Decode,
    /// The remote endpoint returned an HTTP status code.
    HttpStatus(u16),
    /// The operation was cancelled cooperatively.
    Cancelled,
    /// The request failed at the HTTP request layer.
    Request,
    /// The request could not be built locally.
    Builder,
    /// The response body exceeded the configured maximum size and was
    /// refused. This outcome is deterministic for a given response, so it is
    /// never retried.
    ResponseTooLarge,
    /// The transport failure does not match a more specific category.
    Other,
}

impl NetworkErrorKind {
    /// Maps the shared core transport class into a retry-oriented network kind.
    #[must_use]
    pub const fn from_transport_error_class(class: TransportErrorClass) -> Self {
        match class {
            TransportErrorClass::Timeout => Self::Timeout,
            TransportErrorClass::Connect => Self::Connect,
            TransportErrorClass::Decode | TransportErrorClass::Body => Self::Decode,
            TransportErrorClass::Status => Self::HttpStatus(0),
            TransportErrorClass::Request => Self::Request,
            TransportErrorClass::Builder => Self::Builder,
            TransportErrorClass::ResponseTooLarge => Self::ResponseTooLarge,
            _ => Self::Other,
        }
    }
}

/// Classifies transport adapter errors without exposing adapter-specific types.
pub trait ErrorClassifier {
    /// Error type accepted by the classifier.
    type Error;

    /// Returns the retry-oriented category for `error`.
    fn classify(&self, error: &Self::Error) -> NetworkErrorKind;
}

/// Classifier for native `reqwest` transport errors.
#[cfg(all(feature = "reqwest-classifier", not(target_arch = "wasm32")))]
#[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "reqwest-classifier", not(target_arch = "wasm32"))))
)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ReqwestErrorClassifier;

#[cfg(all(feature = "reqwest-classifier", not(target_arch = "wasm32")))]
#[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "reqwest-classifier", not(target_arch = "wasm32"))))
)]
impl ErrorClassifier for ReqwestErrorClassifier {
    type Error = reqwest::Error;

    fn classify(&self, error: &Self::Error) -> NetworkErrorKind {
        if error.is_timeout() {
            return NetworkErrorKind::Timeout;
        }
        if error.is_connect() {
            return NetworkErrorKind::Connect;
        }
        if error.is_decode() || error.is_body() {
            return NetworkErrorKind::Decode;
        }
        if let Some(status) = error.status() {
            return NetworkErrorKind::HttpStatus(status.as_u16());
        }
        if error.is_request() {
            return NetworkErrorKind::Request;
        }
        if error.is_builder() {
            return NetworkErrorKind::Builder;
        }
        NetworkErrorKind::Other
    }
}
