//! Contract tests for typed app-data metadata validation.
//!
//! These cover the typed quote-slippage bound enforced at
//! [`QuoteMetadata`] construction. The canonical pre-upload validation pass for
//! a full app-data document is exercised through `into_validated` /
//! `app_data_info` in the schema and validated-shape contract suites.

use cow_sdk_app_data::{AppDataError, QuoteMetadata};

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
