mod common;

use alloy_primitives::{Address as AlloyAddress, B256};
use sha3::{Digest, Keccak256};

use cow_sdk_contracts::{
    ContractId, DEPLOYER_CONTRACT, Registry, SALT, deployment_address_hash_input,
    deployment_for_chain, deterministic_deployment_address,
};
use cow_sdk_core::{CowEnv, SupportedChainId};

use common::fixture_case;

// Hand-rolled `sha3::Keccak256` helper used by the assertions below.
// Crate code routes through `alloy_primitives::keccak256` per ADR 0052;
// this helper deliberately runs `sha3::Keccak256` directly so the parity
// check compares the crate output against an independent keccak
// implementation.
fn keccak256(bytes: impl AsRef<[u8]>) -> [u8; 32] {
    let digest = Keccak256::digest(bytes.as_ref());
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

#[test]
fn deployment_constants_and_create2_address_match_fixture_contract() {
    let fixture = fixture_case("contracts-deployment-constants");
    let expected_salt: B256 = fixture["expected"]["salt"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let expected_deployer: AlloyAddress = fixture["expected"]["deployer_contract"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(SALT, expected_salt);
    assert_eq!(DEPLOYER_CONTRACT, expected_deployer);

    let bytecode = "0x608060405234801561001057600080fd5b506040516102c73803806102c78339";
    let args = vec![
        "0x9008D19f58AAbD9eD0D60971565AA8510560ab41".to_owned(),
        "0x1234567890123456789012345678901234567890".to_owned(),
    ];

    let actual = deterministic_deployment_address(bytecode, &args).unwrap();

    let mut init_code = hex::decode(bytecode.trim_start_matches("0x")).unwrap();
    for arg in &args {
        init_code.extend_from_slice(&hex::decode(arg.trim_start_matches("0x")).unwrap());
    }
    let mut payload = Vec::with_capacity(85);
    payload.push(0xff);
    payload.extend_from_slice(DEPLOYER_CONTRACT.as_slice());
    payload.extend_from_slice(SALT.as_slice());
    payload.extend_from_slice(&keccak256(init_code));
    let hash = keccak256(payload);
    let expected = format!("0x{}", hex::encode(&hash[12..]));

    assert_eq!(actual.to_hex_string(), expected);
}

#[test]
fn deployment_for_chain_uses_core_protocol_addresses() {
    let mainnet = deployment_for_chain(1).unwrap();
    let registry = Registry::default();
    assert_eq!(
        mainnet.settlement,
        registry
            .address(
                ContractId::Settlement,
                SupportedChainId::Mainnet,
                CowEnv::Prod
            )
            .expect("canonical settlement address is registered on mainnet")
    );
    assert_eq!(
        mainnet.vault_relayer,
        registry
            .address(
                ContractId::VaultRelayer,
                SupportedChainId::Mainnet,
                CowEnv::Prod
            )
            .expect("canonical vault-relayer address is registered on mainnet")
    );
    assert_eq!(
        mainnet.eth_flow,
        registry
            .address(ContractId::EthFlow, SupportedChainId::Mainnet, CowEnv::Prod)
            .expect("canonical EthFlow address is registered on mainnet")
    );

    assert!(deployment_for_chain(999_999).is_err());
}

#[test]
fn registry_canonical_addresses_are_bound_to_the_reviewed_create2_salt_contract() {
    let fixture = fixture_case("contracts-deployment-constants");
    let expected_salt: B256 = fixture["expected"]["salt"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let expected_deployer: AlloyAddress = fixture["expected"]["deployer_contract"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(SALT, expected_salt);
    assert_eq!(DEPLOYER_CONTRACT, expected_deployer);

    let registry = Registry::default();
    let canonical = [
        (
            ContractId::Settlement,
            "0x9008d19f58aabd9ed0d60971565aa8510560ab41",
        ),
        (
            ContractId::VaultRelayer,
            "0xc92e8bdf79f0507f65a392b0ab4667716bfe0110",
        ),
    ];

    for (contract_id, expected_address) in canonical {
        for chain in SupportedChainId::ALL {
            let address = registry
                .address(contract_id, chain, CowEnv::Prod)
                .unwrap_or_else(|| panic!("{contract_id:?} prod address is missing for {chain:?}"));
            assert_eq!(
                address.to_hex_string(),
                expected_address,
                "{contract_id:?} must keep the deterministic CREATE2 deployment address across prod chains",
            );
        }
    }

    let bytecode = "0x6001600055";
    let args = vec!["0x1234".to_owned()];
    let init_hash = deployment_address_hash_input(bytecode, &args).unwrap();
    let mut payload = Vec::with_capacity(85);
    payload.push(0xff);
    payload.extend_from_slice(DEPLOYER_CONTRACT.as_slice());
    payload.extend_from_slice(SALT.as_slice());
    payload.extend_from_slice(&init_hash);
    let expected = keccak256(payload);

    assert_eq!(
        deterministic_deployment_address(bytecode, &args)
            .unwrap()
            .to_hex_string(),
        format!("0x{}", hex::encode(&expected[12..]))
    );
}
