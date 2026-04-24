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

use cow_sdk_core::Amount;
use cow_sdk_orderbook::{
    BuyTokenDestination, GetOrdersRequest, GetTradesRequest, OrderQuoteRequest, OrderQuoteResponse,
    PriceQuality, QuoteSide, SellTokenSource, SigningScheme, calculate_total_fee,
};

use crate::common::{
    sample_app_data_hash, sample_buy_token, sample_order_json, sample_order_uid, sample_owner,
    sample_quote_response_json,
};

const CASE_COUNT: u64 = 128;

#[derive(Clone)]
struct CaseRng {
    state: u64,
}

impl CaseRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0xA076_1D64_78BD_642F,
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
}

fn generated_decimal(rng: &mut CaseRng) -> String {
    let mut value = 1 + (u64::from(rng.next_u32()) % 1_000_000_000_000);
    let mut digits = String::new();
    while value > 0 {
        digits.push(char::from(b'0' + (value % 10) as u8));
        value /= 10;
    }
    digits.chars().rev().collect()
}

fn generated_amount(rng: &mut CaseRng) -> Amount {
    Amount::new(generated_decimal(rng)).expect("generated amount must parse")
}

fn generated_price_quality(rng: &mut CaseRng) -> PriceQuality {
    match rng.next_u32() % 3 {
        0 => PriceQuality::Fast,
        1 => PriceQuality::Optimal,
        _ => PriceQuality::Verified,
    }
}

fn generated_signing_scheme(rng: &mut CaseRng) -> SigningScheme {
    match rng.next_u32() % 4 {
        0 => SigningScheme::Eip712,
        1 => SigningScheme::EthSign,
        2 => SigningScheme::Eip1271,
        _ => SigningScheme::PreSign,
    }
}

fn generated_sell_balance(rng: &mut CaseRng) -> SellTokenSource {
    match rng.next_u32() % 3 {
        0 => SellTokenSource::Erc20,
        1 => SellTokenSource::External,
        _ => SellTokenSource::Internal,
    }
}

fn generated_buy_balance(rng: &mut CaseRng) -> BuyTokenDestination {
    match rng.next_u32() % 2 {
        0 => BuyTokenDestination::Erc20,
        _ => BuyTokenDestination::Internal,
    }
}

fn generated_inline_app_data(seed: u64, rng: &mut CaseRng) -> String {
    format!(
        "{{\"appCode\":\"cow-rs/orderbook-property\",\"metadata\":{{\"seed\":{},\"nonce\":{}}}}}",
        seed,
        rng.next_u32() % 10_000
    )
}

#[test]
fn quote_request_shape_roundtrips_without_side_coercion() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed + 1);
        let is_sell = rng.next_bool();
        let side = if is_sell {
            QuoteSide::sell(generated_amount(&mut rng))
        } else {
            QuoteSide::buy(generated_amount(&mut rng))
        };
        let mut request =
            OrderQuoteRequest::new(sample_owner(), sample_buy_token(), sample_owner(), side)
                .with_price_quality(generated_price_quality(&mut rng))
                .with_signing_scheme(generated_signing_scheme(&mut rng))
                .with_sell_token_balance(generated_sell_balance(&mut rng))
                .with_buy_token_balance(generated_buy_balance(&mut rng));

        if rng.next_bool() {
            request = request.with_receiver(sample_buy_token());
        }
        if rng.next_bool() {
            request = request.with_valid_for(1 + (rng.next_u32() % 86_400));
        }
        if rng.next_bool() {
            request = request.with_valid_to(rng.next_u32());
        }
        if rng.next_bool() {
            request = request.with_timeout(u64::from(1 + (rng.next_u32() % 60_000)));
        }
        if rng.next_bool() {
            request = request.with_verification_gas_limit(u64::from(21_000 + rng.next_u32()));
        }
        if rng.next_bool() {
            request = request.with_partially_fillable();
        }
        if rng.next_bool() {
            request = request.with_onchain_order();
        }

        let value = serde_json::to_value(&request).expect("request serialization must succeed");

        assert!(request.is_valid());
        if is_sell {
            assert!(value.get("sellAmountBeforeFee").is_some());
            assert!(value.get("buyAmountAfterFee").is_none());
            assert_eq!(value.get("kind"), Some(&serde_json::json!("sell")));
        } else {
            assert!(value.get("buyAmountAfterFee").is_some());
            assert!(value.get("sellAmountBeforeFee").is_none());
            assert_eq!(value.get("kind"), Some(&serde_json::json!("buy")));
        }

        assert_eq!(
            value.get("receiver").is_some(),
            request.receiver.is_some(),
            "receiver presence must stay explicit through serialization"
        );
        assert_eq!(
            value.get("validFor").is_some(),
            request.valid_for.is_some(),
            "validFor presence must stay explicit through serialization"
        );
        assert_eq!(
            value.get("validTo").is_some(),
            request.valid_to.is_some(),
            "validTo presence must stay explicit through serialization"
        );
        assert_eq!(
            value.get("timeout").is_some(),
            request.timeout.is_some(),
            "timeout presence must stay explicit through serialization"
        );
        assert_eq!(
            value.get("verificationGasLimit").is_some(),
            request.verification_gas_limit.is_some(),
            "verificationGasLimit presence must stay explicit through serialization"
        );

        let roundtrip: OrderQuoteRequest =
            serde_json::from_value(value).expect("request roundtrip must remain stable");
        assert_eq!(roundtrip, request);
    }
}

#[test]
fn quote_request_app_data_and_pagination_shape_roundtrip_without_normalization() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed + 2_001);
        let mut request = OrderQuoteRequest::new(
            sample_owner(),
            sample_buy_token(),
            sample_owner(),
            if rng.next_bool() {
                QuoteSide::sell(generated_amount(&mut rng))
            } else {
                QuoteSide::buy(generated_amount(&mut rng))
            },
        );

        let inline_app_data = rng
            .next_bool()
            .then(|| generated_inline_app_data(seed, &mut rng));
        let app_data_hash = rng.next_bool().then(sample_app_data_hash);
        if let Some(app_data) = inline_app_data.clone() {
            request = request.with_app_data(app_data);
        }
        if let Some(hash) = app_data_hash.clone() {
            request = request.with_app_data_hash(hash);
        }

        let value = serde_json::to_value(&request).expect("request serialization must succeed");
        let roundtrip: OrderQuoteRequest =
            serde_json::from_value(value.clone()).expect("request roundtrip must remain stable");

        assert_eq!(roundtrip, request);
        assert_eq!(
            value.get("appData").and_then(serde_json::Value::as_str),
            Some(
                inline_app_data.as_deref().unwrap_or(
                    "0x0000000000000000000000000000000000000000000000000000000000000000"
                )
            ),
            "inline app-data must remain explicit through serialization"
        );
        assert_eq!(
            value.get("appDataHash").is_some(),
            app_data_hash.is_some(),
            "appDataHash presence must not be synthesized or dropped"
        );

        let owner_request = if rng.next_bool() {
            GetOrdersRequest::new(sample_owner())
        } else {
            GetOrdersRequest::new(sample_owner())
                .with_offset(rng.next_u32())
                .with_limit(1 + (rng.next_u32() % 5_000))
        };
        let owner_value =
            serde_json::to_value(&owner_request).expect("orders request must serialize");
        let owner_roundtrip: GetOrdersRequest =
            serde_json::from_value(owner_value).expect("orders request must deserialize");

        assert_eq!(owner_roundtrip, owner_request);
        if owner_request.offset == 0 && owner_request.limit == 1_000 {
            assert_eq!(owner_request, GetOrdersRequest::new(sample_owner()));
        }
    }
}

#[test]
fn trades_request_filter_xor_is_never_silently_normalized() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed + 4_001);
        let owner = rng.next_bool().then(sample_owner);
        let order_uid = rng.next_bool().then(sample_order_uid);
        let request = GetTradesRequest::new(owner.clone(), order_uid.clone())
            .with_offset(rng.next_u32() % 500)
            .with_limit(1 + (rng.next_u32() % 100));

        let value = serde_json::to_value(&request).expect("trades request must serialize");
        let roundtrip: GetTradesRequest =
            serde_json::from_value(value).expect("trades request must deserialize");

        assert_eq!(request.is_valid(), owner.is_some() ^ order_uid.is_some());
        assert_eq!(roundtrip, request);
        assert_eq!(roundtrip.is_valid(), request.is_valid());
    }
}

#[test]
fn malformed_payloads_fail_closed_in_decoding_and_transforms() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed + 8_001);
        match rng.next_u32() % 4 {
            0 => {
                let mut response = sample_quote_response_json();
                response["quote"]["appData"] = serde_json::json!("not-a-hash");
                response["quote"]
                    .as_object_mut()
                    .expect("quote fixture must remain an object")
                    .remove("appDataHash");

                let error = serde_json::from_value::<OrderQuoteResponse>(response)
                    .expect_err("malformed app-data hashes must fail closed");
                assert!(!error.to_string().is_empty());
            }
            1 => {
                let mut response = sample_quote_response_json();
                response["quote"]["kind"] = serde_json::json!("hold");

                let error = serde_json::from_value::<OrderQuoteResponse>(response)
                    .expect_err("unsupported order kinds must fail closed");
                assert!(!error.to_string().is_empty());
            }
            2 => {
                let mut order_json = sample_order_json(&sample_order_uid());
                order_json["executedFee"] = serde_json::json!("abc");

                let error = serde_json::from_value::<cow_sdk_orderbook::Order>(order_json)
                    .expect_err("invalid executedFee must fail at typed wire boundary");
                assert!(error.to_string().contains("amount"));
            }
            _ => {
                let mut order_json = sample_order_json(&sample_order_uid());
                order_json["executedFee"] = serde_json::json!("12z");

                let error = serde_json::from_value::<cow_sdk_orderbook::Order>(order_json)
                    .expect_err("invalid executedFee must fail at typed wire boundary");
                assert!(error.to_string().contains("amount"));
            }
        }
    }
}

#[test]
fn fee_normalization_trims_leading_zeroes_across_generated_decimal_inputs() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed + 12_001);
        let value = generated_decimal(&mut rng);
        let padded = format!("{}{}", "0".repeat((rng.next_u32() % 3) as usize), value);

        let expected = value
            .parse::<u128>()
            .expect("generated decimal must parse")
            .to_string();
        let total_fee = calculate_total_fee(Some(&padded))
            .expect("generated decimal normalization must remain valid");

        assert_eq!(
            total_fee,
            Amount::new(expected).expect("expected amount must parse")
        );
    }
}
