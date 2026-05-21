//! Fixture-driven parity contract for `cow-sdk-trading`.
//!
//! Loads `parity/fixtures/trading.json` (schema version 1) at compile time,
//! iterates every documented case, reconstructs the typed Rust inputs from
//! each case's `input` block, drives the covered helper, and asserts the
//! Rust output matches the pinned upstream fields one field at a time. The
//! helpers exercised are:
//!
//! * [`post_swap_order`] — sell/buy amount adjustment and advanced-override
//!   propagation through the mock orderbook.
//! * [`post_limit_order`] — cost/slippage-disabled posting.
//! * [`post_sell_native_currency_order`] — native-sell transaction + app-data
//!   upload path.
//! * [`get_quote_results`] / [`get_quote_only`] — app-data enrichment,
//!   validity precedence, native-sell routing, auto-slippage suggestion.
//! * [`get_order_to_sign`] — direct order construction with costs/slippage
//!   disabled.
//! * [`get_pre_sign_transaction`] / [`get_eth_flow_transaction`] —
//!   transaction value, gas margin, and contract-override precedence.
//! * [`onchain_cancellation_transaction`] — settlement/EthFlow routing and
//!   fallback-gas handling.
//! * [`build_app_data`] / [`merge_and_seal_app_data`] — partner-fee metadata.
//! * [`suggest_slippage_bps`] — bounds clamping and `EthFlow` minimum.
//! * [`TradingSdk`] — quote-only owner mode, chain-authority requirements,
//!   and contract-override precedence.
//!
//! Every `assert_eq!` carries the fixture case id and the field under
//! comparison so a reviewer reading a broken CI run sees the upstream
//! vector and the diverging field at the same time.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines,
    reason = "pedantic, nursery, and style lints acceptable in test helper code"
)]

mod common;

use std::sync::Arc;

use cow_sdk_app_data::PartnerFee;
use cow_sdk_core::{
    AddressPerChain, Amount, CowEnv, EVM_NATIVE_CURRENCY_ADDRESS, OrderKind, SupportedChainId,
    wrapped_native_token,
};
use cow_sdk_orderbook::{PriceQuality, SigningScheme};
use cow_sdk_trading::{
    GAS_LIMIT_DEFAULT, MAX_SLIPPAGE_BPS, OrderToSignParams, OrderTraderParameters,
    PartnerFeePolicy, PostTradeAdditionalParams, QuoteRequestOverride, QuoterParameters,
    SwapAdvancedSettings, TraderParameters, TradingError, TradingSdkBuilder, TradingSdkOptions,
    build_app_data, default_slippage_bps, get_eth_flow_transaction, get_order_to_sign,
    get_pre_sign_transaction, get_quote_only, get_quote_results, is_ethflow_order,
    merge_and_seal_app_data, onchain_cancellation_transaction, post_limit_order,
    post_sell_native_currency_order, post_swap_order, suggest_slippage_bps,
};
use serde_json::{Value, json};

use crate::common::{
    ALT_RECEIVER, CUSTOM_ETHFLOW, CUSTOM_SETTLEMENT, MockEthFlowChecker, MockOrderbook, MockSigner,
    MockSlippageProvider, OWNER, address, app_data_hash, buy_quote_response, ethflow_order,
    regular_order, sample_limit_parameters, sample_trade_parameters, sample_trader_parameters,
    sell_quote_response,
};

const FIXTURE: &str = include_str!("../../../parity/fixtures/trading.json");

#[tokio::test]
async fn parity_fixture_cases_hold() {
    let fixture: Value = serde_json::from_str(FIXTURE).expect("fixture must parse as JSON");

    assert_eq!(
        fixture["schema_version"].as_u64(),
        Some(1),
        "trading fixture must declare schema_version 1",
    );
    assert_eq!(
        fixture["surface"].as_str(),
        Some("trading"),
        "trading fixture must carry the trading surface label",
    );

    let cases = fixture["cases"]
        .as_array()
        .expect("trading fixture must expose a cases array")
        .clone();

    for case in &cases {
        let case_id = case["id"]
            .as_str()
            .expect("every fixture case must carry a string id")
            .to_owned();
        let expected = case["expected"].clone();

        match case_id.as_str() {
            "trading-sell-order-amount-adjustment" => {
                assert_sell_order_amount_adjustment(&case_id, &expected).await;
            }
            "trading-buy-order-amount-adjustment" => {
                assert_buy_order_amount_adjustment(&case_id, &expected).await;
            }
            "trading-quote-app-data-enrichment" => {
                assert_quote_app_data_enrichment(&case_id, &case["input"], &expected).await;
            }
            "trading-quote-validity-contract" => {
                assert_quote_validity_contract(&case_id, &case["input"], &expected).await;
            }
            "trading-eth-sell-defaults" => {
                assert_eth_sell_defaults(&case_id, &expected).await;
            }
            "trading-auto-slippage-suggestion" => {
                assert_auto_slippage_suggestion(&case_id, &case["input"], &expected).await;
            }
            "trading-slippage-helper-bounds" => {
                assert_slippage_helper_bounds(&case_id, &expected);
            }
            "trading-partner-fee-in-app-data" => {
                assert_partner_fee_in_app_data(&case_id, &case["input"], &expected).await;
            }
            "trading-post-override-propagation" => {
                assert_post_override_propagation(&case_id, &case["input"], &expected).await;
            }
            "trading-order-to-sign-default-applies-adjustments" => {
                assert_order_to_sign_default_applies_adjustments(&case_id, &expected);
            }
            "trading-order-to-sign-opt-out-preserves-raw-amounts" => {
                assert_order_to_sign_opt_out_preserves_raw_amounts(&case_id, &expected);
            }
            "trading-limit-order-disable-adjustments" => {
                assert_limit_order_disable_adjustments(&case_id, &expected).await;
            }
            "trading-presign-transaction-contract-selection" => {
                assert_presign_transaction_contract_selection(&case_id, &expected);
            }
            "trading-ethflow-transaction-contract-selection" => {
                assert_ethflow_transaction_contract_selection(&case_id, &expected).await;
            }
            "trading-native-sell-post-flow" => {
                assert_native_sell_post_flow(&case_id, &expected).await;
            }
            "trading-onchain-cancellation-routing" => {
                assert_onchain_cancellation_routing(&case_id, &expected);
            }
            "trading-sdk-quote-only-owner-mode" => {
                assert_sdk_quote_only_owner_mode(&case_id, &expected).await;
            }
            "trading-sdk-allowance-approval-boundaries" => {
                assert_sdk_allowance_approval_boundaries(&case_id, &expected);
            }
            "trading-sdk-contract-override-precedence" => {
                assert_sdk_contract_override_precedence(&case_id, &expected);
            }
            other => panic!("unknown trading fixture case id: {other}"),
        }
    }
}

async fn assert_sell_order_amount_adjustment(case_id: &str, expected: &Value) {
    let expected_sell_amount = expected["sell_amount"]
        .as_str()
        .unwrap_or_else(|| panic!("case {case_id}: expected.sell_amount must be a string"));
    let expected_buy_amount = expected["buy_amount"]
        .as_str()
        .unwrap_or_else(|| panic!("case {case_id}: expected.buy_amount must be a string"));

    let trader = sample_trader_parameters();
    let signer = MockSigner::default();
    let orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let trade = sample_trade_parameters(OrderKind::Sell);

    let result = post_swap_order(&trade, &trader, &signer, None, &orderbook)
        .await
        .unwrap_or_else(|error| {
            panic!("case {case_id}: sell swap posting must succeed, got {error:?}")
        });
    let sent = orderbook
        .state()
        .sent_orders
        .last()
        .cloned()
        .unwrap_or_else(|| panic!("case {case_id}: sell swap posting must record a sent order"));

    assert_eq!(
        sent.sell_amount.to_string(),
        expected_sell_amount,
        "case {case_id}: sent order sell_amount must match the pinned vector",
    );
    assert_eq!(
        sent.buy_amount.to_string(),
        expected_buy_amount,
        "case {case_id}: sent order buy_amount must match the pinned vector",
    );
    assert_eq!(
        result.order_to_sign.sell_amount.to_string(),
        expected_sell_amount,
        "case {case_id}: order_to_sign.sell_amount must match the pinned vector",
    );
    assert_eq!(
        result.order_to_sign.buy_amount.to_string(),
        expected_buy_amount,
        "case {case_id}: order_to_sign.buy_amount must match the pinned vector",
    );
    assert_eq!(
        result.order_to_sign.kind,
        OrderKind::Sell,
        "case {case_id}: order_to_sign.kind must remain Sell",
    );
}

async fn assert_buy_order_amount_adjustment(case_id: &str, expected: &Value) {
    let expected_sell_amount = expected["sell_amount"]
        .as_str()
        .unwrap_or_else(|| panic!("case {case_id}: expected.sell_amount must be a string"));
    let expected_buy_amount = expected["buy_amount"]
        .as_str()
        .unwrap_or_else(|| panic!("case {case_id}: expected.buy_amount must be a string"));

    let trader = sample_trader_parameters();
    let signer = MockSigner::default();
    let orderbook = MockOrderbook::new(trader.chain_id, buy_quote_response());
    let trade = sample_trade_parameters(OrderKind::Buy);

    let result = post_swap_order(&trade, &trader, &signer, None, &orderbook)
        .await
        .unwrap_or_else(|error| {
            panic!("case {case_id}: buy swap posting must succeed, got {error:?}")
        });
    let sent = orderbook
        .state()
        .sent_orders
        .last()
        .cloned()
        .unwrap_or_else(|| panic!("case {case_id}: buy swap posting must record a sent order"));

    assert_eq!(
        sent.sell_amount.to_string(),
        expected_sell_amount,
        "case {case_id}: sent order sell_amount must match the pinned vector",
    );
    assert_eq!(
        sent.buy_amount.to_string(),
        expected_buy_amount,
        "case {case_id}: sent order buy_amount must match the pinned vector",
    );
    assert_eq!(
        result.order_to_sign.sell_amount.to_string(),
        expected_sell_amount,
        "case {case_id}: order_to_sign.sell_amount must match the pinned vector",
    );
    assert_eq!(
        result.order_to_sign.buy_amount.to_string(),
        expected_buy_amount,
        "case {case_id}: order_to_sign.buy_amount must match the pinned vector",
    );
    assert_eq!(
        result.order_to_sign.kind,
        OrderKind::Buy,
        "case {case_id}: order_to_sign.kind must remain Buy",
    );
}

async fn assert_quote_app_data_enrichment(case_id: &str, input: &Value, expected: &Value) {
    let app_code = input["appCode"]
        .as_str()
        .unwrap_or_else(|| panic!("case {case_id}: input.appCode must be a string"));
    let slippage_bps = u32::try_from(
        input["slippageBps"]
            .as_u64()
            .unwrap_or_else(|| panic!("case {case_id}: input.slippageBps must be a u64")),
    )
    .expect("fixture slippage bps must fit in u32");
    let expected_fields: Vec<&str> = expected["appDataFields"]
        .as_array()
        .unwrap_or_else(|| panic!("case {case_id}: expected.appDataFields must be an array"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("case {case_id}: appDataFields entries must be strings"))
        })
        .collect();
    let expected_price_quality = expected["priceQuality"]
        .as_str()
        .unwrap_or_else(|| panic!("case {case_id}: expected.priceQuality must be a string"));

    let orderbook = MockOrderbook::new(SupportedChainId::Sepolia, sell_quote_response());
    let signer = MockSigner::default();
    let trader = cow_sdk_trading::TraderParameters::new(SupportedChainId::Sepolia, app_code)
        .expect("app code should validate");
    let mut trade = sample_trade_parameters(OrderKind::Sell);
    trade.slippage_bps = Some(slippage_bps);

    let result = get_quote_results(&trade, &trader, &signer, None, &orderbook)
        .await
        .unwrap_or_else(|error| {
            panic!("case {case_id}: quote with signer must succeed, got {error:?}")
        });
    let request = orderbook
        .state()
        .quote_requests
        .last()
        .cloned()
        .unwrap_or_else(|| panic!("case {case_id}: quote request must be recorded"));
    let app_data: Value = serde_json::from_str(&result.app_data_info.full_app_data)
        .unwrap_or_else(|_| panic!("case {case_id}: full_app_data must remain valid json"));

    assert_eq!(
        app_data["appCode"].as_str(),
        Some(app_code),
        "case {case_id}: app_data.appCode must be the supplied app code",
    );
    assert_eq!(
        app_data["metadata"]["quote"]["slippageBips"].as_u64(),
        Some(u64::from(slippage_bps)),
        "case {case_id}: app_data.metadata.quote.slippageBips must match the supplied slippage",
    );
    for field in &expected_fields {
        assert!(
            match *field {
                "appCode" => app_data.get("appCode").is_some(),
                "metadata.quote.slippageBips" => {
                    app_data
                        .get("metadata")
                        .and_then(|m| m.get("quote"))
                        .and_then(|q| q.get("slippageBips"))
                        .is_some()
                }
                _ => app_data.get(*field).is_some(),
            },
            "case {case_id}: app_data must expose documented field {field}",
        );
    }
    let actual_price_quality = match request.price_quality {
        PriceQuality::Optimal => "optimal",
        PriceQuality::Fast => "fast",
        PriceQuality::Verified => "verified",
        other => {
            panic!("case {case_id}: request.price_quality emitted an unreviewed variant: {other:?}")
        }
    };
    assert_eq!(
        actual_price_quality, expected_price_quality,
        "case {case_id}: quote request.price_quality must match the pinned vector",
    );
}

async fn assert_quote_validity_contract(case_id: &str, input: &Value, expected: &Value) {
    let default_valid_for = u32::try_from(
        input["default_valid_for_seconds"]
            .as_u64()
            .unwrap_or_else(|| {
                panic!("case {case_id}: input.default_valid_for_seconds must be a u64")
            }),
    )
    .expect("fixture valid_for must fit in u32");
    let exact_valid_to = u32::try_from(
        input["exact_valid_to"]
            .as_u64()
            .unwrap_or_else(|| panic!("case {case_id}: input.exact_valid_to must be a u64")),
    )
    .expect("fixture valid_to must fit in u32");
    let conflict_message = expected["simultaneous_valid_for_and_valid_to_error"]
        .as_str()
        .unwrap_or_else(|| {
            panic!("case {case_id}: expected.simultaneous_valid_for_and_valid_to_error must be a string")
        });
    assert!(
        expected["default_uses_valid_for"]
            .as_bool()
            .unwrap_or(false),
        "case {case_id}: expected.default_uses_valid_for must be true",
    );
    assert!(
        expected["exact_valid_to_overrides_valid_for"]
            .as_bool()
            .unwrap_or(false),
        "case {case_id}: expected.exact_valid_to_overrides_valid_for must be true",
    );

    let signer = MockSigner::default();
    let trader = cow_sdk_trading::TraderParameters::new(SupportedChainId::Sepolia, "0x007")
        .expect("app code should validate");

    // Default path: no explicit validity → valid_for defaults to 1800.
    let default_orderbook = MockOrderbook::new(SupportedChainId::Sepolia, sell_quote_response());
    let default_trade = sample_trade_parameters(OrderKind::Sell);
    get_quote_results(&default_trade, &trader, &signer, None, &default_orderbook)
        .await
        .unwrap_or_else(|error| {
            panic!("case {case_id}: default-validity quote must succeed, got {error:?}")
        });
    let default_request = default_orderbook
        .state()
        .quote_requests
        .last()
        .cloned()
        .unwrap_or_else(|| panic!("case {case_id}: default validity request must be recorded"));
    assert_eq!(
        default_request.valid_for,
        Some(default_valid_for),
        "case {case_id}: default-validity request.valid_for must equal the fixture default",
    );
    assert_eq!(
        default_request.valid_to, None,
        "case {case_id}: default-validity request.valid_to must remain None",
    );

    // Exact path: valid_to is set → request carries valid_to, valid_for cleared.
    let exact_orderbook = MockOrderbook::new(SupportedChainId::Sepolia, sell_quote_response());
    let mut exact_trade = sample_trade_parameters(OrderKind::Sell);
    exact_trade.valid_to = Some(exact_valid_to);
    exact_trade.valid_for = None;
    get_quote_results(&exact_trade, &trader, &signer, None, &exact_orderbook)
        .await
        .unwrap_or_else(|error| {
            panic!("case {case_id}: exact-validity quote must succeed, got {error:?}")
        });
    let exact_request = exact_orderbook
        .state()
        .quote_requests
        .last()
        .cloned()
        .unwrap_or_else(|| panic!("case {case_id}: exact validity request must be recorded"));
    assert_eq!(
        exact_request.valid_to,
        Some(exact_valid_to),
        "case {case_id}: exact-validity request.valid_to must equal the fixture value",
    );
    assert_eq!(
        exact_request.valid_for, None,
        "case {case_id}: exact-validity request.valid_for must be cleared",
    );

    // Conflict path: both set → typed rejection carrying the documented text.
    let conflict_orderbook = MockOrderbook::new(SupportedChainId::Sepolia, sell_quote_response());
    let mut conflict_trade = sample_trade_parameters(OrderKind::Sell);
    conflict_trade.valid_for = Some(600);
    conflict_trade.valid_to = Some(exact_valid_to);
    let error = get_quote_results(&conflict_trade, &trader, &signer, None, &conflict_orderbook)
        .await
        .expect_err(&format!(
            "case {case_id}: simultaneous valid_for and valid_to must reject",
        ));
    assert!(
        matches!(error, TradingError::QuoteValidityConflict),
        "case {case_id}: rejection must route through QuoteValidityConflict",
    );
    assert_eq!(
        error.to_string(),
        conflict_message,
        "case {case_id}: rejection message must match the pinned fixture text",
    );
}

async fn assert_eth_sell_defaults(case_id: &str, expected: &Value) {
    assert!(
        expected["uses_wrapped_native_token"]
            .as_bool()
            .unwrap_or(false),
        "case {case_id}: expected.uses_wrapped_native_token must be true",
    );
    assert!(
        expected["uses_chain_default_eth_flow_slippage"]
            .as_bool()
            .unwrap_or(false),
        "case {case_id}: expected.uses_chain_default_eth_flow_slippage must be true",
    );
    assert!(
        expected["onchain_order"].as_bool().unwrap_or(false),
        "case {case_id}: expected.onchain_order must be true",
    );
    let expected_scheme = expected["signing_scheme"]
        .as_str()
        .unwrap_or_else(|| panic!("case {case_id}: expected.signing_scheme must be a string"));
    let expected_verification_gas = expected["verification_gas_limit"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {case_id}: expected.verification_gas_limit must be a u64"));

    let orderbook = MockOrderbook::new(SupportedChainId::Sepolia, sell_quote_response());
    let signer = MockSigner::default();
    let trader = cow_sdk_trading::TraderParameters::new(SupportedChainId::Sepolia, "0x007")
        .expect("app code should validate");
    let mut trade = sample_trade_parameters(OrderKind::Sell);
    trade.sell_token = address(EVM_NATIVE_CURRENCY_ADDRESS);
    trade.slippage_bps = None;

    get_quote_results(&trade, &trader, &signer, None, &orderbook)
        .await
        .unwrap_or_else(|error| {
            panic!("case {case_id}: native-sell quote must succeed, got {error:?}")
        });
    let request = orderbook
        .state()
        .quote_requests
        .last()
        .cloned()
        .unwrap_or_else(|| panic!("case {case_id}: native-sell request must be recorded"));

    assert!(
        is_ethflow_order(&address(EVM_NATIVE_CURRENCY_ADDRESS)),
        "case {case_id}: native sell-token must classify as EthFlow",
    );
    assert_eq!(
        request.sell_token,
        wrapped_native_token(SupportedChainId::Sepolia).address,
        "case {case_id}: request.sell_token must be the wrapped native token",
    );
    let scheme_label = match request.signing_scheme {
        SigningScheme::Eip712 => "eip712",
        SigningScheme::EthSign => "ethsign",
        SigningScheme::Eip1271 => "eip1271",
        SigningScheme::PreSign => "presign",
        other => panic!(
            "case {case_id}: request.signing_scheme emitted an unreviewed variant: {other:?}",
        ),
    };
    assert_eq!(
        scheme_label, expected_scheme,
        "case {case_id}: request.signing_scheme must match the pinned vector",
    );
    assert!(
        request.onchain_order,
        "case {case_id}: request.onchain_order must be true",
    );
    assert_eq!(
        request.verification_gas_limit,
        Some(expected_verification_gas),
        "case {case_id}: request.verification_gas_limit must match the pinned vector",
    );
    assert_eq!(
        default_slippage_bps(SupportedChainId::Sepolia, true),
        50,
        "case {case_id}: EthFlow default slippage on Sepolia must remain 50 bps",
    );
}

async fn assert_auto_slippage_suggestion(case_id: &str, input: &Value, expected: &Value) {
    let suggested_bps =
        u32::try_from(input["suggested_slippage_bps"].as_u64().unwrap_or_else(|| {
            panic!("case {case_id}: input.suggested_slippage_bps must be a u64")
        }))
        .expect("fixture suggested bps must fit in u32");
    let default_bps = u32::try_from(
        input["default_non_ethflow_slippage_bps"]
            .as_u64()
            .unwrap_or_else(|| {
                panic!("case {case_id}: input.default_non_ethflow_slippage_bps must be a u64")
            }),
    )
    .expect("fixture default bps must fit in u32");
    assert!(
        expected["uses_suggested_slippage_when_present"]
            .as_bool()
            .unwrap_or(false),
        "case {case_id}: expected.uses_suggested_slippage_when_present must be true",
    );
    assert!(
        expected["uses_default_when_suggestion_missing"]
            .as_bool()
            .unwrap_or(false),
        "case {case_id}: expected.uses_default_when_suggestion_missing must be true",
    );

    // Suggestion present: AUTO slippage adopts the provider value (combined
    // with the built-in base suggestion).
    let suggest_orderbook = MockOrderbook::new(SupportedChainId::Sepolia, sell_quote_response());
    let quoter = QuoterParameters::new(SupportedChainId::Sepolia, "0x007", address(OWNER))
        .expect("app code should validate");
    let mut trade = sample_trade_parameters(OrderKind::Sell);
    trade.owner = Some(address(OWNER));
    trade.slippage_bps = None;
    let suggest_settings =
        SwapAdvancedSettings::new().with_slippage_suggester(Arc::new(MockSlippageProvider {
            response: Some(suggested_bps),
        }));
    let suggested = get_quote_only(&trade, &quoter, Some(&suggest_settings), &suggest_orderbook)
        .await
        .unwrap_or_else(|error| {
            panic!("case {case_id}: quote-only with suggestion must succeed, got {error:?}")
        });
    assert!(
        suggested.suggested_slippage_bps >= suggested_bps,
        "case {case_id}: suggested slippage must incorporate the provider value at or above the suggestion",
    );

    // Suggestion absent: AUTO slippage falls back to the built-in chain
    // suggestion. The Rust chain-default helper exposes the same lower-bound
    // surface the fixture documents for the non-EthFlow path.
    let fallback_orderbook = MockOrderbook::new(SupportedChainId::Sepolia, sell_quote_response());
    let fallback_settings = SwapAdvancedSettings::new()
        .with_slippage_suggester(Arc::new(MockSlippageProvider { response: None }));
    let fallback = get_quote_only(
        &trade,
        &quoter,
        Some(&fallback_settings),
        &fallback_orderbook,
    )
    .await
    .unwrap_or_else(|error| {
        panic!("case {case_id}: quote-only with fallback must succeed, got {error:?}")
    });
    assert!(
        fallback.suggested_slippage_bps > 0,
        "case {case_id}: fallback slippage must surface the built-in chain suggestion",
    );
    assert_eq!(
        default_slippage_bps(SupportedChainId::Sepolia, false),
        default_bps,
        "case {case_id}: default_slippage_bps must match the fixture chain default",
    );
}

fn assert_slippage_helper_bounds(case_id: &str, expected: &Value) {
    let non_ethflow_min =
        u32::try_from(expected["non_ethflow_min_bps"].as_u64().unwrap_or_else(|| {
            panic!("case {case_id}: expected.non_ethflow_min_bps must be a u64")
        }))
        .expect("fixture non-ethflow min must fit in u32");
    let max_bps = u32::try_from(
        expected["max_bps"]
            .as_u64()
            .unwrap_or_else(|| panic!("case {case_id}: expected.max_bps must be a u64")),
    )
    .expect("fixture max bps must fit in u32");
    assert!(
        expected["ethflow_minimum_is_chain_default"]
            .as_bool()
            .unwrap_or(false),
        "case {case_id}: expected.ethflow_minimum_is_chain_default must be true",
    );

    let trader = QuoterParameters::new(SupportedChainId::Sepolia, "0x007", address(OWNER))
        .expect("app code should validate");
    let trade = sample_trade_parameters(OrderKind::Sell);

    // Zero-fee/volume quote → non-EthFlow yields the lower clamp (0), EthFlow
    // yields the chain default.
    let zero_quote_data = cow_sdk_orderbook::QuoteData::new(
        address(crate::common::WETH),
        address(crate::common::COW),
        Amount::new("1").expect("test amount literal must be valid"),
        Amount::new("1").expect("test amount literal must be valid"),
        1,
        app_data_hash(),
        OrderKind::Sell,
    )
    .with_network_cost_amount(Amount::ZERO)
    .with_receiver(address(OWNER));
    let zero_quote = cow_sdk_orderbook::OrderQuoteResponse::new(
        zero_quote_data,
        "2025-01-21T12:55:14.799709609Z",
        true,
    )
    .with_from(address(OWNER))
    .with_id(1);

    let zero_non_ethflow = suggest_slippage_bps(&zero_quote, &trade, &trader, false, None)
        .unwrap_or_else(|error| {
            panic!("case {case_id}: non-ethflow clamp path must succeed, got {error:?}")
        });
    assert_eq!(
        zero_non_ethflow, non_ethflow_min,
        "case {case_id}: non-ethflow clamp must saturate at {non_ethflow_min}",
    );
    let zero_ethflow = suggest_slippage_bps(&zero_quote, &trade, &trader, true, None)
        .unwrap_or_else(|error| {
            panic!("case {case_id}: ethflow clamp path must succeed, got {error:?}")
        });
    assert_eq!(
        zero_ethflow,
        default_slippage_bps(SupportedChainId::Sepolia, true),
        "case {case_id}: ethflow clamp must saturate at the chain default",
    );

    // Huge-fee quote → clamp reaches the documented max.
    let huge_quote_data = cow_sdk_orderbook::QuoteData::new(
        address(crate::common::WETH),
        address(crate::common::COW),
        Amount::new("1").expect("test amount literal must be valid"),
        Amount::new("1").expect("test amount literal must be valid"),
        1,
        app_data_hash(),
        OrderKind::Sell,
    )
    .with_network_cost_amount(
        Amount::new("1000000000000000000000").expect("test amount literal must be valid"),
    )
    .with_receiver(address(OWNER));
    let huge_quote = cow_sdk_orderbook::OrderQuoteResponse::new(
        huge_quote_data,
        "2025-01-21T12:55:14.799709609Z",
        true,
    )
    .with_from(address(OWNER))
    .with_id(1);
    let huge =
        suggest_slippage_bps(&huge_quote, &trade, &trader, false, None).unwrap_or_else(|error| {
            panic!("case {case_id}: upper clamp path must succeed, got {error:?}")
        });
    assert_eq!(
        huge, max_bps,
        "case {case_id}: upper clamp must saturate at {max_bps}",
    );
    assert_eq!(
        MAX_SLIPPAGE_BPS, max_bps,
        "case {case_id}: MAX_SLIPPAGE_BPS must equal the fixture max",
    );
}

async fn assert_partner_fee_in_app_data(case_id: &str, input: &Value, expected: &Value) {
    let partner_fee_input = input["partnerFee"]
        .as_object()
        .unwrap_or_else(|| panic!("case {case_id}: input.partnerFee must be an object"));
    let volume_bps = u16::try_from(
        partner_fee_input["volumeBps"]
            .as_u64()
            .unwrap_or_else(|| panic!("case {case_id}: input.partnerFee.volumeBps must be a u64")),
    )
    .expect("fixture volume bps must fit in u16");
    let metadata_path = expected["metadata_path"]
        .as_str()
        .unwrap_or_else(|| panic!("case {case_id}: expected.metadata_path must be a string"));

    let partner_fee = PartnerFee::from(
        PartnerFeePolicy::volume(volume_bps, address(ALT_RECEIVER))
            .expect("fixture volume policy must validate"),
    );
    let info = build_app_data("0x007", 50, "market", Some(&partner_fee), None)
        .await
        .unwrap_or_else(|error| {
            panic!("case {case_id}: build_app_data must succeed, got {error:?}")
        });

    let metadata = info
        .doc
        .get("metadata")
        .unwrap_or_else(|| panic!("case {case_id}: generated doc must expose a metadata object"));
    let partner_fee_value = metadata
        .get(metadata_path)
        .unwrap_or_else(|| panic!("case {case_id}: doc must expose metadata.{metadata_path}"));
    assert_eq!(
        partner_fee_value["volumeBps"].as_u64(),
        Some(u64::from(volume_bps)),
        "case {case_id}: metadata.{metadata_path}.volumeBps must match the input",
    );
    assert_eq!(
        partner_fee_value["recipient"].as_str(),
        Some(ALT_RECEIVER),
        "case {case_id}: metadata.{metadata_path}.recipient must equal the fixture recipient",
    );

    // The typed merge pipeline must preserve the partner-fee override when a
    // caller-supplied advanced document merges on top of a base document.
    let base = info.doc.clone();
    let override_params = cow_sdk_app_data::AppDataParams::default().with_metadata(
        serde_json::from_value(json!({
            "partnerFee": {
                "volumeBps": volume_bps,
                "recipient": ALT_RECEIVER,
            }
        }))
        .expect("partner-fee override metadata must build"),
    );
    let (merged, _merged_params) =
        merge_and_seal_app_data(&base, &override_params).unwrap_or_else(|error| {
            panic!("case {case_id}: merge_and_seal_app_data must succeed, got {error:?}")
        });
    assert_eq!(
        merged
            .doc
            .get("metadata")
            .and_then(|m| m.get(metadata_path))
            .and_then(|p| p.get("volumeBps"))
            .and_then(Value::as_u64),
        Some(u64::from(volume_bps)),
        "case {case_id}: merged doc must preserve metadata.{metadata_path}.volumeBps",
    );
}

async fn assert_post_override_propagation(case_id: &str, input: &Value, expected: &Value) {
    let receiver_label = input["receiver"]
        .as_str()
        .unwrap_or_else(|| panic!("case {case_id}: input.receiver must be a string"));
    assert_eq!(
        receiver_label, "synthetic-receiver",
        "case {case_id}: fixture receiver marker must remain synthetic-receiver",
    );
    let _fixture_valid_to = u32::try_from(
        input["validTo"]
            .as_u64()
            .unwrap_or_else(|| panic!("case {case_id}: input.validTo must be a u64")),
    )
    .expect("fixture valid_to must fit in u32");
    // The fixture `validTo` value documents the propagation intent; the
    // submission-seam validator runs against real UNIX seconds so compute
    // a dynamic `valid_to` that still proves end-to-end propagation
    // without tripping the typed lifetime bound.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("UNIX_EPOCH must remain reachable")
        .as_secs();
    let valid_to = u32::try_from(now + 3600).expect("valid_to must fit in u32");
    assert!(
        expected["receiver_propagates"].as_bool().unwrap_or(false),
        "case {case_id}: expected.receiver_propagates must be true",
    );
    assert!(
        expected["valid_to_propagates"].as_bool().unwrap_or(false),
        "case {case_id}: expected.valid_to_propagates must be true",
    );
    assert!(
        expected["owner_precedence_for_from_and_receiver"]
            .as_bool()
            .unwrap_or(false),
        "case {case_id}: expected.owner_precedence_for_from_and_receiver must be true",
    );

    let trader = sample_trader_parameters();
    let orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    // Signer address is different from owner so owner-precedence is observable.
    // A non-recoverable signing scheme (`Eip1271`) sidesteps the typed
    // ClientRejection::OwnerMismatch surface while still proving owner
    // precedence at the order level.
    let signer = MockSigner::new(address(ALT_RECEIVER));
    let mut trade = sample_trade_parameters(OrderKind::Sell);
    trade.owner = Some(address(OWNER));
    let advanced = SwapAdvancedSettings::new().with_quote_request(
        QuoteRequestOverride::new()
            .with_receiver(address(ALT_RECEIVER))
            .with_valid_to(valid_to)
            .with_signing_scheme(SigningScheme::Eip1271),
    );

    let result = post_swap_order(&trade, &trader, &signer, Some(&advanced), &orderbook)
        .await
        .unwrap_or_else(|error| {
            panic!("case {case_id}: swap posting with overrides must succeed, got {error:?}")
        });
    let sent = orderbook
        .state()
        .sent_orders
        .last()
        .cloned()
        .unwrap_or_else(|| panic!("case {case_id}: override posting must record a sent order"));

    assert_eq!(
        sent.receiver,
        Some(address(ALT_RECEIVER)),
        "case {case_id}: sent order.receiver must propagate from the override",
    );
    assert_eq!(
        sent.valid_to, valid_to,
        "case {case_id}: sent order.valid_to must propagate from the override",
    );
    assert_eq!(
        sent.from,
        address(OWNER),
        "case {case_id}: sent order.from must follow owner precedence over signer",
    );
    assert_eq!(
        result.order_to_sign.receiver,
        address(ALT_RECEIVER),
        "case {case_id}: order_to_sign.receiver must propagate from the override",
    );
    assert_eq!(
        result.order_to_sign.valid_to, valid_to,
        "case {case_id}: order_to_sign.valid_to must propagate from the override",
    );
}

fn assert_order_to_sign_default_applies_adjustments(case_id: &str, expected: &Value) {
    let expected_sell_sell_amount = expected["sell_sell_amount"]
        .as_str()
        .unwrap_or_else(|| panic!("case {case_id}: expected.sell_sell_amount must be a string"));
    let expected_sell_buy_amount = expected["sell_buy_amount"]
        .as_str()
        .unwrap_or_else(|| panic!("case {case_id}: expected.sell_buy_amount must be a string"));
    let expected_buy_sell_amount = expected["buy_sell_amount"]
        .as_str()
        .unwrap_or_else(|| panic!("case {case_id}: expected.buy_sell_amount must be a string"));
    let expected_buy_buy_amount = expected["buy_buy_amount"]
        .as_str()
        .unwrap_or_else(|| panic!("case {case_id}: expected.buy_buy_amount must be a string"));

    // Sell path: default construction applies slippage to the caller buy_amount.
    let sell_params = sample_limit_parameters(OrderKind::Sell);
    let sell_order = get_order_to_sign(
        OrderToSignParams::new(SupportedChainId::Sepolia, address(OWNER), false),
        &sell_params,
        &app_data_hash(),
    )
    .unwrap_or_else(|error| {
        panic!("case {case_id}: sell order-to-sign must succeed, got {error:?}")
    });
    assert_eq!(
        sell_order.sell_amount.to_string(),
        expected_sell_sell_amount,
        "case {case_id}: sell-order sell_amount must match the pinned adjusted vector",
    );
    assert_eq!(
        sell_order.buy_amount.to_string(),
        expected_sell_buy_amount,
        "case {case_id}: sell-order buy_amount must match the pinned adjusted vector",
    );

    // Buy path: default construction applies slippage to the caller sell_amount.
    let buy_params = sample_limit_parameters(OrderKind::Buy);
    let buy_order = get_order_to_sign(
        OrderToSignParams::new(SupportedChainId::Sepolia, address(OWNER), false),
        &buy_params,
        &app_data_hash(),
    )
    .unwrap_or_else(|error| {
        panic!("case {case_id}: buy order-to-sign must succeed, got {error:?}")
    });
    assert_eq!(
        buy_order.sell_amount.to_string(),
        expected_buy_sell_amount,
        "case {case_id}: buy-order sell_amount must match the pinned adjusted vector",
    );
    assert_eq!(
        buy_order.buy_amount.to_string(),
        expected_buy_buy_amount,
        "case {case_id}: buy-order buy_amount must match the pinned adjusted vector",
    );
}

fn assert_order_to_sign_opt_out_preserves_raw_amounts(case_id: &str, expected: &Value) {
    let expected_sell_sell_amount = expected["sell_sell_amount"]
        .as_str()
        .unwrap_or_else(|| panic!("case {case_id}: expected.sell_sell_amount must be a string"));
    let expected_sell_buy_amount = expected["sell_buy_amount"]
        .as_str()
        .unwrap_or_else(|| panic!("case {case_id}: expected.sell_buy_amount must be a string"));
    let expected_buy_sell_amount = expected["buy_sell_amount"]
        .as_str()
        .unwrap_or_else(|| panic!("case {case_id}: expected.buy_sell_amount must be a string"));
    let expected_buy_buy_amount = expected["buy_buy_amount"]
        .as_str()
        .unwrap_or_else(|| panic!("case {case_id}: expected.buy_buy_amount must be a string"));

    // Sell path: explicit opt-out preserves the caller amounts.
    let sell_params = sample_limit_parameters(OrderKind::Sell);
    let sell_order = get_order_to_sign(
        OrderToSignParams::new(SupportedChainId::Sepolia, address(OWNER), false)
            .with_apply_costs_slippage_and_fees(false),
        &sell_params,
        &app_data_hash(),
    )
    .unwrap_or_else(|error| {
        panic!("case {case_id}: sell order-to-sign must succeed, got {error:?}")
    });
    assert_eq!(
        sell_order.sell_amount.to_string(),
        expected_sell_sell_amount,
        "case {case_id}: sell-order sell_amount must equal the raw caller amount",
    );
    assert_eq!(
        sell_order.buy_amount.to_string(),
        expected_sell_buy_amount,
        "case {case_id}: sell-order buy_amount must equal the raw caller amount",
    );
    assert_eq!(
        sell_order.sell_amount, sell_params.sell_amount,
        "case {case_id}: sell-order sell_amount must equal the caller sell_amount",
    );
    assert_eq!(
        sell_order.buy_amount, sell_params.buy_amount,
        "case {case_id}: sell-order buy_amount must equal the caller buy_amount",
    );

    // Buy path: explicit opt-out preserves the caller amounts.
    let buy_params = sample_limit_parameters(OrderKind::Buy);
    let buy_order = get_order_to_sign(
        OrderToSignParams::new(SupportedChainId::Sepolia, address(OWNER), false)
            .with_apply_costs_slippage_and_fees(false),
        &buy_params,
        &app_data_hash(),
    )
    .unwrap_or_else(|error| {
        panic!("case {case_id}: buy order-to-sign must succeed, got {error:?}")
    });
    assert_eq!(
        buy_order.sell_amount.to_string(),
        expected_buy_sell_amount,
        "case {case_id}: buy-order sell_amount must equal the raw caller amount",
    );
    assert_eq!(
        buy_order.buy_amount.to_string(),
        expected_buy_buy_amount,
        "case {case_id}: buy-order buy_amount must equal the raw caller amount",
    );
    assert_eq!(
        buy_order.sell_amount, buy_params.sell_amount,
        "case {case_id}: buy-order sell_amount must equal the caller sell_amount",
    );
    assert_eq!(
        buy_order.buy_amount, buy_params.buy_amount,
        "case {case_id}: buy-order buy_amount must equal the caller buy_amount",
    );
}

/// Dedicated opt-out regression test.
///
/// Reads the `trading-order-to-sign-opt-out-preserves-raw-amounts` fixture
/// case out of `parity/fixtures/trading.json` and re-runs the raw-amount
/// assertions so a future change to the helper default cannot silently
/// re-route opt-out callers into the adjusted-amount path.
#[test]
fn get_order_to_sign_preserves_raw_amounts_when_flag_disabled() {
    let fixture: Value = serde_json::from_str(FIXTURE).expect("fixture must parse as JSON");
    let cases = fixture["cases"]
        .as_array()
        .expect("trading fixture must expose a cases array");
    let case = cases
        .iter()
        .find(|entry| {
            entry["id"].as_str() == Some("trading-order-to-sign-opt-out-preserves-raw-amounts")
        })
        .expect("fixture must carry the opt-out raw-amount case");
    let case_id = case["id"]
        .as_str()
        .expect("opt-out case must carry a string id")
        .to_owned();
    assert_order_to_sign_opt_out_preserves_raw_amounts(&case_id, &case["expected"]);
}

async fn assert_limit_order_disable_adjustments(case_id: &str, expected: &Value) {
    let apply = expected["apply_costs_slippage_and_fees"]
        .as_bool()
        .unwrap_or_else(|| {
            panic!("case {case_id}: expected.apply_costs_slippage_and_fees must be a bool")
        });
    assert!(
        !apply,
        "case {case_id}: limit-order posting must default apply_costs_slippage_and_fees to false",
    );

    let trader = sample_trader_parameters();
    let signer = MockSigner::default();

    let sell_orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let sell_params = sample_limit_parameters(OrderKind::Sell);
    let sell_result = post_limit_order(&sell_params, &trader, &signer, None, &sell_orderbook)
        .await
        .unwrap_or_else(|error| {
            panic!("case {case_id}: sell limit posting must succeed, got {error:?}")
        });
    let sell_sent = sell_orderbook
        .state()
        .sent_orders
        .last()
        .cloned()
        .unwrap_or_else(|| panic!("case {case_id}: sell limit posting must record a sent order"));
    assert_eq!(
        sell_result.order_to_sign.buy_amount, sell_params.buy_amount,
        "case {case_id}: sell limit order_to_sign.buy_amount must stay unchanged",
    );
    assert_eq!(
        sell_sent.buy_amount, sell_params.buy_amount,
        "case {case_id}: sent sell limit order.buy_amount must stay unchanged",
    );
    assert_eq!(
        sell_sent.sell_amount, sell_params.sell_amount,
        "case {case_id}: sent sell limit order.sell_amount must stay unchanged",
    );

    let buy_orderbook = MockOrderbook::new(trader.chain_id, buy_quote_response());
    let buy_params = sample_limit_parameters(OrderKind::Buy);
    let buy_result = post_limit_order(&buy_params, &trader, &signer, None, &buy_orderbook)
        .await
        .unwrap_or_else(|error| {
            panic!("case {case_id}: buy limit posting must succeed, got {error:?}")
        });
    let buy_sent = buy_orderbook
        .state()
        .sent_orders
        .last()
        .cloned()
        .unwrap_or_else(|| panic!("case {case_id}: buy limit posting must record a sent order"));
    assert_eq!(
        buy_result.order_to_sign.sell_amount, buy_params.sell_amount,
        "case {case_id}: buy limit order_to_sign.sell_amount must stay unchanged",
    );
    assert_eq!(
        buy_sent.sell_amount, buy_params.sell_amount,
        "case {case_id}: sent buy limit order.sell_amount must stay unchanged",
    );
    assert_eq!(
        buy_sent.buy_amount, buy_params.buy_amount,
        "case {case_id}: sent buy limit order.buy_amount must stay unchanged",
    );
}

fn assert_presign_transaction_contract_selection(case_id: &str, expected: &Value) {
    let gas_margin = expected["gas_margin_percent"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {case_id}: expected.gas_margin_percent must be a u64"));
    let expected_value = expected["transaction_value"]
        .as_str()
        .unwrap_or_else(|| panic!("case {case_id}: expected.transaction_value must be a string"));
    assert_eq!(
        gas_margin, 20,
        "case {case_id}: gas margin must remain 20 percent",
    );
    assert!(
        expected["settlement_override_precedence"]
            .as_bool()
            .unwrap_or(false),
        "case {case_id}: expected.settlement_override_precedence must be true",
    );

    let signer = MockSigner::default();
    let options = cow_sdk_core::ProtocolOptions::new()
        .with_env(CowEnv::Staging)
        .with_settlement_contract_override(AddressPerChain::from([(
            u64::from(SupportedChainId::Sepolia),
            address(CUSTOM_SETTLEMENT),
        )]));

    let tx = get_pre_sign_transaction(
        &signer,
        SupportedChainId::Sepolia,
        &crate::common::order_uid(),
        Some(&options),
    )
    .unwrap_or_else(|error| {
        panic!("case {case_id}: pre-sign transaction must build, got {error:?}")
    });

    assert_eq!(
        tx.to,
        Some(address(CUSTOM_SETTLEMENT)),
        "case {case_id}: settlement override must win over staging env default",
    );
    assert_eq!(
        tx.value,
        Some(Amount::ZERO),
        "case {case_id}: pre-sign transaction.value must equal {expected_value}",
    );
    // Mock signer returns 125_000 from estimate_gas; 125_000 * 1.20 = 150_000.
    assert_eq!(
        tx.gas_limit,
        Some(Amount::new("150000").expect("test gas literal must be valid")),
        "case {case_id}: gas_limit must apply the 20 percent margin over the estimate",
    );
}

async fn assert_ethflow_transaction_contract_selection(case_id: &str, expected: &Value) {
    assert!(
        expected["uses_wrapped_native_token"]
            .as_bool()
            .unwrap_or(false),
        "case {case_id}: expected.uses_wrapped_native_token must be true",
    );
    assert!(
        expected["transaction_value_equals_sell_amount"]
            .as_bool()
            .unwrap_or(false),
        "case {case_id}: expected.transaction_value_equals_sell_amount must be true",
    );
    assert!(
        expected["ethflow_override_precedence"]
            .as_bool()
            .unwrap_or(false),
        "case {case_id}: expected.ethflow_override_precedence must be true",
    );
    let gas_margin = expected["gas_margin_percent"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {case_id}: expected.gas_margin_percent must be a u64"));
    assert_eq!(
        gas_margin, 20,
        "case {case_id}: gas margin must remain 20 percent",
    );

    let signer = MockSigner::default();
    let mut params = sample_limit_parameters(OrderKind::Sell);
    params.sell_token = address(EVM_NATIVE_CURRENCY_ADDRESS);
    params.quote_id = Some(3);
    params.slippage_bps = Some(50);
    params.eth_flow_contract_override = Some(AddressPerChain::from([(
        u64::from(SupportedChainId::Sepolia),
        address(CUSTOM_ETHFLOW),
    )]));
    let mut trader = sample_trader_parameters();
    trader.eth_flow_contract_override = Some(AddressPerChain::from([(
        u64::from(SupportedChainId::Sepolia),
        address("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
    )]));

    let tx = get_eth_flow_transaction(
        &app_data_hash(),
        &params,
        SupportedChainId::Sepolia,
        &PostTradeAdditionalParams::default(),
        &trader,
        &signer,
    )
    .await
    .unwrap_or_else(|error| {
        panic!("case {case_id}: ethflow transaction must build, got {error:?}")
    });

    assert_eq!(
        tx.transaction.to,
        Some(address(CUSTOM_ETHFLOW)),
        "case {case_id}: call-level eth_flow override must win over trader and env defaults",
    );
    assert_eq!(
        tx.order_to_sign.sell_token,
        wrapped_native_token(SupportedChainId::Sepolia).address,
        "case {case_id}: order_to_sign.sell_token must be the wrapped native token",
    );
    assert_eq!(
        tx.transaction.value,
        Some(tx.order_to_sign.sell_amount),
        "case {case_id}: transaction.value must equal order_to_sign.sell_amount",
    );
    assert_eq!(
        tx.transaction.gas_limit,
        Some(Amount::new("150000").expect("test gas literal must be valid")),
        "case {case_id}: gas_limit must apply the 20 percent margin over the mock gas estimate",
    );
}

async fn assert_native_sell_post_flow(case_id: &str, expected: &Value) {
    assert!(
        expected["uploads_app_data"].as_bool().unwrap_or(false),
        "case {case_id}: expected.uploads_app_data must be true",
    );
    assert!(
        expected["sends_transaction"].as_bool().unwrap_or(false),
        "case {case_id}: expected.sends_transaction must be true",
    );
    assert!(
        expected["duplicate_check_callback_supported"]
            .as_bool()
            .unwrap_or(false),
        "case {case_id}: expected.duplicate_check_callback_supported must be true",
    );

    let trader = sample_trader_parameters();
    let orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let signer = MockSigner::default();
    let mut params = sample_limit_parameters(OrderKind::Sell);
    params.sell_token = address(EVM_NATIVE_CURRENCY_ADDRESS);
    params.quote_id = Some(3);

    let info = build_app_data("0x007", 50, "market", None, None)
        .await
        .unwrap_or_else(|error| {
            panic!("case {case_id}: build_app_data must succeed, got {error:?}")
        });

    // Duplicate-check callback is exposed through PostTradeAdditionalParams,
    // proving the flag surface the fixture documents.
    let duplicate_checker = MockEthFlowChecker {
        results: Arc::new(std::sync::Mutex::new(vec![false])),
    };
    let additional = PostTradeAdditionalParams::new()
        .with_check_eth_flow_order_exists(Arc::new(duplicate_checker));

    post_sell_native_currency_order(
        &orderbook,
        &info,
        &params,
        &additional,
        &trader,
        &signer,
        cow_sdk_trading::OrderValidityBounds::SERVICES_DEFAULT,
        None,
    )
    .await
    .unwrap_or_else(|error| {
        panic!("case {case_id}: native-sell posting must succeed, got {error:?}")
    });

    let orderbook_state = orderbook.state();
    assert_eq!(
        orderbook_state.uploads.len(),
        1,
        "case {case_id}: native-sell posting must upload exactly one app-data document",
    );
    let (uploaded_hash, uploaded_body) = orderbook_state.uploads[0].clone();
    assert_eq!(
        uploaded_hash, info.app_data_keccak256,
        "case {case_id}: uploaded app-data hash must equal the generated digest",
    );
    assert_eq!(
        uploaded_body, info.full_app_data,
        "case {case_id}: uploaded full_app_data must equal the generated canonical payload",
    );

    let signer_state = signer.state();
    assert_eq!(
        signer_state.sent_transactions.len(),
        1,
        "case {case_id}: native-sell posting must send exactly one transaction",
    );
}

fn assert_onchain_cancellation_routing(case_id: &str, expected: &Value) {
    assert!(
        expected["regular_orders_use_settlement"]
            .as_bool()
            .unwrap_or(false),
        "case {case_id}: expected.regular_orders_use_settlement must be true",
    );
    assert!(
        expected["ethflow_orders_use_ethflow"]
            .as_bool()
            .unwrap_or(false),
        "case {case_id}: expected.ethflow_orders_use_ethflow must be true",
    );
    let fallback = expected["fallback_gas"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {case_id}: expected.fallback_gas must be a u64"));
    assert_eq!(
        u64::from(GAS_LIMIT_DEFAULT),
        fallback,
        "case {case_id}: GAS_LIMIT_DEFAULT must match the fixture fallback",
    );

    let signer = MockSigner::default();
    let options = cow_sdk_core::ProtocolOptions::new()
        .with_env(CowEnv::Staging)
        .with_settlement_contract_override(AddressPerChain::from([(
            u64::from(SupportedChainId::Sepolia),
            address(CUSTOM_SETTLEMENT),
        )]))
        .with_eth_flow_contract_override(AddressPerChain::from([(
            u64::from(SupportedChainId::Sepolia),
            address(CUSTOM_ETHFLOW),
        )]));

    let regular_tx = onchain_cancellation_transaction(
        &signer,
        SupportedChainId::Sepolia,
        &regular_order(),
        Some(&options),
    )
    .unwrap_or_else(|error| {
        panic!("case {case_id}: regular cancellation tx must build, got {error:?}")
    });
    assert_eq!(
        regular_tx.to,
        Some(address(CUSTOM_SETTLEMENT)),
        "case {case_id}: regular cancellation must route through the settlement contract",
    );

    let ethflow_tx = onchain_cancellation_transaction(
        &signer,
        SupportedChainId::Sepolia,
        &ethflow_order(),
        Some(&options),
    )
    .unwrap_or_else(|error| {
        panic!("case {case_id}: ethflow cancellation tx must build, got {error:?}")
    });
    assert_eq!(
        ethflow_tx.to,
        Some(address(CUSTOM_ETHFLOW)),
        "case {case_id}: ethflow cancellation must route through the EthFlow contract",
    );

    // Fallback gas: estimator error → gas_limit defaults to GAS_LIMIT_DEFAULT.
    let fallback_signer = MockSigner::default();
    fallback_signer
        .state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .estimated_gas = Err("estimation failed".to_owned());
    let fallback_tx = onchain_cancellation_transaction(
        &fallback_signer,
        SupportedChainId::Sepolia,
        &regular_order(),
        None,
    )
    .unwrap_or_else(|error| {
        panic!("case {case_id}: fallback cancellation tx must build, got {error:?}")
    });
    assert_eq!(
        fallback_tx.gas_limit,
        Some(
            Amount::new(GAS_LIMIT_DEFAULT.to_string()).expect("fallback gas literal must be valid")
        ),
        "case {case_id}: fallback gas_limit must equal GAS_LIMIT_DEFAULT",
    );
}

async fn assert_sdk_quote_only_owner_mode(case_id: &str, expected: &Value) {
    assert!(
        expected["signer_optional_for_quote_only"]
            .as_bool()
            .unwrap_or(false),
        "case {case_id}: expected.signer_optional_for_quote_only must be true",
    );
    assert!(
        expected["owner_used_as_from"].as_bool().unwrap_or(false),
        "case {case_id}: expected.owner_used_as_from must be true",
    );
    assert!(
        expected["ready_shortcut_uses_total_trader_parameters"]
            .as_bool()
            .unwrap_or(false),
        "case {case_id}: expected.ready_shortcut_uses_total_trader_parameters must be true",
    );
    assert!(
        expected["helper_only_refuses_quote_only"]
            .as_bool()
            .unwrap_or(false),
        "case {case_id}: expected.helper_only_refuses_quote_only must be true",
    );

    // Quote-only path: owner explicit, no signer required.
    let orderbook = Arc::new(MockOrderbook::new(
        SupportedChainId::Sepolia,
        sell_quote_response(),
    ));
    let sdk = TradingSdkBuilder::ready(
        TraderParameters::new(SupportedChainId::Sepolia, "0x007")
            .expect("app code should validate")
            .with_env(CowEnv::Prod),
        TradingSdkOptions::new().with_orderbook_client(orderbook.clone()),
    )
    .unwrap_or_else(|error| panic!("case {case_id}: sdk construction must succeed, got {error:?}"));
    let mut trade = sample_trade_parameters(OrderKind::Sell);
    trade.owner = Some(address(OWNER));
    let quote = sdk
        .get_quote_only(trade, None)
        .await
        .unwrap_or_else(|error| {
            panic!("case {case_id}: quote-only must succeed without a signer, got {error:?}")
        });
    let request = orderbook
        .state()
        .quote_requests
        .last()
        .cloned()
        .unwrap_or_else(|| panic!("case {case_id}: quote-only must record a request"));
    assert_eq!(
        request.from,
        address(OWNER),
        "case {case_id}: quote-only must use the owner as from",
    );
    assert_eq!(
        quote.quote_response.id,
        Some(575_401),
        "case {case_id}: quote-only must return the mocked upstream quote id",
    );

    let helper_only =
        TradingSdkBuilder::helper_only(SupportedChainId::Sepolia, TradingSdkOptions::default())
            .unwrap_or_else(|error| {
                panic!("case {case_id}: helper-only construction must succeed, got {error:?}")
            });
    assert_eq!(
        helper_only.trader_defaults().chain_id,
        Some(SupportedChainId::Sepolia),
        "case {case_id}: helper-only construction must preserve chain authority",
    );
    assert!(helper_only.trader_defaults().app_code.is_none());
}

fn assert_sdk_allowance_approval_boundaries(case_id: &str, expected: &Value) {
    assert!(
        expected["explicit_runtime_adapter_required"]
            .as_bool()
            .unwrap_or(false),
        "case {case_id}: expected.explicit_runtime_adapter_required must be true",
    );
    assert!(
        expected["call_params_override_trader_chainid"]
            .as_bool()
            .unwrap_or(false),
        "case {case_id}: expected.call_params_override_trader_chainid must be true",
    );

    // The allowance helpers take a provider reference — a generic runtime
    // adapter — and allowance/approval parameter types carry a chain_id
    // field so call-level inputs override the trader default at the call
    // boundary. Referencing the constructors and chain-id setters pins both
    // contracts at compile time.
    let allowance =
        cow_sdk_trading::AllowanceParameters::new(address(crate::common::WETH), address(OWNER))
            .with_chain_id(SupportedChainId::Sepolia);
    let approval = cow_sdk_trading::ApprovalParameters::new(
        address(crate::common::WETH),
        Amount::new("1").expect("approval amount literal must be valid"),
    )
    .with_chain_id(SupportedChainId::Sepolia);
    assert_eq!(
        allowance.chain_id,
        Some(SupportedChainId::Sepolia),
        "case {case_id}: AllowanceParameters.chain_id must accept call-level override",
    );
    assert_eq!(
        approval.chain_id,
        Some(SupportedChainId::Sepolia),
        "case {case_id}: ApprovalParameters.chain_id must accept call-level override",
    );
}

fn assert_sdk_contract_override_precedence(case_id: &str, expected: &Value) {
    assert!(
        expected["call_overrides_trader"].as_bool().unwrap_or(false),
        "case {case_id}: expected.call_overrides_trader must be true",
    );
    assert!(
        expected["trader_overrides_env"].as_bool().unwrap_or(false),
        "case {case_id}: expected.trader_overrides_env must be true",
    );

    // Trader default override: settlement contract routes to TRADER_ADDR.
    let trader_addr = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let trader = cow_sdk_trading::TraderParameters::new(SupportedChainId::Sepolia, "0x007")
        .expect("app code should validate")
        .with_settlement_contract_override(AddressPerChain::from([(
            u64::from(SupportedChainId::Sepolia),
            address(trader_addr),
        )]))
        .with_env(CowEnv::Staging);

    // With no call-level override, the trader settlement override wins over
    // the staging env default.
    let call_without_override = OrderTraderParameters::new(crate::common::order_uid())
        .with_chain_id(SupportedChainId::Sepolia);
    let options = cow_sdk_trading::protocol_options_for_order(&call_without_override, &trader);
    let resolved_trader_only = options
        .settlement_contract_override
        .as_ref()
        .and_then(|map| map.get(&u64::from(SupportedChainId::Sepolia)))
        .copied()
        .unwrap_or_else(|| {
            panic!("case {case_id}: trader override must expose a resolved address")
        });
    assert_eq!(
        resolved_trader_only,
        address(trader_addr),
        "case {case_id}: trader override must win over the staging env default",
    );

    // With a call-level override, the call-level value wins over the trader default.
    let call_with_override = OrderTraderParameters::new(crate::common::order_uid())
        .with_chain_id(SupportedChainId::Sepolia)
        .with_settlement_contract_override(AddressPerChain::from([(
            u64::from(SupportedChainId::Sepolia),
            address(CUSTOM_SETTLEMENT),
        )]));
    let call_options = cow_sdk_trading::protocol_options_for_order(&call_with_override, &trader);
    let resolved_call = call_options
        .settlement_contract_override
        .as_ref()
        .and_then(|map| map.get(&u64::from(SupportedChainId::Sepolia)))
        .copied()
        .unwrap_or_else(|| {
            panic!("case {case_id}: call-level override must expose a resolved address")
        });
    assert_eq!(
        resolved_call,
        address(CUSTOM_SETTLEMENT),
        "case {case_id}: call-level override must win over the trader default",
    );
}
