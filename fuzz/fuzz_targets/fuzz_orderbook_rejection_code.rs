#![no_main]

//! Fuzz target for the sanitized orderbook rejection-code wrapper.
//!
//! **Surface:** `cow_sdk_orderbook::rejection::OrderbookRejectionCode`.
//! **Property:** `PROP-ORD-008`.
//! **Seed contract:** corpus inputs cover canonical services rejection
//! tags pinned by
//! `parity/fixtures/orderbook.json::orderbook-duplicate-order-error`,
//! length and first-byte boundaries, and adversarial bodies containing
//! UTF-8 escapes, control bytes, or services-style description prose
//! that must collapse to the public redaction placeholder.
//! **Corpus README:** `../corpus/fuzz_orderbook_rejection_code/README.md`.
//!
//! The internal `is_safe_rejection_code` allowlist is crate-private; the
//! target reaches it through the public constructor and asserts the
//! documented invariants:
//!
//! * `OrderbookRejectionCode::new` never panics.
//! * `as_str()` returns either the input verbatim — and only when the
//!   input is a non-empty `[A-Z][A-Za-z0-9_]{0,47}` shape — or the
//!   public [`cow_sdk_core::REDACTED_PLACEHOLDER`] string.
//! * Determinism: identical input always produces identical output.

use cow_sdk_core::REDACTED_PLACEHOLDER;
use cow_sdk_orderbook::rejection::OrderbookRejectionCode;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let candidate = String::from_utf8_lossy(data).into_owned();

    let code = OrderbookRejectionCode::new(candidate.clone());
    let rendered = code.as_str().to_owned();

    // Determinism: the same input must produce the same sanitized output.
    let second = OrderbookRejectionCode::new(candidate.clone());
    assert_eq!(
        rendered,
        second.as_str(),
        "OrderbookRejectionCode::new must be deterministic on identical input",
    );

    // The wrapper's only two documented outputs are the verbatim input
    // (when it passes the allowlist) or the public redaction placeholder.
    let is_verbatim = rendered == candidate;
    let is_placeholder = rendered == REDACTED_PLACEHOLDER;
    assert!(
        is_verbatim || is_placeholder,
        "OrderbookRejectionCode::as_str must be input or REDACTED_PLACEHOLDER: got {rendered:?}",
    );

    // Cross-check the allowlist shape against the reference implementation:
    // accepted codes are non-empty, ≤ 48 bytes, start with ASCII uppercase,
    // and otherwise contain only `[A-Za-z0-9_]`.
    let reference_safe = is_safe_rejection_code_reference(&candidate);
    if reference_safe {
        assert!(
            is_verbatim,
            "allowlist-passing input must surface verbatim: {candidate:?} -> {rendered:?}",
        );
    } else {
        assert!(
            is_placeholder,
            "allowlist-failing input must collapse to REDACTED_PLACEHOLDER: \
             {candidate:?} -> {rendered:?}",
        );
    }

    // The Display impl mirrors `as_str` and must remain deterministic.
    assert_eq!(format!("{code}"), rendered, "Display must match as_str");
    assert_eq!(
        format!("{code}"),
        format!("{code}"),
        "Display must be deterministic on identical input",
    );
});

/// Reference implementation of the crate-private `is_safe_rejection_code`
/// allowlist, replicated so the fuzz target can independently assert which
/// branch of `OrderbookRejectionCode::new` must fire for a given input.
fn is_safe_rejection_code_reference(code: &str) -> bool {
    !code.is_empty()
        && code.len() <= 48
        && code.as_bytes().first().is_some_and(u8::is_ascii_uppercase)
        && code
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}
