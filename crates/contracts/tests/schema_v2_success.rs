//! Positive fixtures for deployment registry schema v2.

use cow_sdk_contracts::{ContractId, DeploymentChainId, DeploymentEnv, Registry};

struct SuccessFixture {
    name: &'static str,
    source: &'static str,
    expected_len: usize,
}

const SUCCESS_FIXTURES: &[SuccessFixture] = &[
    SuccessFixture {
        name: "env_specific_gpv2.toml",
        source: include_str!("fixtures/schema_v2_success/env_specific_gpv2.toml"),
        expected_len: 2,
    },
    SuccessFixture {
        name: "environment_agnostic_composable.toml",
        source: include_str!("fixtures/schema_v2_success/environment_agnostic_composable.toml"),
        expected_len: 1,
    },
    SuccessFixture {
        name: "mixed_contract_families.toml",
        source: include_str!("fixtures/schema_v2_success/mixed_contract_families.toml"),
        expected_len: 3,
    },
];

#[test]
fn schema_v2_success_fixtures_parse_and_resolve_expected_keys() {
    for fixture in SUCCESS_FIXTURES {
        let registry = Registry::from_toml_str(fixture.source)
            .unwrap_or_else(|error| panic!("{} must parse: {error}", fixture.name));
        assert_eq!(
            registry.len(),
            fixture.expected_len,
            "{} row count drifted",
            fixture.name,
        );
    }
}

#[test]
fn schema_v2_environment_agnostic_lookup_falls_back_for_concrete_envs() {
    let registry = Registry::from_toml_str(include_str!(
        "fixtures/schema_v2_success/environment_agnostic_composable.toml"
    ))
    .expect("environment-agnostic capability fixture must parse");

    let expected = registry
        .address(
            ContractId::ComposableCow,
            DeploymentChainId::Polygon,
            DeploymentEnv::EnvironmentAgnostic,
        )
        .expect("environment-agnostic row must resolve directly");

    assert_eq!(
        registry.address(
            ContractId::ComposableCow,
            DeploymentChainId::Polygon,
            DeploymentEnv::Prod,
        ),
        Some(expected),
        "capability contracts must fall back from concrete envs to environment-agnostic rows",
    );
}
