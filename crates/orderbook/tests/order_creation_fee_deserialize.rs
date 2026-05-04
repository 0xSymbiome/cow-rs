//! Regression and property coverage for the order-submission fee boundary.
//!
//! `OrderCreation` intentionally accepts only a zero order-level
//! `feeAmount`. Quote responses keep their separate network-cost `feeAmount`
//! behavior and must continue to accept non-zero values.

#![allow(
    clippy::redundant_clone,
    clippy::too_many_lines,
    reason = "wire DTO tests keep explicit payloads close to their assertions"
)]

use cow_sdk_orderbook::{Amount, OrderCreation, QuoteData};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Value, json};

const ADDRESS_1: &str = "0x0000000000000000000000000000000000000001";
const ADDRESS_2: &str = "0x0000000000000000000000000000000000000002";
const ADDRESS_3: &str = "0x0000000000000000000000000000000000000003";
const APP_DATA: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";
const NON_ZERO_FEE_ERROR: &str = "non-zero feeAmount is not accepted for OrderCreation";
const REGRESSION_FILE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/proptest-regressions/order_creation_fee_deserialize.txt"
);

fn amount(value: &str) -> Amount {
    Amount::new(value).expect("test amount literal must be valid")
}

fn order_creation_payload(fee_amount: Option<&str>) -> Value {
    let mut payload = json!({
        "sellToken": ADDRESS_1,
        "buyToken": ADDRESS_2,
        "receiver": ADDRESS_3,
        "sellAmount": "1000000000000000000",
        "buyAmount": "2000000000000000000",
        "validTo": 1_700_000_000u32,
        "appData": "{\"version\":\"1.0.0\",\"metadata\":{}}",
        "appDataHash": APP_DATA,
        "kind": "sell",
        "partiallyFillable": false,
        "sellTokenBalance": "erc20",
        "buyTokenBalance": "erc20",
        "signingScheme": "eip712",
        "signature": "0x1234",
        "from": ADDRESS_3,
        "quoteId": 7
    });

    if let Some(fee_amount) = fee_amount {
        payload["feeAmount"] = json!(fee_amount);
    }

    payload
}

fn deserialize_order_creation(
    fee_amount: Option<&str>,
) -> Result<OrderCreation, serde_json::Error> {
    serde_json::from_value(order_creation_payload(fee_amount))
}

#[test]
fn order_creation_deserialize_accepts_zero_or_omitted_fee_amount() {
    let explicit_zero =
        deserialize_order_creation(Some("0")).expect("zero feeAmount must deserialize");
    let omitted = deserialize_order_creation(None).expect("omitted feeAmount must deserialize");

    let explicit_zero_wire =
        serde_json::to_value(&explicit_zero).expect("explicit zero order must serialize");
    let omitted_wire = serde_json::to_value(&omitted).expect("omitted-fee order must serialize");

    assert_eq!(explicit_zero_wire["feeAmount"], json!("0"));
    assert_eq!(omitted_wire["feeAmount"], json!("0"));
}

#[test]
fn order_creation_deserialize_rejects_non_zero_fee_amount() {
    for fee_amount in [
        "1",
        "100",
        "0001",
        "340282366920938463463374607431768211455",
    ] {
        let error = deserialize_order_creation(Some(fee_amount))
            .expect_err("non-zero feeAmount must reject during deserialization");
        assert!(
            error.to_string().contains(NON_ZERO_FEE_ERROR),
            "error must carry stable substring for {fee_amount}: {error}"
        );
    }
}

#[test]
fn order_creation_deserialize_keeps_malformed_fee_amount_parser_error() {
    for malformed in ["not-a-decimal", "12z", ""] {
        let error = deserialize_order_creation(Some(malformed))
            .expect_err("malformed feeAmount must reject before the zero-fee guard");
        assert!(
            error.to_string().contains("amount"),
            "malformed feeAmount should retain amount parser context: {error}"
        );
        assert!(
            !error.to_string().contains(NON_ZERO_FEE_ERROR),
            "malformed feeAmount must not be reported as the non-zero fee guard: {error}"
        );
    }
}

#[test]
fn quote_data_deserialize_keeps_non_zero_network_cost_fee_amount() {
    let quote: QuoteData = serde_json::from_value(json!({
        "sellToken": ADDRESS_1,
        "buyToken": ADDRESS_2,
        "receiver": ADDRESS_3,
        "sellAmount": "1000000000000000000",
        "buyAmount": "2000000000000000000",
        "validTo": 1_700_000_000u32,
        "appData": APP_DATA,
        "feeAmount": "3000000000000000",
        "kind": "sell",
        "partiallyFillable": true,
        "sellTokenBalance": "erc20",
        "buyTokenBalance": "internal"
    }))
    .expect("QuoteData must keep accepting non-zero network-cost feeAmount");

    assert_eq!(quote.network_cost_amount(), &amount("3000000000000000"));

    let rendered = serde_json::to_value(&quote).expect("QuoteData must serialize");
    assert_eq!(rendered["feeAmount"], json!("3000000000000000"));
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(REGRESSION_FILE))),
        ..ProptestConfig::default()
    })]

    #[test]
    fn order_creation_deserialize_fee_amount_boundary_is_zero_only(fee in any::<u128>()) {
        let fee_amount = fee.to_string();
        let result = deserialize_order_creation(Some(&fee_amount));

        if fee == 0 {
            prop_assert!(result.is_ok(), "zero feeAmount must deserialize");
        } else {
            let error = result.expect_err("non-zero feeAmount must reject");
            prop_assert!(
                error.to_string().contains(NON_ZERO_FEE_ERROR),
                "non-zero feeAmount error must carry stable substring: {error}"
            );
        }
    }
}
