//! Integration tests for the chain-keyed deployment registry.
//!
//! Exercises three anchors of the public contract:
//!
//! * Every `(ContractId, SupportedChainId, CowEnv)` tuple reachable through
//!   [`Registry::default`] resolves to a non-zero [`Address`], and every
//!   supported chain has at least one registered contract in at least one
//!   deployment environment.
//! * The typed lookup API returns the canonical Ethereum-mainnet addresses
//!   so every downstream resolver reaches the same wire-level bytes the
//!   shipped crate has always pinned.
//! * The runtime parser surfaces every documented failure mode as a typed
//!   [`RegistryError`] — the same taxonomy the compile-time `build.rs`
//!   gate enforces. Exercising the reject path end-to-end keeps regressions
//!   red in CI rather than letting malformed manifests drift through.

use cow_sdk_contracts::{
    ContractId, DeploymentChainId, DeploymentEnv, DeploymentVerificationStatus, Registry,
    RegistryError,
};
use cow_sdk_core::{Address, CowEnv, SupportedChainId};

#[test]
fn registry_default_resolves_every_entry_to_a_non_zero_address() {
    let registry = Registry::default();
    assert!(
        !registry.is_empty(),
        "the embedded manifest must carry entries",
    );

    let zero = Address::new("0x0000000000000000000000000000000000000000")
        .expect("zero-address literal must parse");

    for (contract_id, chain_id, env, address) in registry.entries() {
        assert_ne!(
            address, &zero,
            "{contract_id} / {chain_id:?} / {env:?} must resolve to a non-zero address",
        );
    }
}

#[test]
fn registry_default_covers_every_supported_chain_in_at_least_one_env() {
    let registry = Registry::default();

    for chain_id in SupportedChainId::ALL {
        let has_prod = registry
            .address(ContractId::Settlement, chain_id, CowEnv::Prod)
            .is_some()
            || registry
                .address(ContractId::VaultRelayer, chain_id, CowEnv::Prod)
                .is_some()
            || registry
                .address(ContractId::EthFlow, chain_id, CowEnv::Prod)
                .is_some();
        let has_staging = registry
            .address(ContractId::Settlement, chain_id, CowEnv::Staging)
            .is_some()
            || registry
                .address(ContractId::VaultRelayer, chain_id, CowEnv::Staging)
                .is_some()
            || registry
                .address(ContractId::EthFlow, chain_id, CowEnv::Staging)
                .is_some();
        assert!(
            has_prod || has_staging,
            "{chain_id:?} must have at least one registered contract in at least one environment",
        );
    }
}

#[test]
fn registry_default_resolves_the_canonical_mainnet_addresses() {
    let registry = Registry::default();

    // Canonical lowercase 0x-prefixed wire form per PROP-WB-004; cow Address
    // canonicalizes to lowercase at construction (ADR 0052).
    assert_eq!(
        registry
            .address(
                ContractId::Settlement,
                SupportedChainId::Mainnet,
                CowEnv::Prod
            )
            .expect("settlement/prod/mainnet must be registered")
            .to_hex_string(),
        "0x9008d19f58aabd9ed0d60971565aa8510560ab41",
    );
    assert_eq!(
        registry
            .address(
                ContractId::Settlement,
                SupportedChainId::Mainnet,
                CowEnv::Staging,
            )
            .expect("settlement/staging/mainnet must be registered")
            .to_hex_string(),
        "0xf553d092b50bdcbdded1a99af2ca29fbe5e2cb13",
    );
    assert_eq!(
        registry
            .address(
                ContractId::VaultRelayer,
                SupportedChainId::Mainnet,
                CowEnv::Prod,
            )
            .expect("vault-relayer/prod/mainnet must be registered")
            .to_hex_string(),
        "0xc92e8bdf79f0507f65a392b0ab4667716bfe0110",
    );
    assert_eq!(
        registry
            .address(ContractId::EthFlow, SupportedChainId::Mainnet, CowEnv::Prod)
            .expect("eth-flow/prod/mainnet must be registered")
            .to_hex_string(),
        "0xba3cb449bd2b4adddbc894d8697f5170800eadec",
    );
}

#[test]
fn registry_rejects_unknown_contract_id_variants() {
    let manifest = r#"
schema_version = 2

[[entries]]
contract_id = "UnknownFlashLoan"
chain_id = 1
env = "prod"
address = "0x1111111111111111111111111111111111111111"
[entries.verification]
status = "code_hash_verified"
source = "test"
"#;

    let error = Registry::from_toml_str(manifest)
        .expect_err("unknown contract_id variants must be rejected");
    match error {
        RegistryError::Parse { source } => {
            let rendered = source.to_string();
            assert!(
                rendered.contains("contract_id"),
                "parser must name the offending field, got: {rendered}"
            );
        }
        other => panic!("expected Parse variant for unknown contract_id, got {other:?}"),
    }
}

#[test]
fn registry_rejects_unsupported_chain_ids() {
    let manifest = r#"
schema_version = 2

[[entries]]
contract_id = "Settlement"
chain_id = 424242
env = "prod"
address = "0x1111111111111111111111111111111111111111"
[entries.verification]
status = "code_hash_verified"
source = "test"
"#;

    let error = Registry::from_toml_str(manifest)
        .expect_err("chain ids outside SupportedChainId::ALL must be rejected");
    assert!(matches!(
        error,
        RegistryError::UnsupportedChainId {
            contract_id: ContractId::Settlement,
            chain_id: 424_242,
        }
    ));
}

#[test]
fn registry_rejects_malformed_address_literals() {
    let manifest = r#"
schema_version = 2

[[entries]]
contract_id = "Settlement"
chain_id = 1
env = "prod"
address = "not-even-close-to-hex"
[entries.verification]
status = "code_hash_verified"
source = "test"
"#;

    let error =
        Registry::from_toml_str(manifest).expect_err("non-hex address literals must be rejected");
    assert!(matches!(error, RegistryError::InvalidAddress { .. }));
}

#[test]
fn registry_rejects_duplicate_entries() {
    let manifest = r#"
schema_version = 2

[[entries]]
contract_id = "Settlement"
chain_id = 1
env = "prod"
address = "0x9008D19f58AAbD9eD0D60971565AA8510560ab41"
[entries.verification]
status = "code_hash_verified"
source = "test"

[[entries]]
contract_id = "Settlement"
chain_id = 1
env = "prod"
address = "0xba3cb449bd2b4adddbc894d8697f5170800eadec"
[entries.verification]
status = "code_hash_verified"
source = "test"
"#;

    let error = Registry::from_toml_str(manifest)
        .expect_err("duplicate (contract_id, chain_id, env) tuples must be rejected");
    assert!(matches!(
        error,
        RegistryError::DuplicateEntry {
            contract_id: ContractId::Settlement,
            chain_id: 1,
            env: DeploymentEnv::Prod,
        }
    ));
}

#[test]
fn registry_rejects_unsupported_schema_version() {
    let manifest = r#"
schema_version = 42

[[entries]]
contract_id = "Settlement"
chain_id = 1
env = "prod"
address = "0x9008D19f58AAbD9eD0D60971565AA8510560ab41"
[entries.verification]
status = "code_hash_verified"
source = "test"
"#;

    let error = Registry::from_toml_str(manifest)
        .expect_err("unsupported schema_version values must be rejected");
    assert!(matches!(
        error,
        RegistryError::UnsupportedSchemaVersion { expected: 2, .. }
    ));
}

#[test]
fn registry_address_lookup_matrix_is_exhaustive() {
    let registry = Registry::default();
    assert_eq!(
        registry.len(),
        177,
        "schema v2 registry row count must remain stable"
    );

    for (contract_id, chain_id, env, address) in registry.entries() {
        assert_eq!(
            registry.address(contract_id, chain_id, env),
            Some(*address),
            "{contract_id} / {chain_id:?} / {env:?} lookup must return the manifest address",
        );
    }
}

#[test]
fn empty_registry_manifest_exposes_empty_state() {
    let registry = Registry::from_toml_str("schema_version = 2\n")
        .expect("entry-less manifests are valid for parser consumers");

    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
    assert!(registry.entries().next().is_none());
    assert_eq!(
        registry.address(
            ContractId::Settlement,
            SupportedChainId::Mainnet,
            CowEnv::Prod,
        ),
        None,
    );
}

#[test]
fn registry_override_replaces_only_the_requested_key() {
    let canonical = Registry::default();
    let override_address = Address::new("0x1212121212121212121212121212121212121212").unwrap();
    let overridden = canonical.clone().with_override(
        ContractId::Settlement,
        SupportedChainId::Mainnet,
        CowEnv::Prod,
        override_address,
    );

    assert_eq!(
        overridden.address(
            ContractId::Settlement,
            SupportedChainId::Mainnet,
            CowEnv::Prod,
        ),
        Some(override_address),
    );
    assert_eq!(
        overridden.verification(
            ContractId::Settlement,
            SupportedChainId::Mainnet,
            CowEnv::Prod,
        ),
        Some(DeploymentVerificationStatus::CanonicalUnverified),
    );
    assert_ne!(
        canonical.address(
            ContractId::Settlement,
            SupportedChainId::Mainnet,
            CowEnv::Prod,
        ),
        Some(override_address),
        "with_override must not mutate the source registry",
    );
}

#[test]
fn concrete_env_lookup_does_not_fallback_for_environment_scoped_contracts() {
    let env_agnostic_settlement =
        Address::new("0x3434343434343434343434343434343434343434").unwrap();
    let registry = Registry::default().with_override(
        ContractId::Settlement,
        DeploymentChainId::Lens,
        DeploymentEnv::EnvironmentAgnostic,
        env_agnostic_settlement,
    );

    assert_eq!(
        registry.address(
            ContractId::Settlement,
            DeploymentChainId::Lens,
            DeploymentEnv::EnvironmentAgnostic,
        ),
        Some(env_agnostic_settlement),
    );
    assert_eq!(
        registry.address(
            ContractId::Settlement,
            DeploymentChainId::Lens,
            DeploymentEnv::Prod,
        ),
        None,
        "prod/staging contracts must not borrow environment-agnostic rows",
    );
}

#[test]
fn registry_verification_statuses_stay_in_registry_rows() {
    let registry = Registry::default();
    let statuses = registry
        .entry_details()
        .map(|(_, _, _, status, _)| status)
        .collect::<std::collections::BTreeSet<_>>();

    assert!(statuses.contains(&DeploymentVerificationStatus::CodeHashVerified));
    assert!(statuses.contains(&DeploymentVerificationStatus::ExternalVerified));
    assert!(statuses.contains(&DeploymentVerificationStatus::ReadmeTableUnverified));
    assert!(statuses.contains(&DeploymentVerificationStatus::CanonicalUnverified));
    assert_eq!(
        registry.verification(
            ContractId::ComposableCow,
            cow_sdk_contracts::DeploymentChainId::Polygon,
            DeploymentEnv::EnvironmentAgnostic,
        ),
        Some(DeploymentVerificationStatus::CanonicalUnverified),
    );
}

#[test]
fn deployment_manifest_labels_have_stable_display_spellings() {
    for (env, expected) in [
        (DeploymentEnv::Prod, "prod"),
        (DeploymentEnv::Staging, "staging"),
        (DeploymentEnv::EnvironmentAgnostic, "environment_agnostic"),
    ] {
        assert_eq!(env.as_str(), expected);
        assert_eq!(env.to_string(), expected);
    }

    for (status, expected) in [
        (
            DeploymentVerificationStatus::CodeHashVerified,
            "code_hash_verified",
        ),
        (
            DeploymentVerificationStatus::ExternalVerified,
            "external_verified",
        ),
        (
            DeploymentVerificationStatus::ReadmeTableUnverified,
            "readme_table_unverified",
        ),
        (
            DeploymentVerificationStatus::CanonicalUnverified,
            "canonical_unverified",
        ),
    ] {
        assert_eq!(status.as_str(), expected);
        assert_eq!(status.to_string(), expected);
    }
}
