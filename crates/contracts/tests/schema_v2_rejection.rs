//! Negative fixtures for deployment registry schema v2.

use cow_sdk_contracts::{ContractId, DeploymentEnv, Registry, RegistryError};

struct RejectionFixture {
    name: &'static str,
    source: &'static str,
    matcher: fn(&RegistryError) -> bool,
}

const REJECTION_FIXTURES: &[RejectionFixture] = &[
    RejectionFixture {
        name: "unsupported_schema_version.toml",
        source: include_str!("fixtures/schema_v2_rejection/unsupported_schema_version.toml"),
        matcher: |error| matches!(error, RegistryError::UnsupportedSchemaVersion { .. }),
    },
    RejectionFixture {
        name: "capability_under_prod.toml",
        source: include_str!("fixtures/schema_v2_rejection/capability_under_prod.toml"),
        matcher: |error| {
            matches!(
                error,
                RegistryError::InvalidEnvironmentScope {
                    contract_id: ContractId::ComposableCow,
                    env: DeploymentEnv::Prod,
                }
            )
        },
    },
    RejectionFixture {
        name: "gpv2_environment_agnostic.toml",
        source: include_str!("fixtures/schema_v2_rejection/gpv2_environment_agnostic.toml"),
        matcher: |error| {
            matches!(
                error,
                RegistryError::InvalidEnvironmentScope {
                    contract_id: ContractId::Settlement,
                    env: DeploymentEnv::EnvironmentAgnostic,
                }
            )
        },
    },
    RejectionFixture {
        name: "duplicate_registry_key.toml",
        source: include_str!("fixtures/schema_v2_rejection/duplicate_registry_key.toml"),
        matcher: |error| matches!(error, RegistryError::DuplicateEntry { .. }),
    },
    RejectionFixture {
        name: "unsupported_deployment_chain.toml",
        source: include_str!("fixtures/schema_v2_rejection/unsupported_deployment_chain.toml"),
        matcher: |error| matches!(error, RegistryError::UnsupportedChainId { .. }),
    },
];

#[test]
fn schema_v2_rejection_fixtures_fail_with_typed_errors() {
    for fixture in REJECTION_FIXTURES {
        let Err(error) = Registry::from_toml_str(fixture.source) else {
            panic!("{} unexpectedly parsed", fixture.name);
        };
        assert!(
            (fixture.matcher)(&error),
            "{} produced unexpected error: {error:?}",
            fixture.name,
        );
    }
}
