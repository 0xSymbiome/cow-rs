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
