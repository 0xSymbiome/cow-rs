mod common;

use cow_sdk_contracts::{
    BUY_ETH_ADDRESS, CANCELLATIONS_TYPE_FIELDS, ORDER_TYPE_FIELDS, ORDER_TYPE_HASH, Order,
    OrderCancellations, OrderUidParams, compute_order_uid, extract_order_uid_params, hash_order,
    hash_order_cancellation, hash_order_cancellations, hash_order_for_contract,
    normalize_buy_token_balance, normalize_order, pack_order_uid_params, uid_for_contract,
};
use cow_sdk_core::{
    Address, AppDataHex, OrderBalance, OrderKind, OrderModel, TypedDataDomain, UnsignedOrder,
};

use common::fixture_case;

fn sample_domain() -> TypedDataDomain {
    TypedDataDomain {
        name: "Gnosis Protocol".to_owned(),
        version: "v2".to_owned(),
        chain_id: 1,
        verifying_contract: Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap(),
    }
}

fn sample_order() -> Order {
    Order {
        sell_token: Address::new("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap(),
        buy_token: Address::new("0x6b175474e89094c44da98b954eedeac495271d0f").unwrap(),
        receiver: None,
        sell_amount: "1000000000000000000".to_owned(),
        buy_amount: "2000000000000000000000".to_owned(),
        valid_to: 1_709_990_000,
        app_data: AppDataHex::new(
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap(),
        fee_amount: "5000000000000000".to_owned(),
        kind: OrderKind::Sell,
        partially_fillable: false,
        sell_token_balance: None,
        buy_token_balance: Some(OrderBalance::External),
    }
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
    assert_eq!(
        ORDER_TYPE_HASH,
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
        normalized.receiver.as_str(),
        "0x0000000000000000000000000000000000000000"
    );
    assert_eq!(normalized.sell_token_balance, OrderBalance::Erc20);
    assert_eq!(normalized.buy_token_balance, OrderBalance::Erc20);
    assert_eq!(
        normalize_buy_token_balance(Some(OrderBalance::External)),
        OrderBalance::Erc20
    );
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
    assert_eq!(order_hash.len(), 66);
    assert_eq!(hash_order(&domain, &order).unwrap(), order_hash);

    let owner = Address::new("0x1111111111111111111111111111111111111111").unwrap();
    let uid = compute_order_uid(&domain, &order, &owner).unwrap();

    let uid_case = fixture_case("contracts-order-uid-length");
    assert_eq!(
        uid.as_str().trim_start_matches("0x").len(),
        uid_case["expected"]["hex_chars"].as_u64().unwrap() as usize
    );

    let extracted = extract_order_uid_params(&uid).unwrap();
    assert_eq!(extracted.owner, owner);
    assert_eq!(extracted.valid_to, order.valid_to);
    assert_eq!(extracted.order_digest, order_hash);

    let roundtrip = pack_order_uid_params(&OrderUidParams {
        order_digest: order_hash.clone(),
        owner: owner.clone(),
        valid_to: order.valid_to,
    })
    .unwrap();
    assert_eq!(roundtrip, uid);

    let cancellation = hash_order_cancellation(&domain, &uid).unwrap();
    let batch = hash_order_cancellations(
        &domain,
        &OrderCancellations {
            order_uids: vec![uid.clone(), roundtrip.clone()],
        },
    )
    .unwrap();
    assert_eq!(cancellation.len(), 66);
    assert_eq!(batch.len(), 66);
    assert_ne!(cancellation, batch);
}

#[test]
fn unsigned_order_conversion_makes_user_domain_and_contract_boundaries_explicit() {
    let unsigned = UnsignedOrder {
        sell_token: Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        buy_token: Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        receiver: Address::new("0x3333333333333333333333333333333333333333").unwrap(),
        sell_amount: "1000".to_owned(),
        buy_amount: "900".to_owned(),
        valid_to: 1_700_000_000,
        app_data: AppDataHex::new(
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        )
        .unwrap(),
        fee_amount: "10".to_owned(),
        kind: OrderKind::Sell,
        partially_fillable: true,
        sell_token_balance: OrderBalance::External,
        buy_token_balance: OrderBalance::External,
    };

    let contract = Order::from(&unsigned);

    assert_eq!(contract.sell_token, unsigned.sell_token);
    assert_eq!(contract.buy_token, unsigned.buy_token);
    assert_eq!(contract.receiver, Some(unsigned.receiver.clone()));
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
    assert_eq!(normalized.sell_token_balance, OrderBalance::External);
    assert_eq!(normalized.buy_token_balance, OrderBalance::Erc20);
}

#[test]
fn compatibility_wrappers_remain_available_for_current_workspace() {
    let order = OrderModel {
        kind: OrderKind::Sell,
        sell_token: Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        buy_token: Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        receiver: Address::new("0x3333333333333333333333333333333333333333").unwrap(),
        owner: Address::new("0x4444444444444444444444444444444444444444").unwrap(),
        app_data_hex: AppDataHex::new(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .unwrap(),
    };

    let hash = hash_order_for_contract(&order, 1).unwrap();
    assert_eq!(hash.len(), 32);

    let uid = uid_for_contract(&order, 1, [0x44; 20], 1_700_000_000).unwrap();
    assert_eq!(uid.as_str().trim_start_matches("0x").len(), 112);
}
