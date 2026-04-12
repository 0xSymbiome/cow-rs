mod common;

use cow_sdk_orderbook::{
    GetTradesRequest, Order, OrderBalance, OrderQuoteRequest, OrderQuoteResponse, PriceQuality,
    QuoteSide, SigningScheme, transform_order,
};

use crate::common::{
    sample_buy_token, sample_order_json, sample_order_uid, sample_owner, sample_quote_response_json,
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

fn generated_balance(rng: &mut CaseRng) -> OrderBalance {
    match rng.next_u32() % 3 {
        0 => OrderBalance::Erc20,
        1 => OrderBalance::External,
        _ => OrderBalance::Internal,
    }
}

#[test]
fn quote_request_shape_roundtrips_without_side_coercion() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed + 1);
        let is_sell = rng.next_bool();
        let side = if is_sell {
            QuoteSide::sell(generated_decimal(&mut rng))
        } else {
            QuoteSide::buy(generated_decimal(&mut rng))
        };
        let mut request = OrderQuoteRequest::new(
            sample_owner(),
            sample_buy_token(),
            sample_owner(),
            side,
        )
        .with_price_quality(generated_price_quality(&mut rng))
        .with_signing_scheme(generated_signing_scheme(&mut rng))
        .with_sell_token_balance(generated_balance(&mut rng))
        .with_buy_token_balance(generated_balance(&mut rng));

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
fn trades_request_filter_xor_is_never_silently_normalized() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed + 4_001);
        let owner = rng.next_bool().then(sample_owner);
        let order_uid = rng.next_bool().then(sample_order_uid);
        let request = GetTradesRequest {
            owner: owner.clone(),
            order_uid: order_uid.clone(),
            offset: rng.next_u32() % 500,
            limit: 1 + (rng.next_u32() % 100),
        };

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
                order_json["executedFeeAmount"] = serde_json::json!("abc");

                let order: Order =
                    serde_json::from_value(order_json).expect("order fixture must deserialize");
                let error =
                    transform_order(order).expect_err("invalid executedFeeAmount must fail closed");
                assert!(matches!(
                    error,
                    cow_sdk_orderbook::OrderbookError::InvalidTransform(_)
                ));
            }
            _ => {
                let mut order_json = sample_order_json(&sample_order_uid());
                order_json["executedFee"] = serde_json::json!("12z");

                let order: Order =
                    serde_json::from_value(order_json).expect("order fixture must deserialize");
                let error =
                    transform_order(order).expect_err("invalid executedFee must fail closed");
                assert!(matches!(
                    error,
                    cow_sdk_orderbook::OrderbookError::InvalidTransform(_)
                ));
            }
        }
    }
}
