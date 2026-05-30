mod common;

use std::sync::{Arc, Mutex};

use cow_sdk_core::{
    Amount, AppCode, BuyTokenDestination, CowEnv, MAX_VALID_TO_EPOCH, ProtocolOptions,
    SellTokenSource, SupportedChainId, UnsignedOrder, ValidationReason, wrapped_native_token,
};

fn test_app_code() -> AppCode {
    AppCode::new("0x007").expect("fixture appCode must validate")
}
use cow_sdk_orderbook::{OrderKind, SigningScheme};
use cow_sdk_signing::ORDER_PRIMARY_TYPE;
use cow_sdk_trading::{
    ClientRejection, PartnerFeePolicy, QuoteRequestOverride, QuoterParameters,
    TradeAdvancedSettings, TradeParameters, build_app_data, calculate_unique_order_id,
    get_quote_only, get_quote_results,
};

use crate::common::{
    MockEthFlowChecker, MockOrderbook, MockSigner, MockSlippageProvider, OWNER, address,
    app_data_hash, sample_trade_parameters, sell_quote_response, trading_fixture,
};

#[tokio::test]
async fn quote_app_data_and_request_shape_follow_pinned_contract() {
    let fixture = trading_fixture();
    assert!(
        fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .any(|case| case["id"] == "trading-quote-app-data-enrichment")
    );

    let orderbook = MockOrderbook::new(
        cow_sdk_core::SupportedChainId::Sepolia,
        sell_quote_response(),
    );
    let signer = MockSigner::default();
    let trader =
        cow_sdk_trading::TraderParameters::new(cow_sdk_core::SupportedChainId::Sepolia, "0x007")
            .expect("app code should validate")
            .with_env(CowEnv::Prod);
    let mut trade = sample_trade_parameters(OrderKind::Sell);
    trade.slippage_bps = Some(76);

    let result = get_quote_results(&trade, &trader, &signer, None, &orderbook)
        .await
        .expect("quote with signer should succeed");
    let state = orderbook.state();
    let request = state.quote_requests.last().expect("quote request recorded");
    let app_data = serde_json::from_str::<serde_json::Value>(&result.app_data_info.full_app_data)
        .expect("full app data must remain valid json");

    assert_eq!(app_data["appCode"], serde_json::json!("0x007"));
    assert_eq!(
        app_data["metadata"]["quote"]["slippageBips"],
        serde_json::json!(76)
    );
    assert_eq!(result.order_typed_data.primary_type, ORDER_PRIMARY_TYPE);
    assert!(result.order_typed_data.primary_type_fields().is_some());
    assert!(result.order_typed_data.types.contains_key("EIP712Domain"));
    assert_eq!(
        request.price_quality,
        cow_sdk_orderbook::PriceQuality::Optimal
    );
}

#[tokio::test]
async fn quote_validity_uses_valid_for_by_default_and_exact_valid_to_when_requested() {
    let orderbook = MockOrderbook::new(
        cow_sdk_core::SupportedChainId::Sepolia,
        sell_quote_response(),
    );
    let signer = MockSigner::default();
    let trader =
        cow_sdk_trading::TraderParameters::new(cow_sdk_core::SupportedChainId::Sepolia, "0x007")
            .expect("app code should validate");

    let default_trade = sample_trade_parameters(OrderKind::Sell);
    let _ = get_quote_results(&default_trade, &trader, &signer, None, &orderbook)
        .await
        .expect("default quote should succeed");
    let default_request = orderbook
        .state()
        .quote_requests
        .last()
        .cloned()
        .expect("default quote request recorded");
    assert_eq!(
        default_request.validity,
        cow_sdk_orderbook::QuoteValidity::ValidFor(1_800)
    );

    let mut exact_trade = sample_trade_parameters(OrderKind::Sell);
    exact_trade.valid_to = Some(2_524_608_000);
    exact_trade.valid_for = None;
    let orderbook = MockOrderbook::new(
        cow_sdk_core::SupportedChainId::Sepolia,
        sell_quote_response(),
    );
    let _ = get_quote_results(&exact_trade, &trader, &signer, None, &orderbook)
        .await
        .expect("exact validTo quote should succeed");
    let exact_request = orderbook
        .state()
        .quote_requests
        .last()
        .cloned()
        .expect("exact validTo request recorded");
    assert_eq!(
        exact_request.validity,
        cow_sdk_orderbook::QuoteValidity::ValidTo(2_524_608_000)
    );

    let mut invalid_trade = sample_trade_parameters(OrderKind::Sell);
    invalid_trade.valid_for = Some(600);
    invalid_trade.valid_to = Some(2_524_608_000);
    let error = get_quote_results(&invalid_trade, &trader, &signer, None, &orderbook)
        .await
        .expect_err("simultaneous validFor and validTo must fail");
    assert!(
        error
            .to_string()
            .contains("Cannot specify both validFor and validTo")
    );
}

#[tokio::test]
async fn native_sell_quote_uses_wrapped_native_and_onchain_defaults() {
    let orderbook = MockOrderbook::new(
        cow_sdk_core::SupportedChainId::Sepolia,
        sell_quote_response(),
    );
    let signer = MockSigner::default();
    let trader =
        cow_sdk_trading::TraderParameters::new(cow_sdk_core::SupportedChainId::Sepolia, "0x007")
            .expect("app code should validate");
    let mut trade = sample_trade_parameters(OrderKind::Sell);
    trade.sell_token = address(cow_sdk_core::EVM_NATIVE_CURRENCY_ADDRESS);
    trade.slippage_bps = None;

    let _ = get_quote_results(&trade, &trader, &signer, None, &orderbook)
        .await
        .expect("native sell quote should succeed");
    let request = orderbook
        .state()
        .quote_requests
        .last()
        .cloned()
        .expect("native sell request recorded");

    assert_eq!(
        request.sell_token,
        cow_sdk_core::wrapped_native_token(cow_sdk_core::SupportedChainId::Sepolia).address
    );
    assert_eq!(
        request.signing_scheme,
        cow_sdk_orderbook::QuoteSigningScheme::Eip1271 {
            onchain_order: true,
            verification_gas_limit: 0
        }
    );
}

#[tokio::test]
async fn auto_slippage_uses_provider_suggestion_and_quote_only_uses_owner_without_signer() {
    let orderbook = MockOrderbook::new(
        cow_sdk_core::SupportedChainId::Sepolia,
        sell_quote_response(),
    );
    let quoter = QuoterParameters::new(
        cow_sdk_core::SupportedChainId::Sepolia,
        "0x007",
        address(OWNER),
    )
    .expect("app code should validate");
    let mut trade = sample_trade_parameters(OrderKind::Sell);
    trade.owner = Some(address(OWNER));
    trade.slippage_bps = None;
    let advanced =
        TradeAdvancedSettings::new().with_slippage_suggester(Arc::new(MockSlippageProvider {
            response: Some(200),
        }));

    let result = get_quote_only(&trade, &quoter, Some(&advanced), &orderbook)
        .await
        .expect("quote-only should succeed");
    let request = orderbook
        .state()
        .quote_requests
        .last()
        .cloned()
        .expect("quote-only request recorded");
    let app_data = serde_json::from_str::<serde_json::Value>(&result.app_data_info.full_app_data)
        .expect("full app data must remain valid json");

    assert_eq!(result.suggested_slippage_bps, 269);
    assert_eq!(request.from, address(OWNER));
    assert_eq!(
        app_data["metadata"]["quote"]["slippageBips"],
        serde_json::json!(269)
    );
}

#[tokio::test]
async fn quote_request_override_can_change_receiver_and_price_quality() {
    let orderbook = MockOrderbook::new(
        cow_sdk_core::SupportedChainId::Sepolia,
        sell_quote_response(),
    );
    let signer = MockSigner::default();
    let trader =
        cow_sdk_trading::TraderParameters::new(cow_sdk_core::SupportedChainId::Sepolia, "0x007")
            .expect("app code should validate");
    let trade: TradeParameters = sample_trade_parameters(OrderKind::Sell);
    let advanced = TradeAdvancedSettings::new().with_quote_request(
        QuoteRequestOverride::new()
            .with_receiver(address(crate::common::ALT_RECEIVER))
            .with_price_quality(cow_sdk_orderbook::PriceQuality::Fast),
    );

    let result = get_quote_results(&trade, &trader, &signer, Some(&advanced), &orderbook)
        .await
        .expect("quote with override should succeed");
    let request = orderbook
        .state()
        .quote_requests
        .last()
        .cloned()
        .expect("override request recorded");

    assert_eq!(request.receiver, Some(address(crate::common::ALT_RECEIVER)));
    assert_eq!(request.price_quality, cow_sdk_orderbook::PriceQuality::Fast);
    assert_eq!(
        result.trade_parameters.receiver,
        Some(address(crate::common::ALT_RECEIVER))
    );
    assert_eq!(
        result.order_to_sign.receiver,
        address(crate::common::ALT_RECEIVER)
    );
}

#[tokio::test]
async fn quote_results_preserve_non_default_balance_semantics_from_quote_and_override_request() {
    let mut quote_response = sell_quote_response();
    quote_response.quote.sell_token_balance = SellTokenSource::External;
    quote_response.quote.buy_token_balance = BuyTokenDestination::Internal;
    let orderbook = MockOrderbook::new(cow_sdk_core::SupportedChainId::Sepolia, quote_response);
    let signer = MockSigner::default();
    let trader =
        cow_sdk_trading::TraderParameters::new(cow_sdk_core::SupportedChainId::Sepolia, "0x007")
            .expect("app code should validate");
    let trade: TradeParameters = sample_trade_parameters(OrderKind::Sell);
    let advanced = TradeAdvancedSettings::new().with_quote_request(
        QuoteRequestOverride::new()
            .with_sell_token_balance(SellTokenSource::External)
            .with_buy_token_balance(BuyTokenDestination::Internal),
    );

    let result = get_quote_results(&trade, &trader, &signer, Some(&advanced), &orderbook)
        .await
        .expect("quote with balance overrides should succeed");
    let request = orderbook
        .state()
        .quote_requests
        .last()
        .cloned()
        .expect("quote request recorded");

    assert_eq!(request.sell_token_balance, SellTokenSource::External);
    assert_eq!(request.buy_token_balance, BuyTokenDestination::Internal);
    assert_eq!(
        result.trade_parameters.sell_token_balance,
        SellTokenSource::External
    );
    assert_eq!(
        result.trade_parameters.buy_token_balance,
        BuyTokenDestination::Internal
    );
    assert_eq!(
        result.quote_response.quote.sell_token_balance,
        SellTokenSource::External
    );
    assert_eq!(
        result.quote_response.quote.buy_token_balance,
        BuyTokenDestination::Internal
    );
    assert_eq!(
        result.order_to_sign.sell_token_balance,
        SellTokenSource::External
    );
    assert_eq!(
        result.order_to_sign.buy_token_balance,
        BuyTokenDestination::Internal
    );
}

#[tokio::test]
async fn quote_request_keeps_trade_partial_fill_flag_without_direct_override() {
    let orderbook = MockOrderbook::new(
        cow_sdk_core::SupportedChainId::Sepolia,
        sell_quote_response(),
    );
    let signer = MockSigner::default();
    let trader =
        cow_sdk_trading::TraderParameters::new(cow_sdk_core::SupportedChainId::Sepolia, "0x007")
            .expect("app code should validate");
    let mut trade: TradeParameters = sample_trade_parameters(OrderKind::Sell);
    trade.partially_fillable = true;

    let result = get_quote_results(&trade, &trader, &signer, None, &orderbook)
        .await
        .expect("quote with trade-level partial-fill flag should succeed");
    let request = orderbook
        .state()
        .quote_requests
        .last()
        .cloned()
        .expect("quote request recorded");

    assert!(request.partially_fillable);
    assert!(result.trade_parameters.partially_fillable);
    assert!(result.order_to_sign.partially_fillable);
}

#[tokio::test]
async fn quote_helpers_reject_injected_orderbook_chain_conflicts() {
    let orderbook = MockOrderbook::new(
        cow_sdk_core::SupportedChainId::Sepolia,
        sell_quote_response(),
    );
    let quoter = QuoterParameters::new(
        cow_sdk_core::SupportedChainId::Mainnet,
        "0x007",
        address(OWNER),
    )
    .expect("app code should validate")
    .with_env(CowEnv::Prod);
    let trade = sample_trade_parameters(OrderKind::Sell);

    let error = get_quote_only(&trade, &quoter, None, &orderbook)
        .await
        .expect_err("mismatched quoter chain must fail before quoting");

    assert!(matches!(
        error,
        cow_sdk_trading::TradingError::InjectedOrderbookContextConflict {
            field: "chainId",
            ..
        }
    ));
    assert!(orderbook.state().quote_requests.is_empty());
}

#[tokio::test]
async fn quote_results_capture_originating_orderbook_runtime_binding() {
    let orderbook = MockOrderbook::new_with_base_url(
        cow_sdk_core::SupportedChainId::Sepolia,
        CowEnv::Prod,
        "https://quotes.cow.test",
        sell_quote_response(),
    );
    let signer = MockSigner::default();
    let trader =
        cow_sdk_trading::TraderParameters::new(cow_sdk_core::SupportedChainId::Sepolia, "0x007")
            .expect("app code should validate")
            .with_env(CowEnv::Prod);
    let trade = sample_trade_parameters(OrderKind::Sell);

    let result = get_quote_results(&trade, &trader, &signer, None, &orderbook)
        .await
        .expect("quote with explicit base url should succeed");
    let binding = result
        .orderbook_binding
        .expect("quote results must retain the originating orderbook binding");

    assert_eq!(binding.chain_id, cow_sdk_core::SupportedChainId::Sepolia);
    assert_eq!(binding.env, CowEnv::Prod);
    assert_eq!(
        binding.resolved_base_url.as_deref(),
        Some("https://quotes.cow.test")
    );
}

#[tokio::test]
async fn order_id_collision_retries_with_new_salt_until_success_or_cap() {
    let chain_id = SupportedChainId::Sepolia;
    let order = UnsignedOrder::new(
        address(cow_sdk_core::EVM_NATIVE_CURRENCY_ADDRESS),
        address(crate::common::COW),
        address(OWNER),
        Amount::new("1000000000000000000").expect("test sell amount literal must be valid"),
        Amount::new("3").expect("test buy amount literal must be valid"),
        MAX_VALID_TO_EPOCH,
        app_data_hash(),
        Amount::ZERO,
        OrderKind::Sell,
        false,
        SellTokenSource::Erc20,
        BuyTokenDestination::Erc20,
    );
    let checker = MockEthFlowChecker {
        results: Arc::new(Mutex::new(vec![true, true, false])),
    };

    let generated = calculate_unique_order_id(
        chain_id,
        &order,
        Some(&checker),
        Some(&ProtocolOptions::new().with_env(CowEnv::Prod)),
    )
    .await
    .expect("collision retry must eventually produce a free order id");

    let mut expected_order = order;
    expected_order.sell_token = wrapped_native_token(chain_id).address;
    expected_order.buy_amount =
        Amount::new("1").expect("second collision retry must decrement buy amount twice");
    let expected = cow_sdk_signing::generate_order_id(
        chain_id,
        &expected_order,
        &cow_sdk_contracts::Registry::default()
            .address(
                cow_sdk_contracts::ContractId::EthFlow,
                chain_id,
                CowEnv::Prod,
            )
            .expect("canonical EthFlow address must stay registered"),
        Some(&ProtocolOptions::new().with_env(CowEnv::Prod)),
    )
    .expect("expected retried order id must generate");

    assert_eq!(generated.order_id, expected.order_id);
    assert_eq!(generated.order_digest, expected.order_digest);
}

#[tokio::test]
async fn quote_results_apply_advanced_owner_validity_slippage_and_partner_fee_precedence() {
    let orderbook = MockOrderbook::new(
        cow_sdk_core::SupportedChainId::Sepolia,
        sell_quote_response(),
    );
    let signer = MockSigner::default();
    let trader =
        cow_sdk_trading::TraderParameters::new(cow_sdk_core::SupportedChainId::Sepolia, "0x007")
            .expect("app code should validate")
            .with_env(CowEnv::Prod);
    let mut trade: TradeParameters = sample_trade_parameters(OrderKind::Sell);
    trade.owner = None;
    trade.slippage_bps = None;
    let advanced = TradeAdvancedSettings::new()
        .with_quote_request(
            QuoteRequestOverride::new()
                .with_from(address(crate::common::ALT_RECEIVER))
                .with_receiver(address(crate::common::ALT_RECEIVER))
                .with_valid_to(5_600_000)
                .with_partially_fillable(true),
        )
        .with_app_data(
            cow_sdk_app_data::AppDataParams::default().with_metadata(
                serde_json::from_value(serde_json::json!({
                    "quote": {
                        "slippageBips": 77
                    },
                    "partnerFee": {
                        "volumeBps": 42,
                        "recipient": crate::common::ALT_RECEIVER
                    }
                }))
                .expect("advanced app-data metadata must deserialize"),
            ),
        );

    let result = get_quote_results(&trade, &trader, &signer, Some(&advanced), &orderbook)
        .await
        .expect("quote with advanced precedence should succeed");
    let request = orderbook
        .state()
        .quote_requests
        .last()
        .cloned()
        .expect("quote request must be recorded");
    let app_data: serde_json::Value = serde_json::from_str(&result.app_data_info.full_app_data)
        .expect("advanced app data must remain valid json");

    assert_eq!(
        result.trade_parameters.owner,
        Some(address(crate::common::ALT_RECEIVER))
    );
    assert_eq!(request.from, address(crate::common::ALT_RECEIVER));
    assert_eq!(
        result.trade_parameters.receiver,
        Some(address(crate::common::ALT_RECEIVER))
    );
    assert_eq!(
        result.order_to_sign.receiver,
        address(crate::common::ALT_RECEIVER)
    );
    assert_eq!(result.trade_parameters.valid_to, Some(5_600_000));
    assert_eq!(
        request.validity,
        cow_sdk_orderbook::QuoteValidity::ValidTo(5_600_000)
    );
    assert_eq!(result.order_to_sign.valid_to, 5_600_000);
    assert_eq!(result.trade_parameters.slippage_bps, Some(77));
    assert_eq!(
        result.trade_parameters.partner_fee,
        Some(
            PartnerFeePolicy::volume(42, address(crate::common::ALT_RECEIVER))
                .expect("volume policy must validate")
                .into(),
        )
    );
    assert!(result.trade_parameters.partially_fillable);
    assert!(result.order_to_sign.partially_fillable);
    assert_eq!(
        app_data["metadata"]["partnerFee"]["volumeBps"],
        serde_json::json!(42)
    );
}

#[tokio::test]
async fn quote_results_reject_invalid_partner_fee_metadata_before_quoting() {
    let orderbook = MockOrderbook::new(
        cow_sdk_core::SupportedChainId::Sepolia,
        sell_quote_response(),
    );
    let signer = MockSigner::default();
    let trader =
        cow_sdk_trading::TraderParameters::new(cow_sdk_core::SupportedChainId::Sepolia, "0x007")
            .expect("app code should validate")
            .with_env(CowEnv::Prod);
    let trade: TradeParameters = sample_trade_parameters(OrderKind::Sell);
    let advanced = TradeAdvancedSettings::new().with_app_data(
        cow_sdk_app_data::AppDataParams::default().with_metadata(
            serde_json::from_value(serde_json::json!({
                "partnerFee": {
                    "unexpected": true
                }
            }))
            .expect("invalid metadata shape should still deserialize as json"),
        ),
    );

    let error = get_quote_results(&trade, &trader, &signer, Some(&advanced), &orderbook)
        .await
        .expect_err("invalid partner-fee metadata must fail before quote transport");

    let rendered = error.to_string();
    assert!(
        rendered.contains("appData.metadata.partnerFee"),
        "error text must name the offending field, got: {rendered}"
    );
    assert!(
        rendered.contains("partner-fee schema"),
        "error text must describe the shape violation, got: {rendered}"
    );
    assert!(orderbook.state().quote_requests.is_empty());
}

#[tokio::test]
async fn quote_request_validation_runs_before_orderbook_transport() {
    let orderbook = MockOrderbook::new(
        cow_sdk_core::SupportedChainId::Sepolia,
        sell_quote_response(),
    );
    let signer = MockSigner::default();
    let trader =
        cow_sdk_trading::TraderParameters::new(cow_sdk_core::SupportedChainId::Sepolia, "0x007")
            .expect("app code should validate")
            .with_env(CowEnv::Prod);
    let trade: TradeParameters = sample_trade_parameters(OrderKind::Sell);
    let advanced = TradeAdvancedSettings::new().with_quote_request(
        QuoteRequestOverride::new()
            .with_signing_scheme(SigningScheme::Eip712)
            .with_onchain_order(true),
    );

    let error = get_quote_results(&trade, &trader, &signer, Some(&advanced), &orderbook)
        .await
        .expect_err("incompatible quote signing pair must fail before transport");

    assert!(matches!(
        error,
        cow_sdk_trading::TradingError::Orderbook(
            cow_sdk_orderbook::OrderbookError::IncompatibleSigningScheme {
                signing_scheme: SigningScheme::Eip712,
                onchain_order: true,
            }
        )
    ));
    assert!(orderbook.state().quote_requests.is_empty());
}

#[test]
fn trade_parameters_validate_rejects_zero_address_partner_fee_recipient() {
    let trade = sample_trade_parameters(OrderKind::Sell).with_partner_fee(
        PartnerFeePolicy::Volume {
            volume_bps: 42,
            recipient: address("0x0000000000000000000000000000000000000000"),
        }
        .into(),
    );

    let error = trade
        .validate()
        .expect_err("zero-address partner fee recipient must fail locally");

    assert!(matches!(
        error,
        ClientRejection::InvalidPartnerFee {
            field: "partnerFee.recipient",
            reason: ValidationReason::Precondition { .. },
        }
    ));
}

#[tokio::test]
async fn quote_results_reject_zero_address_partner_fee_before_quoting() {
    let orderbook = MockOrderbook::new(
        cow_sdk_core::SupportedChainId::Sepolia,
        sell_quote_response(),
    );
    let signer = MockSigner::default();
    let trader =
        cow_sdk_trading::TraderParameters::new(cow_sdk_core::SupportedChainId::Sepolia, "0x007")
            .expect("app code should validate")
            .with_env(CowEnv::Prod);
    let trade = sample_trade_parameters(OrderKind::Sell).with_partner_fee(
        PartnerFeePolicy::Volume {
            volume_bps: 42,
            recipient: address("0x0000000000000000000000000000000000000000"),
        }
        .into(),
    );

    let error = get_quote_results(&trade, &trader, &signer, None, &orderbook)
        .await
        .expect_err("zero-address partner fee recipient must fail before quote transport");

    assert!(matches!(
        error,
        cow_sdk_trading::TradingError::AppData(cow_sdk_app_data::AppDataError::InvalidPartnerFee {
            field: "partnerFee.recipient",
            reason: ValidationReason::Precondition { .. },
        })
    ));
    assert!(orderbook.state().quote_requests.is_empty());
}

#[tokio::test]
async fn build_app_data_injects_default_utm_when_override_absent() {
    const EXPECTED_UTM_SOURCE: &str = "cow-sdk";
    const EXPECTED_UTM_CAMPAIGN: &str = "developer-cohort";
    const EXPECTED_UTM_CONTENT: &str = "";
    const EXPECTED_UTM_TERM: &str = "rs";
    const EXPECTED_UTM_MEDIUM_PREFIX: &str = "cow-rs@";

    let info = build_app_data(&test_app_code(), 50, "market", None, None)
        .await
        .expect("default-utm build_app_data must succeed");
    let doc: serde_json::Value = serde_json::from_str(&info.full_app_data)
        .expect("sealed app-data document must remain valid json");
    let utm = doc
        .get("metadata")
        .and_then(|metadata| metadata.get("utm"))
        .expect("default path must stamp a metadata.utm block");

    assert_eq!(
        utm["utmSource"].as_str(),
        Some(EXPECTED_UTM_SOURCE),
        "default utmSource must match the local attribution policy",
    );
    assert_eq!(
        utm["utmCampaign"].as_str(),
        Some(EXPECTED_UTM_CAMPAIGN),
        "default utmCampaign must match the local attribution policy",
    );
    assert_eq!(
        utm["utmContent"].as_str(),
        Some(EXPECTED_UTM_CONTENT),
        "default utmContent must match the local attribution policy",
    );
    assert_eq!(
        utm["utmTerm"].as_str(),
        Some(EXPECTED_UTM_TERM),
        "default utmTerm must match the local attribution policy",
    );
    let utm_medium = utm["utmMedium"]
        .as_str()
        .expect("default utmMedium must be a string");
    assert!(
        utm_medium.starts_with(EXPECTED_UTM_MEDIUM_PREFIX),
        "default utmMedium must start with {EXPECTED_UTM_MEDIUM_PREFIX:?}, got {utm_medium:?}",
    );
    assert!(
        utm_medium.len() > EXPECTED_UTM_MEDIUM_PREFIX.len(),
        "default utmMedium must embed a non-empty crate version after the prefix, got {utm_medium:?}",
    );
}

#[tokio::test]
async fn default_utm_block_uses_env_cargo_pkg_version() {
    let info = build_app_data(&test_app_code(), 50, "market", None, None)
        .await
        .expect("default app-data construction must succeed");
    let doc: serde_json::Value =
        serde_json::from_str(&info.full_app_data).expect("full app data must remain valid json");
    let utm_medium = doc["metadata"]["utm"]["utmMedium"]
        .as_str()
        .expect("default UTM medium must be a string");

    assert_eq!(
        utm_medium,
        format!("cow-rs@{}", env!("CARGO_PKG_VERSION")),
        "default UTM medium must embed the trading crate version exactly",
    );
}

#[tokio::test]
async fn build_app_data_respects_full_utm_override() {
    let override_params = cow_sdk_app_data::AppDataParams::default().with_metadata(
        serde_json::from_value(serde_json::json!({
            "utm": {
                "utmSource": "custom",
                "utmMedium": "custom",
                "utmCampaign": "custom",
                "utmContent": "custom",
                "utmTerm": "custom"
            }
        }))
        .expect("full-utm override metadata must deserialize"),
    );

    let info = build_app_data(&test_app_code(), 50, "market", None, Some(&override_params))
        .await
        .expect("full-utm-override build_app_data must succeed");
    let doc: serde_json::Value = serde_json::from_str(&info.full_app_data)
        .expect("sealed app-data document must remain valid json");
    let utm = doc
        .get("metadata")
        .and_then(|metadata| metadata.get("utm"))
        .cloned()
        .expect("override path must preserve the caller-supplied metadata.utm block");

    assert_eq!(
        utm,
        serde_json::json!({
            "utmSource": "custom",
            "utmMedium": "custom",
            "utmCampaign": "custom",
            "utmContent": "custom",
            "utmTerm": "custom",
        }),
        "caller-supplied full metadata.utm must be carried through byte-identical with no Rust-injected defaults",
    );
}

#[tokio::test]
async fn build_app_data_respects_partial_utm_override() {
    let override_params = cow_sdk_app_data::AppDataParams::default().with_metadata(
        serde_json::from_value(serde_json::json!({
            "utm": {
                "utmTerm": "xyz"
            }
        }))
        .expect("partial-utm override metadata must deserialize"),
    );

    let info = build_app_data(&test_app_code(), 50, "market", None, Some(&override_params))
        .await
        .expect("partial-utm-override build_app_data must succeed");
    let doc: serde_json::Value = serde_json::from_str(&info.full_app_data)
        .expect("sealed app-data document must remain valid json");
    let utm = doc
        .get("metadata")
        .and_then(|metadata| metadata.get("utm"))
        .cloned()
        .expect("override path must preserve the caller-supplied metadata.utm block");

    assert_eq!(
        utm,
        serde_json::json!({
            "utmTerm": "xyz",
        }),
        "partial caller-supplied metadata.utm must stay partial; Rust defaults must not be merged on top",
    );
}
