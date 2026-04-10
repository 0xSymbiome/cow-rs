mod common;

use std::sync::{Arc, Mutex};

use serde_json::json;

use cow_sdk_core::{EVM_NATIVE_CURRENCY_ADDRESS, OrderKind};
use cow_sdk_trading::{
    LimitOrderAdvancedSettings, LimitTradeParameters, PostTradeAdditionalParams,
    QuoteRequestOverride, SwapAdvancedSettings, build_app_data, post_limit_order,
    post_limit_order_async, post_sell_native_currency_order, post_swap_order,
};

use crate::common::{
    ALT_RECEIVER, MockEip1271Provider, MockEthFlowChecker, MockOrderbook, MockSigner, OWNER,
    address, buy_quote_response, sample_limit_parameters, sample_trade_parameters,
    sample_trader_parameters, sell_quote_response, trading_fixture,
};

#[tokio::test]
async fn swap_posting_matches_pinned_sell_and_buy_adjustment_vectors() {
    let fixture = trading_fixture();
    let sell_case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "trading-sell-order-amount-adjustment")
        .unwrap();
    let buy_case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "trading-buy-order-amount-adjustment")
        .unwrap();

    let trader = sample_trader_parameters();
    let signer = MockSigner::default();

    let sell_orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let sell_trade = sample_trade_parameters(OrderKind::Sell);
    let sell_result = post_swap_order(&sell_trade, &trader, &signer, None, &sell_orderbook)
        .await
        .expect("sell swap order should succeed");
    let sell_order = sell_orderbook
        .state()
        .sent_orders
        .last()
        .cloned()
        .expect("sell order must be recorded");

    assert_eq!(
        sell_order.sell_amount,
        sell_case["expected"]["sell_amount"].as_str().unwrap()
    );
    assert_eq!(
        sell_order.buy_amount,
        sell_case["expected"]["buy_amount"].as_str().unwrap()
    );
    assert_eq!(
        sell_result.order_to_sign.sell_amount,
        sell_order.sell_amount
    );
    assert_eq!(sell_result.order_to_sign.buy_amount, sell_order.buy_amount);

    let buy_orderbook = MockOrderbook::new(trader.chain_id, buy_quote_response());
    let buy_trade = sample_trade_parameters(OrderKind::Buy);
    let buy_result = post_swap_order(&buy_trade, &trader, &signer, None, &buy_orderbook)
        .await
        .expect("buy swap order should succeed");
    let buy_order = buy_orderbook
        .state()
        .sent_orders
        .last()
        .cloned()
        .expect("buy order must be recorded");

    assert_eq!(
        buy_order.sell_amount,
        buy_case["expected"]["sell_amount"].as_str().unwrap()
    );
    assert_eq!(
        buy_order.buy_amount,
        buy_case["expected"]["buy_amount"].as_str().unwrap()
    );
    assert_eq!(buy_result.order_to_sign.sell_amount, buy_order.sell_amount);
    assert_eq!(buy_result.order_to_sign.buy_amount, buy_order.buy_amount);
}

#[tokio::test]
async fn posting_propagates_partner_fee_receiver_valid_to_and_owner_precedence() {
    let trader = sample_trader_parameters();
    let orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let signer = MockSigner::new(address(ALT_RECEIVER));
    let mut trade = sample_trade_parameters(OrderKind::Sell);
    trade.owner = Some(address(OWNER));
    trade.partner_fee = Some(json!({
        "volumeBps": 50,
        "recipient": ALT_RECEIVER
    }));

    let advanced = SwapAdvancedSettings {
        quote_request: Some(QuoteRequestOverride {
            receiver: Some(address(ALT_RECEIVER)),
            valid_to: Some(5_600_000),
            ..QuoteRequestOverride::default()
        }),
        app_data: Some(cow_sdk_app_data::AppDataParams {
            app_code: None,
            environment: None,
            metadata: serde_json::from_value(json!({
                "partnerFee": {
                    "volumeBps": 50,
                    "recipient": ALT_RECEIVER
                }
            }))
            .expect("partner fee metadata should build"),
        }),
        ..SwapAdvancedSettings::default()
    };

    let result = post_swap_order(&trade, &trader, &signer, Some(&advanced), &orderbook)
        .await
        .expect("swap order with overrides should succeed");
    let state = orderbook.state();
    let order = state
        .sent_orders
        .last()
        .cloned()
        .expect("order must be sent");
    let uploaded = state
        .uploads
        .last()
        .cloned()
        .expect("app data must be uploaded");
    let uploaded_json: serde_json::Value =
        serde_json::from_str(&uploaded.1).expect("uploaded app data must remain valid json");

    assert_eq!(order.receiver, Some(address(ALT_RECEIVER)));
    assert_eq!(order.valid_to, 5_600_000);
    assert_eq!(order.from, address(OWNER));
    assert_eq!(result.order_to_sign.receiver, address(ALT_RECEIVER));
    assert_eq!(result.order_to_sign.valid_to, 5_600_000);
    assert_eq!(
        uploaded_json["metadata"]["partnerFee"]["volumeBps"],
        serde_json::json!(50)
    );
}

#[tokio::test]
async fn limit_posting_disables_cost_slippage_adjustments_for_sell_and_buy_orders() {
    let trader = sample_trader_parameters();
    let signer = MockSigner::default();

    let sell_orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let sell_params = sample_limit_parameters(OrderKind::Sell);
    let sell_result = post_limit_order(&sell_params, &trader, &signer, None, &sell_orderbook)
        .await
        .expect("sell limit order should succeed");
    let sell_sent = sell_orderbook
        .state()
        .sent_orders
        .last()
        .cloned()
        .expect("sell limit order must be sent");

    assert_eq!(sell_result.order_to_sign.buy_amount, sell_params.buy_amount);
    assert_eq!(sell_sent.buy_amount, sell_params.buy_amount);

    let buy_orderbook = MockOrderbook::new(trader.chain_id, buy_quote_response());
    let buy_params = sample_limit_parameters(OrderKind::Buy);
    let buy_result = post_limit_order(&buy_params, &trader, &signer, None, &buy_orderbook)
        .await
        .expect("buy limit order should succeed");
    let buy_sent = buy_orderbook
        .state()
        .sent_orders
        .last()
        .cloned()
        .expect("buy limit order must be sent");

    assert_eq!(buy_result.order_to_sign.sell_amount, buy_params.sell_amount);
    assert_eq!(buy_sent.sell_amount, buy_params.sell_amount);
}

#[tokio::test]
async fn limit_posting_sync_signer_wrapper_matches_async_suffix_path() {
    let trader = sample_trader_parameters();
    let signer = MockSigner::default();
    let params = sample_limit_parameters(OrderKind::Sell);

    let wrapper_orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let async_orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());

    let wrapper_result = post_limit_order(&params, &trader, &signer, None, &wrapper_orderbook)
        .await
        .expect("wrapper limit order should succeed");
    let async_result = post_limit_order_async(&params, &trader, &signer, None, &async_orderbook)
        .await
        .expect("async suffix limit order should succeed");

    assert_eq!(wrapper_result.order_to_sign, async_result.order_to_sign);
    assert_eq!(wrapper_result.signature, async_result.signature);
    assert_eq!(wrapper_result.signing_scheme, async_result.signing_scheme);
    assert_eq!(
        wrapper_orderbook.state().sent_orders,
        async_orderbook.state().sent_orders
    );
}

#[tokio::test]
async fn native_sell_post_flow_uploads_app_data_sends_transaction_and_supports_collision_checks() {
    let trader = sample_trader_parameters();
    let orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let signer = MockSigner::default();
    let app_data = build_app_data("0x007", 50, "market", None, None)
        .await
        .expect("app data should build");
    let mut params: LimitTradeParameters = sample_limit_parameters(OrderKind::Sell);
    params.sell_token = address(EVM_NATIVE_CURRENCY_ADDRESS);
    params.quote_id = Some(3);
    params.slippage_bps = Some(50);
    let collision_results = Arc::new(Mutex::new(vec![true, false]));
    let additional = PostTradeAdditionalParams {
        check_eth_flow_order_exists: Some(Arc::new(MockEthFlowChecker {
            results: collision_results.clone(),
        })),
        network_costs_amount: Some(sell_quote_response().quote.fee_amount.clone()),
        custom_eip1271_signature: Some(Arc::new(MockEip1271Provider)),
        ..PostTradeAdditionalParams::default()
    };

    let result = post_sell_native_currency_order(
        &orderbook,
        &app_data,
        &params,
        &additional,
        &trader,
        &signer,
    )
    .await
    .expect("native sell posting should succeed");

    let state = orderbook.state();
    let signer_state = signer.state();
    let remaining = collision_results
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();

    assert_eq!(state.uploads.len(), 1);
    assert_eq!(signer_state.sent_transactions.len(), 1);
    assert!(result.tx_hash.is_some());
    assert!(remaining.is_empty(), "collision callback must be consumed");
}

#[tokio::test]
async fn limit_posting_accepts_custom_eip1271_signatures_without_local_re_signing() {
    let trader = sample_trader_parameters();
    let orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let signer = MockSigner::default();
    let params = sample_limit_parameters(OrderKind::Sell);
    let advanced = LimitOrderAdvancedSettings {
        additional_params: Some(PostTradeAdditionalParams {
            signing_scheme: Some(cow_sdk_orderbook::SigningScheme::Eip1271),
            custom_eip1271_signature: Some(Arc::new(MockEip1271Provider)),
            ..PostTradeAdditionalParams::default()
        }),
        ..LimitOrderAdvancedSettings::default()
    };

    let result = post_limit_order(&params, &trader, &signer, Some(&advanced), &orderbook)
        .await
        .expect("eip1271 limit order should succeed");

    assert_eq!(
        result.signing_scheme,
        cow_sdk_orderbook::SigningScheme::Eip1271
    );
    assert_eq!(result.signature, "0x7e57c0de");
}
