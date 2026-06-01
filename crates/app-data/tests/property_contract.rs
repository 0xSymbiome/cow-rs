//! Property-based coverage for the deterministic `cow-sdk-app-data` boundary.
//!
//! Each `proptest!` case exercises a named invariant on one of the CID,
//! schema-validation, canonical-stringifier, or IPFS preflight helpers.
//! Shrinking narrows any counter-example before `cargo test` prints it,
//! and committed seed files under `tests/proptest-regressions/` keep the
//! shrink outcomes reproducible across contributors. Net coverage
//! matches the hand-rolled enumerator this file replaced: every
//! invariant family the enumerator exercised carries a named property
//! here, with malformed-input fail-closed shapes consolidated via
//! `prop_oneof!` where the original already branched over a set.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::doc_markdown,
    clippy::missing_const_for_fn,
    clippy::redundant_clone,
    clippy::redundant_closure,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    reason = "pedantic, nursery, and style lints acceptable in test helper code"
)]

mod common;

use cow_sdk_app_data::{
    AppDataDoc, AppDataError, IpfsFetchPolicy, SchemaVersion, app_data_hex_to_cid,
    cid_to_app_data_hex, get_app_data_info, get_app_data_schema, stringify_deterministic,
};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Map, Number, Value};

/// Path for committed regression seeds; proptest writes new shrink
/// outcomes here so every contributor re-runs prior counter-examples
/// before any novel case is generated.
const REGRESSION_FILE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/proptest-regressions/property_contract.txt"
);

/// Produces the reviewed canonical JSON form for `value` using the same
/// algorithm the original enumerator hand-rolled: lexicographic object
/// keys, compact separators, and `serde_json::to_string` for strings.
fn manual_canonical_json(value: &Value) -> String {
    match value {
        Value::Null => "null".to_owned(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(string) => serde_json::to_string(string).unwrap(),
        Value::Array(array) => format!(
            "[{}]",
            array
                .iter()
                .map(manual_canonical_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::Object(object) => {
            let mut entries = object.iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(right.0));
            format!(
                "{{{}}}",
                entries
                    .into_iter()
                    .map(|(key, item)| {
                        format!(
                            "{}:{}",
                            serde_json::to_string(key).unwrap(),
                            manual_canonical_json(item)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
    }
}

/// Returns a JSON value with every object field reordered recursively.
/// The reviewed canonical-JSON algorithm and [`get_app_data_info`] must
/// produce byte-identical output on `value` and
/// `reordered_document(value)`.
fn reordered_document(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut entries = object.iter().collect::<Vec<_>>();
            entries.reverse();
            let mut reordered = Map::new();
            for (key, item) in entries {
                reordered.insert(key.clone(), reordered_document(item));
            }
            Value::Object(reordered)
        }
        Value::Array(array) => Value::Array(array.iter().map(reordered_document).collect()),
        other => other.clone(),
    }
}

/// Strategy that emits a 32-byte app-data hex digest.
fn app_data_hex_strategy() -> impl Strategy<Value = String> {
    any::<[u8; 32]>().prop_map(|bytes| format!("0x{}", alloy_primitives::hex::encode(bytes)))
}

/// Strategy that emits the union of malformed hex shapes
/// [`app_data_hex_to_cid`] must reject: missing `0x` prefix, wrong
/// byte length, and non-hex characters in the payload.
fn malformed_app_data_hex_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        any::<[u8; 32]>().prop_map(|bytes| alloy_primitives::hex::encode(bytes)),
        any::<[u8; 31]>().prop_map(|bytes| format!("0x{}", alloy_primitives::hex::encode(bytes))),
        any::<[u8; 33]>().prop_map(|bytes| format!("0x{}", alloy_primitives::hex::encode(bytes))),
        any::<[u8; 31]>().prop_map(|bytes| format!("0x{}gg", alloy_primitives::hex::encode(bytes))),
    ]
}

/// Strategy that emits a well-formed SemVer triplet suitable for
/// [`SchemaVersion::new`].
fn schema_version_strategy() -> impl Strategy<Value = String> {
    (0u32..=999u32, 0u32..=999u32, 0u32..=999u32)
        .prop_map(|(major, minor, patch)| format!("{major}.{minor}.{patch}"))
}

/// Strategy that emits the union of malformed schema-version shapes the
/// reviewed parser rejects: decimal pair, four-part dotted, `v`-
/// prefixed, negative component, leading/trailing whitespace, trailing
/// non-digit, and an arbitrary non-semver word.
fn malformed_schema_version_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        (0u32..=999u32, 0u32..=999u32).prop_map(|(major, minor)| format!("{major}.{minor}")),
        (0u32..=9u32, 0u32..=9u32, 0u32..=9u32, 0u32..=9u32)
            .prop_map(|(a, b, c, d)| format!("{a}.{b}.{c}.{d}")),
        schema_version_strategy().prop_map(|version| format!("v{version}")),
        schema_version_strategy().prop_map(|version| format!("-{version}")),
        schema_version_strategy().prop_map(|version| format!(" {version}")),
        schema_version_strategy().prop_map(|version| format!("{version} ")),
        schema_version_strategy().prop_map(|version| format!("{version}x")),
        (0u32..=99u32, 0u32..=99u32).prop_map(|(a, b)| format!("alpha.{a}.{b}")),
        Just("not.semver.value".to_owned()),
    ]
}

/// Recursive strategy that emits arbitrary JSON values for
/// [`stringify_deterministic`] coverage. Depth is bounded so shrinking
/// always terminates.
fn json_value_strategy() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        (0u64..=10_000u64).prop_map(|n| Value::Number(Number::from(n))),
        "[a-z][a-zA-Z0-9_-]{0,12}".prop_map(Value::String),
    ];
    leaf.prop_recursive(3, 16, 4, |element| {
        prop_oneof![
            prop::collection::vec(element.clone(), 0..=4).prop_map(Value::Array),
            prop::collection::btree_map("[a-z][a-zA-Z0-9_-]{0,8}", element, 0..=4).prop_map(
                |map| {
                    let mut object = Map::new();
                    for (key, value) in map {
                        object.insert(key, value);
                    }
                    Value::Object(object)
                },
            ),
        ]
    })
}

/// Strategy that emits a valid app-data document shell with empty
/// metadata and an optional `environment` field; the reviewed schema
/// accepts every emitted document.
fn valid_document_strategy() -> impl Strategy<Value = Value> {
    (
        "[A-Za-z][A-Za-z0-9 ]{0,15}",
        prop::option::of("[a-z][a-z0-9-]{0,12}"),
        any::<bool>(),
    )
        .prop_map(|(app_code, environment, metadata_first)| {
            let mut document = Map::new();
            if metadata_first {
                document.insert("metadata".to_owned(), Value::Object(Map::new()));
                document.insert("version".to_owned(), Value::String("0.7.0".to_owned()));
                document.insert("appCode".to_owned(), Value::String(app_code));
            } else {
                document.insert("appCode".to_owned(), Value::String(app_code));
                document.insert("version".to_owned(), Value::String("0.7.0".to_owned()));
                document.insert("metadata".to_owned(), Value::Object(Map::new()));
            }
            if let Some(env) = environment {
                document.insert("environment".to_owned(), Value::String(env));
            }
            Value::Object(document)
        })
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(REGRESSION_FILE))),
        ..ProptestConfig::default()
    })]

    /// [`app_data_hex_to_cid`] produces CIDs that round-trip through
    /// [`cid_to_app_data_hex`] back to the same hex digest. Malformed
    /// hex input fails closed with [`AppDataError::InvalidAppDataHex`].
    #[test]
    fn cid_roundtrips_hold_and_malformed_hex_inputs_fail_closed(
        hex in app_data_hex_strategy(),
        malformed in malformed_app_data_hex_strategy(),
    ) {
        let latest = app_data_hex_to_cid(&hex).unwrap();

        prop_assert_eq!(cid_to_app_data_hex(&latest).unwrap(), hex.clone());

        prop_assert!(matches!(
            app_data_hex_to_cid(&malformed).unwrap_err(),
            AppDataError::InvalidAppDataHex
        ));
    }

    /// [`SchemaVersion::new`] and [`str::parse::<SchemaVersion>`] are
    /// strict inverses on well-formed SemVer triplets; every malformed
    /// shape the reviewed parser rejects surfaces
    /// [`AppDataError::InvalidSchemaVersion`] through
    /// [`get_app_data_schema`] and fails [`SchemaVersion::new`].
    #[test]
    fn schema_versions_roundtrip_and_reject_malformed(
        valid in schema_version_strategy(),
        malformed in malformed_schema_version_strategy(),
    ) {
        let schema = SchemaVersion::new(valid.clone()).unwrap();
        prop_assert_eq!(schema.as_str(), valid.as_str());
        prop_assert_eq!(schema.to_string(), valid.clone());
        prop_assert_eq!(valid.parse::<SchemaVersion>().unwrap(), schema);

        prop_assert!(matches!(
            get_app_data_schema(&malformed).unwrap_err(),
            AppDataError::InvalidSchemaVersion(ref message) if message.as_inner() == &malformed
        ));
        prop_assert!(SchemaVersion::new(malformed.clone()).is_err());
        prop_assert!(malformed.parse::<SchemaVersion>().is_err());
    }

    /// [`IpfsFetchPolicy::new`] fails closed on every whitespace-only
    /// base URI, surfacing the typed builder error before any read
    /// transport is reached.
    #[test]
    fn ipfs_read_policy_rejects_blank_base_uri(
        whitespace_len in 1usize..=16usize,
    ) {
        let whitespace = " ".repeat(whitespace_len);
        let policy_err = IpfsFetchPolicy::new(whitespace).unwrap_err();
        match policy_err {
            AppDataError::Transport { ref detail, .. } => {
                prop_assert_eq!(detail.as_inner(), "ipfs read base uri must not be empty");
            }
            other => prop_assert!(false, "expected Transport, got {:?}", other),
        }
    }

    /// [`stringify_deterministic`] produces output byte-identical to the
    /// reviewed canonical-JSON algorithm for any JSON value; reparsing
    /// the rendered string recovers the original value (modulo key
    /// ordering), and a second pass produces byte-identical output.
    /// [`reordered_document`] leaves the rendered canonical form
    /// unchanged, covering the key-permutation invariance the original
    /// enumerator checked at deeper nesting.
    #[test]
    fn stringify_deterministic_matches_manual_canonical_form(
        document in json_value_strategy(),
    ) {
        let doc: AppDataDoc = document.clone();
        let rendered = stringify_deterministic(&doc).unwrap();

        prop_assert_eq!(rendered.clone(), manual_canonical_json(&document));
        prop_assert_eq!(
            serde_json::from_str::<Value>(&rendered).unwrap(),
            document.clone(),
        );
        let parsed_back: AppDataDoc = serde_json::from_str(&rendered).unwrap();
        prop_assert_eq!(stringify_deterministic(&parsed_back).unwrap(), rendered.clone());

        let reordered = reordered_document(&document);
        prop_assert_eq!(
            stringify_deterministic(&reordered).unwrap(),
            rendered,
        );
    }

    #[test]
    fn canonical_json_handles_unicode_escapes_consistently(
        generated in prop::collection::vec(any::<char>(), 0..=16).prop_map(|chars| chars.into_iter().collect::<String>()),
    ) {
        let literal = "quote\" slash\\ newline\n tab\t snowman\u{2603}";
        let document = Value::Object(Map::from_iter([
            ("generated".to_owned(), Value::String(generated.clone())),
            (generated.clone(), Value::String(literal.to_owned())),
        ]));

        let rendered = stringify_deterministic(&document).unwrap();

        prop_assert_eq!(rendered.clone(), manual_canonical_json(&document));
        prop_assert_eq!(serde_json::from_str::<Value>(&rendered).unwrap(), document);
        prop_assert!(rendered.contains("\\\""));
        prop_assert!(rendered.contains("\\\\"));
        prop_assert!(rendered.contains("\\n"));
        prop_assert!(rendered.contains("\\t"));
    }

    /// [`get_app_data_info`] is invariant under equivalent top-level
    /// key orderings: permuting the root object preserves the CID, the
    /// app-data content string, and the app-data hex digest.
    #[test]
    fn document_sources_canonicalize_equivalent_top_level_permutations(
        document in valid_document_strategy(),
    ) {
        let reordered = reordered_document(&document);
        let first = get_app_data_info(document).unwrap();
        let second = get_app_data_info(reordered).unwrap();
        prop_assert_eq!(first.cid.clone(), second.cid.clone());
        prop_assert_eq!(
            first.app_data_content.clone(),
            second.app_data_content.clone()
        );
        prop_assert_eq!(first.app_data_hex.clone(), second.app_data_hex.clone());
    }
}
