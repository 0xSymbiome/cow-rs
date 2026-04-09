use cow_sdk_core::{
    Address, Amounts, AppDataHex, Costs, FeeComponent, NetworkFee, ORDER_TYPE_FIELD_NAMES,
    OrderBalance, OrderKind, OrderModel, OrderUid, QUOTE_AMOUNT_STAGE_NAMES, QuoteAmountsAndCosts,
    QuoteModel, UnsignedOrder, addresses_equal, token_id,
};

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
        sell_amount: "100".to_owned(),
        buy_amount: "200".to_owned(),
        valid_to: 1_700_000_000,
        app_data: AppDataHex::new(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .unwrap(),
        fee_amount: "5".to_owned(),
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
                amount_in_sell_currency: "1".to_owned(),
                amount_in_buy_currency: "2".to_owned(),
            },
            partner_fee: FeeComponent {
                amount: "3".to_owned(),
                bps: 4,
            },
            protocol_fee: FeeComponent {
                amount: "5".to_owned(),
                bps: 6,
            },
        },
        before_all_fees: Amounts {
            sell_amount: "10".to_owned(),
            buy_amount: "20".to_owned(),
        },
        before_network_costs: Amounts {
            sell_amount: "11".to_owned(),
            buy_amount: "21".to_owned(),
        },
        after_protocol_fees: Amounts {
            sell_amount: "12".to_owned(),
            buy_amount: "22".to_owned(),
        },
        after_network_costs: Amounts {
            sell_amount: "13".to_owned(),
            buy_amount: "23".to_owned(),
        },
        after_partner_fees: Amounts {
            sell_amount: "14".to_owned(),
            buy_amount: "24".to_owned(),
        },
        after_slippage: Amounts {
            sell_amount: "15".to_owned(),
            buy_amount: "25".to_owned(),
        },
        amounts_to_sign: Amounts {
            sell_amount: "16".to_owned(),
            buy_amount: "26".to_owned(),
        },
    };
    let encoded = serde_json::to_value(amounts).unwrap();
    assert!(encoded.as_object().unwrap().contains_key("amountsToSign"));
}
