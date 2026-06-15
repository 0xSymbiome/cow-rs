#![no_main]

//! Fuzz target for typed orderbook rejection decoding.
//!
//! **Surface:** `cow_sdk_orderbook::parse_rejection`.
//! **Property:** `PROP-ORD-008`.
//! **Seed contract:** corpus inputs cover every known services tag plus
//! malformed-envelope and non-UTF-8 body boundaries.
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
            assert!(
                !rendered.contains('\0'),
                "typed orderbook rejection Display must not carry raw null bytes: {rendered}",
            );
            let debug = format!("{rejection:?}");
            assert_eq!(
                debug,
                format!("{rejection:?}"),
                "typed orderbook rejection Debug must be deterministic",
            );
            let reparsed = parse_rejection(status, body)
                .expect("re-parsing the same body must reproduce the typed rejection");
            assert_eq!(
                reparsed.to_string(),
                rendered,
                "parse_rejection must be deterministic on identical input",
            );
        }
    }
});
