mod common;

use cow_sdk_core::{Amount, EVM_NATIVE_CURRENCY_ADDRESS, OrderKind, SupportedChainId};
use cow_sdk_trading::{
    MAX_SLIPPAGE_BPS, PostTradeAdditionalParams, QuoterParameters, get_eth_flow_transaction,
    suggest_slippage_bps,
};
use num_bigint::BigUint;

use crate::common::{
    MockSigner, OWNER, address, app_data_hash, buy_quote_response, sample_limit_parameters,
    sample_trade_parameters, sample_trader_parameters, sell_quote_response,
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
    QuoterParameters {
        chain_id: SupportedChainId::Sepolia,
        app_code: "0x007".to_owned(),
        account: address(OWNER),
        env: None,
        settlement_contract_override: None,
        eth_flow_contract_override: None,
    }
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

        let low = suggest_slippage_bps(
            &quote,
            &trade,
            &trader,
            is_ethflow,
            Some(low_multiplier),
        )
        .expect("lower multiplier should produce a deterministic slippage suggestion");
        let high = suggest_slippage_bps(
            &quote,
            &trade,
            &trader,
            is_ethflow,
            Some(high_multiplier),
        )
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
            &PostTradeAdditionalParams {
                apply_costs_slippage_and_fees: Some(false),
                ..PostTradeAdditionalParams::default()
            },
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
