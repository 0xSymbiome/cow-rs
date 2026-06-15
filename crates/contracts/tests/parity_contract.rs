//! Fixture-driven parity contract for `cow-sdk-contracts`.
//!
//! Loads `parity/fixtures/contracts.json` (schema version 1) at compile time,
//! iterates every documented case, and asserts the Rust helpers produce the
//! pinned upstream values. The helpers exercised are:
//!
//! * [`ORDER_TYPE_FIELDS`], the `GPv2Order` type hash, [`CANCELLATIONS_TYPE_FIELDS`],
//!   [`ORDER_UID_LENGTH`] — canonical EIP-712 and UID layout constants.
//! * [`extract_order_uid_params`] — UID length validation through
//!   [`ContractsError::InvalidOrderUidLength`].
//! * [`SellTokenSource`] / [`BuyTokenDestination`] — split balance enums; the
//!   parity fixture pins that `BuyTokenDestination` has no `external` variant
//!   so quote-derived and direct trading orders cannot rewrite the buy-side
//!   destination silently.
//! * [`SigningScheme`] — signature scheme discriminants. The EIP-1271
//!   success magic value is the typed selector emitted by the `sol!`-
//!   generated [`IERC1271::isValidSignatureCall`] binding.
//! * [`normalize_interaction`] — interaction defaulting rule.
//!
//! Failure messages carry the fixture case id so a reviewer looking at a
//! broken CI run sees the exact upstream vector that diverged.

use alloy_sol_types::{
    SolCall,
    private::{Address as SolAddress, Bytes as SolBytes, U256},
};
use cow_sdk_contracts::settlement::IGPv2Settlement;
use cow_sdk_contracts::{
    CANCELLATIONS_TYPE_FIELDS, EthFlowOrderData, IERC20, IERC1271, InteractionLike,
    ORDER_TYPE_FIELDS, ORDER_UID_LENGTH, SigningScheme, encode_create_order_calldata,
    encode_invalidate_order_calldata, normalize_interaction,
};
use cow_sdk_core::{
    Address, Amount, AppDataHash, BuyTokenDestination, OrderDigest, OrderUid, SellTokenSource,
};
use serde_json::Value;

// The settlement calldata cases encode through the shipped
// `cow_sdk_contracts::settlement::IGPv2Settlement` binding (imported above) and
// assert the bytes are identical to the upstream-pinned fixture vectors. Driving
// the shipped binding — rather than a test-local re-declaration — is what makes
// this gate guard the encoder the SDK actually ships: a field-order or selector
// drift in the binding now fails here instead of passing against a decoy.
const FIXTURE: &str = include_str!("../../../parity/fixtures/contracts.json");

#[test]
fn parity_fixture_cases_hold() {
    let fixture: Value = serde_json::from_str(FIXTURE).expect("fixture must parse as JSON");

    assert_eq!(
        fixture["surface"].as_str(),
        Some("contracts"),
        "contracts fixture must carry the contracts surface label"
    );

    let cases = fixture["cases"]
        .as_array()
        .expect("contracts fixture must expose a cases array");

    for case in cases {
        let id = case["id"]
            .as_str()
            .expect("every fixture case must carry a string id");
        let expected = &case["expected"];

        match id {
            "contracts-order-type-fields" => assert_order_type_fields(id, expected),
            "contracts-order-type-hash" => assert_order_type_hash(id, expected),
            "contracts-cancellation-type-fields" => assert_cancellation_type_fields(id, expected),
            "contracts-order-uid-length" => assert_order_uid_length(id, expected),
            "contracts-extract-order-uid-invalid-length" => {
                assert_extract_order_uid_invalid_length(id, expected);
            }
            "contracts-buy-balance-domain" => {
                assert_buy_balance_domain(id, expected);
            }
            "contracts-signing-scheme-discriminants" => {
                assert_signing_scheme_discriminants(id, expected);
            }
            "contracts-eip1271-magic-value" => assert_eip1271_magic_value(id, expected),
            "contracts-interaction-defaults" => assert_interaction_defaults(id, expected),
            "contracts-settlement-invalidate-order-calldata" => {
                assert_settlement_invalidate_order_calldata(id, expected);
            }
            "contracts-settlement-set-presignature-calldata" => {
                assert_settlement_set_presignature_calldata(id, expected);
            }
            "contracts-settlement-free-filled-amount-storage-calldata" => {
                assert_settlement_free_filled_amount_storage_calldata(id, expected);
            }
            "contracts-settlement-free-presignature-storage-calldata" => {
                assert_settlement_free_presignature_storage_calldata(id, expected);
            }
            "contracts-ethflow-create-order-calldata" => {
                assert_ethflow_create_order_calldata(id, expected);
            }
            "contracts-ethflow-invalidate-order-calldata" => {
                assert_ethflow_invalidate_order_calldata(id, expected);
            }
            "contracts-erc20-approve-calldata" => {
                assert_erc20_approve_calldata(id, expected);
            }
            "contracts-erc20-transfer-from-calldata" => {
                assert_erc20_transfer_from_calldata(id, expected);
            }
            other => panic!("unknown contracts fixture case id: {other}"),
        }
    }
}

fn assert_order_type_fields(id: &str, expected: &Value) {
    let expected_fields: Vec<&str> = expected["fields"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.fields must be an array"))
        .iter()
        .map(|field| {
            field
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: expected.fields entries must be strings"))
        })
        .collect();

    let actual_fields: Vec<&str> = ORDER_TYPE_FIELDS.iter().map(|field| field.name).collect();

    assert_eq!(
        actual_fields, expected_fields,
        "case {id}: ORDER_TYPE_FIELDS names must match the pinned contracts field order",
    );
}

fn assert_order_type_hash(id: &str, expected: &Value) {
    use cow_sdk_contracts::order_eip712_type_hash;
    let expected_hash = expected["hash"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.hash must be a string"));
    let actual_hash = order_eip712_type_hash().to_hex_string();
    assert_eq!(
        actual_hash, expected_hash,
        "case {id}: order EIP-712 type hash must equal the pinned contracts-ts constant",
    );
}

fn assert_cancellation_type_fields(id: &str, expected: &Value) {
    let expected_fields: Vec<&str> = expected["fields"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.fields must be an array"))
        .iter()
        .map(|field| {
            field
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: expected.fields entries must be strings"))
        })
        .collect();

    let actual_fields: Vec<&str> = CANCELLATIONS_TYPE_FIELDS
        .iter()
        .map(|field| field.name)
        .collect();

    assert_eq!(
        actual_fields, expected_fields,
        "case {id}: CANCELLATIONS_TYPE_FIELDS must expose the single orderUids field",
    );
}

fn assert_order_uid_length(id: &str, expected: &Value) {
    let expected_bytes = expected["bytes"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {id}: expected.bytes must be a u64"));
    let expected_hex = expected["hex_chars"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {id}: expected.hex_chars must be a u64"));

    assert_eq!(
        ORDER_UID_LENGTH as u64, expected_bytes,
        "case {id}: ORDER_UID_LENGTH byte length must match the fixture",
    );
    assert_eq!(
        (ORDER_UID_LENGTH * 2) as u64,
        expected_hex,
        "case {id}: ORDER_UID_LENGTH hex-char length must be twice the byte length",
    );
}

fn assert_extract_order_uid_invalid_length(id: &str, expected: &Value) {
    assert_eq!(
        expected["must_reject"].as_bool(),
        Some(true),
        "case {id}: expected.must_reject must be true",
    );
    let _ = expected["error_contains"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.error_contains must be a string"));

    // The Rust boundary rejects malformed UID length at `OrderUid::new`, which
    // runs before `extract_order_uid_params`. The typed
    // `ValidationError::InvalidHexLength` names the `order_uid` field and the
    // required hex-character count, mirroring the "invalid order UID length"
    // rejection posture expressed in the upstream TypeScript fixture.
    let rejection = OrderUid::new("0x0000000000000000000000000000000000000000");
    let error = rejection
        .expect_err("case contracts-extract-order-uid-invalid-length must reject short UIDs");
    let message = error.to_string();
    assert!(
        message.contains("order_uid") && message.contains("hex"),
        "case {id}: rejection message {message:?} must name the order_uid hex-length invariant",
    );
}

fn assert_buy_balance_domain(id: &str, expected: &Value) {
    let expected_buy = expected["buy_token_destination_variants"]
        .as_array()
        .unwrap_or_else(|| {
            panic!("case {id}: expected.buy_token_destination_variants must be an array")
        })
        .iter()
        .map(|entry| {
            entry.as_str().unwrap_or_else(|| {
                panic!("case {id}: buy_token_destination_variants entries must be strings")
            })
        })
        .collect::<Vec<_>>();
    let expected_sell = expected["sell_token_source_variants"]
        .as_array()
        .unwrap_or_else(|| {
            panic!("case {id}: expected.sell_token_source_variants must be an array")
        })
        .iter()
        .map(|entry| {
            entry.as_str().unwrap_or_else(|| {
                panic!("case {id}: sell_token_source_variants entries must be strings")
            })
        })
        .collect::<Vec<_>>();

    let buy_variants = [BuyTokenDestination::Erc20, BuyTokenDestination::Internal]
        .into_iter()
        .map(|variant| serde_json::to_value(variant).expect("variant serialization"))
        .map(|value| {
            value
                .as_str()
                .expect("BuyTokenDestination must serialize to a snake_case string")
                .to_owned()
        })
        .collect::<Vec<_>>();
    let sell_variants = [
        SellTokenSource::Erc20,
        SellTokenSource::External,
        SellTokenSource::Internal,
    ]
    .into_iter()
    .map(|variant| serde_json::to_value(variant).expect("variant serialization"))
    .map(|value| {
        value
            .as_str()
            .expect("SellTokenSource must serialize to a snake_case string")
            .to_owned()
    })
    .collect::<Vec<_>>();

    assert_eq!(
        buy_variants, expected_buy,
        "case {id}: BuyTokenDestination must expose exactly the services buy-side variant set",
    );
    assert_eq!(
        sell_variants, expected_sell,
        "case {id}: SellTokenSource must expose exactly the services sell-side variant set",
    );
}

fn assert_signing_scheme_discriminants(id: &str, expected: &Value) {
    let expected_eip712 = expected["EIP712"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {id}: expected.EIP712 must be a u64"));
    let expected_ethsign = expected["ETHSIGN"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {id}: expected.ETHSIGN must be a u64"));
    let expected_eip1271 = expected["EIP1271"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {id}: expected.EIP1271 must be a u64"));
    let expected_presign = expected["PRESIGN"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {id}: expected.PRESIGN must be a u64"));

    assert_eq!(
        u64::from(SigningScheme::Eip712.as_u8()),
        expected_eip712,
        "case {id}: SigningScheme::Eip712 discriminant must match the fixture",
    );
    assert_eq!(
        u64::from(SigningScheme::EthSign.as_u8()),
        expected_ethsign,
        "case {id}: SigningScheme::EthSign discriminant must match the fixture",
    );
    assert_eq!(
        u64::from(SigningScheme::Eip1271.as_u8()),
        expected_eip1271,
        "case {id}: SigningScheme::Eip1271 discriminant must match the fixture",
    );
    assert_eq!(
        u64::from(SigningScheme::PreSign.as_u8()),
        expected_presign,
        "case {id}: SigningScheme::PreSign discriminant must match the fixture",
    );
}

fn assert_eip1271_magic_value(id: &str, expected: &Value) {
    let magic = expected["magic_value"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.magic_value must be a string"));
    // Route through the `sol!`-emitted selector on
    // `IERC1271::isValidSignatureCall` so the parity oracle compares the
    // typed-binding byte form against the upstream-documented hex
    // string. The selector is a `[u8; 4]` const; rendering it as
    // `0x{hex}` matches the fixture's wire representation.
    let actual = format!(
        "0x{}",
        alloy_primitives::hex::encode(<IERC1271::isValidSignatureCall as SolCall>::SELECTOR)
    );
    assert_eq!(
        actual.as_str(),
        magic,
        "case {id}: IERC1271::isValidSignatureCall::SELECTOR must equal the standard 0x1626ba7e",
    );
}

fn assert_interaction_defaults(id: &str, expected: &Value) {
    let expected_value = expected["value"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.value must be a string"));
    let expected_call_data = expected["call_data"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.call_data must be a string"));

    let normalized = normalize_interaction(&InteractionLike::new(
        Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        None,
        None,
    ));

    assert_eq!(
        normalized.value.to_string(),
        expected_value,
        "case {id}: normalize_interaction must default missing value to zero",
    );
    assert_eq!(
        normalized.call_data.len(),
        0,
        "case {id}: normalize_interaction must default missing call_data to an empty payload",
    );
    assert_eq!(
        expected_call_data, "0x",
        "case {id}: fixture expects the hex-empty payload marker 0x",
    );
}

fn sample_order_uid() -> OrderUid {
    // 56 byte fixture UID: 32-byte digest | 20-byte owner | 4-byte valid_to.
    let digest =
        OrderDigest::new("0x1111111111111111111111111111111111111111111111111111111111111111")
            .unwrap();
    let owner = Address::new("0x2222222222222222222222222222222222222222").unwrap();

    cow_sdk_contracts::pack_order_uid_params(&cow_sdk_contracts::OrderUidParams::new(
        digest,
        owner,
        0x1234_5678,
    ))
}

fn assert_calldata_hex(id: &str, actual_bytes: &[u8], expected_hex: &str) {
    let actual_hex = format!("0x{}", alloy_primitives::hex::encode(actual_bytes));
    assert_eq!(
        actual_hex.as_str(),
        expected_hex,
        "case {id}: encoded call-data must match the pinned byte-identity fixture",
    );
}

fn parse_address_bytes(address: &Address) -> [u8; 20] {
    let hex_bytes = alloy_primitives::hex::decode(
        address
            .to_hex_string()
            .strip_prefix("0x")
            .expect("Address must carry a 0x prefix"),
    )
    .expect("Address hex must decode");
    <[u8; 20]>::try_from(hex_bytes.as_slice()).expect("Address hex must be 20 bytes")
}

fn to_sol_address(address: &Address) -> SolAddress {
    SolAddress::from(parse_address_bytes(address))
}

fn sample_secondary_order_uid() -> OrderUid {
    // Second 56 byte fixture UID for multi-entry refund encoding; mirrors the
    // primary sample UID's shape but with distinct digest, owner, and
    // valid_to.
    let digest =
        OrderDigest::new("0x3333333333333333333333333333333333333333333333333333333333333333")
            .unwrap();
    let owner = Address::new("0x4444444444444444444444444444444444444444").unwrap();

    cow_sdk_contracts::pack_order_uid_params(&cow_sdk_contracts::OrderUidParams::new(
        digest,
        owner,
        0x9abc_def0,
    ))
}

fn order_uid_as_sol_bytes(uid: &OrderUid) -> SolBytes {
    let hex_bytes = alloy_primitives::hex::decode(
        uid.to_hex_string()
            .strip_prefix("0x")
            .expect("OrderUid must carry a 0x prefix"),
    )
    .expect("OrderUid hex must decode");
    SolBytes::from(hex_bytes)
}

fn sample_ethflow_order() -> EthFlowOrderData {
    EthFlowOrderData::new(
        Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        Amount::new("1000000000000000000").unwrap(),
        Amount::new("2000000000000000000").unwrap(),
        AppDataHash::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap(),
        Amount::ZERO,
        0x1234_5678,
        false,
        42,
    )
    .expect("sample EthFlow order helper uses a non-zero receiver")
}

fn assert_settlement_invalidate_order_calldata(id: &str, expected: &Value) {
    let expected_hex = expected["call_data"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.call_data must be a string"));

    let uid = sample_order_uid();
    let call_data = IGPv2Settlement::invalidateOrderCall {
        orderUid: order_uid_as_sol_bytes(&uid),
    }
    .abi_encode();

    assert_calldata_hex(id, &call_data, expected_hex);
}

fn assert_settlement_set_presignature_calldata(id: &str, expected: &Value) {
    let expected_hex = expected["call_data"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.call_data must be a string"));

    let uid = sample_order_uid();
    let call_data = IGPv2Settlement::setPreSignatureCall {
        orderUid: order_uid_as_sol_bytes(&uid),
        signed: true,
    }
    .abi_encode();

    assert_calldata_hex(id, &call_data, expected_hex);
}

fn assert_settlement_free_filled_amount_storage_calldata(id: &str, expected: &Value) {
    let expected_hex = expected["call_data"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.call_data must be a string"));

    let primary = sample_order_uid();
    let secondary = sample_secondary_order_uid();
    let call_data = IGPv2Settlement::freeFilledAmountStorageCall {
        orderUids: vec![
            order_uid_as_sol_bytes(&primary),
            order_uid_as_sol_bytes(&secondary),
        ],
    }
    .abi_encode();

    assert_calldata_hex(id, &call_data, expected_hex);
}

fn assert_settlement_free_presignature_storage_calldata(id: &str, expected: &Value) {
    let expected_hex = expected["call_data"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.call_data must be a string"));

    let primary = sample_order_uid();
    let secondary = sample_secondary_order_uid();
    let call_data = IGPv2Settlement::freePreSignatureStorageCall {
        orderUids: vec![
            order_uid_as_sol_bytes(&primary),
            order_uid_as_sol_bytes(&secondary),
        ],
    }
    .abi_encode();

    assert_calldata_hex(id, &call_data, expected_hex);
}

fn assert_ethflow_create_order_calldata(id: &str, expected: &Value) {
    let expected_hex = expected["call_data"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.call_data must be a string"));

    let call_data = encode_create_order_calldata(&sample_ethflow_order());

    assert_calldata_hex(id, &call_data, expected_hex);
}

fn assert_ethflow_invalidate_order_calldata(id: &str, expected: &Value) {
    let expected_hex = expected["call_data"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.call_data must be a string"));

    let call_data = encode_invalidate_order_calldata(&sample_ethflow_order());

    assert_calldata_hex(id, &call_data, expected_hex);
}

fn assert_erc20_approve_calldata(id: &str, expected: &Value) {
    let expected_hex = expected["call_data"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.call_data must be a string"));

    let spender = Address::new("0x1111111111111111111111111111111111111111").unwrap();
    let value = U256::from(1_000_000_000_000_000_000_u128);

    let call_data = IERC20::approveCall {
        spender: to_sol_address(&spender),
        value,
    }
    .abi_encode();

    assert_calldata_hex(id, &call_data, expected_hex);
}

fn assert_erc20_transfer_from_calldata(id: &str, expected: &Value) {
    let expected_hex = expected["call_data"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.call_data must be a string"));

    let from = Address::new("0x1111111111111111111111111111111111111111").unwrap();
    let to = Address::new("0x2222222222222222222222222222222222222222").unwrap();
    let value = U256::from(1_000_000_000_000_000_000_u128);

    let call_data = IERC20::transferFromCall {
        from: to_sol_address(&from),
        to: to_sol_address(&to),
        value,
    }
    .abi_encode();

    assert_calldata_hex(id, &call_data, expected_hex);
}
