#![cfg(not(target_arch = "wasm32"))]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::iter_on_single_items,
    clippy::missing_const_for_fn,
    clippy::option_if_let_else,
    clippy::redundant_clone,
    clippy::too_many_lines,
    clippy::unnested_or_patterns,
    reason = "pedantic, nursery, and perf lints acceptable in test helper code"
)]

mod common;

use cow_sdk_core::{Address, CowEnv, ProtocolOptions, SupportedChainId};
use cow_sdk_signing::{domain_separator, get_domain, order_typed_data};
use sha3::{Digest, Keccak256};

use common::{fixture_case, sample_order};

#[test]
fn domain_resolution_honors_default_env_staging_and_override_precedence() {
    let default_domain = get_domain(SupportedChainId::Mainnet, None).unwrap();
    let staging_domain = get_domain(
        SupportedChainId::Mainnet,
        Some(&ProtocolOptions::new().with_env(CowEnv::Staging)),
    )
    .unwrap();
    let override_address = Address::new("0x1111111111111111111111111111111111111111").unwrap();
    let override_domain = get_domain(
        SupportedChainId::Mainnet,
        Some(
            &ProtocolOptions::new()
                .with_env(CowEnv::Staging)
                .with_settlement_contract_override(
                    [(u64::from(SupportedChainId::Mainnet), override_address)]
                        .into_iter()
                        .collect(),
                ),
        ),
    )
    .unwrap();

    assert_eq!(
        default_domain.verifying_contract,
        Some(
            *Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41")
                .unwrap()
                .as_alloy()
        )
    );
    assert_eq!(
        staging_domain.verifying_contract,
        Some(
            *Address::new("0xf553d092b50bdcbddeD1A99aF2cA29FBE5E2CB13")
                .unwrap()
                .as_alloy()
        )
    );
    assert_eq!(
        override_domain.verifying_contract,
        Some(*override_address.as_alloy())
    );
}

#[test]
fn typed_data_domain_and_separator_match_fixture_contract() {
    let fields_case = fixture_case("signing-domain-separator-fields");
    let order = sample_order();
    let typed = order_typed_data(SupportedChainId::Mainnet, &order, None).unwrap();
    let separator = domain_separator(SupportedChainId::Mainnet, None).unwrap();
    let name = typed
        .domain
        .name
        .as_deref()
        .expect("cow EIP-712 domain always sets name");
    let version = typed
        .domain
        .version
        .as_deref()
        .expect("cow EIP-712 domain always sets version");
    let chain_id_u256 = typed
        .domain
        .chain_id
        .expect("cow EIP-712 domain always sets chainId");
    let chain_id = u64::try_from(chain_id_u256).expect("cow EIP-712 chainId fits in u64");
    let verifying_contract = typed
        .domain
        .verifying_contract
        .expect("cow EIP-712 domain always sets verifyingContract");
    let verifying_contract_hex = format!("0x{}", hex::encode(verifying_contract));
    let expected = independent_domain_separator(name, version, chain_id, &verifying_contract_hex);

    assert_eq!(
        typed.types["EIP712Domain"]
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>(),
        fields_case["expected"]["domain_fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|field| field.as_str().unwrap())
            .collect::<Vec<_>>()
    );
    assert_eq!(separator, expected);
    assert_eq!(separator.len(), 66);
}

fn independent_domain_separator(
    name: &str,
    version: &str,
    chain_id: u64,
    verifying_contract: &str,
) -> String {
    let mut encoded = Vec::with_capacity(32 * 5);
    encoded.extend_from_slice(&keccak256(
        "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
            .as_bytes(),
    ));
    encoded.extend_from_slice(&keccak256(name.as_bytes()));
    encoded.extend_from_slice(&keccak256(version.as_bytes()));

    let mut chain_word = [0u8; 32];
    chain_word[24..].copy_from_slice(&chain_id.to_be_bytes());
    encoded.extend_from_slice(&chain_word);

    let mut address_word = [0u8; 32];
    let address_bytes = hex::decode(verifying_contract.trim_start_matches("0x")).unwrap();
    address_word[12..].copy_from_slice(&address_bytes);
    encoded.extend_from_slice(&address_word);

    format!("0x{}", hex::encode(keccak256(encoded)))
}

// Hand-rolled `sha3::Keccak256` helper used by the assertions above.
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
