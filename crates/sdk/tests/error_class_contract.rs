//! Contract for `SdkError::class()` rate-limit classification.
//!
//! A throttled orderbook response is retried with `Retry-After` honoring by
//! the transport layer, so a 429 that survives to `SdkError` is an
//! exhausted-retry throttle. `class()` reports it as `ErrorClass::RateLimited`
//! instead of bucketing it with generic remote responses, on both the
//! recognised-rejection (`Rejected`) and unparsed-envelope (`Api`) paths.

use cow_sdk::{
    ErrorClass, SdkError,
    orderbook::{OrderbookApiError, OrderbookError, ResponseBody},
};
use serde_json::json;

/// Builds the promoted `Rejected` error for a recognised rejection envelope at
/// the given HTTP status (the `From` path derives the `StatusCode` from the
/// `u16`).
fn rejected(status: u16, error_type: &str) -> OrderbookError {
    OrderbookApiError::new(
        status,
        "diagnostic",
        ResponseBody::Json(json!({ "errorType": error_type, "description": "x" })),
    )
    .into()
}

/// Builds the untyped `Api` envelope error for a body with no recognisable
/// rejection envelope at the given HTTP status.
fn api(status: u16) -> OrderbookError {
    OrderbookError::Api(Box::new(OrderbookApiError::new(
        status,
        "diagnostic",
        ResponseBody::Text("opaque body".to_owned()),
    )))
}

#[test]
fn exhausted_retry_429_classifies_as_rate_limited() {
    // A 429 whose body carries no recognisable rejection envelope lands in `Api`.
    assert_eq!(
        SdkError::Orderbook(api(429)).class(),
        ErrorClass::RateLimited,
    );

    // A 429 with a recognised rejection envelope lands in `Rejected`.
    assert_eq!(
        SdkError::Orderbook(rejected(429, "Forbidden")).class(),
        ErrorClass::RateLimited,
    );
}

#[test]
fn non_429_remote_responses_stay_remote() {
    assert_eq!(SdkError::Orderbook(api(500)).class(), ErrorClass::Remote);
    assert_eq!(
        SdkError::Orderbook(rejected(400, "ZeroAmount")).class(),
        ErrorClass::Remote,
    );
}
