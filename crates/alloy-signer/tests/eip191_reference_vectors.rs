#![cfg(not(target_arch = "wasm32"))]

use std::str::FromStr;

use alloy_primitives::Signature as AlloySignature;
use cow_sdk_alloy_signer::LocalAlloySigner;
use cow_sdk_core::{Address, Signer, SupportedChainId};

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
const EXPECTED_LOWERCASE_ADDRESS: &str = "0x70997970c51812dc3a010c7d01b50e0d17dc79c8";
const EXPECTED_CHECKSUM_ADDRESS: &str = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";
const EXPECTED_MESSAGE_SIGNATURE: &str = "0x267c1300572586cc72a2780636139a843ce20866dcc515c62c02909f0bbf3ce71468a683b857347aced6470cd911828201eb0fe21e2ba3bcf14f903916407d101b";

#[tokio::test]
async fn eip191_message_signature_matches_reference_vector_and_recovers_address() {
    let signer = signer();
    let signature = signer.sign_message(b"hello cow").await.unwrap();
    let recovered = AlloySignature::from_str(&signature)
        .unwrap()
        .recover_address_from_msg(b"hello cow")
        .unwrap();

    assert_eq!(signature, EXPECTED_MESSAGE_SIGNATURE);
    assert_eq!(format!("{recovered:#x}"), EXPECTED_LOWERCASE_ADDRESS);
}

#[tokio::test]
async fn address_matches_lowercase_and_eip55_checksum_forms() {
    let address = signer().address().await.unwrap();

    assert_eq!(address, Address::new(EXPECTED_LOWERCASE_ADDRESS).unwrap());
    assert_eq!(address, Address::new(EXPECTED_CHECKSUM_ADDRESS).unwrap());
}

fn signer() -> LocalAlloySigner {
    LocalAlloySigner::builder()
        .private_key(TEST_KEY)
        .unwrap()
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .unwrap()
}
