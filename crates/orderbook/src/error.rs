use std::fmt;

use cow_sdk_core::{
    AppDataHash, Cancelled, CoreError, HostPolicyError, Redacted, TransportErrorClass,
    ValidationReason,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::rejection::{OrderbookRejection, parse_rejection};
use crate::request::{OrderBookApiError, ResponseBody};
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
    Api(Box<OrderBookApiError>),
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
        source: Box<OrderBookApiError>,
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
    /// JSON or text decoding failure while parsing a successful or error response.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
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

impl From<OrderBookApiError> for OrderbookError {
    fn from(value: OrderBookApiError) -> Self {
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

impl From<Cancelled> for OrderbookError {
    fn from(_: Cancelled) -> Self {
        Self::Cancelled
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
