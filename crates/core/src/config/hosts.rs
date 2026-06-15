use std::{fmt, net::IpAddr};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::{ParseError, Url};

use crate::redaction::Redacted;

const ORDERBOOK_CANONICAL_HOSTS: [&str; 4] = [
    "api.cow.fi",
    "barn.api.cow.fi",
    "partners.cow.fi",
    "partners.barn.cow.fi",
];
const SUBGRAPH_CANONICAL_HOSTS: [&str; 1] = ["gateway.thegraph.com"];

/// Host validation policy for SDK-owned external service endpoints.
///
/// The default policy accepts only the canonical `CoW Protocol` hosts compiled
/// into the SDK. Callers that route through a private mirror or test fixture
/// must opt in explicitly so non-canonical service endpoints are visible in
/// code review and telemetry.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExternalHostPolicy {
    /// Accept only canonical production/staging hosts for the target surface.
    #[default]
    Default,
    /// Accept canonical hosts plus the supplied host allow-list.
    Allow(Vec<String>),
    /// Accept every `http` or `https` host.
    AllowAny,
    /// Accept canonical hosts plus loopback hosts for local fixtures.
    Test,
}

impl ExternalHostPolicy {
    const fn label(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Allow(_) => "allow",
            Self::AllowAny => "allow_any",
            Self::Test => "test",
        }
    }
}

/// Sanitized class for URL parse failures surfaced by host policy checks.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UrlParseFailureClass {
    /// The URL did not contain a valid absolute scheme.
    MalformedScheme,
    /// The URL did not contain a usable host component.
    MissingHost,
    /// The URL contained an invalid port component.
    InvalidPort,
    /// The parse failure is intentionally collapsed to avoid echoing raw URL bytes.
    Other,
}

impl fmt::Display for UrlParseFailureClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedScheme => f.write_str("malformedScheme"),
            Self::MissingHost => f.write_str("missingHost"),
            Self::InvalidPort => f.write_str("invalidPort"),
            Self::Other => f.write_str("other"),
        }
    }
}

impl From<ParseError> for UrlParseFailureClass {
    fn from(value: ParseError) -> Self {
        match value {
            ParseError::RelativeUrlWithoutBase | ParseError::RelativeUrlWithCannotBeABaseBase => {
                Self::MalformedScheme
            }
            ParseError::EmptyHost | ParseError::SetHostOnCannotBeABaseUrl => Self::MissingHost,
            ParseError::InvalidPort => Self::InvalidPort,
            _ => Self::Other,
        }
    }
}

/// Sanitized host-policy failure for SDK-owned service endpoint construction.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Error)]
pub enum HostPolicyError {
    /// The URL could not be parsed into an absolute service endpoint.
    #[error("external service URL could not be parsed: {class}")]
    UnparsableUrl {
        /// Sanitized parse-failure class.
        class: UrlParseFailureClass,
    },
    /// The URL host is not canonical and was not explicitly allowed.
    #[error("external service host is not allowed: {host}")]
    HostNotAllowed {
        /// Redacted host component only; scheme, path, credentials, query,
        /// and fragment are never retained.
        host: Redacted<String>,
    },
    /// The URL scheme is outside the supported `http`/`https` set.
    #[error("external service URL scheme is unsupported: {scheme}")]
    UnsupportedScheme {
        /// Sanitized scheme label.
        scheme: &'static str,
    },
}

/// Returns canonical orderbook hosts accepted by [`ExternalHostPolicy::Default`].
#[must_use]
pub const fn canonical_orderbook_hosts() -> &'static [&'static str] {
    &ORDERBOOK_CANONICAL_HOSTS
}

/// Returns canonical subgraph hosts accepted by [`ExternalHostPolicy::Default`].
#[must_use]
pub const fn canonical_subgraph_hosts() -> &'static [&'static str] {
    &SUBGRAPH_CANONICAL_HOSTS
}

/// Validates one SDK-owned external service URL against a host policy.
///
/// # Errors
///
/// Returns [`HostPolicyError`] when the URL cannot be parsed, uses an
/// unsupported scheme, or resolves to a host not accepted by `policy`.
pub fn validate_external_service_url(
    base_url: &str,
    canonical_hosts: &[&str],
    policy: &ExternalHostPolicy,
) -> Result<(), HostPolicyError> {
    let parsed = Url::parse(base_url.trim()).map_err(|error| HostPolicyError::UnparsableUrl {
        class: error.into(),
    })?;

    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(HostPolicyError::UnsupportedScheme {
            scheme: sanitized_scheme(parsed.scheme()),
        });
    }

    let host = parsed
        .host_str()
        .ok_or(HostPolicyError::UnparsableUrl {
            class: UrlParseFailureClass::MissingHost,
        })?
        .to_ascii_lowercase();

    if is_canonical_host(&host, canonical_hosts) {
        return Ok(());
    }

    let allowed = match policy {
        ExternalHostPolicy::Default => false,
        ExternalHostPolicy::Allow(hosts) => hosts.iter().any(|candidate| {
            normalized_allowed_host(candidate)
                .as_deref()
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(&host))
        }),
        ExternalHostPolicy::AllowAny => true,
        ExternalHostPolicy::Test => is_loopback_host(&host),
    };

    warn_noncanonical_external_host(&host, policy.label(), allowed);

    if allowed {
        Ok(())
    } else {
        Err(HostPolicyError::HostNotAllowed {
            host: Redacted::new(host),
        })
    }
}

fn is_canonical_host(host: &str, canonical_hosts: &[&str]) -> bool {
    canonical_hosts
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(host))
}

fn normalized_allowed_host(candidate: &str) -> Option<String> {
    let trimmed = candidate.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(url) = Url::parse(trimmed) {
        return url.host_str().map(str::to_ascii_lowercase);
    }

    Some(
        trimmed
            .trim_start_matches('[')
            .trim_end_matches(']')
            .to_ascii_lowercase(),
    )
}

fn is_loopback_host(host: &str) -> bool {
    let normalized = host.trim_start_matches('[').trim_end_matches(']');
    normalized.eq_ignore_ascii_case("localhost")
        || normalized
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback())
}

fn sanitized_scheme(scheme: &str) -> &'static str {
    match scheme {
        "http" => "http",
        "https" => "https",
        "ftp" => "ftp",
        "ws" => "ws",
        "wss" => "wss",
        "file" => "file",
        "data" => "data",
        _ => "other",
    }
}

#[cfg(feature = "tracing")]
fn warn_noncanonical_external_host(host: &str, policy: &'static str, allowed: bool) {
    tracing::warn!(
        target: "cow_sdk::trust",
        host = ?Redacted::new(host.to_owned()),
        policy,
        allowed,
        "non-canonical external service host evaluated"
    );
}

#[cfg(not(feature = "tracing"))]
const fn warn_noncanonical_external_host(_host: &str, _policy: &'static str, _allowed: bool) {}
