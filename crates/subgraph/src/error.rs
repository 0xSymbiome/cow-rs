//! Typed error surface for subgraph requests.

use cow_sdk_core::{Cancelled, HostPolicyError, Redacted, TransportErrorClass};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use thiserror::Error;

/// A GraphQL error returned in the `errors` array.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SubgraphGraphQlError {
    /// Human-readable error message returned by the GraphQL service.
    pub message: Redacted<String>,
    /// Optional source locations within the submitted document.
    #[serde(default)]
    pub locations: Vec<SubgraphGraphQlErrorLocation>,
    /// Optional GraphQL `extensions` metadata, preserved verbatim (redacted)
    /// when a GraphQL endpoint provides it.
    ///
    /// The Graph's gateway and indexers do not populate this field — their
    /// errors carry only `message` and `locations` — so it is normally absent
    /// and exposes no machine-readable error `code` to classify against. It
    /// remains an opaque pass-through for any GraphQL endpoint that does set
    /// it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Redacted<Value>>,
}

impl SubgraphGraphQlError {
    /// Creates a typed GraphQL error entry.
    #[must_use]
    pub fn new(message: impl Into<String>, locations: Vec<SubgraphGraphQlErrorLocation>) -> Self {
        Self {
            message: message.into().into(),
            locations,
            extensions: None,
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
    pub api: Redacted<String>,
    /// Raw GraphQL document submitted to the endpoint.
    pub document: Redacted<String>,
    /// Optional GraphQL operation name sent with the request.
    pub operation_name: Option<Redacted<String>>,
    /// Optional GraphQL variables sent with the request.
    pub variables: Option<Redacted<Value>>,
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
            api: api.into().into(),
            document: document.into().into(),
            operation_name: operation_name.map(Redacted::new),
            variables: variables.map(Redacted::new),
        }
    }
}

/// Typed failure boundary for subgraph helper and raw-query operations.
///
/// `Display` for every variant pairs the redacted route identity in
/// `context.api` with at least one piece of plaintext structural diagnostic
/// (chain id, error count, source location, HTTP status, transport class, or
/// response-body byte count) so the default `format!("{e}")` path remains
/// actionable without breaching the workspace redaction posture (ADR 0025).
/// The exact format string is not a stability contract; consumers needing
/// structured access pattern-match on the typed variant fields directly.
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
    #[error("subgraph transport error ({class}) for {} (chain {}): {details}", context.api, context.chain_id)]
    Transport {
        /// Resolved request metadata captured at the failure boundary.
        context: Box<SubgraphRequestErrorContext>,
        /// Classification of the underlying transport failure.
        class: TransportErrorClass,
        /// Transport-layer error details from the HTTP client.
        details: Redacted<String>,
    },
    /// The default native transport could not be constructed from the
    /// resolved transport policy before any request was issued.
    ///
    /// Surfaced only by the native default-transport build path, where the
    /// configured user-agent failed HTTP header-value encoding while
    /// constructing the backing
    /// [`ReqwestTransport`](cow_sdk_core::ReqwestTransport). Distinct from
    /// [`SubgraphError::Transport`], which carries per-request
    /// [`SubgraphRequestErrorContext`] for failures observed once a query is
    /// in flight; a transport-construction failure happens before any chain,
    /// route, or document context is bound to a request, so there is no
    /// context to attach.
    #[error("subgraph transport configuration error ({class}): {details}")]
    TransportConfiguration {
        /// Classification of the underlying transport-construction failure.
        class: TransportErrorClass,
        /// Redacted transport-layer detail from the HTTP client builder.
        details: Redacted<String>,
    },
    /// Explicit service endpoint override failed host-policy validation.
    #[error(transparent)]
    HostPolicy(#[from] HostPolicyError),
    /// The endpoint returned a non-success HTTP status code.
    #[error("subgraph http status error for {} (chain {}): {status}: {body}", context.api, context.chain_id)]
    HttpStatus {
        /// Resolved request metadata captured at the failure boundary.
        context: Box<SubgraphRequestErrorContext>,
        /// Numeric HTTP status code.
        status: u16,
        /// Redacted and bounded response body returned with the status code.
        body: Redacted<String>,
    },
    /// The endpoint returned a success status with a body that could not be decoded.
    #[error(
        "subgraph serialization error for {} (chain {}, body {} bytes): {details}: {body}",
        context.api,
        context.chain_id,
        body.as_inner().len(),
    )]
    Serialization {
        /// Resolved request metadata captured at the failure boundary.
        context: Box<SubgraphRequestErrorContext>,
        /// Redacted and bounded response body that failed to decode.
        body: Redacted<String>,
        /// Serde decoding error details.
        details: Redacted<String>,
    },
    /// The GraphQL payload returned one or more typed GraphQL errors.
    ///
    /// `Display` reports the redacted route identity, the chain id, the
    /// error count, and, when available, the first error's source location
    /// as `at line:column`. The free-form `errors[i].message` payload stays
    /// behind `Redacted<String>` and is reached only through explicit typed
    /// access on the carried `errors` vector; the `.as_inner()` call is the
    /// workspace marker that the caller is crossing the redaction boundary
    /// on purpose.
    ///
    /// The Graph returns these failures as an HTTP 200 response carrying a
    /// GraphQL `errors` array whose entries hold only `message` and,
    /// optionally, `locations` — there is no machine-readable error `code` on
    /// the wire. The originating condition (authentication, an unknown
    /// subgraph, an invalid query, or unavailable / unhealthy indexers)
    /// survives only inside the free-form `message`, which stays redacted
    /// because the gateway URL embeds the partner API key and the SDK cannot
    /// assume the upstream message never echoes it. This variant therefore
    /// carries the raw `errors` for opt-in inspection rather than a typed
    /// reason discriminant: there is no stable coded reason on the wire to
    /// classify against.
    ///
    /// ```rust,ignore
    /// use cow_sdk_subgraph::SubgraphError;
    ///
    /// fn route_graphql_error_into_structured_log(err: &SubgraphError) {
    ///     if let SubgraphError::GraphQl { errors, .. } = err {
    ///         if let Some(first) = errors.first() {
    ///             let message_text: &str = first.message.as_inner();
    ///             // Route `message_text` into structured logging
    ///             // deliberately. The SDK's default `format!("{e}")`
    ///             // path keeps the message behind `Redacted<T>` so it
    ///             // never reaches log output without this explicit
    ///             // caller opt-in.
    ///             let _ = message_text;
    ///         }
    ///     }
    /// }
    /// ```
    #[error(
        "subgraph graphql error response for {} (chain {}, {} error{}{})",
        context.api,
        context.chain_id,
        errors.len(),
        if errors.len() == 1 { "" } else { "s" },
        first_graphql_location_suffix(errors),
    )]
    GraphQl {
        /// Resolved request metadata captured at the failure boundary.
        context: Box<SubgraphRequestErrorContext>,
        /// GraphQL errors returned by the endpoint.
        errors: Vec<SubgraphGraphQlError>,
    },
    /// The response was otherwise successful but did not contain `data`.
    #[error("subgraph response missing data for {} (chain {})", context.api, context.chain_id)]
    MissingData {
        /// Resolved request metadata captured at the failure boundary.
        context: Box<SubgraphRequestErrorContext>,
    },
    /// A long-running subgraph operation was cancelled through a cooperative cancellation token.
    #[error("subgraph operation was cancelled")]
    Cancelled,
}

/// Renders the first GraphQL error's first source location as
/// ` at line:column` when present, or the empty string otherwise.
///
/// The values rendered are typed `u32` line and column counters defined by
/// the GraphQL specification as referencing positions within the
/// SDK-submitted document, so they cannot carry credential-bearing content
/// and are safe to interpolate into the public `Display` template.
fn first_graphql_location_suffix(errors: &[SubgraphGraphQlError]) -> String {
    errors
        .first()
        .and_then(|entry| entry.locations.first())
        .map(|location| format!(" at {}:{}", location.line, location.column))
        .unwrap_or_default()
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
            Self::TransportConfiguration { class, details } => {
                map.serialize_entry("type", "TransportConfiguration")?;
                map.serialize_entry("class", &class.to_string())?;
                map.serialize_entry("details", details)?;
            }
            Self::HostPolicy(error) => {
                map.serialize_entry("type", "HostPolicy")?;
                map.serialize_entry("error", error)?;
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
