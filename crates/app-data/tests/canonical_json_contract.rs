//! Fixture-driven contract suite pinning the RFC 8785 canonical-JSON
//! serialisation for app-data documents whose object keys carry code
//! points that diverge between UTF-16 code-unit ordering and bytewise
//! UTF-8 ordering.
//!
//! ASCII-only app-data documents continue to serialise byte-identically
//! under both orderings; the parity fixture
//! `parity/fixtures/app_data/canonical_json_utf16.json` is the locked
//! witness for the documented divergence after the migration to
//! `serde_jcs::to_string`.

use cow_sdk_app_data::stringify_deterministic;
use serde_json::Value;

const CANONICAL_JSON_UTF16: &str =
    include_str!("../../../parity/fixtures/app_data/canonical_json_utf16.json");

#[test]
fn canonical_json_utf16_corpus_serialises_to_expected_canonical_bytes() {
    let document: Value = serde_json::from_str(CANONICAL_JSON_UTF16)
        .expect("canonical_json_utf16 fixture must be valid JSON");
    let cases = document["cases"]
        .as_array()
        .expect("canonical_json_utf16 fixture must carry a top-level cases array");
    assert!(
        !cases.is_empty(),
        "canonical_json_utf16 corpus must carry at least one case",
    );

    for case in cases {
        let id = case["id"].as_str().expect("each case must carry an id");
        let input = &case["inputs"]["document"];
        let expected = case["expected"]["canonical"]
            .as_str()
            .unwrap_or_else(|| panic!("case {id} must carry expected.canonical"));

        let actual = stringify_deterministic(input)
            .unwrap_or_else(|error| panic!("case {id}: stringify_deterministic failed: {error:?}"));
        assert_eq!(
            actual, expected,
            "case {id}: stringify_deterministic must match the canonical RFC 8785 byte sequence",
        );
    }
}
