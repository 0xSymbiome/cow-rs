#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::iter_on_single_items,
    clippy::missing_const_for_fn,
    clippy::option_if_let_else,
    clippy::redundant_clone,
    clippy::too_many_lines,
    clippy::unnested_or_patterns,
    reason = "pedantic, nursery, and perf lints acceptable in test helper code"
)]

mod common;

use std::sync::{Arc, Mutex};

use serde_json::json;

use cow_sdk_core::{
    Amount, BuyTokenDestination, EVM_NATIVE_CURRENCY_ADDRESS, HexData, OrderKind, ProtocolOptions,
    SellTokenSource,
};

fn protocol_options_from_trader(trader: &cow_sdk_trading::TraderParameters) -> ProtocolOptions {
    let mut options = ProtocolOptions::new();
    if let Some(env) = trader.env {
        options = options.with_env(env);
    }
    if let Some(overrides) = trader.settlement_contract_override.clone() {
        options = options.with_settlement_contract_override(overrides);
    }
    if let Some(overrides) = trader.eth_flow_contract_override.clone() {
        options = options.with_eth_flow_contract_override(overrides);
    }
    options
}
use cow_sdk_trading::{
    LimitOrderAdvancedSettings, LimitTradeParameters, PartnerFeePolicy, PostTradeAdditionalParams,
    QuoteRequestOverride, SwapAdvancedSettings, build_app_data, get_quote_results,
    post_limit_order, post_limit_order_async, post_sell_native_currency_order, post_swap_order,
    post_swap_order_from_quote,
};

use crate::common::{
    ALT_RECEIVER, CountingSigner, MockEip1271Provider, MockEthFlowChecker, MockOrderbook,
    MockSigner, OWNER, address, buy_quote_response, sample_limit_parameters,
    sample_trade_parameters, sample_trader_parameters, sell_quote_response, trading_fixture,
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
        Amount::new(sell_order.sell_amount.clone()).expect("sell order amount must be valid")
    );
    assert_eq!(
        sell_result.order_to_sign.buy_amount,
        Amount::new(sell_order.buy_amount.clone()).expect("buy order amount must be valid")
    );

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
    assert_eq!(
        buy_result.order_to_sign.sell_amount,
        Amount::new(buy_order.sell_amount.clone()).expect("sell order amount must be valid")
    );
    assert_eq!(
        buy_result.order_to_sign.buy_amount,
        Amount::new(buy_order.buy_amount.clone()).expect("buy order amount must be valid")
    );
}

#[tokio::test]
async fn posting_propagates_partner_fee_receiver_valid_to_and_owner_precedence() {
    let trader = sample_trader_parameters();
    let orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let signer = MockSigner::new(address(ALT_RECEIVER));
    let mut trade = sample_trade_parameters(OrderKind::Sell);
    trade.owner = Some(address(OWNER));
    trade.partner_fee = Some(
        PartnerFeePolicy::volume(50, address(ALT_RECEIVER))
            .expect("volume policy must validate")
            .into(),
    );

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("UNIX_EPOCH must remain reachable")
        .as_secs();
    let override_valid_to = u32::try_from(now + 3600).expect("valid_to must fit in u32");
    let advanced = SwapAdvancedSettings::new()
        .with_quote_request(
            QuoteRequestOverride::new()
                .with_receiver(address(ALT_RECEIVER))
                .with_valid_to(override_valid_to)
                .with_signing_scheme(cow_sdk_orderbook::SigningScheme::Eip1271),
        )
        .with_app_data(cow_sdk_app_data::AppDataParams {
            app_code: None,
            environment: None,
            signer: None,
            flashloan: None,
            metadata: serde_json::from_value(json!({
                "partnerFee": {
                    "volumeBps": 50,
                    "recipient": ALT_RECEIVER
                }
            }))
            .expect("partner fee metadata should build"),
        });

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
    assert_eq!(order.valid_to, override_valid_to);
    assert_eq!(order.from, address(OWNER));
    assert_eq!(result.order_to_sign.receiver, address(ALT_RECEIVER));
    assert_eq!(result.order_to_sign.valid_to, override_valid_to);
    assert_eq!(
        uploaded_json["metadata"]["partnerFee"]["volumeBps"],
        serde_json::json!(50)
    );
}

#[tokio::test]
async fn swap_posting_preserves_non_default_balance_semantics_from_quote_to_submission() {
    let trader = sample_trader_parameters();
    let mut quote_response = sell_quote_response();
    quote_response.quote.sell_token_balance = SellTokenSource::External;
    quote_response.quote.buy_token_balance = BuyTokenDestination::Internal;
    let orderbook = MockOrderbook::new(trader.chain_id, quote_response);
    let signer = MockSigner::default();
    let trade = sample_trade_parameters(OrderKind::Sell);
    let advanced = SwapAdvancedSettings::new().with_quote_request(
        QuoteRequestOverride::new()
            .with_sell_token_balance(SellTokenSource::External)
            .with_buy_token_balance(BuyTokenDestination::Internal),
    );

    let result = post_swap_order(&trade, &trader, &signer, Some(&advanced), &orderbook)
        .await
        .expect("swap posting with non-default balances should succeed");
    let sent_order = orderbook
        .state()
        .sent_orders
        .last()
        .cloned()
        .expect("order must be recorded");

    assert_eq!(
        result.order_to_sign.sell_token_balance,
        SellTokenSource::External
    );
    assert_eq!(
        result.order_to_sign.buy_token_balance,
        BuyTokenDestination::Internal
    );
    assert_eq!(sent_order.sell_token_balance, SellTokenSource::External);
    assert_eq!(sent_order.buy_token_balance, BuyTokenDestination::Internal);
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
    assert_eq!(sell_sent.buy_amount, sell_params.buy_amount.to_string());

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
    assert_eq!(buy_sent.sell_amount, buy_params.sell_amount.to_string());
}

#[tokio::test]
async fn limit_posting_sync_signer_wrapper_matches_async_suffix_path() {
    let trader = sample_trader_parameters();
    let signer = MockSigner::default();
    let mut params = sample_limit_parameters(OrderKind::Sell);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("UNIX_EPOCH must remain reachable")
        .as_secs();
    params.valid_to = Some(u32::try_from(now + 3600).expect("valid_to must fit in u32"));

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
    let additional = PostTradeAdditionalParams::new()
        .with_check_eth_flow_order_exists(Arc::new(MockEthFlowChecker {
            results: collision_results.clone(),
        }))
        .with_network_costs_amount(
            Amount::new(sell_quote_response().quote.network_cost_amount().to_owned())
                .expect("quote fee amount must be valid"),
        )
        .with_custom_eip1271_signature(Arc::new(MockEip1271Provider));

    let result = post_sell_native_currency_order(
        &orderbook,
        &app_data,
        &params,
        &additional,
        &trader,
        &signer,
        cow_sdk_trading::OrderValidityBounds::SERVICES_DEFAULT,
        None,
    )
    .await
    .expect("native sell posting should succeed");

    let state = orderbook.state();
    let signer_state = signer.state();
    let remaining = collision_results
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
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
    let advanced = LimitOrderAdvancedSettings::new().with_additional_params(
        PostTradeAdditionalParams::new()
            .with_signing_scheme(cow_sdk_orderbook::SigningScheme::Eip1271)
            .with_custom_eip1271_signature(Arc::new(MockEip1271Provider)),
    );

    let result = post_limit_order(&params, &trader, &signer, Some(&advanced), &orderbook)
        .await
        .expect("eip1271 limit order should succeed");

    assert_eq!(
        result.signing_scheme,
        cow_sdk_orderbook::SigningScheme::Eip1271
    );
    assert_eq!(result.signature, "0x7e57c0de");
}

#[tokio::test]
async fn recoverable_limit_posting_rejects_owner_signer_mismatch_before_upload_or_submission() {
    let trader = sample_trader_parameters();
    let orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let signer = MockSigner::new(address(ALT_RECEIVER));
    let params = sample_limit_parameters(OrderKind::Sell);

    let error = post_limit_order(&params, &trader, &signer, None, &orderbook)
        .await
        .expect_err("recoverable signing must reject explicit owner and signer mismatches");

    assert!(matches!(
        error,
        cow_sdk_trading::TradingError::ClientRejected(
            cow_sdk_trading::ClientRejection::OwnerMismatch { .. },
        )
    ));
    assert!(orderbook.state().uploads.is_empty());
    assert!(orderbook.state().sent_orders.is_empty());
    assert!(signer.state().last_typed_data_domain.is_none());
}

#[tokio::test]
async fn post_swap_order_appdata_from_mismatch_does_not_upload_or_sign() {
    let trader = sample_trader_parameters();
    let orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let signer = CountingSigner::new(address(OWNER));
    let params = sample_limit_parameters(OrderKind::Sell);
    let advanced =
        LimitOrderAdvancedSettings::new().with_app_data(cow_sdk_app_data::AppDataParams {
            signer: Some(address(ALT_RECEIVER)),
            ..Default::default()
        });

    let error = post_limit_order_async(&params, &trader, &signer, Some(&advanced), &orderbook)
        .await
        .expect_err("mismatched app-data signer must reject before upload or signing");

    assert!(matches!(
        error,
        cow_sdk_trading::TradingError::ClientRejected(
            cow_sdk_trading::ClientRejection::AppdataFromMismatch { .. },
        )
    ));
    assert!(orderbook.state().uploads.is_empty());
    assert!(orderbook.state().sent_orders.is_empty());
    assert_eq!(signer.sign_calls(), 0);
}

#[tokio::test]
async fn post_swap_order_same_buy_sell_token_does_not_upload_or_sign() {
    let trader = sample_trader_parameters();
    let orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let signer = CountingSigner::new(address(OWNER));
    let mut params = sample_limit_parameters(OrderKind::Sell);
    params.buy_token = params.sell_token.clone();

    let error = post_limit_order_async(&params, &trader, &signer, None, &orderbook)
        .await
        .expect_err("same-token limit order must reject before upload or signing");

    assert!(matches!(
        error,
        cow_sdk_trading::TradingError::ClientRejected(
            cow_sdk_trading::ClientRejection::SameBuyAndSellToken { .. },
        )
    ));
    assert!(orderbook.state().uploads.is_empty());
    assert!(orderbook.state().sent_orders.is_empty());
    assert_eq!(signer.sign_calls(), 0);
}

#[tokio::test]
async fn post_swap_order_zero_amount_does_not_upload_or_sign() {
    let trader = sample_trader_parameters();
    let orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let signer = CountingSigner::new(address(OWNER));
    let mut params = sample_limit_parameters(OrderKind::Sell);
    params.sell_amount = Amount::zero();

    let error = post_limit_order_async(&params, &trader, &signer, None, &orderbook)
        .await
        .expect_err("zero-amount limit order must reject before upload or signing");

    assert!(matches!(
        error,
        cow_sdk_trading::TradingError::ClientRejected(
            cow_sdk_trading::ClientRejection::ZeroAmount { side: _ },
        )
    ));
    assert!(orderbook.state().uploads.is_empty());
    assert!(orderbook.state().sent_orders.is_empty());
    assert_eq!(signer.sign_calls(), 0);
}

#[tokio::test]
async fn limit_posting_rejects_trader_env_conflicts_with_orderbook_context() {
    let mut trader = sample_trader_parameters();
    trader.env = Some(cow_sdk_core::CowEnv::Staging);
    let orderbook = MockOrderbook::new_with_env(
        trader.chain_id,
        cow_sdk_core::CowEnv::Prod,
        sell_quote_response(),
    );
    let signer = MockSigner::default();
    let mut params = sample_limit_parameters(OrderKind::Sell);
    params.env = None;

    let error = post_limit_order(&params, &trader, &signer, None, &orderbook)
        .await
        .expect_err("mismatched trader env must fail before signing or submission");

    assert!(matches!(
        error,
        cow_sdk_trading::TradingError::InjectedOrderbookContextConflict { field: "env", .. }
    ));
    assert!(signer.state().last_typed_data_domain.is_none());
    assert!(orderbook.state().sent_orders.is_empty());
}

#[tokio::test]
async fn post_from_quote_reuses_matching_orderbook_binding_and_submits_order() {
    let trader = sample_trader_parameters();
    let signer = MockSigner::default();
    let orderbook = MockOrderbook::new_with_base_url(
        trader.chain_id,
        cow_sdk_core::CowEnv::Prod,
        "https://quotes.cow.test",
        sell_quote_response(),
    );
    let trade = sample_trade_parameters(OrderKind::Sell);

    let quote_results = get_quote_results(&trade, &trader, &signer, None, &orderbook)
        .await
        .expect("quote flow should succeed");
    let result = post_swap_order_from_quote(&quote_results, &trader, &signer, None, &orderbook)
        .await
        .expect("post-from-quote should succeed when the orderbook binding matches");

    assert_eq!(result.order_id, crate::common::order_uid());
    assert_eq!(orderbook.state().sent_orders.len(), 1);
}

#[tokio::test]
async fn post_from_quote_rejects_orderbook_binding_mismatch_before_signing_or_submission() {
    let trader = sample_trader_parameters();
    let signer = MockSigner::default();
    let quoting_orderbook = MockOrderbook::new_with_base_url(
        trader.chain_id,
        cow_sdk_core::CowEnv::Prod,
        "https://quotes.cow.test",
        sell_quote_response(),
    );
    let posting_orderbook = MockOrderbook::new_with_base_url(
        trader.chain_id,
        cow_sdk_core::CowEnv::Prod,
        "https://submit.cow.test",
        sell_quote_response(),
    );
    let trade = sample_trade_parameters(OrderKind::Sell);

    let quote_results = get_quote_results(&trade, &trader, &signer, None, &quoting_orderbook)
        .await
        .expect("quote flow should succeed");
    let error =
        post_swap_order_from_quote(&quote_results, &trader, &signer, None, &posting_orderbook)
            .await
            .expect_err("mismatched orderbook binding must fail before signing or submission");

    assert!(matches!(
        error,
        cow_sdk_trading::TradingError::QuoteOrderbookBindingConflict {
            field: "baseUrl",
            ..
        }
    ));
    assert!(signer.state().last_typed_data_domain.is_none());
    assert!(posting_orderbook.state().sent_orders.is_empty());
}

#[tokio::test]
async fn async_order_level_eip1271_verification_is_explicit_and_reuses_contract_helpers() {
    let trader = sample_trader_parameters();
    let params = sample_limit_parameters(OrderKind::Sell);
    let app_data = build_app_data("0x007", 50, "limit", None, None)
        .await
        .expect("app data should build");
    let order_to_sign = cow_sdk_trading::get_order_to_sign(
        cow_sdk_trading::OrderToSignParams::new(trader.chain_id, address(OWNER), false)
            .with_apply_costs_slippage_and_fees(false),
        &params,
        &app_data.app_data_keccak256,
    )
    .expect("order to sign should build");
    let provider = crate::common::MockProvider::default();
    let verifier = address(OWNER);
    provider.set_code(&verifier, "0x6001600055");
    provider.set_contract_response("isValidSignature", "\"0x1626ba7e\"");

    cow_sdk_trading::post::verify_eip1271_order_signature_async(
        &provider,
        &order_to_sign,
        trader.chain_id,
        &cow_sdk_trading::types::Eip1271VerificationParameters {
            verifier: verifier.clone(),
            signature: HexData::new("0x7e57c0de").unwrap(),
        },
        Some(&protocol_options_from_trader(&trader)),
    )
    .await
    .expect("verification should succeed");

    let call = provider
        .state()
        .last_contract_call
        .expect("verification call recorded");
    assert_eq!(call.address, verifier);
    assert_eq!(call.method, "isValidSignature");
}

#[tokio::test]
async fn order_level_eip1271_verification_surfaces_contract_failures_explicitly() {
    let trader = sample_trader_parameters();
    let params = sample_limit_parameters(OrderKind::Sell);
    let app_data = build_app_data("0x007", 50, "limit", None, None)
        .await
        .expect("app data should build");
    let order_to_sign = cow_sdk_trading::get_order_to_sign(
        cow_sdk_trading::OrderToSignParams::new(trader.chain_id, address(OWNER), false)
            .with_apply_costs_slippage_and_fees(false),
        &params,
        &app_data.app_data_keccak256,
    )
    .expect("order to sign should build");
    let provider = crate::common::MockProvider::default();
    let verifier = address(OWNER);
    provider.set_code(&verifier, "0x6001600055");
    provider.set_contract_response("isValidSignature", "\"0xffffffff\"");

    let error = cow_sdk_trading::post::verify_eip1271_order_signature_async(
        &provider,
        &order_to_sign,
        trader.chain_id,
        &cow_sdk_trading::types::Eip1271VerificationParameters {
            verifier,
            signature: HexData::new("0x7e57c0de").unwrap(),
        },
        Some(&protocol_options_from_trader(&trader)),
    )
    .await
    .expect_err("wrong magic value must fail");

    assert!(matches!(
        error,
        cow_sdk_trading::TradingError::Contracts(
            cow_sdk_contracts::ContractsError::Eip1271MagicValueMismatch { .. }
        )
    ));
}

// The three tests below pin the eth-flow submission seam's owner-vs-receiver
// threading: the client-side validator reads `OrderCreation.from` from the
// signer-derived owner carried on `EthFlowTransaction.from`, not from the
// payout `receiver`. Owners and receivers may legitimately differ when a
// caller asks the native-currency payout to land at a separate address; the
// validator must fire on the owner identity so the `AppdataFromMismatch`
// check stays bound to the signing authority.

fn ethflow_additional_params(
    quote: &cow_sdk_orderbook::OrderQuoteResponse,
) -> PostTradeAdditionalParams {
    PostTradeAdditionalParams::new()
        .with_check_eth_flow_order_exists(Arc::new(MockEthFlowChecker {
            results: Arc::new(Mutex::new(Vec::new())),
        }))
        .with_network_costs_amount(
            Amount::new(quote.quote.network_cost_amount().to_owned())
                .expect("quote fee amount must be valid"),
        )
        .with_custom_eip1271_signature(Arc::new(MockEip1271Provider))
}

fn ethflow_params_with_receiver(receiver: Option<cow_sdk_core::Address>) -> LimitTradeParameters {
    let mut params: LimitTradeParameters = sample_limit_parameters(OrderKind::Sell);
    params.sell_token = address(EVM_NATIVE_CURRENCY_ADDRESS);
    params.quote_id = Some(3);
    params.slippage_bps = Some(50);
    params.receiver = receiver;
    params
}

#[tokio::test]
async fn ethflow_validation_uses_signer_owner_not_receiver() {
    let trader = sample_trader_parameters();
    let orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    // Signer owner is `OWNER`; caller asks payout to land at `ALT_RECEIVER`,
    // which differs from the owner. The typed app-data signer matches the
    // owner, so validation must accept this legitimate receiver override.
    let signer = MockSigner::default();
    let app_data = build_app_data("0x007", 50, "market", None, None)
        .await
        .expect("app data should build");
    let params = ethflow_params_with_receiver(Some(address(ALT_RECEIVER)));
    let additional = ethflow_additional_params(&sell_quote_response());

    let result = post_sell_native_currency_order(
        &orderbook,
        &app_data,
        &params,
        &additional,
        &trader,
        &signer,
        cow_sdk_trading::OrderValidityBounds::SERVICES_DEFAULT,
        Some(address(OWNER)),
    )
    .await
    .expect("receiver override with matching owner and app-data signer must pass validation");

    assert!(result.tx_hash.is_some());
    assert_eq!(orderbook.state().uploads.len(), 1);
    assert_eq!(signer.state().sent_transactions.len(), 1);
}

#[tokio::test]
async fn ethflow_validation_rejects_mismatched_signer() {
    let trader = sample_trader_parameters();
    let orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    // Signer owner is `OWNER`; receiver is a distinct payout address; the
    // declared app-data signer is a third address that disagrees with the
    // owner. Validation must reject and the surfaced typed rejection must
    // carry the owner as `from`, not the receiver.
    let signer = MockSigner::default();
    let mismatched_signer =
        cow_sdk_core::Address::new("0xcccccccccccccccccccccccccccccccccccccccc")
            .expect("mismatched signer literal must be valid");
    let app_data = build_app_data("0x007", 50, "market", None, None)
        .await
        .expect("app data should build");
    let params = ethflow_params_with_receiver(Some(address(ALT_RECEIVER)));
    let additional = ethflow_additional_params(&sell_quote_response());

    let error = post_sell_native_currency_order(
        &orderbook,
        &app_data,
        &params,
        &additional,
        &trader,
        &signer,
        cow_sdk_trading::OrderValidityBounds::SERVICES_DEFAULT,
        Some(mismatched_signer.clone()),
    )
    .await
    .expect_err("mismatched app-data signer must trigger a typed rejection");

    match error {
        cow_sdk_trading::TradingError::ClientRejected(
            cow_sdk_trading::ClientRejection::AppdataFromMismatch {
                appdata_signer,
                from,
            },
        ) => {
            assert_eq!(
                appdata_signer, mismatched_signer,
                "rejection must surface the declared app-data signer verbatim"
            );
            assert_eq!(
                from,
                address(OWNER),
                "rejection's from must be the signer-derived owner, not the payout receiver"
            );
        }
        other => panic!("expected AppdataFromMismatch, got {other:?}"),
    }
}

#[tokio::test]
async fn ethflow_validation_accepts_matched_signer_with_default_receiver() {
    let trader = sample_trader_parameters();
    let orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    // No explicit receiver: `get_order_to_sign` defaults it to the signer
    // owner so owner and receiver converge. A matching app-data signer must
    // pass validation through the same code path the custom-receiver test
    // exercises.
    let signer = MockSigner::default();
    let app_data = build_app_data("0x007", 50, "market", None, None)
        .await
        .expect("app data should build");
    let params = ethflow_params_with_receiver(None);
    let additional = ethflow_additional_params(&sell_quote_response());

    let result = post_sell_native_currency_order(
        &orderbook,
        &app_data,
        &params,
        &additional,
        &trader,
        &signer,
        cow_sdk_trading::OrderValidityBounds::SERVICES_DEFAULT,
        Some(address(OWNER)),
    )
    .await
    .expect("default-receiver eth-flow post with matching signer must succeed");

    assert!(result.tx_hash.is_some());
    assert_eq!(orderbook.state().uploads.len(), 1);
    assert_eq!(signer.state().sent_transactions.len(), 1);
}
