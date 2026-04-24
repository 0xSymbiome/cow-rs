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

use cow_sdk_contracts::{Order as ContractsOrder, OrderUidParams, SigningScheme, hash_order};
use cow_sdk_core::{Address, Amount, SupportedChainId};
use cow_sdk_signing::{
    GeneratedOrderId, ORDER_PRIMARY_TYPE, SigningError, eip1271_signature_payload,
    generate_order_id, get_domain, order_typed_data, order_typed_data_payload, sign_order,
    sign_order_async, sign_order_with_scheme, sign_order_with_scheme_async,
};
use num_bigint::BigUint;
use sha3::{Digest, Keccak256};

use common::{MockSigner, fixture_case, sample_order};

#[test]
fn order_typed_data_matches_fixture_contract_and_consumer_shape() {
    let order = sample_order();
    let typed = order_typed_data(SupportedChainId::Mainnet, &order, None).unwrap();
    let payload = order_typed_data_payload(SupportedChainId::Mainnet, &order, None).unwrap();
    let fields_case = fixture_case("signing-eip712-order-fields");
    let typed_data_case = fixture_case("signing-typed-data-envelope");

    assert_eq!(typed.primary_type, ORDER_PRIMARY_TYPE);
    assert_eq!(payload.primary_type, ORDER_PRIMARY_TYPE);
    assert_eq!(payload.types, typed.types);
    assert_eq!(payload.message, serde_json::to_string(&order).unwrap());
    assert_eq!(
        typed.types["Order"]
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>(),
        fields_case["expected"]["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|field| field.as_str().unwrap())
            .collect::<Vec<_>>()
    );
    let actual_type_names = typed.types.keys().map(String::as_str).collect::<Vec<_>>();
    let expected_type_names = typed_data_case["expected"]["includes_types"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect::<Vec<_>>();
    for type_name in expected_type_names {
        assert!(actual_type_names.contains(&type_name));
    }
    assert_eq!(typed.message, order);
}

#[test]
fn sign_order_uses_typed_data_for_eip712_and_digest_for_ethsign() {
    let order = sample_order();
    let signer = MockSigner::new();

    let typed_result = sign_order(&order, SupportedChainId::Sepolia, &signer, None).unwrap();
    assert_eq!(typed_result.signing_scheme, SigningScheme::Eip712);
    assert_eq!(typed_result.signature, signer.typed_data_signature);
    assert_eq!(signer.calls.borrow().typed_data.len(), 1);
    assert!(signer.calls.borrow().messages.is_empty());

    let ethsign_result = sign_order_with_scheme(
        &order,
        SupportedChainId::Sepolia,
        &signer,
        SigningScheme::EthSign,
        None,
    )
    .unwrap();
    assert_eq!(ethsign_result.signing_scheme, SigningScheme::EthSign);
    assert_eq!(ethsign_result.signature, signer.message_signature);

    let expected_digest = hash_order(
        &get_domain(SupportedChainId::Sepolia, None).unwrap(),
        &contracts_order(&order),
    )
    .unwrap();

    assert_eq!(
        format!("0x{}", hex::encode(&signer.calls.borrow().messages[0])),
        expected_digest.as_str()
    );
}

#[tokio::test]
async fn async_sign_order_paths_match_sync_signing_behavior() {
    let order = sample_order();
    let signer = MockSigner::new();

    let typed_result = sign_order_async(&order, SupportedChainId::Sepolia, &signer, None)
        .await
        .unwrap();
    assert_eq!(typed_result.signing_scheme, SigningScheme::Eip712);
    assert_eq!(typed_result.signature, signer.typed_data_signature);

    let ethsign_result = sign_order_with_scheme_async(
        &order,
        SupportedChainId::Sepolia,
        &signer,
        SigningScheme::EthSign,
        None,
    )
    .await
    .unwrap();
    assert_eq!(ethsign_result.signing_scheme, SigningScheme::EthSign);
    assert_eq!(ethsign_result.signature, signer.message_signature);
}

#[test]
fn unsupported_local_signer_modes_fail_with_typed_errors() {
    let order = sample_order();
    let signer = MockSigner::new();

    for scheme in [SigningScheme::Eip1271, SigningScheme::PreSign] {
        let error =
            sign_order_with_scheme(&order, SupportedChainId::Mainnet, &signer, scheme, None)
                .unwrap_err();

        assert!(matches!(
            error,
            SigningError::UnsupportedSignerGeneratedScheme { scheme: got }
                if got == scheme
        ));
    }

    assert!(signer.calls.borrow().typed_data.is_empty());
    assert!(signer.calls.borrow().messages.is_empty());
}

#[test]
fn generate_order_id_reuses_contract_hashing_and_uid_packing() {
    let order = sample_order();
    let owner = Address::new("0x1111111111111111111111111111111111111111").unwrap();

    let generated = generate_order_id(SupportedChainId::Sepolia, &order, &owner, None).unwrap();
    let expected_digest = hash_order(
        &get_domain(SupportedChainId::Sepolia, None).unwrap(),
        &contracts_order(&order),
    )
    .unwrap();
    let expected_uid = cow_sdk_contracts::pack_order_uid_params(&OrderUidParams::new(
        expected_digest.clone(),
        owner.clone(),
        order.valid_to,
    ))
    .unwrap();

    assert_eq!(
        generated,
        GeneratedOrderId {
            order_id: expected_uid,
            order_digest: expected_digest,
        }
    );
}

#[test]
fn eip1271_signature_payload_matches_the_manual_contract_encoding() {
    let mut order = sample_order();
    order.sell_amount = Amount::new(format!("0x{}", "ff".repeat(32))).unwrap();
    order.buy_amount = Amount::new("0x01").unwrap();
    order.fee_amount = Amount::new("0x02").unwrap();
    order.app_data = cow_sdk_core::AppDataHex::new(format!("0x{}", "11".repeat(32))).unwrap();

    let signature = format!("0x{}1b", "aa".repeat(64));
    let payload = eip1271_signature_payload(&order, &signature).unwrap();
    let signature_bytes = hex::decode(signature.trim_start_matches("0x")).unwrap();

    let mut expected = Vec::with_capacity(32 * 15 + padded_len_manual(signature_bytes.len()));
    expected.extend_from_slice(&encode_address_word(order.sell_token.as_str()));
    expected.extend_from_slice(&encode_address_word(order.buy_token.as_str()));
    expected.extend_from_slice(&encode_address_word(order.receiver.as_str()));
    expected.extend_from_slice(&encode_u256_word(&order.sell_amount.to_string()));
    expected.extend_from_slice(&encode_u256_word(&order.buy_amount.to_string()));
    expected.extend_from_slice(&encode_u32_word(order.valid_to));
    expected.extend_from_slice(&encode_bytes32_word(order.app_data.as_str()));
    expected.extend_from_slice(&encode_u256_word(&order.fee_amount.to_string()));
    expected.extend_from_slice(&keccak_word("sell"));
    expected.extend_from_slice(&encode_bool_word(order.partially_fillable));
    expected.extend_from_slice(&keccak_word("erc20"));
    expected.extend_from_slice(&keccak_word("erc20"));
    expected.extend_from_slice(&encode_usize_word(32 * 13));
    expected.extend_from_slice(&encode_usize_word(signature_bytes.len()));
    expected.extend_from_slice(&signature_bytes);
    expected.extend(std::iter::repeat_n(
        0u8,
        padded_len_manual(signature_bytes.len()) - signature_bytes.len(),
    ));

    assert_eq!(payload, format!("0x{}", hex::encode(expected)));
}

#[test]
fn eip1271_signature_payload_keeps_full_bytes32_app_data_and_exact_word_padding() {
    let mut order = sample_order();
    order.app_data = cow_sdk_core::AppDataHex::new(format!("0x{}", "ab".repeat(32))).unwrap();

    let signature = format!("0x{}1b", "cd".repeat(64));
    let payload = eip1271_signature_payload(&order, &signature).unwrap();
    let encoded = hex::decode(payload.trim_start_matches("0x")).unwrap();
    assert_eq!(encoded.len(), 32 * 17);

    let app_data_word_offset = 32 * 6;
    assert_eq!(
        &encoded[app_data_word_offset..app_data_word_offset + 32],
        &parse_hex_word(order.app_data.as_str(), 32)
    );

    let dynamic_length_offset = 32 * 13;
    assert_eq!(
        &encoded[dynamic_length_offset..dynamic_length_offset + 32],
        &encode_usize_word(65)
    );
    assert_eq!(
        &encoded[dynamic_length_offset + 32..dynamic_length_offset + 32 + 65],
        &hex::decode(signature.trim_start_matches("0x")).unwrap()
    );
    assert!(
        encoded[dynamic_length_offset + 32 + 65..]
            .iter()
            .all(|byte| *byte == 0)
    );
}

fn contracts_order(order: &cow_sdk_core::UnsignedOrder) -> ContractsOrder {
    ContractsOrder::new(
        order.sell_token.clone(),
        order.buy_token.clone(),
        Some(order.receiver.clone()),
        order.sell_amount.clone(),
        order.buy_amount.clone(),
        order.valid_to,
        order.app_data.clone(),
        order.fee_amount.clone(),
        order.kind,
        order.partially_fillable,
        Some(order.sell_token_balance),
        Some(order.buy_token_balance),
    )
}

fn parse_hex_word(value: &str, expected_len: usize) -> Vec<u8> {
    let bytes = hex::decode(value.trim_start_matches("0x")).unwrap();
    assert_eq!(bytes.len(), expected_len);
    bytes
}

fn encode_address_word(value: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[12..].copy_from_slice(&parse_hex_word(value, 20));
    out
}

fn encode_bytes32_word(value: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    out.copy_from_slice(&parse_hex_word(value, 32));
    out
}

fn encode_u32_word(value: u32) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[28..].copy_from_slice(&value.to_be_bytes());
    out
}

fn encode_u256_word(value: &str) -> [u8; 32] {
    let parsed = if let Some(stripped) = value.strip_prefix("0x") {
        BigUint::parse_bytes(stripped.as_bytes(), 16)
    } else {
        BigUint::parse_bytes(value.as_bytes(), 10)
    }
    .unwrap();
    let bytes = parsed.to_bytes_be();
    assert!(bytes.len() <= 32);

    let mut out = [0u8; 32];
    out[32 - bytes.len()..].copy_from_slice(&bytes);
    out
}

fn encode_usize_word(value: usize) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[24..].copy_from_slice(&(value as u64).to_be_bytes());
    out
}

fn encode_bool_word(value: bool) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[31] = u8::from(value);
    out
}

fn keccak_word(value: &str) -> [u8; 32] {
    let digest = Keccak256::digest(value.as_bytes());
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

fn padded_len_manual(len: usize) -> usize {
    if len == 0 {
        0
    } else {
        ((len - 1) / 32 + 1) * 32
    }
}
