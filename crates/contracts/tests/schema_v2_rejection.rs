//! Negative-fixture matrix for the deployment registry schema-v2 validator.
//!
//! Every fixture is a manifest carrying one shape a well-formed registry must
//! avoid. The test drives them through the runtime [`Registry::from_toml_str`]
//! loader and asserts each produces the expected typed [`RegistryError`]. The
//! `build.rs` gate enforces the same invariants at compile time through its own
//! validator; this matrix is the runtime counterpart that exercises the
//! loader's rejection arms directly.

use cow_sdk_contracts::{ContractId, DeploymentEnv, Registry, RegistryError};

struct RejectionFixture {
    name: &'static str,
    source: &'static str,
    matcher: fn(&RegistryError) -> bool,
}

const REJECTION_FIXTURES: &[RejectionFixture] = &[
    // Schema-version, environment-scope, duplicate, and chain rejections.
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
    // Additional rejection shapes: the loader also rejects unknown contract
    // ids, malformed addresses, and TOML syntax errors.
    RejectionFixture {
        name: "bad_schema_version.toml",
        source: include_str!("fixtures/schema_v2_rejection/bad_schema_version.toml"),
        matcher: |error| matches!(error, RegistryError::UnsupportedSchemaVersion { .. }),
    },
    RejectionFixture {
        name: "unknown_contract_id.toml",
        source: include_str!("fixtures/schema_v2_rejection/unknown_contract_id.toml"),
        matcher: |error| matches!(error, RegistryError::Parse { .. }),
    },
    RejectionFixture {
        name: "unsupported_chain.toml",
        source: include_str!("fixtures/schema_v2_rejection/unsupported_chain.toml"),
        matcher: |error| matches!(error, RegistryError::UnsupportedChainId { .. }),
    },
    RejectionFixture {
        name: "invalid_address.toml",
        source: include_str!("fixtures/schema_v2_rejection/invalid_address.toml"),
        matcher: |error| matches!(error, RegistryError::InvalidAddress { .. }),
    },
    RejectionFixture {
        name: "duplicate_entry.toml",
        source: include_str!("fixtures/schema_v2_rejection/duplicate_entry.toml"),
        matcher: |error| matches!(error, RegistryError::DuplicateEntry { .. }),
    },
    RejectionFixture {
        name: "malformed_syntax.toml",
        source: include_str!("fixtures/schema_v2_rejection/malformed_syntax.toml"),
        matcher: |error| matches!(error, RegistryError::Parse { .. }),
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
