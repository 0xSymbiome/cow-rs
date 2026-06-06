use std::fmt;
use std::time::Duration;

use cow_sdk_core::{
    AppDataHash, Cancelled, CoreError, ErrorClass, HostPolicyError, Redacted, TransportError,
    TransportErrorClass, ValidationReason,
};
use cow_sdk_transport_policy::{NetworkErrorKind, RetryPolicy, is_retryable_status};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::rejection::{OrderbookRejection, parse_rejection};
use crate::request::{OrderbookApiError, ResponseBody};
use crate::types::SigningScheme;

/// Stage at which an app-data hash mismatch was detected by the typed
/// orderbook client.
///
/// [`HashMismatchStage::ClientPrecheck`] indicates the caller-supplied hash
/// did not match `keccak256(full_app_data.as_bytes())` and was rejected
/// before any network call.
///
/// [`HashMismatchStage::ServerEcho`] indicates the orderbook responded
/// successfully but the hash returned in the response body did not equal the
/// locally derived digest. A successful order signed under the caller's hash
/// would not resolve to the document the SDK intended to register, so the
/// SDK surfaces the disagreement instead of reporting success.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum HashMismatchStage {
    /// Detected before any network call by re-hashing the supplied body.
    ClientPrecheck,
    /// Detected after the server responded with a hash that disagrees with
    /// the locally derived digest.
    ServerEcho,
}

impl fmt::Display for HashMismatchStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClientPrecheck => f.write_str("client precheck"),
            Self::ServerEcho => f.write_str("server echo"),
        }
    }
}

/// Errors returned by the typed orderbook client and transport helpers.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum OrderbookError {
    /// Error bubbled up from shared core configuration or type validation.
    #[error(transparent)]
    Core(#[from] CoreError),
    /// Structured non-2xx response returned by the orderbook API whose body
    /// did not carry a recognisable rejection envelope.
    #[error(transparent)]
    Api(Box<OrderbookApiError>),
    /// Structured rejection classified from the non-2xx response body using
    /// the typed [`OrderbookRejection`] taxonomy.
    #[error("orderbook rejected the request ({status}): {rejection}")]
    Rejected {
        /// HTTP status code surfaced by the orderbook service.
        status: StatusCode,
        /// Typed rejection classification parsed from the response body.
        rejection: OrderbookRejection,
        /// Raw transport-level envelope preserved for diagnostics.
        #[source]
        source: Box<OrderbookApiError>,
    },
    /// Network or request-execution failure before a structured API response was decoded.
    #[error("transport error ({class}): {detail}")]
    Transport {
        /// Classification of the underlying REST-transport failure.
        class: TransportErrorClass,
        /// Redacted detail message sourced from the transport layer.
        detail: Redacted<String>,
    },
    /// Explicit service endpoint override failed host-policy validation.
    #[error(transparent)]
    HostPolicy(#[from] HostPolicyError),
    /// JSON decoding of an orderbook response body failed.
    ///
    /// Only the serde failure category and the structural position are
    /// surfaced. The raw `serde_json::Error` rendering can echo bytes from
    /// the decoded upstream response body, so the orderbook client never
    /// renders it into a `Display` or `Debug` surface (ADR 0025); the
    /// `category`/`line`/`column` triple is the safe structural diagnostic.
    #[error("serialization error ({category}) at line {line} column {column}")]
    Serialization {
        /// serde failure category: `"syntax"`, `"data"`, `"eof"`, or `"io"`.
        category: &'static str,
        /// 1-based line in the response body where decoding failed, or `0`
        /// when the position is unknown.
        line: usize,
        /// 1-based column in the response body where decoding failed, or `0`
        /// when the position is unknown.
        column: usize,
    },
    /// Invalid trades query assembled locally before any network request was sent.
    #[error("invalid trades query for field `{field}`: {reason}")]
    InvalidTradesQuery {
        /// Public field name that failed validation.
        field: &'static str,
        /// Canonical validation-failure mode.
        reason: ValidationReason,
    },
    /// Invalid quote request assembled locally before any network request was sent.
    #[error("invalid quote request for field `{field}`: {reason}")]
    InvalidQuoteRequest {
        /// Public field name that failed validation.
        field: &'static str,
        /// Canonical validation-failure mode.
        reason: ValidationReason,
    },
    /// Quote signing-scheme and on-chain-order flags are incompatible before transport.
    #[error(
        "incompatible signing scheme `{signing_scheme:?}` for on-chain order flag `{onchain_order}`"
    )]
    IncompatibleSigningScheme {
        /// Signing scheme supplied for the quote request.
        signing_scheme: SigningScheme,
        /// Whether the eventual order is expected to be on-chain.
        onchain_order: bool,
    },
    /// Invalid normalized orderbook data encountered after a successful HTTP response.
    #[error("invalid transform for field `{field}`: {reason}")]
    InvalidTransform {
        /// Public field name that failed validation.
        field: &'static str,
        /// Canonical validation-failure mode.
        reason: ValidationReason,
    },
    /// App-data hash does not match the keccak256 digest of the supplied body.
    ///
    /// Surfaced in two stages of the upload flow. When
    /// [`stage`](HashMismatchStage) is
    /// [`HashMismatchStage::ClientPrecheck`] the caller-supplied hash did
    /// not equal `keccak256(full_app_data)` and the SDK rejected the request
    /// before any network call. When [`stage`](HashMismatchStage) is
    /// [`HashMismatchStage::ServerEcho`] the orderbook responded
    /// successfully but the hash carried in the response body did not match
    /// the locally derived digest.
    ///
    /// Both cases indicate a content-addressed-write invariant violation: an
    /// order signed under `expected` would not resolve to the document the
    /// SDK intended to register. Distinct from
    /// [`OrderbookRejection::AppDataHashMismatch`], which is the
    /// services-emitted 400-class envelope for the same invariant detected
    /// server-side.
    #[error(
        "app-data hash mismatch ({stage}): expected {expected}, observed {observed}. \
         If both sides represent the same document, verify the body is canonical-JSON \
         serialized before computing the digest."
    )]
    AppDataHashMismatch {
        /// Hash the caller supplied, or the hash the SDK locally derived for
        /// the no-hash upload path.
        expected: AppDataHash,
        /// Hash observed by the verifier (locally computed or server-returned).
        observed: AppDataHash,
        /// Stage of the upload flow that detected the mismatch.
        stage: HashMismatchStage,
    },
    /// A long-running orderbook operation was cancelled through a cooperative cancellation token.
    #[error("orderbook operation was cancelled")]
    Cancelled,
}

impl From<OrderbookApiError> for OrderbookError {
    fn from(value: OrderbookApiError) -> Self {
        let status = StatusCode::from_u16(value.status).ok();
        let rejection = match (status, value.body.as_inner()) {
            (Some(status_code), ResponseBody::Json(body)) => serde_json::to_vec(body)
                .ok()
                .and_then(|bytes| parse_rejection(status_code, &bytes)),
            _ => None,
        };

        match (status, rejection) {
            (Some(status), Some(rejection)) => Self::Rejected {
                status,
                rejection,
                source: Box::new(value),
            },
            _ => Self::Api(Box::new(value)),
        }
    }
}

impl From<TransportError> for OrderbookError {
    fn from(error: TransportError) -> Self {
        match error {
            TransportError::Transport { class, detail } => Self::Transport { class, detail },
            TransportError::Configuration { message } => Self::Transport {
                class: TransportErrorClass::Builder,
                detail: message,
            },
            TransportError::HttpStatus { status, .. } => Self::Transport {
                class: TransportErrorClass::Status,
                detail: Redacted::new(format!("transport returned HTTP status {status}")),
            },
            _ => Self::Transport {
                class: TransportErrorClass::Other,
                detail: Redacted::new("transport error".to_owned()),
            },
        }
    }
}

impl From<Cancelled> for OrderbookError {
    fn from(_: Cancelled) -> Self {
        Self::Cancelled
    }
}

impl OrderbookError {
    /// Returns the coarse-grained [`ErrorClass`] for this error.
    ///
    /// A 429 rejection that outlived the transport retry budget classifies as
    /// [`ErrorClass::RateLimited`]; other structured non-2xx responses are
    /// [`ErrorClass::Remote`].
    #[must_use]
    pub const fn class(&self) -> ErrorClass {
        match self {
            Self::Core(error) => error.class(),
            Self::Rejected { status, .. } if status.as_u16() == 429 => ErrorClass::RateLimited,
            Self::Api(error) if error.status == 429 => ErrorClass::RateLimited,
            Self::Api(_) | Self::Rejected { .. } => ErrorClass::Remote,
            Self::Transport { .. } => ErrorClass::Transport,
            Self::InvalidTradesQuery { .. } | Self::InvalidQuoteRequest { .. } => {
                ErrorClass::Validation
            }
            Self::Cancelled => ErrorClass::Cancelled,
            // HostPolicy, Serialization, IncompatibleSigningScheme,
            // InvalidTransform, and AppDataHashMismatch plus future additive
            // variants classify as internal.
            _ => ErrorClass::Internal,
        }
    }

    /// Returns `true` when retrying the same request may succeed.
    ///
    /// A structured non-2xx response is retryable when its HTTP status is one
    /// the default transport policy retries (`408`, `425`, `429`, and the
    /// `500`/`502`/`503`/`504` server-fault range). A transport failure is
    /// retryable when its class is a transient network fault (a timeout,
    /// connection failure, request-layer failure, or unclassified transport
    /// error) rather than a deterministic decode, body, builder, status, or
    /// oversize-response failure. Validation, serialization,
    /// signing-scheme, transform, hash-mismatch, host-policy, core, and
    /// cancellation faults are never retryable.
    ///
    /// This is the same verdict the SDK's own transport retry loop reaches, so
    /// a consumer that drives its own retry loop over a returned error does not
    /// re-derive the retryable-status set. Pair it with
    /// [`OrderbookError::backoff_hint`] for the suggested wait before the next
    /// attempt.
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        match self {
            Self::Rejected { status, .. } => is_retryable_status(status.as_u16()),
            Self::Api(error) => is_retryable_status(error.status),
            Self::Transport { class, .. } => RetryPolicy::builder()
                .build()
                .should_retry_network(NetworkErrorKind::from_transport_error_class(*class)),
            _ => false,
        }
    }

    /// Returns the server-suggested backoff before the next attempt, when the
    /// failing response carried a `Retry-After` header.
    ///
    /// The delay is parsed once, when the error is constructed, from the
    /// response `Retry-After` header per RFC 7231 (delta-seconds or HTTP-date);
    /// an HTTP-date in the past resolves to [`Duration::ZERO`]. Returns [`None`]
    /// for transport failures and for responses that carried no `Retry-After`
    /// header. A retryable error with a [`None`] hint means retry under your
    /// own backoff policy rather than a server-pinned delay.
    #[must_use]
    pub fn backoff_hint(&self) -> Option<Duration> {
        match self {
            Self::Rejected { source, .. } => source.retry_after(),
            Self::Api(error) => error.retry_after(),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for OrderbookError {
    /// Captures only the serde failure category and structural position.
    ///
    /// The raw `serde_json::Error` rendering can echo bytes from the decoded
    /// upstream response body (a `data` failure renders the offending value,
    /// an unknown field renders its name), so it is intentionally dropped
    /// here. Surfacing only the `category`/`line`/`column` triple keeps the
    /// orderbook decode-failure diagnostic free of upstream-authored content
    /// (ADR 0025).
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization {
            category: serialization_error_category(&error),
            line: error.line(),
            column: error.column(),
        }
    }
}

/// Maps a `serde_json` failure to its stable category tag.
fn serialization_error_category(error: &serde_json::Error) -> &'static str {
    match error.classify() {
        serde_json::error::Category::Io => "io",
        serde_json::error::Category::Syntax => "syntax",
        serde_json::error::Category::Data => "data",
        serde_json::error::Category::Eof => "eof",
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<reqwest::Error> for OrderbookError {
    fn from(error: reqwest::Error) -> Self {
        let (class, detail) = classify_reqwest_error(error);
        Self::Transport {
            class,
            detail: detail.into(),
        }
    }
}

/// Classifies a `reqwest::Error`, strips any attached URL, and returns a typed
/// `(class, detail)` pair.
///
/// [`reqwest::Error::without_url`] is called before the
/// [`std::fmt::Display`] implementation runs so partner-route URLs and their
/// query parameters cannot leak through error text; the typed
/// [`TransportErrorClass`] captures the classification produced by the
/// documented `is_timeout`, `is_connect`, `is_redirect`, `is_decode`,
/// `is_body`, `is_builder`, `is_request`, and `is_status` partition.
#[must_use]
#[cfg(not(target_arch = "wasm32"))]
pub fn classify_reqwest_error(error: reqwest::Error) -> (TransportErrorClass, String) {
    let sanitized = error.without_url();
    let class = reqwest_error_class(&sanitized);
    (class, sanitized.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn reqwest_error_class(error: &reqwest::Error) -> TransportErrorClass {
    if error.is_timeout() {
        return TransportErrorClass::Timeout;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if error.is_connect() {
            return TransportErrorClass::Connect;
        }
        if error.is_redirect() {
            return TransportErrorClass::Redirect;
        }
    }
    if error.is_decode() {
        TransportErrorClass::Decode
    } else if error.is_body() {
        TransportErrorClass::Body
    } else if error.is_builder() {
        TransportErrorClass::Builder
    } else if error.is_request() {
        TransportErrorClass::Request
    } else if error.is_status() {
        TransportErrorClass::Status
    } else {
        TransportErrorClass::Other
    }
}

#[cfg(test)]
mod retry_classification_tests {
    use std::time::Duration;

    use cow_sdk_core::{Redacted, TransportErrorClass};

    use super::OrderbookError;
    use crate::request::{OrderbookApiError, ResponseBody};

    fn api_error(status: u16, retry_after: Option<Duration>) -> OrderbookError {
        OrderbookError::Api(Box::new(
            OrderbookApiError::new(status, "", ResponseBody::Empty).with_retry_after(retry_after),
        ))
    }

    fn transport(class: TransportErrorClass) -> OrderbookError {
        OrderbookError::Transport {
            class,
            detail: Redacted::new(String::new()),
        }
    }

    #[test]
    fn retryable_statuses_classify_as_retryable() {
        for status in [408, 425, 429, 500, 502, 503, 504] {
            assert!(
                api_error(status, None).is_retryable(),
                "status {status} must be retryable"
            );
        }
    }

    #[test]
    fn client_error_statuses_are_not_retryable() {
        for status in [400, 401, 403, 404, 409, 422] {
            assert!(
                !api_error(status, None).is_retryable(),
                "status {status} must not be retryable"
            );
        }
    }

    #[test]
    fn transient_transport_classes_are_retryable() {
        for class in [
            TransportErrorClass::Timeout,
            TransportErrorClass::Connect,
            TransportErrorClass::Request,
            TransportErrorClass::Other,
        ] {
            assert!(
                transport(class).is_retryable(),
                "{class:?} must be retryable"
            );
        }
    }

    #[test]
    fn deterministic_transport_classes_are_not_retryable() {
        for class in [
            TransportErrorClass::Decode,
            TransportErrorClass::Body,
            TransportErrorClass::Builder,
            TransportErrorClass::Status,
        ] {
            assert!(
                !transport(class).is_retryable(),
                "{class:?} must not be retryable"
            );
        }
    }

    #[test]
    fn non_transport_faults_are_not_retryable() {
        assert!(!OrderbookError::Cancelled.is_retryable());
    }

    #[test]
    fn backoff_hint_surfaces_parsed_retry_after() {
        assert_eq!(
            api_error(429, Some(Duration::from_secs(120))).backoff_hint(),
            Some(Duration::from_secs(120))
        );
    }

    #[test]
    fn backoff_hint_is_absent_without_a_header() {
        assert_eq!(api_error(503, None).backoff_hint(), None);
        assert_eq!(transport(TransportErrorClass::Timeout).backoff_hint(), None);
    }
}
