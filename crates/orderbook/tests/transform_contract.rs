mod common;

use cow_sdk_orderbook::{
    AuctionOrder, EVM_NATIVE_CURRENCY_ADDRESS, OnchainOrderData, Order, OrderQuoteResponse,
    StoredOrderQuote, Trade, calculate_total_fee, transform_order,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::common::{sample_ethflow_order_json, sample_order_json, sample_order_uid, sample_owner};

fn amount(value: &str) -> cow_sdk_core::Amount {
    cow_sdk_core::Amount::new(value).expect("test amount literal must be valid")
}

fn assert_fixture_fields_roundtrip<T>(
    fixture_name: &str,
    raw: &str,
    fields: &[&str],
) -> (T, Value, Value)
where
    T: DeserializeOwned + Serialize,
{
    let expected: Value = serde_json::from_str(raw)
        .unwrap_or_else(|error| panic!("{fixture_name} must be valid JSON: {error}"));
    let typed: T = serde_json::from_value(expected.clone())
        .unwrap_or_else(|error| panic!("{fixture_name} must deserialize: {error}"));
    let actual = serde_json::to_value(&typed)
        .unwrap_or_else(|error| panic!("{fixture_name} must serialize: {error}"));

    for field in fields {
        assert_eq!(
            actual.get(*field),
            expected.get(*field),
            "{fixture_name}: OpenAPI inventory field `{field}` must round-trip",
        );
    }

    (typed, expected, actual)
}

#[test]
fn order_fixture_matches_openapi_inventory() {
    let (order, _, _) = assert_fixture_fields_roundtrip::<Order>(
        "order_with_full_metadata.json",
        include_str!("../../../parity/fixtures/orderbook/order_with_full_metadata.json"),
        &[
            "appData",
            "appDataHash",
            "availableBalance",
            "buyAmount",
            "buyToken",
            "buyTokenBalance",
            "class",
            "creationDate",
            "ethflowData",
            "executedBuyAmount",
            "executedFee",
            "executedFeeAmount",
            "executedFeeToken",
            "executedSellAmount",
            "executedSellAmountBeforeFees",
            "feeAmount",
            "from",
            "fullAppData",
            "fullBalanceCheck",
            "interactions",
            "invalidated",
            "isLiquidityOrder",
            "kind",
            "onchainOrderData",
            "onchainUser",
            "owner",
            "partiallyFillable",
            "quote",
            "quoteId",
            "receiver",
            "sellAmount",
            "sellToken",
            "sellTokenBalance",
            "settlementContract",
            "signature",
            "signingScheme",
            "status",
            "uid",
            "validTo",
        ],
    );

    assert_eq!(
        order.settlement_contract.as_str(),
        "0x0000000000000000000000000000000000000006"
    );
    assert_eq!(order.is_liquidity_order, Some(false));
    assert_eq!(order.executed_fee_amount, amount("4000000000000000"));
    assert!(
        order
            .interactions
            .as_ref()
            .and_then(|value| value.pre.as_ref())
            .is_some(),
        "order_with_full_metadata.json: interactions.pre must deserialize",
    );
    assert!(
        order.quote.as_ref().is_some_and(|quote| quote.verified),
        "order_with_full_metadata.json: quote.verified must deserialize",
    );
}

#[test]
fn auction_order_fixture_matches_openapi_inventory() {
    let (order, _, _) = assert_fixture_fields_roundtrip::<AuctionOrder>(
        "auction_order_with_protocol_fees.json",
        include_str!("../../../parity/fixtures/orderbook/auction_order_with_protocol_fees.json"),
        &[
            "appData",
            "buyAmount",
            "buyToken",
            "buyTokenBalance",
            "class",
            "created",
            "executed",
            "kind",
            "owner",
            "partiallyFillable",
            "postInteractions",
            "preInteractions",
            "protocolFees",
            "quote",
            "receiver",
            "sellAmount",
            "sellToken",
            "sellTokenBalance",
            "signature",
            "uid",
            "validTo",
        ],
    );

    assert_eq!(order.protocol_fees.len(), 1);
    assert_eq!(order.pre_interactions[0].call_data, "0x");
    assert_eq!(
        order.quote.as_ref().and_then(|quote| quote.fee.as_ref()),
        Some(&amount("3000000000000000")),
        "auction_order_with_protocol_fees.json: auction-side quote fee must deserialize",
    );
}

#[test]
fn order_quote_response_fixture_matches_openapi_inventory() {
    let (response, _, _) = assert_fixture_fields_roundtrip::<OrderQuoteResponse>(
        "order_quote_response.json",
        include_str!("../../../parity/fixtures/orderbook/order_quote_response.json"),
        &[
            "expiration",
            "from",
            "id",
            "protocolFeeBps",
            "quote",
            "verified",
        ],
    );

    assert_eq!(response.id, Some(42));
    assert_eq!(response.protocol_fee_bps.as_deref(), Some("2"));
    assert_eq!(
        response.quote.network_cost_amount(),
        &amount("3000000000000000")
    );
}

#[test]
fn trade_fixture_matches_openapi_inventory() {
    let (trade, _, _) = assert_fixture_fields_roundtrip::<Trade>(
        "trade.json",
        include_str!("../../../parity/fixtures/orderbook/trade.json"),
        &[
            "blockNumber",
            "buyAmount",
            "buyToken",
            "executedProtocolFees",
            "logIndex",
            "orderUid",
            "owner",
            "sellAmount",
            "sellAmountBeforeFees",
            "sellToken",
            "txHash",
        ],
    );

    assert_eq!(trade.sell_amount_before_fees, amount("90000000000000000"));
    assert_eq!(trade.executed_protocol_fees.as_ref().map(Vec::len), Some(1));
    assert_eq!(
        trade.tx_hash.as_ref().map(ToString::to_string).as_deref(),
        Some("0x1111111111111111111111111111111111111111111111111111111111111111"),
    );
}

#[test]
fn stored_order_quote_fixture_matches_openapi_inventory() {
    let (quote, _, _) = assert_fixture_fields_roundtrip::<StoredOrderQuote>(
        "stored_order_quote.json",
        include_str!("../../../parity/fixtures/orderbook/stored_order_quote.json"),
        &[
            "buyAmount",
            "feeAmount",
            "gasAmount",
            "gasPrice",
            "metadata",
            "sellAmount",
            "sellTokenPrice",
            "solver",
            "verified",
        ],
    );

    assert_eq!(quote.gas_amount, "150000");
    assert_eq!(quote.fee_amount, amount("3000000000000000"));
    assert!(
        quote.metadata.is_some(),
        "stored_order_quote.json: metadata must deserialize"
    );
}

#[test]
fn onchain_order_data_fixture_matches_openapi_inventory() {
    let (data, _, _) = assert_fixture_fields_roundtrip::<OnchainOrderData>(
        "onchain_order_data.json",
        include_str!("../../../parity/fixtures/orderbook/onchain_order_data.json"),
        &["placementError", "sender"],
    );

    assert_eq!(
        data.sender.as_str(),
        "0x0000000000000000000000000000000000000005"
    );
    assert_eq!(data.placement_error.as_deref(), Some("none"));
}

fn order_with_fee_fields(executed_fee: Option<&str>, executed_fee_amount: Option<&str>) -> Order {
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
    if let Some(value) = executed_fee_amount {
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

    assert_eq!(total_fee, amount("9"));
}

#[test]
fn total_fee_transform_defaults_missing_executed_fee_to_zero() {
    let total_fee = calculate_total_fee(None).expect("missing executed fee defaults to zero");

    assert_eq!(total_fee, amount("0"));
}

#[test]
fn total_fee_transform_trims_leading_zeroes_on_normalized_input() {
    let total_fee = calculate_total_fee(Some("000099")).expect("normalization must succeed");

    assert_eq!(total_fee, amount("99"));
}

#[test]
fn total_fee_transform_treats_all_zero_input_as_single_zero() {
    let total_fee = calculate_total_fee(Some("0000")).expect("all-zero input must normalize");

    assert_eq!(total_fee, amount("0"));
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
    assert_eq!(transformed.total_fee, amount("10"));
}

#[test]
fn regular_order_transform_keeps_order_shape_and_adds_total_fee() {
    let uid = sample_order_uid();
    let order = serde_json::from_value::<Order>(sample_order_json(&uid))
        .expect("order fixture must deserialize");
    let transformed = transform_order(order).expect("order must transform");

    assert_eq!(transformed.uid, uid);
    assert_eq!(transformed.owner, sample_owner());
    assert_eq!(transformed.total_fee, amount("20"));
}

#[test]
fn total_fee_is_executed_fee_when_both_populated() {
    let order = order_with_fee_fields(Some("10"), Some("20"));
    assert_eq!(order.executed_fee.as_ref(), Some(&amount("10")));
    assert_eq!(
        order.executed_fee_amount,
        amount("20"),
        "legacy executedFeeAmount must deserialize into the read-only sibling field",
    );

    let transformed = transform_order(order).expect("order must transform");
    assert_eq!(
        transformed.total_fee,
        amount("10"),
        "total_fee must equal the canonical executedFee value when both fields are populated",
    );
    assert_eq!(
        transformed.executed_fee_amount,
        amount("20"),
        "transform must preserve the legacy field byte-identical without folding it into total_fee",
    );
}

#[test]
fn total_fee_is_executed_fee_when_only_executed_fee_present() {
    let order = order_with_fee_fields(Some("10"), None);
    assert_eq!(order.executed_fee.as_ref(), Some(&amount("10")));
    assert_eq!(
        order.executed_fee_amount,
        amount("0"),
        "absent executedFeeAmount on the wire must deserialize as zero",
    );

    let transformed = transform_order(order).expect("order must transform");
    assert_eq!(
        transformed.total_fee,
        amount("10"),
        "total_fee must equal the canonical executedFee value when the legacy field is absent",
    );
    assert_eq!(
        transformed.executed_fee_amount,
        amount("0"),
        "transform must keep the default legacy value at zero when none was on the wire",
    );
}

#[test]
fn total_fee_is_zero_when_only_legacy_field_present() {
    let order = order_with_fee_fields(None, Some("20"));
    assert_eq!(
        order.executed_fee, None,
        "missing executedFee on the wire must deserialize as None",
    );
    assert_eq!(order.executed_fee_amount, amount("20"));

    let transformed = transform_order(order).expect("order must transform");
    assert_eq!(
        transformed.total_fee,
        amount("0"),
        "total_fee must default to zero when the canonical executedFee is absent, regardless of the legacy field",
    );
    assert_eq!(
        transformed.executed_fee_amount,
        amount("20"),
        "transform must keep the legacy value reachable for callers that need to compute the legacy summation explicitly",
    );
}

#[test]
fn total_fee_is_zero_when_neither_field_populated() {
    let order = order_with_fee_fields(None, None);
    assert_eq!(order.executed_fee, None);
    assert_eq!(order.executed_fee_amount, amount("0"));

    let transformed = transform_order(order).expect("order must transform");
    assert_eq!(
        transformed.total_fee,
        amount("0"),
        "total_fee must default to zero when neither fee field is populated",
    );
    assert_eq!(
        transformed.executed_fee_amount,
        amount("0"),
        "transform must keep the default legacy value at zero when neither fee field is populated",
    );
}
