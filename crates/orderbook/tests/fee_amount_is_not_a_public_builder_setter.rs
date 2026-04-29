//! Code-level enforcement that `fee_amount` is not a public builder setter
//! on any cow-sdk-orderbook DTO, together with a positive round-trip witness
//! that order submissions still emit `"feeAmount": "0"` on the wire.
//!
//! The cow-protocol services backend rejects orders with a non-zero
//! order-level fee, so the submission path always wires `"feeAmount": "0"`
//! and no public Rust builder exposes a `fee_amount(...)` setter. The
//! compile-fail invariant is asserted by the module-level `compile_fail`
//! doctests on `cow_sdk_orderbook`, which are executed by `cargo test
//! --doc`. The runtime tests in this file prove that submission and
//! order-response wire shapes stay aligned with the retained EIP-712
//! struct-hash contract.

use cow_sdk_core::{Address, Amount, AppDataHash, OrderKind};
use cow_sdk_orderbook::{OrderCreation, QuoteData, SigningScheme};

fn amount(value: &str) -> Amount {
    Amount::new(value).expect("test amount literal must be valid")
}

#[test]
fn order_creation_wire_emits_fee_amount_zero_exactly_once() {
    let address = Address::new("0x0000000000000000000000000000000000000001")
        .expect("literal address must parse");

    let order = OrderCreation::new(
        address.clone(),
        address.clone(),
        amount("1000"),
        amount("900"),
        1_700_000_000,
        OrderKind::Sell,
        SigningScheme::Eip712,
        "0x",
        address,
    );

    let wire = serde_json::to_value(&order).expect("OrderCreation must serialize");
    assert_eq!(
        wire.get("feeAmount"),
        Some(&serde_json::Value::String("0".to_owned())),
        "OrderCreation must wire feeAmount as the zero string for EIP-712 compatibility",
    );

    let wire_string = serde_json::to_string(&order).expect("OrderCreation must stringify");
    let occurrences = wire_string.matches("\"feeAmount\":\"0\"").count();
    assert_eq!(
        occurrences, 1,
        "OrderCreation wire form must emit the feeAmount zero string exactly once",
    );
}

#[test]
fn order_creation_from_quote_zeroes_fee_amount_on_submission() {
    let address = Address::new("0x0000000000000000000000000000000000000001")
        .expect("literal address must parse");
    let app_data =
        AppDataHash::new("0x0000000000000000000000000000000000000000000000000000000000000000")
            .expect("literal app-data hash must parse");

    let quote = QuoteData::new(
        address.clone(),
        address.clone(),
        amount("1000"),
        amount("900"),
        1_700_000_000,
        app_data,
        OrderKind::Sell,
    )
    .with_network_cost_amount(amount("12345"));

    assert_eq!(
        quote.network_cost_amount(),
        &amount("12345"),
        "QuoteData must surface the configured network-cost amount through its accessor",
    );

    let order = OrderCreation::from_quote(&quote, address, None, SigningScheme::Eip712, "0x");
    let wire = serde_json::to_value(&order).expect("OrderCreation must serialize");
    assert_eq!(
        wire.get("feeAmount"),
        Some(&serde_json::Value::String("0".to_owned())),
        "OrderCreation::from_quote must never propagate the quote network cost into the signed order",
    );
}

#[test]
fn order_response_wire_form_excludes_zero_legacy_executed_fee_amount_and_full_fee_amount() {
    use serde_json::json;

    // The current services schema retains `executedFee`; the legacy
    // `executedFeeAmount` value stays read-only and is not re-emitted when it
    // was absent or zero on the wire.
    let payload = json!({
        "sellToken": "0x0000000000000000000000000000000000000002",
        "buyToken": "0x0000000000000000000000000000000000000003",
        "sellAmount": "1000",
        "buyAmount": "900",
        "validTo": 1_700_000_000u32,
        "appData": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "feeAmount": "0",
        "kind": "sell",
        "partiallyFillable": false,
        "sellTokenBalance": "erc20",
        "buyTokenBalance": "erc20",
        "signingScheme": "eip712",
        "signature": "0x",
        "owner": "0x0000000000000000000000000000000000000004",
        "uid": "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        "creationDate": "2024-01-01T00:00:00Z",
        "status": "open",
        "class": "market",
        "executedSellAmount": "0",
        "executedSellAmountBeforeFees": "0",
        "executedBuyAmount": "0",
        "executedFee": "0",
        "executedFeeAmount": "0",
        "settlementContract": "0x0000000000000000000000000000000000000004"
    });
    let order: cow_sdk_orderbook::Order =
        serde_json::from_value(payload).expect("order response must deserialize");
    let roundtrip =
        serde_json::to_value(&order).expect("order response must re-serialize for inspection");

    assert!(
        roundtrip.get("executedFeeAmount").is_none(),
        "Order responses must not re-emit a zero legacy executedFeeAmount descriptor",
    );
    assert!(
        roundtrip.get("fullFeeAmount").is_none(),
        "Order responses must not re-emit the retired fullFeeAmount descriptor",
    );
    assert_eq!(
        roundtrip.get("executedFee"),
        Some(&serde_json::Value::String("0".to_owned())),
        "Order responses must surface the executedFee component on the wire",
    );
}
