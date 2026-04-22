mod common;

use cow_sdk_orderbook::{EVM_NATIVE_CURRENCY_ADDRESS, Order, calculate_total_fee, transform_order};
use serde_json::Value;

use crate::common::{sample_ethflow_order_json, sample_order_json, sample_order_uid, sample_owner};

fn order_with_fee_fields(
    executed_fee: Option<&str>,
    executed_fee_amount_legacy: Option<&str>,
) -> Order {
    let uid = sample_order_uid();
    let mut payload = sample_order_json(&uid);
    let object = payload
        .as_object_mut()
        .expect("sample order fixture must be a JSON object");
    object.remove("executedFee");
    object.remove("executedFeeAmount");
    if let Some(value) = executed_fee {
        object.insert("executedFee".to_owned(), Value::String(value.to_owned()));
    }
    if let Some(value) = executed_fee_amount_legacy {
        object.insert(
            "executedFeeAmount".to_owned(),
            Value::String(value.to_owned()),
        );
    }
    serde_json::from_value::<Order>(payload).expect("test order payload must deserialize")
}

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

#[test]
fn total_fee_is_executed_fee_when_both_populated() {
    let order = order_with_fee_fields(Some("10"), Some("20"));
    assert_eq!(order.executed_fee.as_deref(), Some("10"));
    assert_eq!(
        order.executed_fee_amount_legacy.as_deref(),
        Some("20"),
        "legacy executedFeeAmount must deserialize into the read-only sibling field",
    );

    let transformed = transform_order(order).expect("order must transform");
    assert_eq!(
        transformed.total_fee, "10",
        "total_fee must equal the canonical executedFee value when both fields are populated",
    );
    assert_eq!(
        transformed.executed_fee_amount_legacy.as_deref(),
        Some("20"),
        "transform must preserve the legacy field byte-identical without folding it into total_fee",
    );
}

#[test]
fn total_fee_is_executed_fee_when_only_executed_fee_present() {
    let order = order_with_fee_fields(Some("10"), None);
    assert_eq!(order.executed_fee.as_deref(), Some("10"));
    assert_eq!(
        order.executed_fee_amount_legacy, None,
        "absent executedFeeAmount on the wire must deserialize as None",
    );

    let transformed = transform_order(order).expect("order must transform");
    assert_eq!(
        transformed.total_fee, "10",
        "total_fee must equal the canonical executedFee value when the legacy field is absent",
    );
    assert_eq!(
        transformed.executed_fee_amount_legacy, None,
        "transform must not invent a legacy value when none was on the wire",
    );
}

#[test]
fn total_fee_is_zero_when_only_legacy_field_present() {
    let order = order_with_fee_fields(None, Some("20"));
    assert_eq!(
        order.executed_fee, None,
        "missing executedFee on the wire must deserialize as None",
    );
    assert_eq!(order.executed_fee_amount_legacy.as_deref(), Some("20"));

    let transformed = transform_order(order).expect("order must transform");
    assert_eq!(
        transformed.total_fee, "0",
        "total_fee must default to zero when the canonical executedFee is absent, regardless of the legacy field",
    );
    assert_eq!(
        transformed.executed_fee_amount_legacy.as_deref(),
        Some("20"),
        "transform must keep the legacy value reachable for callers that need to compute the legacy summation explicitly",
    );
}

#[test]
fn total_fee_is_zero_when_neither_field_populated() {
    let order = order_with_fee_fields(None, None);
    assert_eq!(order.executed_fee, None);
    assert_eq!(order.executed_fee_amount_legacy, None);

    let transformed = transform_order(order).expect("order must transform");
    assert_eq!(
        transformed.total_fee, "0",
        "total_fee must default to zero when neither fee field is populated",
    );
    assert_eq!(
        transformed.executed_fee_amount_legacy, None,
        "transform must not synthesize a legacy value when neither fee field is populated",
    );
}
