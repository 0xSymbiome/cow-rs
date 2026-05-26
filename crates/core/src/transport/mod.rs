//! HTTP transport injection point shared across `cow-sdk` crates.
//!
//! The [`HttpTransport`] trait is the production seam that downstream clients
//! use to dispatch REST requests without committing to a concrete backend.
//! Every method is `async` so implementations can bridge either a native
//! runtime (through [`ReqwestTransport`]) or a browser runtime (through a
//! `JsFuture`-backed adapter in `cow-sdk-transport-wasm`).
//!
//! Every method carries the per-call header set and an optional per-call
//! timeout so typed consumers compose one injection point without holding a
//! parallel HTTP client for header or deadline overrides. Adapters surface
//! non-2xx responses through [`TransportError::HttpStatus`] so the calling
//! layer receives the numeric status, response headers, and raw response body
//! together through the typed error channel.
//!
//! The companion [`TransportError`] enum is the typed failure surface for
//! transport adapters. Native adapters that bridge `reqwest::Error` classify
//! each failure through [`TransportErrorClass`] before wrapping and call
//! [`reqwest::Error::without_url`] to keep endpoint URLs out of the error
//! text. Callers that want to partition telemetry or shape retry policy on
//! the failure category match on the [`class`](TransportError::class) of the
//! [`TransportError::Transport`] variant; callers that need the numeric
//! HTTP status on a non-success response match on
//! [`TransportError::HttpStatus`].

mod error;
mod http;

#[cfg(not(target_arch = "wasm32"))]
pub mod reqwest;

pub use error::TransportError;
pub use http::HttpTransport;

#[cfg(not(target_arch = "wasm32"))]
pub use self::reqwest::{ReqwestTransport, ReqwestTransportConfig, classify_reqwest_error};

pub use crate::validation::TransportErrorClass;

const CUSTOM_OVERRIDE_ROUTE_IDENTITY: &str = "<custom override>";

/// Returns the public origin for a base URL without path, query, fragment, or credentials.
///
/// The helper is intended for diagnostic and telemetry surfaces that need to
/// identify a configured endpoint without echoing credential-bearing path or
/// query material. Invalid URLs and URL forms without a public origin return a
/// stable custom-override marker.
#[cfg(not(target_arch = "wasm32"))]
#[must_use]
#[allow(
    clippy::option_if_let_else,
    reason = "the Ok arm binds an intermediate origin and carries a nested conditional; the combinator form would collapse that multi-statement body into a closure and obscure the two-branch parallel structure"
)]
pub fn sanitize_public_base_url(base_url: &str) -> String {
    match ::reqwest::Url::parse(base_url) {
        Ok(url) => {
            let origin = url.origin().ascii_serialization();
            if origin == "null" {
                CUSTOM_OVERRIDE_ROUTE_IDENTITY.to_owned()
            } else {
                origin.trim_end_matches('/').to_owned()
            }
        }
        Err(_) => CUSTOM_OVERRIDE_ROUTE_IDENTITY.to_owned(),
    }
}

/// Returns the public origin for a base URL without path, query, fragment, or credentials.
///
/// The helper is intended for diagnostic and telemetry surfaces that need to
/// identify a configured endpoint without echoing credential-bearing path or
/// query material. Invalid URLs and URL forms without a public origin return a
/// stable custom-override marker.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn sanitize_public_base_url(base_url: &str) -> String {
    let Some((scheme, after_scheme)) = base_url.split_once("://") else {
        return CUSTOM_OVERRIDE_ROUTE_IDENTITY.to_owned();
    };
    if !is_supported_public_scheme(scheme) {
        return CUSTOM_OVERRIDE_ROUTE_IDENTITY.to_owned();
    }

    let authority = after_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default();
    let public_authority = authority
        .rsplit_once('@')
        .map_or(authority, |(_, public_authority)| public_authority);
    if public_authority.is_empty() || public_authority.starts_with(':') {
        return CUSTOM_OVERRIDE_ROUTE_IDENTITY.to_owned();
    }

    format!(
        "{}://{}",
        scheme.to_ascii_lowercase(),
        public_authority.to_ascii_lowercase()
    )
}

#[cfg(target_arch = "wasm32")]
fn is_supported_public_scheme(scheme: &str) -> bool {
    scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")
}

/// Joins a request `path` against a `base_url` using the canonical
/// SDK transport join semantics.
///
/// This helper is the single seam every workspace `HttpTransport`
/// implementation uses to compose request URLs, so cross-transport
/// URL composition stays byte-identical between the native
/// [`ReqwestTransport`], the browser fetch transport, and any
/// callback-backed transport built on top of `HttpTransport`.
///
/// The join rules are:
///
/// - if `path` is itself an absolute `http://` or `https://` URL, the
///   `base_url` is ignored and `path` is returned verbatim;
/// - if `base_url` is empty, `path` is returned verbatim;
/// - if `path` starts with `/`, the result is `format!("{base_url}{path}")`;
/// - otherwise, the result is `format!("{base_url}/{path}")`.
///
/// The helper does not perform URL validation; both inputs are
/// assumed pre-validated by the calling transport configuration.
/// The helper does not log or store either input, and reads the
/// credential-bearing `base_url` only through the caller-supplied
/// `&str` slice produced by `Redacted::as_inner()` at the dispatch
/// seam, preserving the workspace credential-redaction posture.
#[must_use]
pub fn join_request_url(base_url: &str, path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") || base_url.is_empty() {
        path.to_owned()
    } else if path.starts_with('/') {
        format!("{base_url}{path}")
    } else {
        format!("{base_url}/{path}")
    }
}

#[cfg(test)]
mod join_request_url_tests {
    use super::join_request_url;

    #[test]
    fn absolute_http_path_returns_path_verbatim() {
        assert_eq!(
            join_request_url("https://api.example.com", "http://other.example/orders"),
            "http://other.example/orders"
        );
    }

    #[test]
    fn absolute_https_path_returns_path_verbatim() {
        assert_eq!(
            join_request_url("https://api.example.com", "https://other.example/orders"),
            "https://other.example/orders"
        );
    }

    #[test]
    fn empty_base_returns_path_verbatim() {
        assert_eq!(join_request_url("", "/api/v1/orders"), "/api/v1/orders");
    }

    #[test]
    fn leading_slash_path_concatenates_without_inserted_slash() {
        assert_eq!(
            join_request_url("https://api.example.com", "/api/v1/orders"),
            "https://api.example.com/api/v1/orders"
        );
    }

    #[test]
    fn non_leading_slash_path_concatenates_with_inserted_slash() {
        assert_eq!(
            join_request_url("https://api.example.com", "api/v1/orders"),
            "https://api.example.com/api/v1/orders"
        );
    }
}
