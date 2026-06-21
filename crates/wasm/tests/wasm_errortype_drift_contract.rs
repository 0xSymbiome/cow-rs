#![cfg(not(target_arch = "wasm32"))]

//! Drift gate binding the two hand-maintained orderbook `errorType` tag lists:
//! the Rust `OrderbookRejection::classify` match arms (the producer) and the
//! TypeScript `OrderBookErrorType` union (the consumer-facing type). A real
//! consumer branches on `error.errorType`, so a tag the SDK can emit but the
//! type omits — or a phantom type member the SDK never emits — is a silent
//! contract break. The `exports::errors` projection test pins that `classify`
//! round-trips into `errorType`; this keeps the two hand-authored lists exactly
//! equal so neither drifts from the other.
//!
//! The services `openapi.yml` is the upstream source of these tags, but it is a
//! local vendored checkout rather than a committed artifact, so it cannot back a
//! CI gate; `crates/orderbook/tests/rejection_contract.rs` pins the Rust set
//! against vendored fixtures, and this test pins the TS set against the Rust set.

use std::{collections::BTreeSet, fs, path::PathBuf};

fn read(parts: &[&str]) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for part in parts {
        path.push(part);
    }
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

/// Every identifier-shaped quoted string in `block` (`"PascalCase"`,
/// `"SCREAMING_SNAKE"`), skipping punctuated strings like the open `(string &
/// {})` arm or comments.
fn quoted_identifiers(block: &str) -> BTreeSet<String> {
    let mut tags = BTreeSet::new();
    let mut rest = block;
    while let Some(open) = rest.find('"') {
        rest = &rest[open + 1..];
        let Some(close) = rest.find('"') else { break };
        let tag = &rest[..close];
        if !tag.is_empty()
            && tag
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            tags.insert(tag.to_owned());
        }
        rest = &rest[close + 1..];
    }
    tags
}

/// The `errorType` tags the Rust producer classifies, read from the `classify`
/// match arms in `cow-sdk-orderbook` (every arm is `"Tag" => ...`; the body
/// carries no other quoted strings).
fn rust_classify_tags() -> BTreeSet<String> {
    let source = read(&["..", "orderbook", "src", "rejection.rs"]);
    let start = source.find("fn classify(").expect("classify fn present");
    let body = &source[start..];
    let end = body[1..].find("\nfn ").map_or(body.len(), |idx| idx + 1);
    quoted_identifiers(&body[..end])
}

/// The `errorType` tags the TypeScript type exposes, read from the
/// `OrderBookErrorType` union in the wasm facade.
fn ts_union_tags() -> BTreeSet<String> {
    let source = read(&["npm", "src", "errors.ts"]);
    let start = source
        .find("export type OrderBookErrorType")
        .expect("OrderBookErrorType union present");
    let body = &source[start..];
    let end = body.find(';').expect("union terminator");
    quoted_identifiers(&body[..end])
}

#[test]
fn rust_classify_and_ts_errortype_union_stay_in_sync() {
    let rust = rust_classify_tags();
    let ts = ts_union_tags();

    assert!(
        rust.len() > 40,
        "expected the full classify tag set, parsed only {}",
        rust.len(),
    );

    let missing_from_ts: Vec<_> = rust.difference(&ts).collect();
    assert!(
        missing_from_ts.is_empty(),
        "Rust classify() emits errorType tags the TS OrderBookErrorType union omits \
         (add them to crates/wasm/npm/src/errors.ts): {missing_from_ts:?}",
    );

    let phantom_in_ts: Vec<_> = ts.difference(&rust).collect();
    assert!(
        phantom_in_ts.is_empty(),
        "the TS OrderBookErrorType union lists errorType tags the Rust classify() never emits \
         (remove them, or add a classify arm in crates/orderbook/src/rejection.rs): \
         {phantom_in_ts:?}",
    );
}
