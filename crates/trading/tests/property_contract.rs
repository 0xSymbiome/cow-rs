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

use cow_sdk_core::{Amount, CowEnv, EVM_NATIVE_CURRENCY_ADDRESS, OrderKind, SupportedChainId};
use cow_sdk_orderbook::PriceQuality;
use cow_sdk_trading::{
    MAX_SLIPPAGE_BPS, PartnerFee, PartnerFeePolicy, PostTradeAdditionalParams,
    QuoteRequestOverride, QuoterParameters, SwapAdvancedSettings, get_eth_flow_transaction,
    get_quote_results, suggest_slippage_bps, swap_params_to_limit_order_params,
};
use num_bigint::BigUint;

use crate::common::{
    ALT_RECEIVER, CUSTOM_SETTLEMENT, MockOrderbook, MockSigner, OWNER, address, app_data_hash,
    buy_quote_response, sample_limit_parameters, sample_trade_parameters, sample_trader_parameters,
    sell_quote_response,
};

const CASE_COUNT: u64 = 128;

#[derive(Clone)]
struct CaseRng {
    state: u64,
}

impl CaseRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0x9E37_79B9_7F4A_7C15,
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value << 7;
        value ^= value >> 9;
        value ^= value << 8;
        self.state = value;
        value
    }

    fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 16) as u32
    }

    fn next_bool(&mut self) -> bool {
        (self.next_u64() & 1) == 1
    }

    fn fill(&mut self, bytes: &mut [u8]) {
        for byte in bytes {
            *byte = (self.next_u64() & 0xff) as u8;
        }
    }
}

fn trader() -> QuoterParameters {
    QuoterParameters::new(SupportedChainId::Sepolia, "0x007", address(OWNER))
}

fn generated_uint256_decimal(rng: &mut CaseRng, max_bytes: usize) -> String {
    let mut bytes = vec![0u8; 1 + (rng.next_u64() as usize % max_bytes.max(1))];
    rng.fill(&mut bytes);
    if bytes.iter().all(|byte| *byte == 0) {
        let last = bytes.len() - 1;
        bytes[last] = 1;
    }
    BigUint::from_bytes_be(&bytes).to_string()
}

fn generated_quote(kind: OrderKind, rng: &mut CaseRng) -> cow_sdk_orderbook::OrderQuoteResponse {
    let mut quote = if kind == OrderKind::Sell {
        sell_quote_response()
    } else {
        buy_quote_response()
    };
    let sell_amount = 1_000_000u64 + (rng.next_u64() % 10_000_000_000);
    let buy_amount = 1_000_000u64 + (rng.next_u64() % 10_000_000_000);
    let fee_amount = 1 + (rng.next_u64() % (sell_amount / 10).max(1));

    quote.quote.sell_amount = sell_amount.to_string();
    quote.quote.buy_amount = buy_amount.to_string();
    quote.quote.fee_amount = fee_amount.to_string();
    quote.protocol_fee_bps = None;
    quote
}

fn generated_price_quality(rng: &mut CaseRng) -> PriceQuality {
    match rng.next_u32() % 3 {
        0 => PriceQuality::Fast,
        1 => PriceQuality::Optimal,
        _ => PriceQuality::Verified,
    }
}

fn generated_validity(rng: &mut CaseRng) -> (Option<u32>, Option<u32>) {
    if rng.next_bool() {
        (Some(60 + (rng.next_u32() % 86_400)), None)
    } else {
        (None, Some(1_700_000_000 + (rng.next_u32() % 500_000)))
    }
}

fn generated_partner_fee(rng: &mut CaseRng) -> Option<PartnerFee> {
    rng.next_bool()
        .then(|| PartnerFeePolicy::volume(1 + (rng.next_u32() % 100), address(ALT_RECEIVER)).into())
}

fn generated_optional_override_validity(rng: &mut CaseRng) -> (Option<u32>, Option<u32>) {
    match rng.next_u32() % 3 {
        0 => (None, None),
        1 => (Some(120 + (rng.next_u32() % 14_400)), None),
        _ => (None, Some(1_800_000_000 + (rng.next_u32() % 500_000))),
    }
}

fn calldata_word(data: &str, index: usize) -> &str {
    let stripped = data
        .strip_prefix("0x")
        .expect("encoded calldata must include 0x prefix");
    let start = 8 + (index * 64);
    &stripped[start..start + 64]
}

fn hex_word_to_biguint(word: &str) -> BigUint {
    BigUint::parse_bytes(word.as_bytes(), 16).expect("encoded calldata words must remain hex")
}

#[test]
fn slippage_suggestions_are_monotonic_for_increasing_volume_multipliers() {
    let trader = trader();

    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed + 1);
        let kind = if rng.next_bool() {
            OrderKind::Sell
        } else {
            OrderKind::Buy
        };
        let trade = sample_trade_parameters(kind);
        let quote = generated_quote(kind, &mut rng);
        let is_ethflow = rng.next_bool();
        let low_multiplier = f64::from(1 + (rng.next_u32() % 500)) / 10.0;
        let high_multiplier = low_multiplier + f64::from(1 + (rng.next_u32() % 500)) / 10.0;

        let low = suggest_slippage_bps(&quote, &trade, &trader, is_ethflow, Some(low_multiplier))
            .expect("lower multiplier should produce a deterministic slippage suggestion");
        let high = suggest_slippage_bps(&quote, &trade, &trader, is_ethflow, Some(high_multiplier))
            .expect("higher multiplier should produce a deterministic slippage suggestion");

        assert!(
            low <= high,
            "slippage must not decrease when the volume multiplier increases"
        );
        assert!(high <= MAX_SLIPPAGE_BPS);
        if is_ethflow {
            assert!(low >= 50);
        }
    }
}

#[test]
fn swap_params_to_limit_order_params_preserves_generated_quote_to_limit_shape() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed + 5_001);
        let kind = if rng.next_bool() {
            OrderKind::Sell
        } else {
            OrderKind::Buy
        };
        let mut trade = sample_trade_parameters(kind);
        let mut quote = generated_quote(kind, &mut rng);

        trade.owner = rng.next_bool().then(|| {
            if rng.next_bool() {
                address(OWNER)
            } else {
                address(ALT_RECEIVER)
            }
        });
        trade.receiver = rng.next_bool().then(|| address(CUSTOM_SETTLEMENT));
        trade.env = rng.next_bool().then_some(CowEnv::Staging);
        trade.partially_fillable = rng.next_bool();
        trade.slippage_bps = rng.next_bool().then(|| 1 + (rng.next_u32() % 500));
        trade.partner_fee = generated_partner_fee(&mut rng);
        let (valid_for, valid_to) = generated_validity(&mut rng);
        trade.valid_for = valid_for;
        trade.valid_to = valid_to;
        quote.id = Some(i64::from(rng.next_u32()));

        let limit = swap_params_to_limit_order_params(&trade, &quote)
            .expect("quote-to-limit conversion should remain deterministic");

        assert_eq!(limit.kind, trade.kind);
        assert_eq!(limit.owner, trade.owner);
        assert_eq!(limit.sell_token, trade.sell_token);
        assert_eq!(limit.buy_token, trade.buy_token);
        assert_eq!(limit.sell_token_decimals, trade.sell_token_decimals);
        assert_eq!(limit.buy_token_decimals, trade.buy_token_decimals);
        assert_eq!(limit.sell_amount.as_str(), quote.quote.sell_amount);
        assert_eq!(limit.buy_amount.as_str(), quote.quote.buy_amount);
        assert_eq!(limit.quote_id, quote.id);
        assert_eq!(limit.env, trade.env);
        assert_eq!(limit.partially_fillable, trade.partially_fillable);
        assert_eq!(limit.slippage_bps, trade.slippage_bps);
        assert_eq!(limit.receiver, trade.receiver);
        assert_eq!(limit.valid_for, trade.valid_for);
        assert_eq!(limit.valid_to, trade.valid_to);
        assert_eq!(limit.partner_fee, trade.partner_fee);
    }
}

#[tokio::test]
async fn ethflow_calldata_preserves_uint256_boundary_values() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed + 10_001);
        let signer = MockSigner::default();
        let trader = sample_trader_parameters();
        let mut params = sample_limit_parameters(OrderKind::Sell);
        let sell_amount = generated_uint256_decimal(&mut rng, 32);
        let buy_amount = generated_uint256_decimal(&mut rng, 32);
        let quote_id = i64::from(rng.next_u32());
        let valid_to = rng.next_u32();

        params.sell_token = address(EVM_NATIVE_CURRENCY_ADDRESS);
        params.sell_amount =
            Amount::new(sell_amount.clone()).expect("generated sell amount must remain valid");
        params.buy_amount =
            Amount::new(buy_amount.clone()).expect("generated buy amount must remain valid");
        params.quote_id = Some(quote_id);
        params.valid_to = Some(valid_to);

        let transaction = get_eth_flow_transaction(
            &app_data_hash(),
            &params,
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

        assert_eq!(transaction.order_to_sign.sell_amount.as_str(), sell_amount);
        assert_eq!(transaction.order_to_sign.buy_amount.as_str(), buy_amount);
        assert_eq!(
            hex_word_to_biguint(calldata_word(data.as_str(), 2)),
            BigUint::parse_bytes(sell_amount.as_bytes(), 10).unwrap()
        );
        assert_eq!(
            hex_word_to_biguint(calldata_word(data.as_str(), 3)),
            BigUint::parse_bytes(buy_amount.as_bytes(), 10).unwrap()
        );
        assert_eq!(
            hex_word_to_biguint(calldata_word(data.as_str(), 6)),
            BigUint::from(quote_id as u64)
        );
        assert_eq!(
            hex_word_to_biguint(calldata_word(data.as_str(), 8)),
            BigUint::from(valid_to)
        );
    }
}

#[tokio::test]
async fn quote_results_preserve_generated_override_shape_across_request_and_order_boundaries() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed + 20_001);
        let kind = if rng.next_bool() {
            OrderKind::Sell
        } else {
            OrderKind::Buy
        };
        let orderbook =
            MockOrderbook::new(SupportedChainId::Sepolia, generated_quote(kind, &mut rng));
        let signer = MockSigner::default();
        let mut trader = sample_trader_parameters();
        let mut trade = sample_trade_parameters(kind);
        let (valid_for, valid_to) = generated_validity(&mut rng);

        trader.env = rng.next_bool().then_some(CowEnv::Prod);
        trade.owner = rng.next_bool().then(|| address(OWNER));
        trade.receiver = rng.next_bool().then(|| address(ALT_RECEIVER));
        trade.partially_fillable = rng.next_bool();
        trade.slippage_bps = rng.next_bool().then(|| 1 + (rng.next_u32() % 500));
        trade.partner_fee = generated_partner_fee(&mut rng);
        trade.valid_for = valid_for;
        trade.valid_to = valid_to;
        let (override_valid_for, override_valid_to) =
            generated_optional_override_validity(&mut rng);

        let mut quote_request = QuoteRequestOverride::new();
        if let Some(receiver) = rng.next_bool().then(|| address(CUSTOM_SETTLEMENT)) {
            quote_request = quote_request.with_receiver(receiver);
        }
        if let Some(valid_for) = override_valid_for {
            quote_request = quote_request.with_valid_for(valid_for);
        }
        if let Some(valid_to) = override_valid_to {
            quote_request = quote_request.with_valid_to(valid_to);
        }
        if let Some(from) = rng.next_bool().then(|| address(ALT_RECEIVER)) {
            quote_request = quote_request.with_from(from);
        }
        if let Some(price_quality) = rng.next_bool().then(|| generated_price_quality(&mut rng)) {
            quote_request = quote_request.with_price_quality(price_quality);
        }
        if let Some(partially_fillable) = rng.next_bool().then(|| rng.next_bool()) {
            quote_request = quote_request.with_partially_fillable(partially_fillable);
        }
        let advanced = SwapAdvancedSettings::new().with_quote_request(quote_request.clone());

        let result = get_quote_results(&trade, &trader, &signer, Some(&advanced), &orderbook)
            .await
            .expect("generated quote flow should remain deterministic");
        let request = orderbook
            .state()
            .quote_requests
            .last()
            .cloned()
            .expect("quote request must be recorded");
        let limit =
            swap_params_to_limit_order_params(&result.trade_parameters, &result.quote_response)
                .expect("quote result should remain convertible into limit params");

        let expected_owner = quote_request
            .from
            .clone()
            .or_else(|| trade.owner.clone())
            .unwrap_or_else(|| signer.address.clone());
        let expected_trade_receiver = quote_request
            .receiver
            .clone()
            .or_else(|| trade.receiver.clone());
        let expected_request_receiver = Some(
            expected_trade_receiver
                .clone()
                .unwrap_or_else(|| expected_owner.clone()),
        );
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

        assert_eq!(result.trade_parameters.owner, Some(expected_owner.clone()));
        assert_eq!(request.from, expected_owner);
        assert_eq!(result.trade_parameters.receiver, expected_trade_receiver);
        assert_eq!(request.receiver, expected_request_receiver);
        assert_eq!(result.trade_parameters.valid_for, expected_valid_for);
        assert_eq!(result.trade_parameters.valid_to, expected_valid_to);
        assert_eq!(request.valid_for, expected_valid_for);
        assert_eq!(request.valid_to, expected_valid_to);
        assert_eq!(
            result.trade_parameters.partially_fillable,
            expected_partially_fillable
        );
        assert_eq!(request.partially_fillable, expected_partially_fillable);
        if let Some(price_quality) = quote_request.price_quality {
            assert_eq!(request.price_quality, price_quality);
        }
        assert_eq!(
            result.order_to_sign.receiver,
            expected_trade_receiver.unwrap_or_else(|| request.from.clone())
        );
        assert_eq!(limit.owner, result.trade_parameters.owner);
        assert_eq!(limit.sell_token, result.trade_parameters.sell_token);
        assert_eq!(limit.buy_token, result.trade_parameters.buy_token);
        assert_eq!(
            limit.sell_amount.as_str(),
            result.quote_response.quote.sell_amount
        );
        assert_eq!(
            limit.buy_amount.as_str(),
            result.quote_response.quote.buy_amount
        );
        assert_eq!(limit.quote_id, result.quote_response.id);
        assert_eq!(limit.env, result.trade_parameters.env);
        assert_eq!(
            limit.partially_fillable,
            result.trade_parameters.partially_fillable
        );
        assert_eq!(limit.slippage_bps, result.trade_parameters.slippage_bps);
        assert_eq!(limit.receiver, result.trade_parameters.receiver);
        assert_eq!(limit.valid_for, result.trade_parameters.valid_for);
        assert_eq!(limit.valid_to, result.trade_parameters.valid_to);
        assert_eq!(limit.partner_fee, result.trade_parameters.partner_fee);
    }
}
