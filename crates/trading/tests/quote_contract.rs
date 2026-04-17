mod common;

use std::sync::Arc;

use cow_sdk_core::{CowEnv, OrderBalance};
use cow_sdk_orderbook::OrderKind;
use cow_sdk_signing::ORDER_PRIMARY_TYPE;
use cow_sdk_trading::{
    PartnerFeePolicy, QuoteRequestOverride, QuoterParameters, SwapAdvancedSettings,
    TradeParameters, get_quote_only, get_quote_results,
};

use crate::common::{
    MockOrderbook, MockSigner, MockSlippageProvider, OWNER, address, sample_trade_parameters,
    sell_quote_response, trading_fixture,
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
        cow_sdk_trading::TraderParameters::new(cow_sdk_core::SupportedChainId::Sepolia, "0x007");

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
    assert_eq!(default_request.valid_for, Some(1_800));
    assert_eq!(default_request.valid_to, None);

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
    assert_eq!(exact_request.valid_to, Some(2_524_608_000));
    assert_eq!(exact_request.valid_for, None);

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
        cow_sdk_trading::TraderParameters::new(cow_sdk_core::SupportedChainId::Sepolia, "0x007");
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
        cow_sdk_orderbook::SigningScheme::Eip1271
    );
    assert!(request.onchain_order);
    assert_eq!(request.verification_gas_limit, Some(0));
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
    );
    let mut trade = sample_trade_parameters(OrderKind::Sell);
    trade.owner = Some(address(OWNER));
    trade.slippage_bps = None;
    let advanced =
        SwapAdvancedSettings::new().with_slippage_suggester(Arc::new(MockSlippageProvider {
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
        cow_sdk_trading::TraderParameters::new(cow_sdk_core::SupportedChainId::Sepolia, "0x007");
    let trade: TradeParameters = sample_trade_parameters(OrderKind::Sell);
    let advanced = SwapAdvancedSettings::new().with_quote_request(
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
    quote_response.quote.sell_token_balance = OrderBalance::External;
    quote_response.quote.buy_token_balance = OrderBalance::Internal;
    let orderbook = MockOrderbook::new(cow_sdk_core::SupportedChainId::Sepolia, quote_response);
    let signer = MockSigner::default();
    let trader =
        cow_sdk_trading::TraderParameters::new(cow_sdk_core::SupportedChainId::Sepolia, "0x007");
    let trade: TradeParameters = sample_trade_parameters(OrderKind::Sell);
    let advanced = SwapAdvancedSettings::new().with_quote_request(
        QuoteRequestOverride::new()
            .with_sell_token_balance(OrderBalance::External)
            .with_buy_token_balance(OrderBalance::Internal),
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

    assert_eq!(request.sell_token_balance, OrderBalance::External);
    assert_eq!(request.buy_token_balance, OrderBalance::Internal);
    assert_eq!(
        result.trade_parameters.sell_token_balance,
        OrderBalance::External
    );
    assert_eq!(
        result.trade_parameters.buy_token_balance,
        OrderBalance::Internal
    );
    assert_eq!(
        result.quote_response.quote.sell_token_balance,
        OrderBalance::External
    );
    assert_eq!(
        result.quote_response.quote.buy_token_balance,
        OrderBalance::Internal
    );
    assert_eq!(
        result.order_to_sign.sell_token_balance,
        OrderBalance::External
    );
    assert_eq!(
        result.order_to_sign.buy_token_balance,
        OrderBalance::Internal
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
        cow_sdk_trading::TraderParameters::new(cow_sdk_core::SupportedChainId::Sepolia, "0x007");
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
async fn quote_results_apply_advanced_owner_validity_slippage_and_partner_fee_precedence() {
    let orderbook = MockOrderbook::new(
        cow_sdk_core::SupportedChainId::Sepolia,
        sell_quote_response(),
    );
    let signer = MockSigner::default();
    let trader =
        cow_sdk_trading::TraderParameters::new(cow_sdk_core::SupportedChainId::Sepolia, "0x007")
            .with_env(CowEnv::Prod);
    let mut trade: TradeParameters = sample_trade_parameters(OrderKind::Sell);
    trade.owner = None;
    trade.slippage_bps = None;
    let advanced = SwapAdvancedSettings::new()
        .with_quote_request(
            QuoteRequestOverride::new()
                .with_from(address(crate::common::ALT_RECEIVER))
                .with_receiver(address(crate::common::ALT_RECEIVER))
                .with_valid_to(5_600_000)
                .with_partially_fillable(true),
        )
        .with_app_data(cow_sdk_app_data::AppDataParams {
            app_code: None,
            environment: None,
            metadata: serde_json::from_value(serde_json::json!({
                "quote": {
                    "slippageBips": 77
                },
                "partnerFee": {
                    "volumeBps": 42,
                    "recipient": crate::common::ALT_RECEIVER
                }
            }))
            .expect("advanced app-data metadata must deserialize"),
        });

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
    assert_eq!(request.valid_to, Some(5_600_000));
    assert_eq!(result.order_to_sign.valid_to, 5_600_000);
    assert_eq!(result.trade_parameters.slippage_bps, Some(77));
    assert_eq!(
        result.trade_parameters.partner_fee,
        Some(PartnerFeePolicy::volume(42, address(crate::common::ALT_RECEIVER)).into())
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
            .with_env(CowEnv::Prod);
    let trade: TradeParameters = sample_trade_parameters(OrderKind::Sell);
    let advanced = SwapAdvancedSettings::new().with_app_data(cow_sdk_app_data::AppDataParams {
        app_code: None,
        environment: None,
        metadata: serde_json::from_value(serde_json::json!({
            "partnerFee": {
                "unexpected": true
            }
        }))
        .expect("invalid metadata shape should still deserialize as json"),
    });

    let error = get_quote_results(&trade, &trader, &signer, Some(&advanced), &orderbook)
        .await
        .expect_err("invalid partner-fee metadata must fail before quote transport");

    assert!(
        error
            .to_string()
            .contains("appData.metadata.partnerFee must match the partner-fee schema")
    );
    assert!(orderbook.state().quote_requests.is_empty());
}
