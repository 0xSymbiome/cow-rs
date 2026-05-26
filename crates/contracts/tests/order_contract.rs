#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::derive_partial_eq_without_eq,
    clippy::iter_on_single_items,
    clippy::missing_const_for_fn,
    clippy::option_if_let_else,
    clippy::redundant_clone,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::unnested_or_patterns,
    reason = "pedantic, nursery, style, and perf lints acceptable in test helper code"
)]

mod common;

use alloy_sol_types::SolStruct;
use cow_sdk_contracts::{
    BUY_ETH_ADDRESS, CANCELLATIONS_TYPE_FIELDS, ContractId, GPv2Order, ORDER_TYPE_FIELDS, Order,
    OrderCancellations, OrderFlags, OrderUidParams, Registry, compute_order_uid,
    decode_order_flags, encode_order_flags, extract_order_uid_params, hash_order,
    hash_order_cancellation, hash_order_cancellations, normalize_order, pack_order_uid_params,
};

fn gpv2_order_type_hash_hex() -> String {
    format!(
        "0x{}",
        alloy_primitives::hex::encode(GPv2Order::default().eip712_type_hash().as_slice())
    )
}
use cow_sdk_core::{
    Address, Amount, AppDataHex, BuyTokenDestination, CowEnv, OrderKind, SellTokenSource,
    SupportedChainId, TypedDataDomain, UnsignedOrder,
};

use common::fixture_case;

const UPSTREAM_SEPOLIA_ORDER_DIGEST: &str =
    "0xc95c0093ac625698d627b6a16b20ea16a8a735493b6f9c7b72d996de978eb823";
const UPSTREAM_SEPOLIA_ORDER_UID: &str = "0xc95c0093ac625698d627b6a16b20ea16a8a735493b6f9c7b72d996de978eb823fb3c7eb936caa12b5a884d612393969a557d4307004c4c1e";
const UPSTREAM_SEPOLIA_ORDER_OWNER: &str = "0xfb3c7eb936caa12b5a884d612393969a557d4307";

fn sample_domain() -> TypedDataDomain {
    TypedDataDomain::new(
        "Gnosis Protocol".to_owned(),
        "v2".to_owned(),
        1,
        Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap(),
    )
}

fn sample_order() -> Order {
    Order::new(
        Address::new("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap(),
        Address::new("0x6b175474e89094c44da98b954eedeac495271d0f").unwrap(),
        None,
        Amount::new("1000000000000000000").unwrap(),
        Amount::new("2000000000000000000000").unwrap(),
        1_709_990_000,
        AppDataHex::new("0x0000000000000000000000000000000000000000000000000000000000000000")
            .unwrap(),
        Amount::new("5000000000000000").unwrap(),
        OrderKind::Sell,
        false,
        None,
        Some(BuyTokenDestination::Internal),
    )
}

fn signing_fixture_case(id: &str) -> serde_json::Value {
    serde_json::from_str::<serde_json::Value>(include_str!("../../../parity/fixtures/signing.json"))
        .expect("signing fixture must remain valid json")["cases"]
        .as_array()
        .expect("signing fixture cases must be an array")
        .iter()
        .find(|case| case["id"] == id)
        .cloned()
        .unwrap_or_else(|| panic!("missing signing fixture case {id}"))
}

fn upstream_signing_sample_order() -> UnsignedOrder {
    UnsignedOrder::new(
        Address::new("0xd057b63f5e69cf1b929b356b579cba08d7688048").unwrap(),
        Address::new("0x7b878668cd1a3adf89764d3a331e0a7bb832192d").unwrap(),
        Address::new("0xa6ddbd0de6b310819b49f680f65871bee85f517e").unwrap(),
        Amount::new("500000000000000").unwrap(),
        Amount::new("23000020000").unwrap(),
        5_000_222,
        AppDataHex::new("0x0000000000000000000000000000000000000000000000000000000000000000")
            .unwrap(),
        Amount::new("2300000").unwrap(),
        OrderKind::Sell,
        true,
        SellTokenSource::Erc20,
        BuyTokenDestination::Erc20,
    )
}

fn upstream_signing_domain() -> TypedDataDomain {
    TypedDataDomain::new(
        "Gnosis Protocol".to_owned(),
        "v2".to_owned(),
        u64::from(SupportedChainId::Sepolia),
        Registry::default()
            .address(
                ContractId::Settlement,
                SupportedChainId::Sepolia,
                CowEnv::Prod,
            )
            .expect("canonical settlement address is registered for sepolia"),
    )
}

#[test]
fn order_contract_matches_fixture_and_normalization_rules() {
    let fields = fixture_case("contracts-order-type-fields");
    let expected_fields = fields["expected"]["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        ORDER_TYPE_FIELDS
            .iter()
            .map(|field| field.name)
            .collect::<Vec<_>>(),
        expected_fields
    );

    let type_hash = fixture_case("contracts-order-type-hash");
    let actual_type_hash = gpv2_order_type_hash_hex();
    assert_eq!(
        actual_type_hash,
        type_hash["expected"]["hash"].as_str().unwrap()
    );

    let cancellation_fields = fixture_case("contracts-cancellation-type-fields");
    assert_eq!(
        CANCELLATIONS_TYPE_FIELDS
            .iter()
            .map(|field| field.name)
            .collect::<Vec<_>>(),
        cancellation_fields["expected"]["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>()
    );

    let order = sample_order();
    let normalized = normalize_order(&order).unwrap();
    assert_eq!(
        normalized.receiver.to_hex_string(),
        "0x0000000000000000000000000000000000000000"
    );
    assert_eq!(normalized.sell_token_balance, SellTokenSource::Erc20);
    assert_eq!(normalized.buy_token_balance, BuyTokenDestination::Internal);
    assert_eq!(
        BUY_ETH_ADDRESS,
        "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE"
    );
}

#[test]
fn order_hash_and_uid_helpers_are_consistent() {
    let order = sample_order();
    let domain = sample_domain();

    let order_hash = hash_order(&domain, &order).unwrap();
    assert_eq!(order_hash.to_hex_string().len(), 66);
    assert_eq!(hash_order(&domain, &order).unwrap(), order_hash);

    let owner = Address::new("0x1111111111111111111111111111111111111111").unwrap();
    let uid = compute_order_uid(&domain, &order, &owner).unwrap();

    let uid_case = fixture_case("contracts-order-uid-length");
    assert_eq!(
        uid.to_hex_string().trim_start_matches("0x").len(),
        usize::try_from(uid_case["expected"]["hex_chars"].as_u64().unwrap())
            .expect("fixture uid length must fit in usize")
    );

    let extracted = extract_order_uid_params(&uid).unwrap();
    assert_eq!(extracted.owner, owner);
    assert_eq!(extracted.valid_to, order.valid_to);
    assert_eq!(extracted.order_digest, order_hash);

    let roundtrip =
        pack_order_uid_params(&OrderUidParams::new(order_hash, owner, order.valid_to)).unwrap();
    assert_eq!(roundtrip, uid);

    let cancellation = hash_order_cancellation(&domain, &uid).unwrap();
    let batch =
        hash_order_cancellations(&domain, &OrderCancellations::new(vec![uid, roundtrip])).unwrap();
    assert_eq!(cancellation.to_hex_string().len(), 66);
    assert_eq!(batch.to_hex_string().len(), 66);
    assert_ne!(cancellation, batch);
}

#[test]
fn unsigned_order_conversion_makes_user_domain_and_contract_boundaries_explicit() {
    let unsigned = UnsignedOrder::new(
        Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        Address::new("0x3333333333333333333333333333333333333333").unwrap(),
        Amount::new("1000").unwrap(),
        Amount::new("900").unwrap(),
        1_700_000_000,
        AppDataHex::new("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
            .unwrap(),
        Amount::new("10").unwrap(),
        OrderKind::Sell,
        true,
        SellTokenSource::External,
        BuyTokenDestination::Internal,
    );

    let contract = Order::from(&unsigned);

    assert_eq!(contract.sell_token, unsigned.sell_token);
    assert_eq!(contract.buy_token, unsigned.buy_token);
    assert_eq!(contract.receiver, Some(unsigned.receiver));
    assert_eq!(contract.sell_amount, unsigned.sell_amount);
    assert_eq!(contract.buy_amount, unsigned.buy_amount);
    assert_eq!(contract.valid_to, unsigned.valid_to);
    assert_eq!(contract.app_data, unsigned.app_data);
    assert_eq!(contract.fee_amount, unsigned.fee_amount);
    assert_eq!(contract.kind, unsigned.kind);
    assert_eq!(contract.partially_fillable, unsigned.partially_fillable);
    assert_eq!(
        contract.sell_token_balance,
        Some(unsigned.sell_token_balance)
    );
    assert_eq!(contract.buy_token_balance, Some(unsigned.buy_token_balance));

    let normalized = contract.normalize().unwrap();
    assert_eq!(normalized.receiver, unsigned.receiver);
    assert_eq!(normalized.sell_token_balance, SellTokenSource::External);
    assert_eq!(normalized.buy_token_balance, BuyTokenDestination::Internal);
}

#[test]
fn canonical_unsigned_order_path_matches_upstream_signing_fixture_digest_and_uid() {
    let fixture = signing_fixture_case("signing-generate-order-id");
    assert_eq!(
        fixture["expected"]["returns"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["orderId", "orderDigest"]
    );
    assert!(fixture["expected"]["owner_required"].as_bool().unwrap());
    assert_eq!(
        fixture["expected"]["uid_valid_to_source"].as_str().unwrap(),
        "order.validTo"
    );

    let unsigned = upstream_signing_sample_order();
    let order = Order::from(&unsigned);
    let domain = upstream_signing_domain();
    let owner = Address::new(UPSTREAM_SEPOLIA_ORDER_OWNER).unwrap();

    let digest = hash_order(&domain, &order).unwrap();
    assert_eq!(digest.to_hex_string(), UPSTREAM_SEPOLIA_ORDER_DIGEST);

    let uid = compute_order_uid(&domain, &order, &owner).unwrap();
    assert_eq!(uid.to_hex_string(), UPSTREAM_SEPOLIA_ORDER_UID);

    let unpacked = extract_order_uid_params(&uid).unwrap();
    assert_eq!(unpacked.owner, owner);
    assert_eq!(unpacked.valid_to, unsigned.valid_to);
    assert_eq!(
        unpacked.order_digest.to_hex_string(),
        UPSTREAM_SEPOLIA_ORDER_DIGEST
    );
}

#[test]
fn order_flag_matrix_enumerates_all_twelve_combinations() {
    let mut encoded = Vec::new();

    for kind in [OrderKind::Sell, OrderKind::Buy] {
        for sell_token_balance in [
            SellTokenSource::Erc20,
            SellTokenSource::External,
            SellTokenSource::Internal,
        ] {
            for buy_token_balance in [BuyTokenDestination::Erc20, BuyTokenDestination::Internal] {
                let flags = OrderFlags::new(kind, false, sell_token_balance, buy_token_balance);
                let encoded_flags = encode_order_flags(&flags).expect("flag tuple must encode");

                assert_eq!(
                    decode_order_flags(encoded_flags).expect("encoded flag tuple must decode"),
                    flags,
                    "order flag tuple must round-trip for {flags:?}",
                );
                encoded.push(encoded_flags);
            }
        }
    }

    encoded.sort_unstable();
    encoded.dedup();
    assert_eq!(
        encoded.len(),
        12,
        "2 order kinds x 3 sell balance sources x 2 buy destinations x 1 fill policy",
    );
}
