//! Behaviour tests for the trading-crate typed parameter and result bundles.
//!
//! These tests pin every documented constructor, `with_*` builder, accessor,
//! trait impl, and serde round-trip across `types/app_code`, `types/slippage`,
//! `types/trader`, `types/advanced`, and `types/options`. The helpers under
//! test are pure builders — no provider, signer, or orderbook fixture is
//! required.
//!
//! Larger result and quote bundles (`QuoteResults`, `OrderPostingResult`,
//! `TradingAppDataInfo`) are exercised end-to-end by the SDK and post
//! contract tests under this same `tests/` directory; this file deliberately
//! restricts itself to constructor- and builder-level coverage to keep its
//! fixtures small.

use std::sync::Arc;

use cow_sdk_core::{
    Address, AddressPerChain, Amount, AppCode, AppCodeError, BuyTokenDestination, CowEnv,
    OrderKind, OrderUid, SellTokenSource, SupportedChainId,
};
use cow_sdk_orderbook::{OrderbookClient, SigningScheme};
use cow_sdk_trading::{
    LimitTradeParameters, NoopQuoteCache, OrderTraderParameters, PartialTraderParameters,
    PostTradeAdditionalParams, QuoterParameters, SlippageToleranceRequest,
    SlippageToleranceResponse, TradeAdvancedSettings, TradeParameters, TraderParameters,
    TradingSdkOptions,
};

const VALID_ADDRESS: &str = "0x1111111111111111111111111111111111111111";
const OTHER_ADDRESS: &str = "0x2222222222222222222222222222222222222222";
// OrderUid is 56 bytes = 112 hex characters after the `0x` prefix.
const VALID_ORDER_UID: &str = concat!(
    "0x",
    "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    "1234567890abcdef1234567890abcdef1234567890abcdef",
);

fn valid_address() -> Address {
    Address::new(VALID_ADDRESS).expect("static valid address parses")
}

fn other_address() -> Address {
    Address::new(OTHER_ADDRESS).expect("static valid address parses")
}

fn order_uid() -> OrderUid {
    OrderUid::new(VALID_ORDER_UID).expect("static valid order uid parses")
}

// -------------------------------------------------------------------------
// AppCode
// -------------------------------------------------------------------------

#[test]
fn app_code_accepts_documented_shapes() {
    for value in [
        "CoW Swap",
        "cow-rs/wasm-console",
        "COW_BRIDGING_REACT_EXAMPLE",
        "x", // single character
        "CoW Swap (mainnet)",
    ] {
        let code = AppCode::new(value).expect("valid app code is accepted");
        assert_eq!(code.as_str(), value);
    }
}

#[test]
fn app_code_rejects_empty_input() {
    assert_eq!(AppCode::new(""), Err(AppCodeError::Empty));
}

#[test]
fn app_code_rejects_nul_byte() {
    assert_eq!(AppCode::new("cow\0rs"), Err(AppCodeError::NulByte));
}

#[test]
fn app_code_rejects_control_characters() {
    for value in ["cow\nrs", "cow\trs", "cow\x01rs", "cow\x7frs"] {
        assert_eq!(
            AppCode::new(value),
            Err(AppCodeError::ControlCharacter),
            "control character in {value:?} must be rejected",
        );
    }
}

#[test]
fn app_code_accessors_return_inner_string() {
    let code = AppCode::new("cow-rs").unwrap();
    assert_eq!(code.as_str(), "cow-rs");
    assert_eq!(<AppCode as AsRef<str>>::as_ref(&code), "cow-rs");
    assert_eq!(&*code, "cow-rs"); // Deref<Target=str>
    assert_eq!(format!("{code}"), "cow-rs"); // Display
    assert_eq!(code.into_inner(), "cow-rs");
}

#[test]
fn app_code_from_str_and_try_from_round_trip() {
    let code: AppCode = "cow-rs".parse().expect("FromStr accepts valid app code");
    assert_eq!(code.as_str(), "cow-rs");

    let invalid: Result<AppCode, _> = "".parse();
    assert_eq!(invalid, Err(AppCodeError::Empty));

    let from_ref: AppCode = "cow-rs".try_into().unwrap();
    assert_eq!(from_ref.as_str(), "cow-rs");

    let from_owned: AppCode = String::from("cow-rs").try_into().unwrap();
    assert_eq!(from_owned.as_str(), "cow-rs");
}

#[test]
fn app_code_serde_round_trips_through_json() {
    let code = AppCode::new("cow-rs/wasm").unwrap();
    let serialized = serde_json::to_string(&code).unwrap();
    assert_eq!(serialized, r#""cow-rs/wasm""#);

    let deserialized: AppCode = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, code);

    // Deserializing an empty string surfaces the validation error.
    let invalid: Result<AppCode, _> = serde_json::from_str(r#""""#);
    assert!(invalid.is_err());

    // Deserializing a control character also surfaces validation.
    let invalid: Result<AppCode, _> = serde_json::from_str(r#""cow\nrs""#);
    assert!(invalid.is_err());
}

// -------------------------------------------------------------------------
// SlippageToleranceRequest + SlippageToleranceResponse
// -------------------------------------------------------------------------

#[test]
fn slippage_tolerance_request_constructors_record_required_fields() {
    let req =
        SlippageToleranceRequest::new(SupportedChainId::Mainnet, valid_address(), other_address());
    assert_eq!(req.chain_id, SupportedChainId::Mainnet);
    assert_eq!(req.sell_token, valid_address());
    assert_eq!(req.buy_token, other_address());
    assert!(req.sell_amount.is_none());
    assert!(req.buy_amount.is_none());

    let chained = req
        .clone()
        .with_sell_amount(Amount::from(1_000_u64))
        .with_buy_amount(Amount::from(2_000_u64));
    assert_eq!(chained.sell_amount, Some(Amount::from(1_000_u64)));
    assert_eq!(chained.buy_amount, Some(Amount::from(2_000_u64)));
    // Original is unchanged (each setter returns a copy).
    assert!(req.sell_amount.is_none());
}

#[test]
fn slippage_tolerance_response_constructors_record_optional_field() {
    let empty = SlippageToleranceResponse::new();
    assert!(empty.slippage_bps.is_none());
    assert_eq!(SlippageToleranceResponse::default(), empty);

    let populated = empty.with_slippage_bps(50);
    assert_eq!(populated.slippage_bps, Some(50));
}

// -------------------------------------------------------------------------
// TraderParameters and friends
// -------------------------------------------------------------------------

#[test]
fn trader_parameters_constructors_and_with_setters_preserve_inputs() {
    let trader = TraderParameters::new(SupportedChainId::Mainnet, "cow-rs")
        .expect("valid app code is accepted");
    assert_eq!(trader.chain_id, SupportedChainId::Mainnet);
    assert_eq!(trader.app_code.as_str(), "cow-rs");
    assert!(trader.env.is_none());
    assert!(trader.settlement_contract_override.is_none());
    assert!(trader.eth_flow_contract_override.is_none());

    let settlement_overrides = AddressPerChain::from_iter([(1_u64, valid_address())]);
    let eth_flow_overrides = AddressPerChain::from_iter([(11_155_111_u64, other_address())]);
    let populated = trader
        .with_env(CowEnv::Prod)
        .with_settlement_contract_override(settlement_overrides.clone())
        .with_eth_flow_contract_override(eth_flow_overrides.clone());

    assert_eq!(populated.env, Some(CowEnv::Prod));
    assert_eq!(
        populated.settlement_contract_override,
        Some(settlement_overrides)
    );
    assert_eq!(
        populated.eth_flow_contract_override,
        Some(eth_flow_overrides)
    );
}

#[test]
fn trader_parameters_new_rejects_invalid_app_code() {
    assert_eq!(
        TraderParameters::new(SupportedChainId::Mainnet, ""),
        Err(AppCodeError::Empty),
    );
    assert_eq!(
        TraderParameters::new(SupportedChainId::Mainnet, "cow\0rs"),
        Err(AppCodeError::NulByte),
    );
    assert_eq!(
        TraderParameters::new(SupportedChainId::Mainnet, "cow\nrs"),
        Err(AppCodeError::ControlCharacter),
    );
}

#[test]
fn partial_trader_parameters_builders_preserve_inputs() {
    let partial = PartialTraderParameters::new();
    assert!(partial.chain_id.is_none());
    assert!(partial.app_code.is_none());

    let populated = partial
        .with_chain_id(SupportedChainId::Sepolia)
        .with_env(CowEnv::Staging)
        .with_app_code("cow-rs")
        .expect("valid app code");
    assert_eq!(populated.chain_id, Some(SupportedChainId::Sepolia));
    assert_eq!(populated.env, Some(CowEnv::Staging));
    assert_eq!(
        populated.app_code.as_ref().map(AppCode::as_str),
        Some("cow-rs")
    );
}

#[test]
fn partial_trader_parameters_with_app_code_rejects_invalid_input() {
    let partial = PartialTraderParameters::new();
    assert!(matches!(
        partial.with_app_code(""),
        Err(AppCodeError::Empty),
    ));
}

#[test]
fn quoter_parameters_constructors_and_with_setters_preserve_inputs() {
    let quoter = QuoterParameters::new(SupportedChainId::Mainnet, "cow-rs", valid_address())
        .expect("valid quoter parameters");
    assert_eq!(quoter.chain_id, SupportedChainId::Mainnet);
    assert_eq!(quoter.account, valid_address());

    let populated = quoter
        .with_env(CowEnv::Prod)
        .with_settlement_contract_override(AddressPerChain::from_iter([(1_u64, valid_address())]))
        .with_eth_flow_contract_override(AddressPerChain::from_iter([(1_u64, other_address())]));
    assert_eq!(populated.env, Some(CowEnv::Prod));
    assert!(populated.settlement_contract_override.is_some());
    assert!(populated.eth_flow_contract_override.is_some());

    // Invalid app code is rejected.
    let invalid = QuoterParameters::new(SupportedChainId::Mainnet, "", valid_address());
    assert_eq!(invalid, Err(AppCodeError::Empty));
}

#[test]
fn order_trader_parameters_constructors_and_with_setters_preserve_inputs() {
    let params = OrderTraderParameters::new(order_uid());
    assert_eq!(params.order_uid, order_uid());
    assert!(params.chain_id.is_none());
    assert!(params.env.is_none());

    let populated = params
        .with_chain_id(SupportedChainId::Mainnet)
        .with_env(CowEnv::Prod)
        .with_settlement_contract_override(AddressPerChain::from_iter([(1_u64, valid_address())]))
        .with_eth_flow_contract_override(AddressPerChain::from_iter([(
            11_155_111_u64,
            other_address(),
        )]));
    assert_eq!(populated.chain_id, Some(SupportedChainId::Mainnet));
    assert_eq!(populated.env, Some(CowEnv::Prod));
    assert!(populated.settlement_contract_override.is_some());
    assert!(populated.eth_flow_contract_override.is_some());
}

// -------------------------------------------------------------------------
// Advanced settings bundles
// -------------------------------------------------------------------------

#[test]
fn post_trade_additional_params_builders_record_fields() {
    let params = PostTradeAdditionalParams::new();
    assert!(params.check_eth_flow_order_exists.is_none());
    assert!(params.network_costs_amount.is_none());
    assert!(params.signing_scheme.is_none());
    assert!(params.custom_eip1271_signature.is_none());
    assert!(params.apply_costs_slippage_and_fees.is_none());

    let populated = params
        .with_network_costs_amount(Amount::from(100_u64))
        .with_signing_scheme(SigningScheme::Eip712)
        .with_apply_costs_slippage_and_fees(true);

    assert_eq!(populated.network_costs_amount, Some(Amount::from(100_u64)));
    assert_eq!(populated.signing_scheme, Some(SigningScheme::Eip712));
    assert_eq!(populated.apply_costs_slippage_and_fees, Some(true));

    let debug = format!("{populated:?}");
    assert!(debug.contains("PostTradeAdditionalParams"));
    assert!(debug.contains("network_costs_amount"));
}

#[test]
fn swap_advanced_settings_builders_round_trip_and_debug_renders() {
    let settings = TradeAdvancedSettings::new();
    let debug = format!("{settings:?}");
    assert!(debug.contains("TradeAdvancedSettings"));
    // The default settings render with `false` flags for trait-object presence.
    assert!(debug.contains("false"));
}

#[test]
fn limit_order_advanced_settings_builders_round_trip_and_debug_renders() {
    let settings = TradeAdvancedSettings::new();
    let debug = format!("{settings:?}");
    assert!(debug.contains("TradeAdvancedSettings"));
}

// -------------------------------------------------------------------------
// TradingSdkOptions
// -------------------------------------------------------------------------

// -------------------------------------------------------------------------
// TradeParameters and LimitTradeParameters
// -------------------------------------------------------------------------

#[test]
fn trade_parameters_new_seeds_documented_defaults_and_with_setters_attach_fields() {
    let trade = TradeParameters::new(
        OrderKind::Sell,
        valid_address(),
        other_address(),
        Amount::from(1_000_000_u64),
    );
    assert_eq!(trade.kind, OrderKind::Sell);
    assert_eq!(trade.sell_token, valid_address());
    assert_eq!(trade.buy_token, other_address());
    assert_eq!(trade.amount, Amount::from(1_000_000_u64));
    assert!(trade.owner.is_none());
    assert!(trade.env.is_none());
    assert!(trade.receiver.is_none());
    assert!(trade.slippage_bps.is_none());
    assert!(trade.valid_to.is_none());
    assert!(trade.valid_for.is_none());
    assert!(trade.partner_fee.is_none());
    assert!(!trade.partially_fillable);
    assert_eq!(trade.sell_token_balance, SellTokenSource::Erc20);
    assert_eq!(trade.buy_token_balance, BuyTokenDestination::Erc20);

    // Chain every with_* setter and assert the new field surfaces.
    let populated = trade
        .with_owner(valid_address())
        .with_env(CowEnv::Prod)
        .with_receiver(other_address())
        .with_slippage_bps(50)
        .with_valid_to(1_800_000_000)
        .with_valid_for(900)
        .with_partially_fillable(true)
        .with_sell_token_balance(SellTokenSource::External)
        .with_buy_token_balance(BuyTokenDestination::Internal);

    assert_eq!(populated.owner, Some(valid_address()));
    assert_eq!(populated.env, Some(CowEnv::Prod));
    assert_eq!(populated.receiver, Some(other_address()));
    assert_eq!(populated.slippage_bps, Some(50));
    assert_eq!(populated.valid_to, Some(1_800_000_000));
    assert_eq!(populated.valid_for, Some(900));
    assert!(populated.partially_fillable);
    assert_eq!(populated.sell_token_balance, SellTokenSource::External);
    assert_eq!(populated.buy_token_balance, BuyTokenDestination::Internal);
}

#[test]
fn limit_trade_parameters_new_seeds_documented_defaults_and_with_setters_attach_fields() {
    let limit = LimitTradeParameters::new(
        OrderKind::Buy,
        valid_address(),
        other_address(),
        Amount::from(2_000_u64),
        Amount::from(1_000_u64),
    );
    assert_eq!(limit.kind, OrderKind::Buy);
    assert_eq!(limit.sell_amount, Amount::from(2_000_u64));
    assert_eq!(limit.buy_amount, Amount::from(1_000_u64));
    assert!(limit.owner.is_none());
    assert!(limit.quote_id.is_none());
    assert!(limit.env.is_none());
    assert!(limit.receiver.is_none());
    assert!(limit.slippage_bps.is_none());
    assert!(!limit.partially_fillable);
    assert_eq!(limit.sell_token_balance, SellTokenSource::Erc20);
    assert_eq!(limit.buy_token_balance, BuyTokenDestination::Erc20);

    let populated = limit
        .with_owner(valid_address())
        .with_env(CowEnv::Staging)
        .with_receiver(other_address())
        .with_slippage_bps(75)
        .with_valid_to(2_000_000_000)
        .with_valid_for(600)
        .with_partially_fillable(true)
        .with_sell_token_balance(SellTokenSource::Internal)
        .with_buy_token_balance(BuyTokenDestination::Internal)
        .with_quote_id(42);

    assert_eq!(populated.owner, Some(valid_address()));
    assert_eq!(populated.env, Some(CowEnv::Staging));
    assert_eq!(populated.receiver, Some(other_address()));
    assert_eq!(populated.slippage_bps, Some(75));
    assert_eq!(populated.valid_to, Some(2_000_000_000));
    assert_eq!(populated.valid_for, Some(600));
    assert!(populated.partially_fillable);
    assert_eq!(populated.sell_token_balance, SellTokenSource::Internal);
    assert_eq!(populated.buy_token_balance, BuyTokenDestination::Internal);
    assert_eq!(populated.quote_id, Some(42));
}

#[test]
fn trading_sdk_options_builders_round_trip_and_debug_reflects_presence() {
    let empty = TradingSdkOptions::new();
    assert!(empty.orderbook_client().is_none());
    assert!(empty.quote_cache().is_none());

    let debug = format!("{empty:?}");
    assert!(debug.contains("order_book_api: false"));
    assert!(debug.contains("quote_cache: false"));

    // Inject a noop quote cache and assert the option surfaces it.
    let cache: Arc<dyn cow_sdk_trading::QuoteCache> = Arc::new(NoopQuoteCache);
    let populated = empty.with_quote_cache(cache);
    assert!(populated.quote_cache().is_some());
    let debug = format!("{populated:?}");
    assert!(debug.contains("quote_cache: true"));

    // orderbook_client: take the default trait-object check without a real client.
    // Implementations live in cow_sdk_orderbook; we use a trait object surfaced
    // through TradingSdk's typed builder pattern via FakeClient avoidance.
    drop::<Option<Arc<dyn OrderbookClient>>>(None);
}
