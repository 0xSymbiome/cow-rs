//! Build-gate fixture coverage for deployment registry schema v2.

use cow_sdk_contracts::Registry;

const SUCCESS_SOURCES: &[&str] = &[
    include_str!("fixtures/schema_v2_success/env_specific_gpv2.toml"),
    include_str!("fixtures/schema_v2_success/environment_agnostic_composable.toml"),
    include_str!("fixtures/schema_v2_success/mixed_contract_families.toml"),
];

const REJECTION_SOURCES: &[&str] = &[
    include_str!("fixtures/schema_v2_rejection/unsupported_schema_version.toml"),
    include_str!("fixtures/schema_v2_rejection/capability_under_prod.toml"),
    include_str!("fixtures/schema_v2_rejection/gpv2_environment_agnostic.toml"),
    include_str!("fixtures/schema_v2_rejection/duplicate_registry_key.toml"),
    include_str!("fixtures/schema_v2_rejection/unsupported_deployment_chain.toml"),
];

#[test]
fn schema_v2_fixture_set_covers_success_and_rejection_paths() {
    for source in SUCCESS_SOURCES {
        Registry::from_toml_str(source).expect("success fixture must parse");
    }

    for source in REJECTION_SOURCES {
        Registry::from_toml_str(source).expect_err("rejection fixture must fail");
    }
}
