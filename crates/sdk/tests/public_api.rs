use cow_sdk::core::{AppDataHex, BuyTokenDestination, OrderKind, SellTokenSource, UnsignedOrder};
use cow_sdk::prelude::{
    Address, Amount, SupportedChainId, TradeParameters, TraderParameters, TradingSdkBuilder,
    TradingSdkOptions,
};
use cow_sdk::signing::{ORDER_PRIMARY_TYPE, generate_order_id, order_typed_data};
use cow_sdk::trading::{PartialTraderParameters, PartnerFee, PartnerFeePolicy};

#[test]
fn public_api_reexports_cover_primary_root_surface() {
    let _ready_sdk = TradingSdkBuilder::ready(
        TraderParameters::new(SupportedChainId::Sepolia, "cow-rs/public-api"),
        TradingSdkOptions::default(),
    )
    .expect("ready trading sdk construction should succeed");
    let _helper_only_sdk =
        TradingSdkBuilder::helper_only(SupportedChainId::Sepolia, TradingSdkOptions::default())
            .expect("helper-only trading sdk construction should succeed");
    let _builder = TradingSdkBuilder::new()
        .with_trader_defaults(PartialTraderParameters::default())
        .with_chain_id(SupportedChainId::Sepolia)
        .build_helper_only()
        .expect("helper-only builder construction should succeed");
    assert_eq!(ORDER_PRIMARY_TYPE, "Order");

    let owner = Address::new("0x4444444444444444444444444444444444444444").unwrap();
    let order = UnsignedOrder::new(
        Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        Address::new("0x3333333333333333333333333333333333333333").unwrap(),
        Amount::new("100000000000000000").unwrap(),
        Amount::new("250000000000000000").unwrap(),
        1_700_000_000,
        AppDataHex::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap(),
        Amount::zero(),
        OrderKind::Sell,
        false,
        SellTokenSource::Erc20,
        BuyTokenDestination::Erc20,
    );
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
fn cancelled_errors_project_to_the_facade_cancelled_class() {
    assert_eq!(
        cow_sdk::SdkError::AppData(cow_sdk::app_data::AppDataError::Cancelled).class(),
        cow_sdk::ErrorClass::Cancelled,
    );
    assert_eq!(
        cow_sdk::SdkError::Contracts(cow_sdk::contracts::ContractsError::Cancelled).class(),
        cow_sdk::ErrorClass::Cancelled,
    );
    assert_eq!(
        cow_sdk::SdkError::Trading(cow_sdk::trading::TradingError::AppData(
            cow_sdk::app_data::AppDataError::Cancelled,
        ))
        .class(),
        cow_sdk::ErrorClass::Cancelled,
    );
    assert_eq!(
        cow_sdk::SdkError::Trading(cow_sdk::trading::TradingError::Contracts(
            cow_sdk::contracts::ContractsError::Cancelled,
        ))
        .class(),
        cow_sdk::ErrorClass::Cancelled,
    );
}

#[test]
fn module_reexports_cover_expected_leaf_crates() {
    let doc = cow_sdk::app_data::generate_app_data_doc(
        cow_sdk::app_data::AppDataParams::default().with_app_code("cow-rs"),
    );
    let validation = cow_sdk::app_data::validate_app_data_doc(&doc);
    let schema =
        cow_sdk::app_data::get_app_data_schema(cow_sdk::app_data::SchemaVersion::latest().as_str())
            .unwrap();
    let deployment = cow_sdk::contracts::deployment_for_chain(11_155_111).unwrap();
    let api = cow_sdk::orderbook::OrderBookApi::builder_from_context(
        cow_sdk::core::ApiContext::default(),
    )
    .build()
    .expect("default facade orderbook client must build");
    let _sdk = cow_sdk::trading::TradingSdkBuilder::helper_only(
        cow_sdk::core::SupportedChainId::Sepolia,
        cow_sdk::trading::TradingSdkOptions::default(),
    )
    .expect("default facade helper-only trading sdk construction should succeed");

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

#[test]
fn prelude_does_not_export_low_level_encoders() {
    let prelude = include_str!("../src/prelude.rs");
    let forbidden = [
        "encode_create_order_calldata",
        "encode_invalidate_order_calldata",
        "function_magic_value",
        "calculate_total_fee",
        "transform_order",
        "parse_rejection",
    ];

    for symbol in forbidden {
        assert!(
            !prelude.contains(symbol),
            "prelude must not export low-level helper `{symbol}`",
        );
    }
}
