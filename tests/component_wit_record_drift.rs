//! Drift gate for the component's hand-mirrored `book.order` / `book.trade` WIT
//! records.
//!
//! The `cow-sdk-component` WIT mirrors the native `cow_sdk_orderbook::Order` /
//! `Trade` serde shapes by hand so a polyglot consumer destructures typed
//! records instead of parsing JSON. This test pins that mirror: it serializes
//! the native type (via the full-metadata parity fixture) and asserts every
//! emitted wire field is present in the WIT record. A native field added
//! upstream that the WIT does not yet mirror **fails CI** here, rather than
//! being silently dropped from the typed surface — the cost-container that keeps
//! the hand-mirror safe (ADR 0071).

use cow_sdk_orderbook::{Order, Trade};

const WIT: &str = include_str!("../crates/component/wit/world.wit");
const ORDER_FIXTURE: &str =
    include_str!("../parity/fixtures/orderbook/order_with_full_metadata.json");
const TRADE_FIXTURE: &str = include_str!("../parity/fixtures/orderbook/trade.json");

/// kebab-case → camelCase (`sell-token` → `sellToken`); `%`-escaped WIT keywords
/// (`%from`) keep their bare name.
fn kebab_to_camel(name: &str) -> String {
    let name = name.trim_start_matches('%');
    let mut out = String::new();
    let mut upper = false;
    for ch in name.chars() {
        if ch == '-' {
            upper = true;
        } else if upper {
            out.extend(ch.to_uppercase());
            upper = false;
        } else {
            out.push(ch);
        }
    }
    out
}

/// Extracts the camelCased field names of a flat `record <name> { … }` from the
/// WIT source. The mirrored records are flat (their nested records are separate
/// `record` items), so the first `}` closes the body.
fn wit_record_fields(record: &str) -> Vec<String> {
    let head = format!("record {record} {{");
    let start = WIT
        .find(&head)
        .unwrap_or_else(|| panic!("`{head}` not found in world.wit"));
    let body = &WIT[start + head.len()..];
    let end = body.find('}').expect("unterminated record");
    body[..end]
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                return None;
            }
            let field = line.split(':').next()?.trim();
            (!field.is_empty()).then(|| kebab_to_camel(field))
        })
        .collect()
}

/// Asserts every wire field the native type serializes is mirrored in the WIT
/// record. (Subset, not equality: the WIT may carry optional fields the fixture
/// omits — the drop we guard against is a *native* field missing from the WIT.)
fn assert_no_drift(native: &serde_json::Value, record: &str) {
    let wit_fields = wit_record_fields(record);
    let object = native
        .as_object()
        .expect("native value serializes to an object");
    let missing: Vec<&String> = object
        .keys()
        .filter(|key| !wit_fields.contains(key))
        .collect();
    assert!(
        missing.is_empty(),
        "WIT drift: native `{record}` serializes field(s) {missing:?} that the \
         component `book.{record}` WIT record does not mirror. Add them to \
         crates/component/wit/world.wit and the `to_wit_{record}!` lowering \
         (crates/component/src/client/orderbook.rs). Mirrored WIT fields: {wit_fields:?}",
    );
}

/// The parity fixtures wrap the DTO under a `payload` key alongside provenance
/// metadata; the typed value is that payload.
fn payload(fixture: &str) -> serde_json::Value {
    serde_json::from_str::<serde_json::Value>(fixture)
        .expect("fixture is valid JSON")
        .get("payload")
        .expect("fixture has a payload")
        .clone()
}

#[test]
fn book_order_record_mirrors_native_order() {
    let order: Order =
        serde_json::from_value(payload(ORDER_FIXTURE)).expect("order fixture deserializes");
    let value = serde_json::to_value(&order).expect("order serializes");
    assert_no_drift(&value, "order");
}

#[test]
fn book_trade_record_mirrors_native_trade() {
    let trade: Trade =
        serde_json::from_value(payload(TRADE_FIXTURE)).expect("trade fixture deserializes");
    let value = serde_json::to_value(&trade).expect("trade serializes");
    assert_no_drift(&value, "trade");
}
