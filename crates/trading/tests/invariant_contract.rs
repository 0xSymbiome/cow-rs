//! Property coverage for the trading quote, slippage, projection, and `EthFlow`
//! encoding invariants.
//!
//! These properties pin cow-rs-specific trading logic — slippage-suggestion
//! monotonicity, the quote→limit projection field-for-field, `EthFlow` calldata
//! uint256 fidelity, and quote-request override precedence — across generated
//! inputs. Coverage uses `proptest` (shrinking + a committed regression file) to
//! match the crate's existing property-test convention in `property_contract.rs`.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::missing_const_for_fn,
    clippy::option_if_let_else,
    clippy::redundant_clone,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    reason = "property-test bodies keep explicit construction and casts close to their assertions"
)]

mod common;

use alloy_primitives::U256;
use cow_sdk_core::{Amount, CowEnv, EVM_NATIVE_CURRENCY_ADDRESS, OrderKind, SupportedChainId};
use cow_sdk_orderbook::{OrderQuoteResponse, PriceQuality, QuoteValidity};
use cow_sdk_trading::{
    LimitTradeParamsFromQuote, MAX_SLIPPAGE_BPS, PartnerFee, PartnerFeePolicy,
    PostTradeAdditionalParams, QuoteRequestOverride, QuoterParams, TradeAdvancedSettings,
    eth_flow_transaction, quote_results, suggest_slippage_bps, swap_params_to_limit_order_params,
};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

use crate::common::{
    ALT_RECEIVER, CUSTOM_SETTLEMENT, MockOrderbook, MockSigner, OWNER, address, app_data_hash,
    buy_quote_response, sample_limit_parameters, sample_trade_parameters, sample_trader_parameters,
    sell_quote_response,
};

const REGRESSION_FILE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/proptest-regressions/invariant_contract.txt"
);

fn trader() -> QuoterParams {
    QuoterParams::new(SupportedChainId::Sepolia, "0x007", address(OWNER))
        .expect("app code should validate")
}

fn make_quote(
    kind: OrderKind,
    sell_amount: u64,
    buy_amount: u64,
    fee_amount: u64,
) -> OrderQuoteResponse {
    let mut quote = if kind == OrderKind::Sell {
        sell_quote_response()
    } else {
        buy_quote_response()
    };
    quote.quote.sell_amount =
        Amount::new(sell_amount.to_string()).expect("generated sell amount must parse");
    quote.quote.buy_amount =
        Amount::new(buy_amount.to_string()).expect("generated buy amount must parse");
    quote.quote.set_network_cost_amount(
        Amount::new(fee_amount.to_string()).expect("generated fee amount must parse"),
    );
    quote.protocol_fee_bps = None;
    quote
}

fn current_thread_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio current-thread runtime must build")
}

fn calldata_word(data: &str, index: usize) -> &str {
    let stripped = data
        .strip_prefix("0x")
        .expect("encoded calldata must include 0x prefix");
    let start = 8 + (index * 64);
    &stripped[start..start + 64]
}

fn hex_word_to_u256(word: &str) -> U256 {
    U256::from_str_radix(word, 16).expect("encoded calldata words must remain hex")
}

fn order_kind_strategy() -> impl Strategy<Value = OrderKind> {
    any::<bool>().prop_map(|is_sell| {
        if is_sell {
            OrderKind::Sell
        } else {
            OrderKind::Buy
        }
    })
}

fn quote_amount_strategy() -> impl Strategy<Value = u64> {
    1_000_000u64..10_001_000_000u64
}

fn fee_strategy() -> impl Strategy<Value = u64> {
    1u64..1_000_000_000u64
}

fn price_quality_strategy() -> impl Strategy<Value = PriceQuality> {
    prop_oneof![
        Just(PriceQuality::Fast),
        Just(PriceQuality::Optimal),
        Just(PriceQuality::Verified),
    ]
}

fn partner_fee_strategy() -> impl Strategy<Value = Option<PartnerFee>> {
    proptest::option::of((1u16..=100).prop_map(|bps| {
        PartnerFeePolicy::volume(bps, address(ALT_RECEIVER))
            .expect("generated volume policy must validate")
            .into()
    }))
}

/// Mutually exclusive (`validFor` XOR `validTo`) trade validity.
fn validity_strategy() -> impl Strategy<Value = (Option<u32>, Option<u32>)> {
    prop_oneof![
        (60u32..86_460).prop_map(|valid_for| (Some(valid_for), None)),
        (1_700_000_000u32..1_700_500_000).prop_map(|valid_to| (None, Some(valid_to))),
    ]
}

/// Three-way quote-request override validity: unset, `validFor`, or `validTo`.
fn override_validity_strategy() -> impl Strategy<Value = (Option<u32>, Option<u32>)> {
    prop_oneof![
        Just((None, None)),
        (120u32..14_520).prop_map(|valid_for| (Some(valid_for), None)),
        (1_800_000_000u32..1_800_500_000).prop_map(|valid_to| (None, Some(valid_to))),
    ]
}

fn uint256_decimal_strategy() -> impl Strategy<Value = String> {
    any::<[u8; 32]>().prop_map(|bytes| {
        let value = U256::from_be_bytes(bytes);
        if value.is_zero() {
            U256::from(1u8).to_string()
        } else {
            value.to_string()
        }
    })
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(REGRESSION_FILE))),
        ..ProptestConfig::default()
    })]

    #[test]
    fn slippage_suggestions_are_monotonic_for_increasing_volume_multipliers(
        kind in order_kind_strategy(),
        sell_amount in quote_amount_strategy(),
        buy_amount in quote_amount_strategy(),
        fee_amount in fee_strategy(),
        is_eth_flow in any::<bool>(),
        low_tenths in 1u32..=500,
        extra_tenths in 1u32..=500,
    ) {
        let trader = trader();
        let trade = sample_trade_parameters(kind);
        let quote = make_quote(kind, sell_amount, buy_amount, fee_amount);
        let low_multiplier = f64::from(low_tenths) / 10.0;
        let high_multiplier = f64::from(low_tenths + extra_tenths) / 10.0;

        let low = suggest_slippage_bps(&quote, &trade, &trader, is_eth_flow, Some(low_multiplier))
            .expect("lower multiplier should produce a deterministic slippage suggestion");
        let high = suggest_slippage_bps(&quote, &trade, &trader, is_eth_flow, Some(high_multiplier))
            .expect("higher multiplier should produce a deterministic slippage suggestion");

        prop_assert!(
            low <= high,
            "slippage must not decrease when the volume multiplier increases"
        );
        prop_assert!(high <= MAX_SLIPPAGE_BPS);
        if is_eth_flow {
            prop_assert!(low >= 50);
        }
    }

    #[test]
    fn swap_params_to_limit_order_params_preserves_generated_quote_to_limit_shape(
        kind in order_kind_strategy(),
        sell_amount in quote_amount_strategy(),
        buy_amount in quote_amount_strategy(),
        fee_amount in fee_strategy(),
        owner_is_alt in proptest::option::of(any::<bool>()),
        set_receiver in any::<bool>(),
        set_staging in any::<bool>(),
        partially_fillable in any::<bool>(),
        slippage_bps in proptest::option::of(1u32..=500),
        partner_fee in partner_fee_strategy(),
        validity in validity_strategy(),
        quote_id_seed in any::<u32>(),
    ) {
        let mut trade = sample_trade_parameters(kind);
        let mut quote = make_quote(kind, sell_amount, buy_amount, fee_amount);

        trade.owner = owner_is_alt.map(|is_alt| {
            if is_alt {
                address(ALT_RECEIVER)
            } else {
                address(OWNER)
            }
        });
        trade.receiver = set_receiver.then(|| address(CUSTOM_SETTLEMENT));
        trade.env = set_staging.then_some(CowEnv::Staging);
        trade.partially_fillable = partially_fillable;
        trade.slippage_bps = slippage_bps;
        trade.partner_fee = partner_fee;
        trade.valid_for = validity.0;
        trade.valid_to = validity.1;
        quote.id = Some(i64::from(quote_id_seed));

        let from_quote = swap_params_to_limit_order_params(&trade, &quote)
            .expect("quote-to-limit conversion should remain deterministic");
        let limit = from_quote.as_limit();

        prop_assert_eq!(limit.kind, trade.kind);
        prop_assert_eq!(limit.owner, trade.owner);
        prop_assert_eq!(limit.sell_token, trade.sell_token);
        prop_assert_eq!(limit.buy_token, trade.buy_token);
        prop_assert_eq!(&limit.sell_amount, &quote.quote.sell_amount);
        prop_assert_eq!(&limit.buy_amount, &quote.quote.buy_amount);
        prop_assert_eq!(limit.quote_id, quote.id);
        prop_assert_eq!(from_quote.quote_id(), quote.id.expect("test seeds a quote id"));
        prop_assert_eq!(limit.env, trade.env);
        prop_assert_eq!(limit.partially_fillable, trade.partially_fillable);
        prop_assert_eq!(limit.slippage_bps, trade.slippage_bps);
        prop_assert_eq!(limit.receiver, trade.receiver);
        prop_assert_eq!(limit.valid_for, trade.valid_for);
        prop_assert_eq!(limit.valid_to, trade.valid_to);
        prop_assert_eq!(&limit.partner_fee, &trade.partner_fee);
    }

    #[test]
    fn ethflow_calldata_preserves_uint256_boundary_values(
        sell_amount in uint256_decimal_strategy(),
        buy_amount in uint256_decimal_strategy(),
        quote_id_seed in any::<u32>(),
        valid_to in any::<u32>(),
    ) {
        let runtime = current_thread_runtime();
        runtime.block_on(async {
            let signer = MockSigner::default();
            let trader = sample_trader_parameters();
            let mut params = sample_limit_parameters(OrderKind::Sell);
            let quote_id = i64::from(quote_id_seed);

            params.sell_token = address(EVM_NATIVE_CURRENCY_ADDRESS);
            params.sell_amount =
                Amount::new(sell_amount.clone()).expect("generated sell amount must remain valid");
            params.buy_amount =
                Amount::new(buy_amount.clone()).expect("generated buy amount must remain valid");
            params.quote_id = Some(quote_id);
            params.valid_to = Some(valid_to);

            let from_quote = LimitTradeParamsFromQuote::try_from_limit(params)
                .expect("test params carry a quote id");
            let transaction = eth_flow_transaction(
                &app_data_hash(),
                &from_quote,
                SupportedChainId::Sepolia,
                &PostTradeAdditionalParams::new().with_apply_costs_slippage_and_fees(false),
                &trader,
                &signer,
            )
            .await
            .expect("ethflow helper should encode deterministic uint256 values");
            let data = transaction
                .transaction
                .data
                .as_ref()
                .expect("ethflow transaction must include calldata");
            let hex = data.to_hex_string();

            prop_assert_eq!(
                transaction.order_to_sign.sell_amount.to_string(),
                sell_amount.clone()
            );
            prop_assert_eq!(
                transaction.order_to_sign.buy_amount.to_string(),
                buy_amount.clone()
            );
            prop_assert_eq!(
                hex_word_to_u256(calldata_word(&hex, 2)),
                U256::from_str_radix(&sell_amount, 10).unwrap()
            );
            prop_assert_eq!(
                hex_word_to_u256(calldata_word(&hex, 3)),
                U256::from_str_radix(&buy_amount, 10).unwrap()
            );
            // The canonical upstream EthFlowOrder.Data tuple lays validTo out at
            // word index 6 and quoteId at word index 8; intermediate words carry
            // appData, feeAmount, and partiallyFillable.
            prop_assert_eq!(hex_word_to_u256(calldata_word(&hex, 6)), U256::from(valid_to));
            prop_assert_eq!(
                hex_word_to_u256(calldata_word(&hex, 8)),
                U256::from(quote_id as u64)
            );
            Ok(())
        })?;
    }

    #[test]
    fn quote_results_preserve_generated_override_shape_across_request_and_order_boundaries(
        kind in order_kind_strategy(),
        sell_amount in quote_amount_strategy(),
        buy_amount in quote_amount_strategy(),
        fee_amount in fee_strategy(),
        trader_prod in any::<bool>(),
        set_trade_owner in any::<bool>(),
        set_trade_receiver in any::<bool>(),
        trade_partially_fillable in any::<bool>(),
        trade_slippage in proptest::option::of(1u32..=500),
        trade_partner_fee in partner_fee_strategy(),
        trade_validity in validity_strategy(),
        override_receiver in any::<bool>(),
        override_validity in override_validity_strategy(),
        override_from in any::<bool>(),
        override_price_quality in proptest::option::of(price_quality_strategy()),
        override_partially_fillable in proptest::option::of(any::<bool>()),
    ) {
        let runtime = current_thread_runtime();
        runtime.block_on(async {
            let orderbook = MockOrderbook::new(
                SupportedChainId::Sepolia,
                make_quote(kind, sell_amount, buy_amount, fee_amount),
            );
            let signer = MockSigner::default();
            let mut trader = sample_trader_parameters();
            let mut trade = sample_trade_parameters(kind);

            trader.env = trader_prod.then_some(CowEnv::Prod);
            trade.owner = set_trade_owner.then(|| address(OWNER));
            trade.receiver = set_trade_receiver.then(|| address(ALT_RECEIVER));
            trade.partially_fillable = trade_partially_fillable;
            trade.slippage_bps = trade_slippage;
            trade.partner_fee = trade_partner_fee;
            trade.valid_for = trade_validity.0;
            trade.valid_to = trade_validity.1;

            let mut quote_request = QuoteRequestOverride::new();
            if override_receiver {
                quote_request = quote_request.with_receiver(address(CUSTOM_SETTLEMENT));
            }
            if let Some(valid_for) = override_validity.0 {
                quote_request = quote_request.with_valid_for(valid_for);
            }
            if let Some(valid_to) = override_validity.1 {
                quote_request = quote_request.with_valid_to(valid_to);
            }
            if override_from {
                quote_request = quote_request.with_from(address(ALT_RECEIVER));
            }
            if let Some(price_quality) = override_price_quality {
                quote_request = quote_request.with_price_quality(price_quality);
            }
            if let Some(partially_fillable) = override_partially_fillable {
                quote_request = quote_request.with_partially_fillable(partially_fillable);
            }
            let advanced = TradeAdvancedSettings::new().with_quote_request(quote_request.clone());

            let result = quote_results(&trade, &trader, &signer, Some(&advanced), &orderbook)
                .await
                .expect("generated quote flow should remain deterministic");
            let request = orderbook
                .state()
                .quote_requests
                .last()
                .cloned()
                .expect("quote request must be recorded");
            let from_quote =
                swap_params_to_limit_order_params(&result.trade_parameters, &result.quote_response)
                    .expect("quote result should remain convertible into limit params");
            let limit = from_quote.as_limit();

            let expected_owner = quote_request.from.or(trade.owner).unwrap_or(signer.address);
            let expected_trade_receiver = quote_request.receiver.or(trade.receiver);
            let expected_request_receiver = Some(expected_trade_receiver.unwrap_or(expected_owner));
            let (expected_valid_for, expected_valid_to) =
                if let Some(valid_for) = quote_request.valid_for {
                    (Some(valid_for), None)
                } else if let Some(valid_to) = quote_request.valid_to {
                    (None, Some(valid_to))
                } else {
                    (trade.valid_for, trade.valid_to)
                };
            let expected_partially_fillable = quote_request
                .partially_fillable
                .unwrap_or(trade.partially_fillable);

            prop_assert_eq!(result.trade_parameters.owner, Some(expected_owner));
            prop_assert_eq!(request.from, expected_owner);
            prop_assert_eq!(result.trade_parameters.receiver, expected_trade_receiver);
            prop_assert_eq!(request.receiver, expected_request_receiver);
            prop_assert_eq!(result.trade_parameters.valid_for, expected_valid_for);
            prop_assert_eq!(result.trade_parameters.valid_to, expected_valid_to);
            let expected_validity = match (expected_valid_for, expected_valid_to) {
                (Some(valid_for), None) => QuoteValidity::ValidFor(valid_for),
                (None, Some(valid_to)) => QuoteValidity::ValidTo(valid_to),
                // build_quote_request defaults to the protocol 30-minute window
                // when neither side supplies a validity.
                (None, None) => QuoteValidity::ValidFor(1_800),
                (Some(_), Some(_)) => unreachable!("quote validity is mutually exclusive"),
            };
            prop_assert_eq!(request.validity, expected_validity);
            prop_assert_eq!(
                result.trade_parameters.partially_fillable,
                expected_partially_fillable
            );
            prop_assert_eq!(request.partially_fillable, expected_partially_fillable);
            if let Some(price_quality) = quote_request.price_quality {
                prop_assert_eq!(request.price_quality, price_quality);
            }
            prop_assert_eq!(
                result.order_to_sign.receiver,
                expected_trade_receiver.unwrap_or(request.from)
            );
            prop_assert_eq!(limit.owner, result.trade_parameters.owner);
            prop_assert_eq!(limit.sell_token, result.trade_parameters.sell_token);
            prop_assert_eq!(limit.buy_token, result.trade_parameters.buy_token);
            prop_assert_eq!(&limit.sell_amount, &result.quote_response.quote.sell_amount);
            prop_assert_eq!(&limit.buy_amount, &result.quote_response.quote.buy_amount);
            prop_assert_eq!(limit.quote_id, result.quote_response.id);
            prop_assert_eq!(limit.env, result.trade_parameters.env);
            prop_assert_eq!(
                limit.partially_fillable,
                result.trade_parameters.partially_fillable
            );
            prop_assert_eq!(limit.slippage_bps, result.trade_parameters.slippage_bps);
            prop_assert_eq!(limit.receiver, result.trade_parameters.receiver);
            prop_assert_eq!(limit.valid_for, result.trade_parameters.valid_for);
            prop_assert_eq!(limit.valid_to, result.trade_parameters.valid_to);
            prop_assert_eq!(&limit.partner_fee, &result.trade_parameters.partner_fee);
            Ok(())
        })?;
    }
}
