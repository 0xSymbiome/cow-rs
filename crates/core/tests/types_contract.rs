use cow_sdk_core::{
    Address, Amount, Amounts, AppDataHex, AtomAmount, Costs, DecimalAmount, FeeComponent, Hash32,
    HexData, NetworkFee, ORDER_TYPE_FIELD_NAMES, OrderBalance, OrderKind, OrderModel, OrderUid,
    QUOTE_AMOUNT_STAGE_NAMES, QuoteAmountsAndCosts, QuoteModel, SignedAmount, UnsignedOrder,
    addresses_equal, token_id,
};
use num_bigint::BigUint;

fn core_fixture() -> serde_json::Value {
    serde_json::from_str(include_str!("../../../parity/fixtures/core.json"))
        .expect("core fixture must remain valid json")
}

#[test]
fn shared_type_contract_matches_core_fixture() {
    let fixture = core_fixture();
    assert_eq!(fixture["surface"].as_str().unwrap(), "core");

    let address_case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "core-evm-address-contract")
        .unwrap();
    assert_eq!(
        address_case["expected"]["address_prefix"].as_str().unwrap(),
        "0x"
    );
    assert_eq!(address_case["expected"]["hex_chars"].as_u64().unwrap(), 40);

    let checksummed = Address::new("0x742D35CC6634C0532925A3B844BC9E7595F0BEBD").unwrap();
    let lowercase = Address::new("0x742d35cc6634c0532925a3b844bc9e7595f0bebd").unwrap();

    assert_eq!(
        checksummed.normalized_key(),
        "0x742d35cc6634c0532925a3b844bc9e7595f0bebd"
    );
    assert!(addresses_equal(&checksummed, &lowercase));
    assert_eq!(
        checksummed, lowercase,
        "PartialEq must agree with addresses_equal on case variants"
    );
    assert_eq!(
        checksummed.byte_length(),
        20,
        "byte_length must match the fixed EVM address width"
    );
    assert_eq!(
        checksummed.as_bytes().len(),
        42,
        "as_bytes exposes the stored hex string as a byte slice"
    );

    let token_case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "core-token-identity-contract")
        .unwrap();
    assert_eq!(
        token_id(1, &checksummed),
        token_case["expected"]["token_id"].as_str().unwrap()
    );
}

#[test]
fn canonical_order_and_quote_shapes_are_pinned() {
    let fixture = core_fixture();
    let shared_case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "core-shared-order-and-quote-surfaces")
        .unwrap();

    let expected_fields: Vec<&str> = shared_case["expected"]["order_fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    let expected_stages: Vec<&str> = shared_case["expected"]["quote_amount_stages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();

    assert_eq!(ORDER_TYPE_FIELD_NAMES.to_vec(), expected_fields);
    assert_eq!(QUOTE_AMOUNT_STAGE_NAMES.to_vec(), expected_stages);

    let order = UnsignedOrder {
        sell_token: Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        buy_token: Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        receiver: Address::new("0x3333333333333333333333333333333333333333").unwrap(),
        sell_amount: Amount::new("100").unwrap(),
        buy_amount: Amount::new("200").unwrap(),
        valid_to: 1_700_000_000,
        app_data: AppDataHex::new(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .unwrap(),
        fee_amount: Amount::new("5").unwrap(),
        kind: OrderKind::Sell,
        partially_fillable: true,
        sell_token_balance: OrderBalance::External,
        buy_token_balance: OrderBalance::External,
    };

    assert_eq!(order.normalized_buy_token_balance(), OrderBalance::Erc20);

    let json = serde_json::to_value(&order).unwrap();
    let object = json.as_object().unwrap();
    assert!(object.contains_key("sellToken"));
    assert!(object.contains_key("buyToken"));
    assert!(object.contains_key("receiver"));
    assert!(object.contains_key("appData"));
}

#[test]
fn compatibility_models_remain_stable_for_current_workspace_consumers() {
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

    let round_trip: OrderModel = serde_json::from_str(&serde_json::to_string(&order).unwrap())
        .expect("compatibility order model should remain serializable");
    assert_eq!(round_trip, order);

    let quote = QuoteModel {
        kind: OrderKind::Buy,
        sell_amount: "1".to_owned(),
        buy_amount: "2".to_owned(),
        fee_amount: "0".to_owned(),
        order_uid: Some(OrderUid::new(format!("0x{}", "b".repeat(112))).unwrap()),
    };

    let parsed: QuoteModel = serde_json::from_str(&serde_json::to_string(&quote).unwrap())
        .expect("compatibility quote model should remain serializable");
    assert_eq!(parsed, quote);

    let amounts = QuoteAmountsAndCosts {
        is_sell: true,
        costs: Costs {
            network_fee: NetworkFee {
                amount_in_sell_currency: Amount::new("1").unwrap(),
                amount_in_buy_currency: Amount::new("2").unwrap(),
            },
            partner_fee: FeeComponent {
                amount: Amount::new("3").unwrap(),
                bps: 4,
            },
            protocol_fee: FeeComponent {
                amount: Amount::new("5").unwrap(),
                bps: 6,
            },
        },
        before_all_fees: Amounts {
            sell_amount: Amount::new("10").unwrap(),
            buy_amount: Amount::new("20").unwrap(),
        },
        before_network_costs: Amounts {
            sell_amount: Amount::new("11").unwrap(),
            buy_amount: Amount::new("21").unwrap(),
        },
        after_protocol_fees: Amounts {
            sell_amount: Amount::new("12").unwrap(),
            buy_amount: Amount::new("22").unwrap(),
        },
        after_network_costs: Amounts {
            sell_amount: Amount::new("13").unwrap(),
            buy_amount: Amount::new("23").unwrap(),
        },
        after_partner_fees: Amounts {
            sell_amount: Amount::new("14").unwrap(),
            buy_amount: Amount::new("24").unwrap(),
        },
        after_slippage: Amounts {
            sell_amount: Amount::new("15").unwrap(),
            buy_amount: Amount::new("25").unwrap(),
        },
        amounts_to_sign: Amounts {
            sell_amount: Amount::new("16").unwrap(),
            buy_amount: Amount::new("26").unwrap(),
        },
    };
    let encoded = serde_json::to_value(amounts).unwrap();
    assert!(encoded.as_object().unwrap().contains_key("amountsToSign"));
}

#[test]
fn typed_atom_and_decimal_amounts_expose_semantic_accessors() {
    let atom = AtomAmount::from_atoms(BigUint::from(1_000_000_000_000_000_000u128));
    assert_eq!(atom.to_string(), "1000000000000000000");
    assert_eq!(
        atom.as_biguint(),
        &BigUint::from(1_000_000_000_000_000_000u128)
    );
    let as_amount: Amount = atom.clone().into();
    assert_eq!(as_amount.as_str(), "1000000000000000000");

    let parsed: AtomAmount = "1000000000000000000".try_into().unwrap();
    assert_eq!(parsed, atom);

    let decimal = DecimalAmount::new(BigUint::from(1_000_000_000_000_000_000u128), 18);
    assert_eq!(decimal.decimals(), 18);
    assert_eq!(
        decimal.atoms(),
        &BigUint::from(1_000_000_000_000_000_000u128)
    );
    assert!((decimal.to_f64_approx() - 1.0).abs() < 1e-12);

    let clamped = DecimalAmount::from_whole_approx(-0.5, 18);
    assert_eq!(clamped.atoms(), &BigUint::from(0u32));
}

#[test]
fn typed_primitives_normalize_and_fail_closed() {
    assert_eq!(Amount::new("00042").unwrap().as_str(), "42");
    assert_eq!(Amount::new("0x2a").unwrap().as_str(), "42");
    assert!(Amount::new("-1").is_err());
    assert!(Amount::new("abc").is_err());
    assert!(Amount::new(format!("0x1{}", "0".repeat(64))).is_err());

    assert_eq!(SignedAmount::new("-0005").unwrap().as_str(), "-5");
    assert_eq!(SignedAmount::new("0").unwrap().as_str(), "0");
    assert!(SignedAmount::new("0x5").is_err());

    assert_eq!(HexData::new("0xabc").unwrap().as_str(), "0x0abc");
    assert_eq!(HexData::empty().as_str(), "0x");
    assert!(HexData::new("1234").is_err());

    let hash = Hash32::new(format!("0x{}", "ab".repeat(32))).unwrap();
    assert_eq!(hash.as_str().len(), 66);
    assert!(Hash32::new("0x1234").is_err());
}
