//! Regression coverage for the JSON recursion guard on untrusted documents.
//!
//! App-data documents fetched from an untrusted IPFS gateway are parsed as
//! free-form JSON values. `serde_json` enforces a default recursion limit, so a
//! deeply nested document is rejected with a typed error rather than
//! overflowing the stack. This pins that behavior as the depth bound the SDK
//! relies on instead of a bespoke nesting cap.

use serde_json::Value;

#[test]
fn deeply_nested_json_is_rejected_by_the_recursion_guard() {
    // Far beyond serde_json's default recursion limit; a hostile gateway
    // cannot use nesting depth to exhaust the stack.
    let depth = 512;
    let nested = format!("{}{}", "[".repeat(depth), "]".repeat(depth));

    let parsed = serde_json::from_str::<Value>(&nested);

    assert!(
        parsed.is_err(),
        "deeply nested JSON must be rejected by the recursion guard rather than parsed"
    );
}
