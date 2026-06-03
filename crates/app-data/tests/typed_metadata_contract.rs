//! Contract tests for typed app-data metadata validation.
//!
//! These cover the typed slippage bound and the `AppDataParams::validate`
//! pass that bound-checks SDK-modelled metadata families while leaving
//! unmodelled and earlier-shaped metadata untouched.

use cow_sdk_app_data::{AppDataError, AppDataParams, QuoteMetadata};
use cow_sdk_core::AppCode;
use serde_json::{Map, Value, json};

fn metadata_with(key: &str, value: Value) -> Map<String, Value> {
    let mut metadata = Map::new();
    metadata.insert(key.to_owned(), value);
    metadata
}

#[test]
fn quote_metadata_rejects_slippage_above_ten_thousand_bips() {
    let error = QuoteMetadata::new(20_000).expect_err("slippage above 10000 bps must be rejected");
    assert!(matches!(
        error,
        AppDataError::InvalidAppDataProvided { field, .. }
            if field == "metadata.quote.slippageBips"
    ));
}

#[test]
fn quote_metadata_accepts_slippage_within_bound() {
    assert!(QuoteMetadata::new(50).is_ok());
    assert!(QuoteMetadata::new(10_000).is_ok());
}

#[test]
fn params_validate_rejects_out_of_range_quote_in_metadata() {
    let code = AppCode::new("my-app").expect("valid app code");
    let params = AppDataParams::new(code)
        .with_metadata(metadata_with("quote", json!({ "slippageBips": 20_000 })));

    assert!(params.validate().is_err());
}

#[test]
fn params_validate_passes_for_valid_quote_and_unmodelled_metadata() {
    let code = AppCode::new("my-app").expect("valid app code");
    let mut metadata = Map::new();
    metadata.insert("quote".to_owned(), json!({ "slippageBips": 50 }));
    // An attribution-only family the SDK does not model passes through.
    metadata.insert(
        "referrer".to_owned(),
        json!({ "address": "0x0000000000000000000000000000000000000001" }),
    );
    let params = AppDataParams::new(code).with_metadata(metadata);

    assert!(params.validate().is_ok());
}

#[test]
fn params_validate_ignores_legacy_string_slippage_shape() {
    // Earlier quote metadata schema versions carried `slippageBips` as a
    // string. That shape no longer parses into the current typed quote, so it
    // is passed through rather than rejected.
    let code = AppCode::new("my-app").expect("valid app code");
    let params = AppDataParams::new(code)
        .with_metadata(metadata_with("quote", json!({ "slippageBips": "5" })));

    assert!(params.validate().is_ok());
}
