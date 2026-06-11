mod common;

use cow_sdk_orderbook::{
    NATIVE_CURRENCY_ADDRESS, OnchainOrderData, Order, OrderQuoteResponse,
    SolverCompetitionResponse, StoredOrderQuote, Trade, calculate_total_fee, transform_order,
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
    let expected = fixture_payload(fixture_name, raw);
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

/// Extracts the wire document from a fixture's `payload` envelope; the
/// provenance header around it is validated by `cargo parity-validate`.
fn fixture_payload(fixture_name: &str, raw: &str) -> Value {
    let fixture: Value = serde_json::from_str(raw)
        .unwrap_or_else(|error| panic!("{fixture_name} must be valid JSON: {error}"));
    fixture
        .get("payload")
        .cloned()
        .unwrap_or_else(|| panic!("{fixture_name} must carry a payload envelope"))
}

#[test]
fn order_fixture_deserializes_nested_typed_accessors() {
    // The `Order` wire shape round-trips byte-for-byte in `wire_contract.rs`, so
    // this test pins only the typed accessors and nested deserialization a
    // round-trip comparison does not surface.
    let order: Order = serde_json::from_value(fixture_payload(
        "order_with_full_metadata.json",
        include_str!("../../../parity/fixtures/orderbook/order_with_full_metadata.json"),
    ))
    .expect("order_with_full_metadata.json must deserialize");

    assert_eq!(
        order.settlement_contract.to_hex_string(),
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
fn order_quote_response_fixture_deserializes_typed_accessors() {
    let response: OrderQuoteResponse = serde_json::from_value(fixture_payload(
        "order_quote_response.json",
        include_str!("../../../parity/fixtures/orderbook/order_quote_response.json"),
    ))
    .expect("order_quote_response.json must deserialize");

    assert_eq!(response.id, Some(42));
    assert_eq!(response.protocol_fee_bps.as_deref(), Some("2"));
    assert_eq!(
        response.quote.network_cost_amount(),
        &amount("3000000000000000")
    );
}

#[test]
fn trade_fixture_deserializes_typed_accessors() {
    let trade: Trade = serde_json::from_value(fixture_payload(
        "trade.json",
        include_str!("../../../parity/fixtures/orderbook/trade.json"),
    ))
    .expect("trade.json must deserialize");

    assert_eq!(trade.sell_amount_before_fees, amount("90000000000000000"));
    assert_eq!(trade.executed_protocol_fees.as_ref().map(Vec::len), Some(1));
    assert_eq!(
        trade.tx_hash.as_ref().map(ToString::to_string).as_deref(),
        Some("0x1111111111111111111111111111111111111111111111111111111111111111"),
    );
}

#[test]
fn stored_order_quote_fixture_deserializes_typed_accessors() {
    let quote: StoredOrderQuote = serde_json::from_value(fixture_payload(
        "stored_order_quote.json",
        include_str!("../../../parity/fixtures/orderbook/stored_order_quote.json"),
    ))
    .expect("stored_order_quote.json must deserialize");

    assert_eq!(quote.gas_amount, "150000");
    assert_eq!(quote.fee_amount, amount("3000000000000000"));
    assert!(
        quote.metadata.is_some(),
        "stored_order_quote.json: metadata must deserialize"
    );
}

#[test]
fn onchain_order_data_fixture_deserializes_typed_accessors() {
    let data: OnchainOrderData = serde_json::from_value(fixture_payload(
        "onchain_order_data.json",
        include_str!("../../../parity/fixtures/orderbook/onchain_order_data.json"),
    ))
    .expect("onchain_order_data.json must deserialize");

    assert_eq!(
        data.sender.to_hex_string(),
        "0x0000000000000000000000000000000000000005"
    );
    assert_eq!(data.placement_error.as_deref(), Some("none"));
}

// Source-locked to the upstream services producer's own v2 serialization
// vector: the `Response` type in `services/crates/model/src/solver_competition_v2.rs`
// is the struct serialized behind `/api/v2/solver_competition/*`. The vendored
// orderbook OpenAPI omits a `required:` block for this schema, so the producer's
// optionality (identity and collection fields non-optional; only `txHash` and
// `referenceScore` optional) is the authoritative contract this fixture pins.
// The type is therefore covered here by a producer-pinned round-trip rather than
// the OpenAPI-optionality manifest (see ADR 0031 and docs/parity.md).
#[test]
fn solver_competition_response_fixture_roundtrips_upstream_producer_vector() {
    let (response, _, _) = assert_fixture_fields_roundtrip::<SolverCompetitionResponse>(
        "solver_competition_response.json",
        include_str!("../../../parity/fixtures/orderbook/solver_competition_response.json"),
        &[
            "auctionId",
            "auctionStartBlock",
            "auctionDeadlineBlock",
            "transactionHashes",
            "referenceScores",
            "auction",
            "solutions",
        ],
    );

    assert_eq!(response.auction_id, 0);
    assert_eq!(response.auction_start_block, 13);
    assert_eq!(response.auction_deadline_block, 100);
    assert_eq!(response.transaction_hashes.len(), 1);
    assert_eq!(response.reference_scores.len(), 1);
    assert_eq!(response.auction.orders.len(), 1);
    assert_eq!(response.auction.prices.len(), 1);

    let solution = response
        .solutions
        .first()
        .expect("upstream vector carries one solution");
    assert_eq!(solution.ranking, 1);
    assert!(solution.is_winner);
    assert!(!solution.filtered_out);
    assert_eq!(solution.score, amount("123"));
    assert_eq!(
        solution.reference_score.as_ref(),
        Some(&amount("10")),
        "solver-settlement reference score must deserialize through the typed Amount",
    );
    assert!(
        solution.tx_hash.is_some(),
        "settlement transaction hash must deserialize"
    );
    assert_eq!(solution.clearing_prices.len(), 1);

    let touched = solution
        .orders
        .first()
        .expect("upstream vector carries one touched order");
    assert_eq!(touched.sell_amount, amount("12"));
    assert_eq!(touched.buy_amount, amount("13"));
    assert!(
        touched.buy_token.is_some() && touched.sell_token.is_some(),
        "touched-order token addresses must be captured rather than dropped",
    );
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
    assert_eq!(transformed.sell_token, NATIVE_CURRENCY_ADDRESS);
    assert_eq!(transformed.valid_to, 1_700_000_123);
    assert_eq!(transformed.total_fee, amount("10"));
}

#[test]
fn signing_order_projects_regular_order_and_fails_closed_for_ethflow() {
    let uid = sample_order_uid();

    // A regular order's response fields still equal the signed order, so the
    // projection is available and preserves every hashing input verbatim.
    let order = serde_json::from_value::<Order>(sample_order_json(&uid))
        .expect("order fixture must deserialize");
    let projected = order
        .signing_order()
        .expect("a regular order must project to a user-domain order");
    let projected = serde_json::to_value(&projected).expect("user-domain order must serialize");
    let wire = serde_json::to_value(&order).expect("response order must serialize");
    for field in [
        "sellToken",
        "buyToken",
        "receiver",
        "sellAmount",
        "buyAmount",
        "validTo",
        "appData",
        "kind",
        "sellTokenBalance",
        "buyTokenBalance",
    ] {
        assert_eq!(
            projected.get(field),
            wire.get(field),
            "signing_order projection must preserve `{field}` from the response order",
        );
    }

    // An eth-flow order's response fields are rewritten by the transform
    // (valid_to/owner/sell_token), so the projection fails closed instead of
    // returning a user-domain order that cannot reproduce the on-chain digest.
    let ethflow = serde_json::from_value::<Order>(sample_ethflow_order_json(&uid))
        .expect("ethflow fixture must deserialize");
    assert!(
        ethflow.signing_order().is_none(),
        "an eth-flow order must not project to a re-derivable user-domain order",
    );
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
fn total_fee_x_executed_fee_amount_matrix_holds_for_zero_legacy_zero_canonical_legacy_only_canonical_only()
 {
    for (label, executed_fee, executed_fee_amount, expected_total_fee, expected_legacy) in [
        ("zero legacy", Some("0"), Some("20"), "0", "20"),
        ("zero canonical", Some("10"), Some("0"), "10", "0"),
        ("legacy only", None, Some("20"), "0", "20"),
        ("canonical only", Some("10"), None, "10", "0"),
        ("both populated", Some("10"), Some("20"), "10", "20"),
        ("neither", None, None, "0", "0"),
    ] {
        let order = order_with_fee_fields(executed_fee, executed_fee_amount);
        let transformed = transform_order(order)
            .unwrap_or_else(|error| panic!("{label} must transform: {error}"));

        assert_eq!(
            transformed.total_fee,
            amount(expected_total_fee),
            "{label}: total_fee must follow canonical executedFee only",
        );
        assert_eq!(
            transformed.executed_fee_amount,
            amount(expected_legacy),
            "{label}: legacy executedFeeAmount must remain preserved",
        );
    }
}
