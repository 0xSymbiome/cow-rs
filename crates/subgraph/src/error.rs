//! Typed error surface for subgraph requests.

use cow_sdk_core::{Cancelled, Redacted, TransportErrorClass};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use thiserror::Error;

/// A GraphQL error returned in the `errors` array.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SubgraphGraphQlError {
    /// Human-readable error message returned by the GraphQL service.
    pub message: String,
    /// Optional source locations within the submitted document.
    #[serde(default)]
    pub locations: Vec<SubgraphGraphQlErrorLocation>,
}

impl SubgraphGraphQlError {
    /// Creates a typed GraphQL error entry.
    #[must_use]
    pub fn new(message: impl Into<String>, locations: Vec<SubgraphGraphQlErrorLocation>) -> Self {
        Self {
            message: message.into(),
            locations,
        }
    }
}

/// A single GraphQL error location.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SubgraphGraphQlErrorLocation {
    /// One-based line number within the submitted document.
    pub line: u32,
    /// One-based column number within the submitted document.
    pub column: u32,
}

impl SubgraphGraphQlErrorLocation {
    /// Creates a one-based GraphQL source location.
    #[must_use]
    pub const fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }
}

/// Request metadata captured in typed subgraph errors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SubgraphRequestErrorContext {
    /// Numeric chain id selected for the request.
    pub chain_id: u64,
    /// Public route identity used for the request.
    ///
    /// Production-derived routes are redacted before they reach this public
    /// error surface, and explicit overrides are normalized to non-secret route
    /// identity.
    pub api: String,
    /// Raw GraphQL document submitted to the endpoint.
    pub document: String,
    /// Optional GraphQL operation name sent with the request.
    pub operation_name: Option<String>,
    /// Optional GraphQL variables sent with the request.
    pub variables: Option<Value>,
}

impl SubgraphRequestErrorContext {
    /// Creates request metadata captured in typed subgraph errors.
    #[must_use]
    pub fn new(
        chain_id: u64,
        api: impl Into<String>,
        document: impl Into<String>,
        operation_name: Option<String>,
        variables: Option<Value>,
    ) -> Self {
        Self {
            chain_id,
            api: api.into(),
            document: document.into(),
            operation_name,
            variables,
        }
    }
}

/// Typed failure boundary for subgraph helper and raw-query operations.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SubgraphError {
    /// The selected chain does not have a configured subgraph endpoint.
    #[error("Unsupported Network. The subgraph API is not available in the Network {chain_id}")]
    UnsupportedNetwork {
        /// Numeric chain id that could not be resolved to a supported endpoint.
        chain_id: u64,
    },
    /// The canonical totals query returned an empty list.
    #[error("No totals found")]
    NoTotalsFound,
    /// Request execution failed before a complete HTTP response was received.
    #[error("subgraph transport error for {}: {details}", context.api)]
    Transport {
        /// Resolved request metadata captured at the failure boundary.
        context: Box<SubgraphRequestErrorContext>,
        /// Classification of the underlying transport failure.
        class: TransportErrorClass,
        /// Transport-layer error details from the HTTP client.
        details: String,
    },
    /// The endpoint returned a non-success HTTP status code.
    #[error("subgraph http status error for {}: {status}: {body}", context.api)]
    HttpStatus {
        /// Resolved request metadata captured at the failure boundary.
        context: Box<SubgraphRequestErrorContext>,
        /// Numeric HTTP status code.
        status: u16,
        /// Redacted and bounded response body returned with the status code.
        body: Redacted<String>,
    },
    /// The endpoint returned a success status with a body that could not be decoded.
    #[error("subgraph serialization error for {}: {details}: {body}", context.api)]
    Serialization {
        /// Resolved request metadata captured at the failure boundary.
        context: Box<SubgraphRequestErrorContext>,
        /// Redacted and bounded response body that failed to decode.
        body: Redacted<String>,
        /// Serde decoding error details.
        details: String,
    },
    /// The GraphQL payload returned one or more typed GraphQL errors.
    #[error("subgraph graphql error response for {}", context.api)]
    GraphQl {
        /// Resolved request metadata captured at the failure boundary.
        context: Box<SubgraphRequestErrorContext>,
        /// GraphQL errors returned by the endpoint.
        errors: Vec<SubgraphGraphQlError>,
    },
    /// The response was otherwise successful but did not contain `data`.
    #[error("subgraph response missing data for {}", context.api)]
    MissingData {
        /// Resolved request metadata captured at the failure boundary.
        context: Box<SubgraphRequestErrorContext>,
    },
    /// A long-running subgraph operation was cancelled through a cooperative cancellation token.
    #[error("subgraph operation was cancelled")]
    Cancelled,
}

impl From<Cancelled> for SubgraphError {
    fn from(_: Cancelled) -> Self {
        Self::Cancelled
    }
}

impl Serialize for SubgraphError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;

        match self {
            Self::UnsupportedNetwork { chain_id } => {
                map.serialize_entry("type", "UnsupportedNetwork")?;
                map.serialize_entry("chainId", chain_id)?;
            }
            Self::NoTotalsFound => {
                map.serialize_entry("type", "NoTotalsFound")?;
            }
            Self::Transport {
                context,
                class,
                details,
            } => {
                map.serialize_entry("type", "Transport")?;
                map.serialize_entry("context", context)?;
                map.serialize_entry("class", &class.to_string())?;
                map.serialize_entry("details", details)?;
            }
            Self::HttpStatus {
                context,
                status,
                body,
            } => {
                map.serialize_entry("type", "HttpStatus")?;
                map.serialize_entry("context", context)?;
                map.serialize_entry("status", status)?;
                map.serialize_entry("body", body)?;
            }
            Self::Serialization {
                context,
                body,
                details,
            } => {
                map.serialize_entry("type", "Serialization")?;
                map.serialize_entry("context", context)?;
                map.serialize_entry("body", body)?;
                map.serialize_entry("details", details)?;
            }
            Self::GraphQl { context, errors } => {
                map.serialize_entry("type", "GraphQl")?;
                map.serialize_entry("context", context)?;
                map.serialize_entry("errors", errors)?;
            }
            Self::MissingData { context } => {
                map.serialize_entry("type", "MissingData")?;
                map.serialize_entry("context", context)?;
            }
            Self::Cancelled => {
                map.serialize_entry("type", "Cancelled")?;
            }
        }

        map.end()
    }
}

/// Classifies a `reqwest::Error`, strips any attached URL, and returns a typed
/// `(class, detail)` pair.
///
/// The transport error is partitioned through `is_timeout`, `is_connect`,
/// `is_redirect`, `is_decode`, `is_body`, `is_builder`, `is_request`, and
/// `is_status`. [`reqwest::Error::without_url`] is called before the
/// [`std::fmt::Display`] implementation runs so gateway URLs and their
/// query-string API keys cannot leak through error text.
#[must_use]
pub fn classify_reqwest_error(error: reqwest::Error) -> (TransportErrorClass, String) {
    let sanitized = error.without_url();
    let class = reqwest_error_class(&sanitized);
    (class, sanitized.to_string())
}

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
