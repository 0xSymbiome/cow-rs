mod common;

use sha3::{Digest, Keccak256};

use cow_sdk_contracts::{
    DEPLOYER_CONTRACT, SALT, deployment_for_chain, deterministic_deployment_address,
};
use cow_sdk_core::{
    CowEnv, SupportedChainId, eth_flow_contract_address, settlement_contract_address,
    vault_relayer_address,
};

use common::fixture_case;

fn keccak256(bytes: impl AsRef<[u8]>) -> [u8; 32] {
    let digest = Keccak256::digest(bytes.as_ref());
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

#[test]
fn deployment_constants_and_create2_address_match_fixture_contract() {
    let fixture = fixture_case("contracts-deployment-constants");
    assert_eq!(SALT, fixture["expected"]["salt"].as_str().unwrap());
    assert_eq!(
        DEPLOYER_CONTRACT,
        fixture["expected"]["deployer_contract"].as_str().unwrap()
    );

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
    payload.extend_from_slice(&hex::decode(DEPLOYER_CONTRACT.trim_start_matches("0x")).unwrap());
    payload.extend_from_slice(&hex::decode(SALT.trim_start_matches("0x")).unwrap());
    payload.extend_from_slice(&keccak256(init_code));
    let hash = keccak256(payload);
    let expected = format!("0x{}", hex::encode(&hash[12..]));

    assert_eq!(actual.as_str(), expected);
}

#[test]
fn deployment_for_chain_uses_core_protocol_addresses() {
    let mainnet = deployment_for_chain(1).unwrap();
    assert_eq!(
        mainnet.settlement,
        settlement_contract_address(SupportedChainId::Mainnet, CowEnv::Prod)
    );
    assert_eq!(
        mainnet.vault_relayer,
        vault_relayer_address(SupportedChainId::Mainnet, CowEnv::Prod)
    );
    assert_eq!(
        mainnet.eth_flow,
        eth_flow_contract_address(SupportedChainId::Mainnet, CowEnv::Prod)
    );

    assert!(deployment_for_chain(999_999).is_err());
}
