use std::collections::HashMap;

use cow_sdk_core::Amount;
use serde_json::Value;

const CORE_FIXTURE: &str = include_str!("../../../parity/fixtures/core.json");
const ORDERBOOK_FIXTURE: &str = include_str!("../../../parity/fixtures/orderbook.json");
const TRADING_FIXTURE: &str = include_str!("../../../parity/fixtures/trading.json");

#[test]
fn cross_fixture_amount_roundtrip() {
    let mut all_amounts = Vec::new();

    for (fixture, raw) in [
        ("core", CORE_FIXTURE),
        ("orderbook", ORDERBOOK_FIXTURE),
        ("trading", TRADING_FIXTURE),
    ] {
        let value: Value =
            serde_json::from_str(raw).unwrap_or_else(|err| panic!("{fixture}: {err}"));
        collect_amount_strings(&value, fixture, &mut all_amounts);
    }

    assert!(
        !all_amounts.is_empty(),
        "fixtures must expose at least one amount-shaped string"
    );

    // The cow `Amount` newtype is `#[repr(transparent)]` over
    // `alloy_primitives::U256` per ADR 0052, so cross-fixture parity is
    // checked at the typed level: two fixtures that share an amount
    // literal must decode to the same typed `Amount`, which by the
    // newtype's `Eq` / `Ord` impls compares the inner U256 bit-for-bit.
    let mut by_literal: HashMap<String, (String, String, Amount)> = HashMap::new();
    for (fixture, path, amount_str) in all_amounts {
        let amount = Amount::new(&amount_str).unwrap_or_else(|err| panic!("{path}: {err}"));
        assert_eq!(
            amount.to_string(),
            amount_str,
            "{path}: amount string did not round-trip byte-identically"
        );

        match by_literal.insert(amount_str, (fixture.clone(), path.clone(), amount)) {
            Some((prior_fixture, prior_path, prior_amount)) if prior_fixture != fixture => {
                assert_eq!(
                    prior_amount, amount,
                    "{path} and {prior_path} disagree on the decoded Amount"
                );
            }
            _ => {}
        }
    }
}

fn collect_amount_strings(value: &Value, fixture: &str, into: &mut Vec<(String, String, String)>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let child_path = format!("{fixture}.{key}");
                match child {
                    Value::String(raw) if marks_amount_value(key) => {
                        into.push((fixture_root(fixture).to_string(), child_path, raw.clone()));
                    }
                    _ => collect_amount_strings(child, &child_path, into),
                }
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                collect_amount_strings(child, &format!("{fixture}[{index}]"), into);
            }
        }
        _ => {}
    }
}

fn fixture_root(path: &str) -> &str {
    path.split(['.', '[']).next().unwrap_or(path)
}

fn marks_amount_value(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase();
    if normalized.ends_with("formula")
        || normalized.ends_with("path")
        || normalized.ends_with("template")
        || normalized.ends_with("fields")
        || normalized.ends_with("methods")
    {
        return false;
    }

    normalized.contains("amount")
        || normalized.contains("fee")
        || normalized.ends_with("value")
        || normalized.ends_with("gas")
}
