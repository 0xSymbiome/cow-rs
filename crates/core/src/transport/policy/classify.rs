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
