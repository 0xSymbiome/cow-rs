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

use cow_sdk_contracts::{ContractId, Registry, RegistryError};
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

    assert_eq!(
        registry
            .address(
                ContractId::Settlement,
                SupportedChainId::Mainnet,
                CowEnv::Prod
            )
            .expect("settlement/prod/mainnet must be registered")
            .as_str(),
        "0x9008D19f58AAbD9eD0D60971565AA8510560ab41",
    );
    assert_eq!(
        registry
            .address(
                ContractId::Settlement,
                SupportedChainId::Mainnet,
                CowEnv::Staging,
            )
            .expect("settlement/staging/mainnet must be registered")
            .as_str(),
        "0xf553d092b50bdcbddeD1A99aF2cA29FBE5E2CB13",
    );
    assert_eq!(
        registry
            .address(
                ContractId::VaultRelayer,
                SupportedChainId::Mainnet,
                CowEnv::Prod,
            )
            .expect("vault-relayer/prod/mainnet must be registered")
            .as_str(),
        "0xC92E8bdf79f0507f65a392b0ab4667716BFE0110",
    );
    assert_eq!(
        registry
            .address(ContractId::EthFlow, SupportedChainId::Mainnet, CowEnv::Prod)
            .expect("eth-flow/prod/mainnet must be registered")
            .as_str(),
        "0xba3cb449bd2b4adddbc894d8697f5170800eadec",
    );
}

#[test]
fn registry_rejects_unknown_contract_id_variants() {
    let manifest = r#"
schema_version = 1

[[entries]]
contract_id = "UnknownFlashLoan"
chain_id = 1
env = "prod"
address = "0x1111111111111111111111111111111111111111"
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
schema_version = 1

[[entries]]
contract_id = "Settlement"
chain_id = 424242
env = "prod"
address = "0x1111111111111111111111111111111111111111"
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
schema_version = 1

[[entries]]
contract_id = "Settlement"
chain_id = 1
env = "prod"
address = "not-even-close-to-hex"
"#;

    let error =
        Registry::from_toml_str(manifest).expect_err("non-hex address literals must be rejected");
    assert!(matches!(error, RegistryError::InvalidAddress { .. }));
}

#[test]
fn registry_rejects_duplicate_entries() {
    let manifest = r#"
schema_version = 1

[[entries]]
contract_id = "Settlement"
chain_id = 1
env = "prod"
address = "0x9008D19f58AAbD9eD0D60971565AA8510560ab41"

[[entries]]
contract_id = "Settlement"
chain_id = 1
env = "prod"
address = "0xba3cb449bd2b4adddbc894d8697f5170800eadec"
"#;

    let error = Registry::from_toml_str(manifest)
        .expect_err("duplicate (contract_id, chain_id, env) tuples must be rejected");
    assert!(matches!(
        error,
        RegistryError::DuplicateEntry {
            contract_id: ContractId::Settlement,
            chain_id: 1,
            env: CowEnv::Prod,
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
"#;

    let error = Registry::from_toml_str(manifest)
        .expect_err("unsupported schema_version values must be rejected");
    assert!(matches!(
        error,
        RegistryError::UnsupportedSchemaVersion {
            expected: 1,
            actual: 42,
        }
    ));
}
