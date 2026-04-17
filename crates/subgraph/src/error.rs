//! Typed error surface for subgraph requests.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// A GraphQL error returned in the `errors` array.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubgraphGraphQlError {
    /// Human-readable error message returned by the GraphQL service.
    pub message: String,
    /// Optional source locations within the submitted document.
    #[serde(default)]
    pub locations: Vec<SubgraphGraphQlErrorLocation>,
}

/// A single GraphQL error location.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubgraphGraphQlErrorLocation {
    /// One-based line number within the submitted document.
    pub line: u32,
    /// One-based column number within the submitted document.
    pub column: u32,
}

/// Request metadata captured in typed subgraph errors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
        /// Transport-layer error details from the HTTP client.
        details: String,
    },
    /// The endpoint returned a non-success HTTP status code.
    #[error("subgraph http status error for {}: {status}", context.api)]
    HttpStatus {
        /// Resolved request metadata captured at the failure boundary.
        context: Box<SubgraphRequestErrorContext>,
        /// Numeric HTTP status code.
        status: u16,
        /// Raw response body returned with the status code.
        body: String,
    },
    /// The endpoint returned a success status with a body that could not be decoded.
    #[error("subgraph serialization error for {}: {details}", context.api)]
    Serialization {
        /// Resolved request metadata captured at the failure boundary.
        context: Box<SubgraphRequestErrorContext>,
        /// Raw response body that failed to decode.
        body: String,
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

/// Classifies a `reqwest::Error`, strips any attached URL, and returns a sanitized message.
///
/// The transport error is partitioned through `is_timeout`, `is_connect`,
/// `is_redirect`, `is_decode`, `is_body`, `is_builder`, `is_request`, and
/// `is_status`. [`reqwest::Error::without_url`] is called before the
/// [`std::fmt::Display`] implementation runs so gateway URLs and their
/// query-string API keys cannot leak through error text.
#[must_use]
pub fn classify_reqwest_error(error: reqwest::Error) -> String {
    let sanitized = error.without_url();
    let class = reqwest_error_class(&sanitized);
    format!("{class}: {sanitized}")
}

fn reqwest_error_class(error: &reqwest::Error) -> &'static str {
    if error.is_timeout() {
        return "timeout";
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if error.is_connect() {
            return "connect";
        }
        if error.is_redirect() {
            return "redirect";
        }
    }
    if error.is_decode() {
        "decode"
    } else if error.is_body() {
        "body"
    } else if error.is_builder() {
        "builder"
    } else if error.is_request() {
        "request"
    } else if error.is_status() {
        "status"
    } else {
        "other"
    }
}
