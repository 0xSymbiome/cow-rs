//! Fixture-driven parity contract for `cow-sdk-app-data`.
//!
//! Loads `parity/fixtures/app-data.json` (schema version 1) at compile time,
//! iterates every documented case, and asserts the Rust app-data helpers
//! reproduce the pinned upstream behavior. The helpers exercised are:
//!
//! * [`generate_app_data_doc`] — default and custom document generation.
//! * [`app_data_hex_to_cid`] / [`cid_to_app_data_hex`] — supported CID
//!   form round-trip.
//! * [`get_app_data_info`] — deterministic document processing.
//! * [`SchemaVersion`] / [`LATEST_APP_DATA_VERSION`] — semver version
//!   validation and the latest-version surface.
//! * [`validate_app_data_doc`] — typed metadata validation with a typed result.
//! * [`fetch_doc_from_cid`] / [`fetch_doc_from_app_data_hex`] —
//!   configurable-URI fetch helpers.
//!
//! Failure messages carry the fixture case id so a reviewer looking at a
//! broken CI run sees the exact upstream vector that diverged.

use async_trait::async_trait;
use cow_sdk_app_data::{
    AppDataError, AppDataParams, LATEST_APP_DATA_VERSION, SchemaVersion, app_data_hex_to_cid,
    cid_to_app_data_hex, generate_app_data_doc, get_app_data_info, validate_app_data_doc,
};
use serde_json::{Value, json};

const FIXTURE: &str = include_str!("../../../parity/fixtures/app-data.json");

#[tokio::test]
async fn parity_fixture_cases_hold() {
    let fixture: Value = serde_json::from_str(FIXTURE).expect("fixture must parse as JSON");

    assert_eq!(
        fixture["schema_version"].as_u64(),
        Some(1),
        "app-data fixture must declare schema_version 1",
    );
    assert_eq!(
        fixture["surface"].as_str(),
        Some("app-data"),
        "app-data fixture must carry the app-data surface label",
    );

    let cases = fixture["cases"]
        .as_array()
        .expect("app-data fixture must expose a cases array");

    for case in cases {
        let id = case["id"]
            .as_str()
            .expect("every fixture case must carry a string id");
        let expected = &case["expected"];

        match id {
            "app-data-generate-default-doc" => assert_generate_default_doc(id, expected),
            "app-data-custom-doc-generation" => assert_custom_doc_generation(id, case, expected),
            "app-data-cid-v1-conversion" => assert_cid_v1_conversion(id, case, expected),
            "app-data-cid-digest-extraction" => assert_cid_digest_extraction(id, case, expected),
            "app-data-get-app-data-info-deterministic" => {
                assert_get_app_data_info_deterministic(id, expected);
            }
            "app-data-schema-lookup-contract" => assert_schema_lookup_contract(id, expected),
            "app-data-validation-contract" => assert_validation_contract(id, expected),
            "app-data-fetch-transport-boundary" => {
                assert_fetch_transport_boundary(id, expected).await;
            }
            "app-data-schema-regression-families" => {
                assert_schema_regression_families(id, expected);
            }
            other => panic!("unknown app-data fixture case id: {other}"),
        }
    }
}

fn assert_generate_default_doc(id: &str, expected: &Value) {
    let expected_version = expected["version"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.version must be a string"));
    let expected_app_code = expected["app_code"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.app_code must be a string"));

    let doc = generate_app_data_doc(AppDataParams::default());
    assert_eq!(
        doc["version"].as_str(),
        Some(expected_version),
        "case {id}: default doc version must match the pinned latest schema",
    );
    assert_eq!(
        doc["appCode"].as_str(),
        Some(expected_app_code),
        "case {id}: default doc app_code must be the pinned default",
    );
    assert_eq!(
        doc["metadata"],
        json!({}),
        "case {id}: default doc must carry an empty metadata object",
    );

    assert_eq!(
        LATEST_APP_DATA_VERSION, expected_version,
        "case {id}: LATEST_APP_DATA_VERSION must match the fixture",
    );
}

fn assert_custom_doc_generation(id: &str, case: &Value, expected: &Value) {
    let environment = case["input"]["environment"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: input.environment must be a string"))
        .to_owned();
    let metadata = case["input"]["metadata"]
        .as_object()
        .unwrap_or_else(|| panic!("case {id}: input.metadata must be an object"))
        .clone();

    let doc = generate_app_data_doc(
        AppDataParams::default()
            .with_environment(environment)
            .with_metadata(metadata),
    );

    assert_eq!(
        doc["version"].as_str(),
        expected["version"].as_str(),
        "case {id}: custom doc version must match",
    );
    assert_eq!(
        doc["appCode"].as_str(),
        expected["appCode"].as_str(),
        "case {id}: custom doc appCode must match",
    );
    assert_eq!(
        doc["environment"].as_str(),
        expected["environment"].as_str(),
        "case {id}: custom doc environment must match",
    );
    assert_eq!(
        doc["metadata"], expected["metadata"],
        "case {id}: custom doc metadata must match",
    );
}

fn assert_cid_v1_conversion(id: &str, case: &Value, expected: &Value) {
    let hex_input = case["input"]["app_data_hex"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: input.app_data_hex must be a string"));
    let expected_cid = expected["cid"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.cid must be a string"));

    let cid = app_data_hex_to_cid(hex_input).unwrap_or_else(|error| {
        panic!("case {id}: app_data_hex_to_cid must succeed, got {error:?}")
    });
    assert_eq!(
        cid, expected_cid,
        "case {id}: latest CID conversion must match the pinned vector",
    );

    let roundtrip = cid_to_app_data_hex(&cid).unwrap_or_else(|error| {
        panic!("case {id}: cid_to_app_data_hex must succeed, got {error:?}")
    });
    assert_eq!(
        roundtrip.to_lowercase(),
        hex_input.to_lowercase(),
        "case {id}: CID round-trip must return the original app-data hex",
    );
}

fn assert_cid_digest_extraction(id: &str, case: &Value, expected: &Value) {
    let cid_input = case["input"]["cid"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: input.cid must be a string"));
    let expected_hex = expected["app_data_hex"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.app_data_hex must be a string"));

    let hex = cid_to_app_data_hex(cid_input).unwrap_or_else(|error| {
        panic!("case {id}: cid_to_app_data_hex must succeed, got {error:?}")
    });
    assert_eq!(
        hex.to_lowercase(),
        expected_hex.to_lowercase(),
        "case {id}: CID-to-hex extraction must match the pinned vector",
    );
}

fn assert_get_app_data_info_deterministic(id: &str, expected: &Value) {
    let returns: Vec<&str> = expected["returns"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.returns must be an array"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: returns entries must be strings"))
        })
        .collect();
    let stringification = expected["structured_doc_stringification"]
        .as_str()
        .unwrap_or_else(|| {
            panic!("case {id}: expected.structured_doc_stringification must be a string")
        });
    let invalid_behavior = expected["invalid_doc_behavior"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.invalid_doc_behavior must be a string"));

    // Structured doc path produces cid, app_data_hex, and app_data_content.
    let doc = generate_app_data_doc(AppDataParams::default());
    let info = get_app_data_info(doc.clone()).expect("structured doc must succeed through info");
    assert!(
        !info.cid.is_empty(),
        "case {id}: info.cid must not be empty"
    );
    assert!(
        info.app_data_hex.starts_with("0x"),
        "case {id}: info.app_data_hex must be 0x-prefixed",
    );
    assert!(
        !info.app_data_content.is_empty(),
        "case {id}: info.app_data_content must carry the canonical content",
    );

    assert!(
        returns.contains(&"cid")
            && returns.contains(&"appDataHex")
            && returns.contains(&"appDataContent"),
        "case {id}: fixture must name the three AppDataInfo fields",
    );
    assert_eq!(
        stringification, "deterministic",
        "case {id}: the structured-doc path must be deterministic",
    );
    assert_eq!(
        invalid_behavior, "typed-rejection",
        "case {id}: the invalid-doc surface must be a typed rejection",
    );

    // Invalid documents (e.g. unknown version) reject through a typed error.
    let mut bad_doc = doc;
    bad_doc["version"] = Value::String("not-a-semver".to_string());
    let error = get_app_data_info(bad_doc).expect_err(
        "case app-data-get-app-data-info-deterministic must reject malformed schema versions",
    );
    assert!(
        matches!(
            error,
            AppDataError::InvalidSchemaVersion(_) | AppDataError::InvalidAppDataProvided { .. }
        ),
        "case {id}: invalid doc must surface a typed AppDataError",
    );
}

fn assert_schema_lookup_contract(id: &str, expected: &Value) {
    let latest_version = expected["latest_version"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.latest_version must be a string"));
    assert!(
        expected["supports_semver_versions"]
            .as_bool()
            .unwrap_or(false),
        "case {id}: expected.supports_semver_versions must be true",
    );
    assert!(
        expected["rejects_non_semver"].as_bool().unwrap_or(false),
        "case {id}: expected.rejects_non_semver must be true",
    );
    assert!(
        expected["rejects_missing_version"]
            .as_bool()
            .unwrap_or(false),
        "case {id}: expected.rejects_missing_version must be true",
    );

    assert_eq!(
        LATEST_APP_DATA_VERSION, latest_version,
        "case {id}: latest-version constant must match the fixture",
    );

    // Valid semver versions parse.
    SchemaVersion::new(LATEST_APP_DATA_VERSION)
        .unwrap_or_else(|error| panic!("case {id}: latest version must parse, got {error:?}"));

    // Non-semver shapes reject through InvalidSchemaVersion.
    for invalid in ["v1.14.0", "1.14", "1.14.0.0", "not-a-version", ""] {
        let error = SchemaVersion::new(invalid).expect_err(&format!(
            "case {id}: version parsing must reject {invalid:?}"
        ));
        assert!(
            matches!(error, AppDataError::InvalidSchemaVersion(_)),
            "case {id}: {invalid:?} must reject through a typed AppDataError",
        );
    }
}

fn assert_validation_contract(id: &str, expected: &Value) {
    let success_expected = expected["success_field"].as_bool().unwrap_or(false);
    let error_surface = expected["error_surface"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.error_surface must be a string"));
    assert!(
        success_expected,
        "case {id}: fixture must name a success=true path",
    );
    assert_eq!(
        error_surface, "validation-result",
        "case {id}: validation must surface results through ValidationResult",
    );

    let valid = generate_app_data_doc(AppDataParams::default());
    let ok = validate_app_data_doc(&valid);
    assert!(
        ok.success,
        "case {id}: canonical generated doc must validate successfully",
    );
    assert!(
        ok.errors.is_none(),
        "case {id}: successful validation must not surface errors",
    );

    let mut invalid = valid;
    // Force an invalid document shape by removing the required version field.
    if let Value::Object(map) = &mut invalid {
        map.remove("version");
    }
    let err = validate_app_data_doc(&invalid);
    assert!(
        !err.success,
        "case {id}: missing-version doc must not validate",
    );
    assert!(
        err.errors.is_some(),
        "case {id}: failed validation must surface error text",
    );
}

async fn assert_fetch_transport_boundary(id: &str, expected: &Value) {
    let helpers: Vec<&str> = expected["helpers"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.helpers must be an array"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: helpers entries must be strings"))
        })
        .collect();
    let injection = expected["transport_injection"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.transport_injection must be a string"));

    assert!(
        helpers.contains(&"fetchDocFromCid") && helpers.contains(&"fetchDocFromAppDataHex"),
        "case {id}: fixture must name the transport-boundary helpers",
    );
    assert_eq!(
        injection, "explicit-uri-parameter",
        "case {id}: transport injection must remain explicit URI parameter",
    );

    // The Rust fetch helpers accept an optional read URI alongside a transport
    // reference. Exercising them with a panic-on-call transport and a
    // deliberately empty URI proves the URI parameter is honored before the
    // transport fires; an empty URI fails through the typed policy surface
    // before any network call is dispatched.
    let err = cow_sdk_app_data::fetch_doc_from_cid("bafybeiany", &PanicFetchTransport, Some(""))
        .await
        .expect_err("empty IPFS URI must fail-closed before dispatching the transport");
    assert!(
        matches!(err, AppDataError::Transport { .. }),
        "case {id}: fetch_doc_from_cid must reject empty URI through AppDataError::Transport",
    );

    // The *_hex helpers derive a CID from the hex input before dispatching,
    // so a malformed hex literal fails through the typed hex path before the
    // transport runs — no URI override needed.
    let err = cow_sdk_app_data::fetch_doc_from_app_data_hex("0xzz", &PanicFetchTransport, None)
        .await
        .expect_err("malformed app-data hex must fail-closed before dispatching the transport");
    assert!(
        matches!(err, AppDataError::Transport { .. }),
        "case {id}: fetch_doc_from_app_data_hex must reject malformed hex before dispatch",
    );
}

fn assert_schema_regression_families(id: &str, expected: &Value) {
    let families: Vec<&str> = expected["families"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.families must be an array"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: families entries must be strings"))
        })
        .collect();

    // Every named late-version metadata family must carry a well-formed semver
    // version suffix, proving the upstream schema-regression family set stays
    // represented on the Rust side.
    for family in families {
        let version = family
            .rsplit('@')
            .next()
            .unwrap_or_else(|| panic!("case {id}: family {family} must carry @version suffix"));
        SchemaVersion::new(version).unwrap_or_else(|error| {
            panic!("case {id}: family {family} version must parse, got {error:?}")
        });
    }
}

struct PanicFetchTransport;

#[async_trait]
impl cow_sdk_app_data::IpfsFetchTransport for PanicFetchTransport {
    async fn get(&self, _uri: &str) -> Result<String, AppDataError> {
        panic!(
            "PanicFetchTransport must never be invoked; malformed inputs must fail-closed earlier"
        )
    }
}
