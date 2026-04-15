use cow_sdk::{
    Address, Amount, AppDataHex, ORDER_PRIMARY_TYPE, OrderBalance, OrderKind,
    PartialTraderParameters, PartnerFee, PartnerFeePolicy, SupportedChainId, TradeParameters,
    TradingSdk, TradingSdkBuilder, TradingSdkOptions, UnsignedOrder, generate_order_id,
    order_typed_data,
};

#[test]
fn public_api_reexports_cover_primary_root_surface() {
    let _sdk = TradingSdk::new(
        PartialTraderParameters::default(),
        TradingSdkOptions::default(),
    )
    .expect("default trading sdk construction should succeed");
    let _builder =
        TradingSdkBuilder::new().with_trader_defaults(PartialTraderParameters::default());
    assert_eq!(ORDER_PRIMARY_TYPE, "Order");

    let owner = Address::new("0x4444444444444444444444444444444444444444").unwrap();
    let order = UnsignedOrder {
        sell_token: Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        buy_token: Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        receiver: Address::new("0x3333333333333333333333333333333333333333").unwrap(),
        sell_amount: Amount::new("100000000000000000").unwrap(),
        buy_amount: Amount::new("250000000000000000").unwrap(),
        valid_to: 1_700_000_000,
        app_data: AppDataHex::new(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .unwrap(),
        fee_amount: Amount::zero(),
        kind: OrderKind::Sell,
        partially_fillable: false,
        sell_token_balance: OrderBalance::Erc20,
        buy_token_balance: OrderBalance::Erc20,
    };
    let typed = order_typed_data(SupportedChainId::Sepolia, &order, None).unwrap();
    let generated = generate_order_id(SupportedChainId::Sepolia, &order, &owner, None).unwrap();
    let partner_fee = PartnerFee::from(PartnerFeePolicy::volume(50, owner.clone()));

    assert_eq!(typed.primary_type, ORDER_PRIMARY_TYPE);
    assert_eq!(generated.order_digest.as_str().len(), 66);
    assert_eq!(generated.order_id.as_str().len(), 114);

    let _trade = TradeParameters {
        kind: OrderKind::Sell,
        owner: Some(owner),
        sell_token: Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        sell_token_decimals: 18,
        buy_token: Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        buy_token_decimals: 18,
        amount: Amount::new("100000000000000000").unwrap(),
        env: None,
        settlement_contract_override: None,
        eth_flow_contract_override: None,
        partially_fillable: false,
        sell_token_balance: OrderBalance::Erc20,
        buy_token_balance: OrderBalance::Erc20,
        slippage_bps: Some(50),
        receiver: None,
        valid_for: None,
        valid_to: None,
        partner_fee: Some(partner_fee),
    };
}

#[test]
fn module_reexports_cover_expected_leaf_crates() {
    let doc = cow_sdk::app_data::generate_app_data_doc(cow_sdk::app_data::AppDataParams {
        app_code: Some("cow-rs".to_owned()),
        ..Default::default()
    });
    let validation = cow_sdk::app_data::validate_app_data_doc(&doc);
    let schema =
        cow_sdk::app_data::get_app_data_schema(cow_sdk::app_data::SchemaVersion::latest().as_str())
            .unwrap();
    let deployment = cow_sdk::contracts::deployment_for_chain(11_155_111).unwrap();
    let api = cow_sdk::orderbook::OrderBookApi::new(cow_sdk::core::ApiContext::default());
    let _sdk = cow_sdk::trading::TradingSdk::new(
        cow_sdk::trading::PartialTraderParameters::default(),
        cow_sdk::trading::TradingSdkOptions::default(),
    )
    .expect("default facade trading sdk construction should succeed");

    assert!(validation.success);
    assert!(schema.is_object());
    assert_eq!(
        cow_sdk::contracts::BUY_ETH_ADDRESS,
        "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE"
    );
    assert_eq!(cow_sdk::contracts::ORDER_UID_LENGTH, 56);
    assert_ne!(deployment.settlement, deployment.eth_flow);
    assert_eq!(api.context().env, cow_sdk::core::CowEnv::Prod);
    assert!(cow_sdk::signing::SigningScheme::Eip712.is_ecdsa());
}

#[test]
fn crate_docs_and_manifest_keep_the_facade_trading_first() {
    let lib_rs = include_str!("../src/lib.rs");
    let manifest = include_str!("../Cargo.toml");

    assert!(lib_rs.contains("Top-level docs are trading-first"));
    assert!(
        lib_rs.contains(
            "Optional browser-runtime support does not change the default facade identity."
        )
    );
    assert!(lib_rs.contains("the full browser-runtime contract stays in `cow-sdk-browser-wallet`"));
    assert!(lib_rs.contains("is a separate crate surface"));
    assert!(manifest.contains("default = []"));
    assert!(manifest.contains("browser-wallet = [\"dep:cow-sdk-browser-wallet\"]"));
    assert!(manifest.contains("cow-sdk-trading"));
    assert!(!manifest.contains("cow-sdk-subgraph"));
}
