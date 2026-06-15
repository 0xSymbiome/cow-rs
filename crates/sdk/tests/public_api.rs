use cow_sdk::core::{
    Address, Amount, AppDataHash, BuyTokenDestination, OrderData, OrderKind, SellTokenSource,
    SupportedChainId,
};
use cow_sdk::signing::{ORDER_PRIMARY_TYPE, generate_order_id, order_typed_data};
use cow_sdk::trading::{PartnerFee, PartnerFeePolicy, TradeParams, TraderParams, TradingBuilder};

#[test]
fn module_paths_cover_primary_workflow_surface() {
    let _ready_trading = TradingBuilder::ready(
        TraderParams::new(SupportedChainId::Sepolia, "cow-rs/public-api")
            .expect("app code should validate"),
    );
    let _trading = TradingBuilder::new()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("cow-rs/public-api")
        .build()
        .expect("ready builder construction should succeed");
    assert_eq!(ORDER_PRIMARY_TYPE, "Order");

    let owner = Address::new("0x4444444444444444444444444444444444444444").unwrap();
    let order = OrderData::new(
        Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        Address::new("0x3333333333333333333333333333333333333333").unwrap(),
        Amount::new("100000000000000000").unwrap(),
        Amount::new("250000000000000000").unwrap(),
        1_700_000_000,
        AppDataHash::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap(),
        Amount::ZERO,
        OrderKind::Sell,
        false,
        SellTokenSource::Erc20,
        BuyTokenDestination::Erc20,
    );
    let typed = order_typed_data(SupportedChainId::Sepolia, &order, None).unwrap();
    let generated = generate_order_id(SupportedChainId::Sepolia, &order, &owner, None).unwrap();
    let partner_fee =
        PartnerFee::from(PartnerFeePolicy::volume(50, owner).expect("volume policy must validate"));

    assert_eq!(typed.primary_type, ORDER_PRIMARY_TYPE);
    assert_eq!(generated.order_digest.to_hex_string().len(), 66);
    assert_eq!(generated.order_id.to_hex_string().len(), 114);

    let _trade = TradeParams::new(
        OrderKind::Sell,
        Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        Amount::new("100000000000000000").unwrap(),
    )
    .with_owner(owner)
    .with_slippage_bps(50)
    .with_partner_fee(partner_fee);
}

#[test]
fn cancelled_errors_project_to_the_facade_cancelled_class() {
    assert_eq!(
        cow_sdk::CowError::AppData(cow_sdk::app_data::AppDataError::Cancelled).class(),
        cow_sdk::ErrorClass::Cancelled,
    );
    assert_eq!(
        cow_sdk::CowError::Contracts(cow_sdk::contracts::ContractsError::Cancelled).class(),
        cow_sdk::ErrorClass::Cancelled,
    );
    assert_eq!(
        cow_sdk::CowError::Trading(cow_sdk::trading::TradingError::AppData(
            cow_sdk::app_data::AppDataError::Cancelled,
        ))
        .class(),
        cow_sdk::ErrorClass::Cancelled,
    );
    assert_eq!(
        cow_sdk::CowError::Trading(cow_sdk::trading::TradingError::Contracts(
            cow_sdk::contracts::ContractsError::Cancelled,
        ))
        .class(),
        cow_sdk::ErrorClass::Cancelled,
    );
}

#[test]
fn module_reexports_cover_expected_leaf_crates() {
    let doc = cow_sdk::app_data::generate_app_data_doc(cow_sdk::app_data::AppDataParams::new(
        cow_sdk::core::AppCode::new("cow-rs").expect("fixture appCode must validate"),
    ));
    let validation = cow_sdk::app_data::validate_app_data_doc(&doc);
    let latest_version = cow_sdk::app_data::SchemaVersion::latest();
    let api = cow_sdk::orderbook::OrderbookApi::builder_from_context(
        cow_sdk::core::ApiContext::default(),
    )
    .build()
    .expect("default facade orderbook client must build");
    let _trading = cow_sdk::trading::TradingBuilder::ready(
        cow_sdk::trading::TraderParams::new(
            cow_sdk::core::SupportedChainId::Sepolia,
            "cow-rs/public-api",
        )
        .expect("app code should validate"),
    );

    assert!(validation.is_ok());
    assert_eq!(
        latest_version.as_str(),
        cow_sdk::app_data::LATEST_APP_DATA_VERSION
    );
    assert_eq!(
        cow_sdk::contracts::BUY_ETH_ADDRESS.to_hex_string(),
        "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
    );
    assert_eq!(cow_sdk::contracts::ORDER_UID_LENGTH, 56);
    assert_eq!(api.context().env, cow_sdk::core::CowEnv::Prod);
    assert!(cow_sdk::signing::SigningScheme::Eip712.is_ecdsa());
}

// Pins the leaf-convenience re-export: `OrderbookClient` is reachable through the
// `trading` module as well as `orderbook` (see `orderbook/src/lib.rs`). A compile
// assertion is the whole pin — trybuild earns its keep on compile-fail cases, not
// pass cases.
fn _orderbook_client_reachable_via_trading(_: Option<&dyn cow_sdk::trading::OrderbookClient>) {}
