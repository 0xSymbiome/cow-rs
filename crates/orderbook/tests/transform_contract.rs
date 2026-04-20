mod common;

use cow_sdk_orderbook::{EVM_NATIVE_CURRENCY_ADDRESS, Order, calculate_total_fee, transform_order};

use crate::common::{sample_ethflow_order_json, sample_order_json, sample_order_uid, sample_owner};

#[test]
fn total_fee_transform_surfaces_executed_fee_value() {
    let total_fee = calculate_total_fee(Some("9")).expect("fee normalization must succeed");

    assert_eq!(total_fee, "9");
}

#[test]
fn total_fee_transform_defaults_missing_executed_fee_to_zero() {
    let total_fee = calculate_total_fee(None).expect("missing executed fee defaults to zero");

    assert_eq!(total_fee, "0");
}

#[test]
fn total_fee_transform_trims_leading_zeroes_on_normalized_input() {
    let total_fee = calculate_total_fee(Some("000099")).expect("normalization must succeed");

    assert_eq!(total_fee, "99");
}

#[test]
fn total_fee_transform_treats_all_zero_input_as_single_zero() {
    let total_fee = calculate_total_fee(Some("0000")).expect("all-zero input must normalize");

    assert_eq!(total_fee, "0");
}

#[test]
fn total_fee_transform_rejects_invalid_decimal_input() {
    let error = calculate_total_fee(Some("nope")).expect_err("invalid decimal should fail");

    match error {
        cow_sdk_orderbook::OrderbookError::InvalidTransform { field, reason } => {
            assert_eq!(field, "executedFee");
            let rendered = reason.to_string();
            assert!(rendered.contains("unsigned decimal string"));
        }
        other => panic!("expected InvalidTransform, got {other:?}"),
    }
}

#[test]
fn ethflow_transform_rewrites_owner_sell_token_and_valid_to() {
    let uid = sample_order_uid();
    let order = serde_json::from_value::<Order>(sample_ethflow_order_json(&uid))
        .expect("ethflow fixture must deserialize");
    let transformed = transform_order(order).expect("ethflow order must transform");

    assert_eq!(transformed.owner, sample_owner());
    assert_eq!(transformed.sell_token.as_str(), EVM_NATIVE_CURRENCY_ADDRESS);
    assert_eq!(transformed.valid_to, 1_700_000_123);
    assert_eq!(transformed.total_fee, "10");
}

#[test]
fn regular_order_transform_keeps_order_shape_and_adds_total_fee() {
    let uid = sample_order_uid();
    let order = serde_json::from_value::<Order>(sample_order_json(&uid))
        .expect("order fixture must deserialize");
    let transformed = transform_order(order).expect("order must transform");

    assert_eq!(transformed.uid, uid);
    assert_eq!(transformed.owner, sample_owner());
    assert_eq!(transformed.total_fee, "20");
}
