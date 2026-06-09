#![cfg(feature = "transport-policy")]

//! Fixture-driven contract suite pinning the `parse_retry_after` HTTP-date
//! branch byte-for-byte against the committed parity fixture rows.
//!
//! Three fixture files under `parity/fixtures/retry_after/` capture the
//! accept and reject corpora for the parser:
//!
//! - `imf_fixdate_accept.json` — every row carries a valid RFC 7231
//!   IMF-fixdate value and the canonical Unix timestamp it resolves to;
//! - `imf_fixdate_reject.json` — every row carries a malformed HTTP-date
//!   value that must surface as `None`;
//! - `legacy_rfc850.json` — every row carries an RFC 850 legacy form that
//!   the parser accepts via `httpdate::parse_http_date` (a capability gain
//!   relative to the pre-`httpdate` IMF-fixdate-only path).
//!
//! Each accept row is exercised at the Unix epoch as `now` so the
//! returned delay is exactly `expected.parsed_unix_timestamp` seconds.

use std::time::{Duration, SystemTime};

use cow_sdk_core::transport::policy::parse_retry_after;
use serde_json::Value;

const IMF_FIXDATE_ACCEPT: &str =
    include_str!("../../../parity/fixtures/retry_after/imf_fixdate_accept.json");
const IMF_FIXDATE_REJECT: &str =
    include_str!("../../../parity/fixtures/retry_after/imf_fixdate_reject.json");
const LEGACY_RFC850: &str = include_str!("../../../parity/fixtures/retry_after/legacy_rfc850.json");

fn parse_fixture(raw: &str) -> Vec<Value> {
    let document: Value =
        serde_json::from_str(raw).expect("retry_after fixture must be valid JSON");
    document["cases"]
        .as_array()
        .expect("retry_after fixture must carry a top-level cases array")
        .clone()
}

#[test]
fn imf_fixdate_accept_corpus_resolves_to_expected_unix_timestamps() {
    let cases = parse_fixture(IMF_FIXDATE_ACCEPT);
    assert!(
        !cases.is_empty(),
        "imf_fixdate_accept corpus must carry at least one case"
    );

    for case in cases {
        let id = case["id"].as_str().expect("each case must carry an id");
        let raw = case["inputs"]["raw_header"]
            .as_str()
            .unwrap_or_else(|| panic!("case {id} must carry inputs.raw_header"));
        let expected = case["expected"]["parsed_unix_timestamp"]
            .as_u64()
            .unwrap_or_else(|| {
                panic!("case {id} must carry expected.parsed_unix_timestamp as u64")
            });

        let parsed = parse_retry_after(raw, SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|| panic!("case {id}: parse_retry_after must accept `{raw}`"));
        assert_eq!(
            parsed.delay(),
            Duration::from_secs(expected),
            "case {id}: parsed delay must equal expected.parsed_unix_timestamp",
        );
    }
}

#[test]
fn imf_fixdate_reject_corpus_returns_none() {
    let cases = parse_fixture(IMF_FIXDATE_REJECT);
    assert!(
        !cases.is_empty(),
        "imf_fixdate_reject corpus must carry at least one case"
    );

    for case in cases {
        let id = case["id"].as_str().expect("each case must carry an id");
        let raw = case["inputs"]["raw_header"]
            .as_str()
            .unwrap_or_else(|| panic!("case {id} must carry inputs.raw_header"));

        assert!(
            parse_retry_after(raw, SystemTime::UNIX_EPOCH).is_none(),
            "case {id}: parse_retry_after must reject `{raw}`",
        );
    }
}

#[test]
fn legacy_rfc850_corpus_resolves_to_expected_unix_timestamps() {
    let cases = parse_fixture(LEGACY_RFC850);
    assert!(
        !cases.is_empty(),
        "legacy_rfc850 corpus must carry at least one case"
    );

    for case in cases {
        let id = case["id"].as_str().expect("each case must carry an id");
        let raw = case["inputs"]["raw_header"]
            .as_str()
            .unwrap_or_else(|| panic!("case {id} must carry inputs.raw_header"));
        let expected = case["expected"]["parsed_unix_timestamp"]
            .as_u64()
            .unwrap_or_else(|| {
                panic!("case {id} must carry expected.parsed_unix_timestamp as u64")
            });

        let parsed = parse_retry_after(raw, SystemTime::UNIX_EPOCH).unwrap_or_else(|| {
            panic!("case {id}: parse_retry_after must accept legacy RFC 850 form `{raw}`")
        });
        assert_eq!(
            parsed.delay(),
            Duration::from_secs(expected),
            "case {id}: legacy RFC 850 form must resolve to expected.parsed_unix_timestamp",
        );
    }
}
