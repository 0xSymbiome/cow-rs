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

use cow_sdk_contracts::{OrderUidParams, SigningScheme, hash_order};
use cow_sdk_core::{Address, Amount, SupportedChainId};

use cow_sdk_signing::{
    GeneratedOrderId, ORDER_PRIMARY_TYPE, SigningError, domain, eip1271_signature_payload,
    generate_order_id, order_typed_data, order_typed_data_payload, sign_order,
    sign_order_with_scheme,
};

use cow_sdk_test_utils::eip712::{
    encode_address_word, encode_bool_word, encode_bytes32_word, encode_u32_word, encode_u256_word,
    encode_usize_word, keccak_word,
};
use cow_sdk_test_utils::mocks::RecordingSigner;

use common::sample_order;

#[test]
fn order_typed_data_matches_fixture_contract_and_consumer_shape() {
    let order = sample_order();
    let typed = order_typed_data(SupportedChainId::Mainnet, &order, None).unwrap();
    let payload = order_typed_data_payload(SupportedChainId::Mainnet, &order, None).unwrap();
    assert_eq!(typed.primary_type, ORDER_PRIMARY_TYPE);
    assert_eq!(payload.primary_type, ORDER_PRIMARY_TYPE);
    assert_eq!(payload.types, typed.types);
    assert_eq!(payload.message, serde_json::to_string(&order).unwrap());
    // Canonical CoW `Order` EIP-712 field order (formerly pinned in the retired
    // signing parity fixture).
    assert_eq!(
        typed.types["Order"]
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>(),
        [
            "sellToken",
            "buyToken",
            "receiver",
            "sellAmount",
            "buyAmount",
            "validTo",
            "appData",
            "feeAmount",
            "kind",
            "partiallyFillable",
            "sellTokenBalance",
            "buyTokenBalance",
        ]
    );
    let actual_type_names = typed.types.keys().map(String::as_str).collect::<Vec<_>>();
    for type_name in ["Order", "EIP712Domain"] {
        assert!(actual_type_names.contains(&type_name));
    }
    assert_eq!(typed.message, order);
}

#[tokio::test]
async fn sign_order_uses_typed_data_for_eip712_and_digest_for_ethsign() {
    let order = sample_order();
    let signer = RecordingSigner::new();

    let typed_result = sign_order(&order, SupportedChainId::Sepolia, &signer, None)
        .await
        .unwrap();
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
    .await
    .unwrap();
    assert_eq!(ethsign_result.signing_scheme, SigningScheme::EthSign);
    assert_eq!(ethsign_result.signature, signer.message_signature);

    let expected_digest =
        hash_order(&domain(SupportedChainId::Sepolia, None).unwrap(), &order).unwrap();

    assert_eq!(
        format!(
            "0x{}",
            alloy_primitives::hex::encode(&signer.calls.borrow().messages[0])
        ),
        expected_digest.to_hex_string()
    );
}

#[tokio::test]
async fn eth_sign_routes_raw_32_byte_digest_to_sign_message() {
    let order = sample_order();
    let signer = RecordingSigner::new();
    let expected_digest =
        hash_order(&domain(SupportedChainId::Sepolia, None).unwrap(), &order).unwrap();

    let result = sign_order_with_scheme(
        &order,
        SupportedChainId::Sepolia,
        &signer,
        SigningScheme::EthSign,
        None,
    )
    .await
    .unwrap();

    assert_eq!(result.signing_scheme, SigningScheme::EthSign);
    let captured = signer.calls.borrow().messages[0].clone();
    assert_eq!(captured.len(), 32);
    assert_eq!(
        format!("0x{}", alloy_primitives::hex::encode(&captured)),
        expected_digest.to_hex_string()
    );
    assert!(!captured.starts_with(b"\x19Ethereum Signed Message:\n32"));
}

#[tokio::test]
async fn sign_order_routes_typed_data_fields_to_signer() {
    let order = sample_order();
    let signer = RecordingSigner::new();

    let typed_result = sign_order(&order, SupportedChainId::Sepolia, &signer, None)
        .await
        .unwrap();
    assert_eq!(typed_result.signing_scheme, SigningScheme::Eip712);
    assert_eq!(typed_result.signature, signer.typed_data_signature);
    {
        let calls = signer.calls.borrow();
        assert_eq!(calls.typed_data.len(), 1);
        assert!(calls.messages.is_empty());
        assert_eq!(calls.typed_data[0].domain.chain_id, 11_155_111);
        assert_eq!(
            calls.typed_data[0].message,
            serde_json::to_string(&order).unwrap()
        );
        assert!(
            calls.typed_data[0]
                .primary_type_fields()
                .unwrap_or_default()
                .iter()
                .any(|field| field.name == "sellToken" && field.kind == "address"),
            "EIP-712 signing must route the order typed-data payload to sign_typed_data_payload",
        );
    }

    let ethsign_result = sign_order_with_scheme(
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
    assert_eq!(signer.calls.borrow().typed_data.len(), 1);
    assert_eq!(signer.calls.borrow().messages.len(), 1);
}

#[tokio::test]
async fn unsupported_local_signer_modes_fail_with_typed_errors() {
    let order = sample_order();
    let signer = RecordingSigner::new();

    for scheme in [SigningScheme::Eip1271, SigningScheme::PreSign] {
        let error =
            sign_order_with_scheme(&order, SupportedChainId::Mainnet, &signer, scheme, None)
                .await
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
    let expected_digest =
        hash_order(&domain(SupportedChainId::Sepolia, None).unwrap(), &order).unwrap();
    let expected_uid = cow_sdk_contracts::pack_order_uid_params(&OrderUidParams::new(
        expected_digest,
        owner,
        order.valid_to,
    ));

    assert_eq!(
        generated,
        GeneratedOrderId::new(expected_uid, expected_digest)
    );
}

#[test]
fn eip1271_signature_payload_matches_the_manual_contract_encoding() {
    let mut order = sample_order();
    order.sell_amount = Amount::new(format!("0x{}", "ff".repeat(32))).unwrap();
    order.buy_amount = Amount::new("0x01").unwrap();
    order.fee_amount = Amount::new("0x02").unwrap();
    order.app_data = cow_sdk_core::AppDataHash::new(format!("0x{}", "11".repeat(32))).unwrap();

    let signature = format!("0x{}1b", "aa".repeat(64));
    let payload = eip1271_signature_payload(&order, &signature).unwrap();
    let signature_bytes =
        alloy_primitives::hex::decode(signature.trim_start_matches("0x")).unwrap();

    let mut expected = Vec::with_capacity(32 * 15 + padded_len_manual(signature_bytes.len()));
    expected.extend_from_slice(&encode_address_word(&order.sell_token.to_hex_string()));
    expected.extend_from_slice(&encode_address_word(&order.buy_token.to_hex_string()));
    expected.extend_from_slice(&encode_address_word(&order.receiver.to_hex_string()));
    expected.extend_from_slice(&encode_u256_word(&order.sell_amount.to_string()));
    expected.extend_from_slice(&encode_u256_word(&order.buy_amount.to_string()));
    expected.extend_from_slice(&encode_u32_word(order.valid_to));
    expected.extend_from_slice(&encode_bytes32_word(&order.app_data.to_hex_string()));
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

    assert_eq!(
        payload,
        format!("0x{}", alloy_primitives::hex::encode(expected))
    );
}

#[test]
fn eip1271_signature_payload_keeps_full_bytes32_app_data_and_exact_word_padding() {
    let mut order = sample_order();
    order.app_data = cow_sdk_core::AppDataHash::new(format!("0x{}", "ab".repeat(32))).unwrap();

    let signature = format!("0x{}1b", "cd".repeat(64));
    let payload = eip1271_signature_payload(&order, &signature).unwrap();
    let encoded = alloy_primitives::hex::decode(payload.trim_start_matches("0x")).unwrap();
    assert_eq!(encoded.len(), 32 * 17);

    let app_data_word_offset = 32 * 6;
    assert_eq!(
        &encoded[app_data_word_offset..app_data_word_offset + 32],
        &parse_hex_word(&order.app_data.to_hex_string(), 32)
    );

    let dynamic_length_offset = 32 * 13;
    assert_eq!(
        &encoded[dynamic_length_offset..dynamic_length_offset + 32],
        &encode_usize_word(65)
    );
    assert_eq!(
        &encoded[dynamic_length_offset + 32..dynamic_length_offset + 32 + 65],
        &alloy_primitives::hex::decode(signature.trim_start_matches("0x")).unwrap()
    );
    assert!(
        encoded[dynamic_length_offset + 32 + 65..]
            .iter()
            .all(|byte| *byte == 0)
    );
}

fn parse_hex_word(value: &str, expected_len: usize) -> Vec<u8> {
    let bytes = alloy_primitives::hex::decode(value.trim_start_matches("0x")).unwrap();
    assert_eq!(bytes.len(), expected_len);
    bytes
}

fn padded_len_manual(len: usize) -> usize {
    if len == 0 {
        0
    } else {
        ((len - 1) / 32 + 1) * 32
    }
}
