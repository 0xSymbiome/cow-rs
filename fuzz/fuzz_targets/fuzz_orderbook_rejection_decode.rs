#![no_main]

//! Fuzz target for typed orderbook rejection decoding.
//!
//! **Surface:** `cow_sdk_orderbook::parse_rejection`.
//! **Property:** `PROP-ORD-008`.
//! **Seed contract:** corpus inputs cover every known services tag plus
//! malformed-envelope and non-UTF-8 body boundaries.
//! **Corpus README:** `../corpus/fuzz_orderbook_rejection_decode/README.md`.
//!
//! The target feeds arbitrary bytes to the parser under both `400 Bad Request`
//! and `500 Internal Server Error`, matching the two statuses that carry most
//! typed rejection envelopes. Parser failures are acceptable for malformed
//! input; successful classifications must have a deterministic, non-empty
//! `Display` representation.

use cow_sdk_orderbook::parse_rejection;
use http::StatusCode;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|body: &[u8]| {
    for status in [StatusCode::BAD_REQUEST, StatusCode::INTERNAL_SERVER_ERROR] {
        if let Some(rejection) = parse_rejection(status, body) {
            let rendered = rejection.to_string();
            assert!(
                !rendered.is_empty(),
                "typed orderbook rejection Display must stay non-empty",
            );
            assert_eq!(
                rendered,
                rejection.to_string(),
                "typed orderbook rejection Display must be deterministic",
            );
        }
    }
});
