use chrono::NaiveDate;
use policy_maintainer::check_wasm_runner_freshness::{analyze_value, validate_report};
use serde_json::json;

#[test]
fn wasm_runner_freshness_accepts_recent_release_date() {
    let today = NaiveDate::from_ymd_opt(2026, 4, 28).unwrap();
    let value = json!({
        "chrome": {
            "version": "124.0.0",
            "released_at": "2026-04-01"
        }
    });

    let report = analyze_value(&value, today).unwrap();
    assert!(validate_report(&report).is_empty());
}

#[test]
fn wasm_runner_freshness_rejects_stale_or_missing_release_date() {
    let today = NaiveDate::from_ymd_opt(2026, 4, 28).unwrap();
    let stale = json!({"chrome": {"released_at": "2025-12-01"}});
    let stale_report = analyze_value(&stale, today).unwrap();
    assert!(validate_report(&stale_report)[0].contains("maximum allowed age"));

    let missing = analyze_value(&json!({"chrome": {"version": "124"}}), today).unwrap();
    assert!(validate_report(&missing)[0].contains("does not contain"));
}
