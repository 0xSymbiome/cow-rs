use cow_sdk_app_data::validate_app_data_doc;
use serde_json::{Value, json};

fn bundled_schema_versions() -> Vec<String> {
    let mut versions = std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/schemas"))
        .expect("schema directory must be readable")
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter_map(|name| {
            name.strip_prefix('v')
                .and_then(|rest| rest.strip_suffix(".json"))
                .map(str::to_owned)
        })
        .collect::<Vec<_>>();
    versions.sort();
    versions
}

fn minimal_doc(version: &str) -> Value {
    json!({
        "version": version,
        "metadata": {}
    })
}

#[test]
fn root_schema_version_matrix_validates_minimal_docs_and_rejects_missing_required_fields() {
    let versions = bundled_schema_versions();
    assert!(
        !versions.is_empty(),
        "bundled schema matrix must include root schemas"
    );

    for version in versions {
        let valid = minimal_doc(&version);
        let validation = validate_app_data_doc(&valid);
        assert!(
            validation.success,
            "minimal root document for schema {version} must validate, got {:?}",
            validation.errors,
        );

        for required_field in ["version", "metadata"] {
            let mut invalid = valid.clone();
            invalid
                .as_object_mut()
                .expect("minimal doc is an object")
                .remove(required_field);

            let validation = validate_app_data_doc(&invalid);
            assert!(
                !validation.success,
                "schema {version} must reject documents missing {required_field}",
            );
            let errors = validation
                .errors
                .as_deref()
                .expect("invalid schema validation carries a rendered error");
            assert!(
                errors.contains(required_field) || errors.contains("required"),
                "schema {version} missing-field error should identify the required-field failure, got {errors:?}",
            );
        }
    }
}
