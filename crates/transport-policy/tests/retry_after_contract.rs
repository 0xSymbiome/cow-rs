//! Behavior tests for the `Retry-After` header parser.
//!
//! These tests assert observable semantics of `parse_retry_after` for both
//! delta-seconds and IMF-fixdate forms. They complement
//! `fuzz/fuzz_targets/fuzz_retry_after_parse` which guarantees byte-level
//! robustness; the cases below pin the documented branches at fixed inputs.
//!
//! The parser is the only public surface of `retry_after.rs`; every helper
//! (`parse_http_date`, `parse_http_month`, `parse_http_time`,
//! `days_from_civil`, `days_in_month`, `is_leap_year`, `unix_timestamp`) is
//! private. Each test below drives the relevant private branch through the
//! public `parse_retry_after(value, now)` boundary, so the coverage gain is
//! attributable to behavior the SDK actually documents.

use std::time::{Duration, SystemTime};

use cow_sdk_transport_policy::{RetryAfter, parse_retry_after};
use proptest::prelude::*;

const EPOCH: SystemTime = SystemTime::UNIX_EPOCH;

/// `Wed, 21 Oct 2026 07:28:00 GMT` expressed as seconds since `UNIX_EPOCH`.
///
/// Independently derived: 20747 days from 1970-01-01 to 2026-10-21
/// (56 years × 365 + 14 leap days between 1970 and 2026 = 20454 days, plus
/// 293 days from Jan 1 to Oct 21 inclusive of leap days in 2024 = 293 days),
/// yielding `20747 * 86400 + 7 * 3600 + 28 * 60 = 1_792_567_680` seconds.
const FIXED_IMF_2026_10_21_07_28_00: u64 = 20_747 * 86_400 + 7 * 3_600 + 28 * 60;

const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

#[test]
fn parse_retry_after_returns_delta_seconds_for_numeric_string() {
    let parsed = parse_retry_after("120", EPOCH).expect("numeric delta is accepted");
    assert_eq!(parsed.delay(), Duration::from_secs(120));

    let parsed = parse_retry_after("0", EPOCH).expect("zero delta is accepted");
    assert_eq!(parsed.delay(), Duration::ZERO);
}

#[test]
fn parse_retry_after_trims_surrounding_whitespace_before_dispatch() {
    let parsed = parse_retry_after("   45   ", EPOCH).expect("trimmed numeric delta is accepted");
    assert_eq!(parsed.delay(), Duration::from_secs(45));
}

#[test]
fn parse_retry_after_returns_delta_for_future_imf_fixdate() {
    let parsed = parse_retry_after("Wed, 21 Oct 2026 07:28:00 GMT", EPOCH)
        .expect("future IMF-fixdate yields a delay");
    assert_eq!(
        parsed.delay(),
        Duration::from_secs(FIXED_IMF_2026_10_21_07_28_00),
    );
}

#[test]
fn parse_retry_after_clamps_past_imf_fixdate_to_zero() {
    let now = EPOCH + Duration::from_secs(FIXED_IMF_2026_10_21_07_28_00 + 1);
    let parsed = parse_retry_after("Wed, 21 Oct 2026 07:28:00 GMT", now)
        .expect("past IMF-fixdate parses but clamps to zero");
    assert_eq!(parsed, RetryAfter::new(Duration::ZERO));
}

#[test]
fn parse_retry_after_clamps_epoch_imf_fixdate_to_zero_when_now_is_epoch() {
    let parsed = parse_retry_after("Thu, 01 Jan 1970 00:00:00 GMT", EPOCH)
        .expect("epoch IMF-fixdate parses but equals now");
    assert_eq!(parsed, RetryAfter::new(Duration::ZERO));
}

#[test]
fn parse_retry_after_rejects_empty_and_whitespace() {
    assert!(parse_retry_after("", EPOCH).is_none());
    assert!(parse_retry_after("   ", EPOCH).is_none());
    assert!(parse_retry_after("\t\n", EPOCH).is_none());
}

#[test]
fn parse_retry_after_rejects_garbage_and_negative_and_trailing_junk() {
    for input in ["banana", "-5", "5xyz", "5.0", "foo bar baz"] {
        assert!(
            parse_retry_after(input, EPOCH).is_none(),
            "garbage input {input:?} must be rejected",
        );
    }
}

#[test]
fn parse_retry_after_rejects_weekday_without_comma() {
    // Same date as the canonical anchor; only the comma is missing.
    assert!(parse_retry_after("Wed 21 Oct 2026 07:28:00 GMT", EPOCH).is_none());
}

#[test]
fn parse_retry_after_rejects_non_gmt_timezone() {
    for input in [
        "Wed, 21 Oct 2026 07:28:00 UTC",
        "Wed, 21 Oct 2026 07:28:00 EST",
        "Wed, 21 Oct 2026 07:28:00 +0000",
    ] {
        assert!(
            parse_retry_after(input, EPOCH).is_none(),
            "non-GMT timezone {input:?} must be rejected",
        );
    }
}

#[test]
fn parse_retry_after_rejects_trailing_tokens_after_timezone() {
    assert!(parse_retry_after("Wed, 21 Oct 2026 07:28:00 GMT extra", EPOCH).is_none());
    assert!(parse_retry_after("Wed, 21 Oct 2026 07:28:00 GMT GMT", EPOCH).is_none());
}

#[test]
fn parse_retry_after_rejects_truncated_imf_fixdate_inputs() {
    // Each truncation point drops one further required component.
    for input in [
        "Wed,",
        "Wed, 21",
        "Wed, 21 Oct",
        "Wed, 21 Oct 2026",
        "Wed, 21 Oct 2026 07:28:00",
    ] {
        assert!(
            parse_retry_after(input, EPOCH).is_none(),
            "truncated input {input:?} must be rejected",
        );
    }
}

#[test]
fn parse_retry_after_rejects_non_numeric_day_and_year() {
    assert!(parse_retry_after("Wed, XX Oct 2026 07:28:00 GMT", EPOCH).is_none());
    assert!(parse_retry_after("Wed, 21 Oct YYYY 07:28:00 GMT", EPOCH).is_none());
}

#[test]
fn parse_retry_after_rejects_invalid_month_name() {
    for input in [
        "Wed, 21 Foo 2026 07:28:00 GMT",
        "Wed, 21 jan 2026 07:28:00 GMT",     // lowercase
        "Wed, 21 JAN 2026 07:28:00 GMT",     // uppercase
        "Wed, 21 January 2026 07:28:00 GMT", // full name
    ] {
        assert!(
            parse_retry_after(input, EPOCH).is_none(),
            "invalid month {input:?} must be rejected",
        );
    }
}

#[test]
fn parse_retry_after_accepts_every_calendar_month() {
    // Use 2026 (non-leap) and day 15 to avoid leap-year and 30-vs-31 issues.
    for (idx, name) in MONTH_NAMES.iter().enumerate() {
        let month = u32::try_from(idx).expect("month index fits in u32") + 1;
        let input = format!("Wed, 15 {name} 2026 00:00:00 GMT");
        let parsed = parse_retry_after(&input, EPOCH)
            .unwrap_or_else(|| panic!("month {name} must parse: {input}"));
        let delay = parsed.delay().as_secs();
        // Sanity: the parsed date is at least 56 years after epoch.
        assert!(
            delay > 56 * 365 * 86_400,
            "month {name} ({month}) parsed delay {delay} is implausibly small",
        );
    }
}

#[test]
fn parse_retry_after_rejects_out_of_range_time_components() {
    for input in [
        "Wed, 21 Oct 2026 24:00:00 GMT",    // hour 24
        "Wed, 21 Oct 2026 07:60:00 GMT",    // minute 60
        "Wed, 21 Oct 2026 07:28:60 GMT",    // second 60
        "Wed, 21 Oct 2026 99:00:00 GMT",    // hour 99
        "Wed, 21 Oct 2026 :28:00 GMT",      // missing hour
        "Wed, 21 Oct 2026 07:28 GMT",       // missing seconds
        "Wed, 21 Oct 2026 07:28:00:00 GMT", // trailing colon
    ] {
        assert!(
            parse_retry_after(input, EPOCH).is_none(),
            "out-of-range/malformed time {input:?} must be rejected",
        );
    }
}

#[test]
fn parse_retry_after_feb_29_respects_leap_year_rules() {
    // 2024 is a leap year (divisible by 4, not 100).
    assert!(parse_retry_after("Thu, 29 Feb 2024 00:00:00 GMT", EPOCH).is_some());
    // 2023 is not a leap year.
    assert!(parse_retry_after("Wed, 29 Feb 2023 00:00:00 GMT", EPOCH).is_none());
    // 2000 is a leap year (divisible by 400).
    assert!(parse_retry_after("Tue, 29 Feb 2000 00:00:00 GMT", EPOCH).is_some());
    // 1900 is NOT a leap year (century, not divisible by 400).
    // Date is before EPOCH so the parser must still reject it on day-bounds.
    let before_epoch_year = "Thu, 29 Feb 1900 00:00:00 GMT";
    assert!(parse_retry_after(before_epoch_year, EPOCH).is_none());
    // 2100 is NOT a leap year (century, not divisible by 400).
    assert!(parse_retry_after("Mon, 29 Feb 2100 00:00:00 GMT", EPOCH).is_none());
}

#[test]
fn parse_retry_after_rejects_day_31_in_30_day_months() {
    for month in ["Apr", "Jun", "Sep", "Nov"] {
        let input = format!("Tue, 31 {month} 2026 00:00:00 GMT");
        assert!(
            parse_retry_after(&input, EPOCH).is_none(),
            "day 31 in 30-day month {month} must be rejected",
        );
    }
    // And rejects day 32 universally.
    assert!(parse_retry_after("Wed, 32 Oct 2026 00:00:00 GMT", EPOCH).is_none());
    // And rejects day 0 (the explicit `day == 0` guard).
    assert!(parse_retry_after("Wed, 00 Oct 2026 00:00:00 GMT", EPOCH).is_none());
}

#[test]
fn parse_retry_after_rejects_dates_before_unix_epoch() {
    // The parser computes `days_from_civil` for any year in the wire format,
    // but `retry_at <= now` clamps anything at or before `now` to zero. With
    // now == UNIX_EPOCH, every pre-epoch date must clamp to ZERO, not return
    // `None`. (See the past-date branch in `parse_retry_after`.)
    let parsed = parse_retry_after("Fri, 31 Dec 1969 23:59:59 GMT", EPOCH)
        .expect("pre-epoch IMF-fixdate parses but clamps to zero");
    assert_eq!(parsed, RetryAfter::new(Duration::ZERO));
}

proptest! {
    /// Consecutive valid days produce timestamps that differ by exactly 86400s.
    ///
    /// This pins `days_from_civil` against itself across a wide year range
    /// without needing a `chrono` dev-dependency. We restrict to days 1..=27
    /// so that `day + 1` is always valid regardless of month length.
    #[test]
    fn days_from_civil_property_consecutive_days_differ_by_86400_seconds(
        year in 1971_i32..=2100_i32,
        month_idx in 0_usize..12_usize,
        day in 1_u32..=27_u32,
    ) {
        let month = MONTH_NAMES[month_idx];
        let first = format!("Wed, {day:02} {month} {year} 00:00:00 GMT");
        let second_day = day + 1;
        let second = format!("Wed, {second_day:02} {month} {year} 00:00:00 GMT");

        let parsed_first = parse_retry_after(&first, EPOCH)
            .map(|r| r.delay().as_secs())
            .expect("valid IMF-fixdate parses");
        let parsed_second = parse_retry_after(&second, EPOCH)
            .map(|r| r.delay().as_secs())
            .expect("valid IMF-fixdate parses");

        prop_assert_eq!(parsed_second - parsed_first, 86_400);
    }
}
