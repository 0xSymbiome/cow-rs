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
/// The parser accepts delta-seconds and IMF-fixdate values. Invalid, empty,
/// negative, or unsupported date formats return [`None`].
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

    let retry_at = parse_http_date(trimmed)?;
    let now = unix_timestamp(now)?;
    if retry_at <= now {
        return Some(RetryAfter::new(Duration::from_secs(0)));
    }
    Some(RetryAfter::new(Duration::from_secs(
        (retry_at - now).cast_unsigned(),
    )))
}

fn parse_http_date(value: &str) -> Option<i64> {
    let mut parts = value.split_ascii_whitespace();
    let weekday = parts.next()?;
    let day = parts.next()?.parse::<u32>().ok()?;
    let month = parse_http_month(parts.next()?)?;
    let year = parts.next()?.parse::<i32>().ok()?;
    let (hour, minute, second) = parse_http_time(parts.next()?)?;
    let timezone = parts.next()?;

    if !weekday.ends_with(',') || timezone != "GMT" || parts.next().is_some() {
        return None;
    }

    let days = days_from_civil(year, month, day)?;
    let seconds = i64::from(hour) * 3_600 + i64::from(minute) * 60 + i64::from(second);
    days.checked_mul(86_400)?.checked_add(seconds)
}

fn parse_http_month(value: &str) -> Option<u32> {
    match value {
        "Jan" => Some(1),
        "Feb" => Some(2),
        "Mar" => Some(3),
        "Apr" => Some(4),
        "May" => Some(5),
        "Jun" => Some(6),
        "Jul" => Some(7),
        "Aug" => Some(8),
        "Sep" => Some(9),
        "Oct" => Some(10),
        "Nov" => Some(11),
        "Dec" => Some(12),
        _ => None,
    }
}

fn parse_http_time(value: &str) -> Option<(u32, u32, u32)> {
    let mut parts = value.split(':');
    let hour = parts.next()?.parse::<u32>().ok()?;
    let minute = parts.next()?.parse::<u32>().ok()?;
    let second = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some() || hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    Some((hour, minute, second))
}

fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i64> {
    if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
        return None;
    }

    // Promote every intermediate to i64 so an attacker-controlled
    // `Retry-After: <imf-fixdate>` header carrying an out-of-range year
    // (the wire format admits any decimal year) cannot panic the
    // checked-arithmetic-disabled release build with an `i32` overflow on
    // `era * 146_097` or `year_of_era * 365`.
    let adjusted_year = i64::from(year) - i64::from(month <= 2);
    let era = if adjusted_year >= 0 {
        adjusted_year
    } else {
        adjusted_year - 399
    } / 400;
    let year_of_era = adjusted_year - era * 400;
    let month_prime = i64::from(month.cast_signed()) + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + i64::from(day.cast_signed()) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

    Some(era * 146_097 + day_of_era - 719_468)
}

const fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

const fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn unix_timestamp(time: SystemTime) -> Option<i64> {
    time.duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs()
        .try_into()
        .ok()
}
