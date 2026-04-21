use cow_sdk::{
    Address, Amount, AppDataHex, BuyTokenDestination, ORDER_PRIMARY_TYPE, OrderKind,
    PartialTraderParameters, PartnerFee, PartnerFeePolicy, SellTokenSource, SupportedChainId,
    TradeParameters, TradingSdk, TradingSdkBuilder, TradingSdkOptions, UnsignedOrder,
    generate_order_id, order_typed_data,
};

#[test]
fn public_api_reexports_cover_primary_root_surface() {
    let _ready_sdk = TradingSdk::new(
        PartialTraderParameters::new()
            .with_chain_id(SupportedChainId::Sepolia)
            .with_app_code("cow-rs/public-api".to_owned()),
        TradingSdkOptions::default(),
    )
    .expect("ready trading sdk construction should succeed");
    let _partial_sdk = TradingSdk::new_partial(
        PartialTraderParameters::default(),
        TradingSdkOptions::default(),
    )
    .expect("partial trading sdk construction should succeed");
    let _builder = TradingSdkBuilder::new()
        .with_trader_defaults(PartialTraderParameters::default())
        .build_partial()
        .expect("partial builder construction should succeed");
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
        sell_token_balance: SellTokenSource::Erc20,
        buy_token_balance: BuyTokenDestination::Erc20,
    };
    let typed = order_typed_data(SupportedChainId::Sepolia, &order, None).unwrap();
    let generated = generate_order_id(SupportedChainId::Sepolia, &order, &owner, None).unwrap();
    let partner_fee = PartnerFee::from(
        PartnerFeePolicy::volume(50, owner.clone()).expect("volume policy must validate"),
    );

    assert_eq!(typed.primary_type, ORDER_PRIMARY_TYPE);
    assert_eq!(generated.order_digest.as_str().len(), 66);
    assert_eq!(generated.order_id.as_str().len(), 114);

    let _trade = TradeParameters::new(
        OrderKind::Sell,
        Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        18,
        Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        18,
        Amount::new("100000000000000000").unwrap(),
    )
    .with_owner(owner)
    .with_slippage_bps(50)
    .with_partner_fee(partner_fee);
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
    let api = cow_sdk::orderbook::OrderBookApi::builder_from_context(
        cow_sdk::core::ApiContext::default(),
    )
    .build();
    let _sdk = cow_sdk::trading::TradingSdk::new_partial(
        cow_sdk::trading::PartialTraderParameters::default(),
        cow_sdk::trading::TradingSdkOptions::default(),
    )
    .expect("default facade partial trading sdk construction should succeed");

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
