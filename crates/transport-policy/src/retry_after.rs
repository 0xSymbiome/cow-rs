//! Parser for the HTTP `Retry-After` response header.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Parsed `Retry-After` delay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RetryAfter {
    delay: Duration,
}

impl RetryAfter {
    /// Creates a parsed `Retry-After` wrapper from a concrete delay.
    #[must_use]
    pub const fn new(delay: Duration) -> Self {
        Self { delay }
    }

    /// Returns the delay requested by the remote endpoint.
    #[must_use]
    pub const fn delay(self) -> Duration {
        self.delay
    }
}

/// Parses a `Retry-After` header value.
///
/// The parser accepts delta-seconds and HTTP-date values per RFC 7231
/// section 7.1.1.1, with HTTP-date parsing delegated to
/// [`httpdate::parse_http_date`]. Accepted HTTP-date forms include
/// IMF-fixdate (`Sun, 06 Nov 1994 08:49:37 GMT`), the legacy RFC 850
/// form (`Sunday, 06-Nov-94 08:49:37 GMT`), and ANSI C `asctime`
/// (`Sun Nov  6 08:49:37 1994`). Invalid, empty, negative, or
/// unsupported date formats return [`None`]. Past or epoch-equal dates
/// clamp to `Duration::ZERO`.
// DO NOT SWAP for alloy_transport's parse_retry_after.
//
// This parser handles the HTTP `Retry-After` *response header* per
// RFC 7231 §7.1.1.1: a delta-seconds integer OR an HTTP-date
// (IMF-fixdate, RFC 850, or ANSI C `asctime`, delegated to
// `httpdate::parse_http_date`).
//
// alloy's namesake parses JSON-RPC error-message strings like
// `"try again in 4ms"` and has no concept of the HTTP header.
// Swapping would silently ignore the orderbook backend's
// `Retry-After` header on 429/503, retry too aggressively, and
// trigger harder rate limits.
//
// The IMF-fixdate parse itself (`httpdate::parse_http_date`) is
// Bucket 1 in the alloy doctrine; only the RFC 7231 *dispatch
// policy* around it is Bucket 2.
//
// ADR: docs/adr/0010-runtime-neutral-async-and-transport-posture.md
// (runtime-neutral transport),
// docs/adr/0019-http-transport-sole-dispatch.md (HttpTransport sole
// dispatch),
// docs/adr/0041-transport-policy-l3-layering.md (transport policy
// layering, lines 36-43),
// docs/adr/0046-transport-policy-js-exposure.md (transport policy
// JS exposure).
// Doctrine: docs/alloy-doctrine.md, Bucket 2 row for `parse_retry_after`
// for the HTTP `Retry-After` header.
// CI gate: .github/workflows/never-swap-gates.yml#gate-transport-stack.
#[must_use]
pub fn parse_retry_after(value: &str, now: SystemTime) -> Option<RetryAfter> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.chars().all(|character| character.is_ascii_digit()) {
        return trimmed
            .parse::<u64>()
            .ok()
            .map(Duration::from_secs)
            .map(RetryAfter::new);
    }

    let retry_at = httpdate::parse_http_date(trimmed).ok()?;
    let retry_at_secs = unix_timestamp(retry_at)?;
    let now_secs = unix_timestamp(now)?;
    if retry_at_secs <= now_secs {
        return Some(RetryAfter::new(Duration::from_secs(0)));
    }
    Some(RetryAfter::new(Duration::from_secs(
        (retry_at_secs - now_secs).cast_unsigned(),
    )))
}

fn unix_timestamp(time: SystemTime) -> Option<i64> {
    time.duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs()
        .try_into()
        .ok()
}

/// Resolves the `Retry-After` delay carried by a set of response headers.
///
/// Scans `headers` for a `Retry-After` field name (ASCII case-insensitive) and
/// parses its value with [`parse_retry_after`] against the wasm-safe wall clock
/// ([`crate::system_now`]). Returns [`None`] when no `Retry-After` header is
/// present or its value does not parse; an HTTP-date in the past resolves to
/// [`Duration::ZERO`].
///
/// This is the consumer-facing accessor for surfacing a server backoff hint on
/// a returned error. The retry driver computes its own clock-injected backoff
/// internally and does not use this helper.
#[must_use]
pub fn retry_after_from_headers(headers: &[(String, String)]) -> Option<Duration> {
    headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("retry-after"))
        .and_then(|(_, value)| parse_retry_after(value, crate::system_now()))
        .map(RetryAfter::delay)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::retry_after_from_headers;

    #[test]
    fn absent_header_resolves_to_none() {
        assert_eq!(retry_after_from_headers(&[]), None);
        let headers = [("content-type".to_owned(), "application/json".to_owned())];
        assert_eq!(retry_after_from_headers(&headers), None);
    }

    #[test]
    fn delta_seconds_resolves_without_a_clock_dependency() {
        let headers = [("Retry-After".to_owned(), "120".to_owned())];
        assert_eq!(
            retry_after_from_headers(&headers),
            Some(Duration::from_secs(120))
        );
    }

    #[test]
    fn header_name_match_is_case_insensitive() {
        let headers = [("retry-after".to_owned(), "5".to_owned())];
        assert_eq!(
            retry_after_from_headers(&headers),
            Some(Duration::from_secs(5))
        );
    }

    #[test]
    fn unparsable_value_resolves_to_none() {
        let headers = [("retry-after".to_owned(), "soon".to_owned())];
        assert_eq!(retry_after_from_headers(&headers), None);
    }
}
