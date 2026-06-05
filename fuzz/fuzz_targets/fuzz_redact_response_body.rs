#![no_main]

//! Fuzz target for the response-body redaction boundary.
//!
//! **Surface:** `cow_sdk_core::redact_response_body`.
//! **Property:** `PROP-CORE-015`.
//! **Seed contract:** corpus inputs cover the canonical empty input,
//! credential-key/value combinations, URL userinfo material, JWT-shaped
//! tokens, and length-boundary inputs at and beyond
//! `REDACTED_RESPONSE_BODY_MAX_BYTES`.
//!
//! The target maps arbitrary bytes through `String::from_utf8_lossy` into the
//! redaction helper, then asserts that the sanitized output is bounded in
//! length, free of credential-shaped substrings, deterministic across repeated
//! calls, and always valid UTF-8.

use cow_sdk_core::{
    REDACTED_RESPONSE_BODY_MAX_BYTES, RESPONSE_BODY_TRUNCATION_MARKER, redact_response_body,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);
    let redacted = redact_response_body(&input);

    // UTF-8 validity is implied by the `String` return type; assert the length
    // bound documented on `REDACTED_RESPONSE_BODY_MAX_BYTES`.
    let length_ceiling = REDACTED_RESPONSE_BODY_MAX_BYTES + RESPONSE_BODY_TRUNCATION_MARKER.len();
    assert!(
        redacted.len() <= length_ceiling,
        "redact_response_body output exceeded the documented length ceiling: \
         len = {}, ceiling = {}",
        redacted.len(),
        length_ceiling,
    );

    assert!(
        !contains_jwt_prefix(&redacted),
        "redact_response_body output leaked a JWT-shaped token: {redacted}",
    );
    assert!(
        !redacted.contains("://user:pass@"),
        "redact_response_body output leaked URL userinfo credentials: {redacted}",
    );
    assert!(
        !contains_credential_key_value(&redacted),
        "redact_response_body output leaked credential key=value material: {redacted}",
    );

    let again = redact_response_body(&input);
    assert_eq!(
        redacted, again,
        "redact_response_body must be deterministic on identical input",
    );
});

fn contains_credential_key_value(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    const CREDENTIAL_NEEDLES: &[&str] = &[
        "apikey=secret",
        "apikey:secret",
        "api_key=secret",
        "x-api-key=secret",
        "token=secret",
        "password=secret",
        "authorization: secret",
        "bearer secret",
    ];
    CREDENTIAL_NEEDLES
        .iter()
        .any(|needle| lowered.contains(needle))
}

fn contains_jwt_prefix(value: &str) -> bool {
    value
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-' && c != '.')
        .any(|token| {
            token.starts_with("eyJ")
                && token.len() >= 26
                && token
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
        })
}
