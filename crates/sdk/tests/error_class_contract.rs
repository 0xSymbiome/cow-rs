//! Contract for the coarse-grained [`ErrorClass`] partition.
//!
//! Every public error type exposes a `class()` accessor and the facade
//! [`SdkError::class()`] delegates to them, so the classification is identical
//! whether a caller holds the facade error or a bare leaf error. A throttled
//! orderbook response is retried with `Retry-After` honoring by the transport
//! layer, so a 429 that survives is an exhausted-retry throttle and reports as
//! [`ErrorClass::RateLimited`] on both the recognised-rejection (`Rejected`)
//! and unparsed-envelope (`Api`) paths.

use cow_sdk::{
    ErrorClass, SdkError,
    core::{CoreError, TransportErrorClass, ValidationError},
    orderbook::{OrderbookApiError, OrderbookError, ResponseBody},
    signing::SigningError,
    trading::TradingError,
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

#[test]
fn error_class_partitions_every_bucket() {
    // Validation — caller-side input boundaries.
    assert_eq!(
        CoreError::Validation(ValidationError::EmptyField { field: "appCode" }).class(),
        ErrorClass::Validation
    );
    assert_eq!(TradingError::MissingOwner.class(), ErrorClass::Validation);

    // Transport — failure before a complete response.
    assert_eq!(
        OrderbookError::Transport {
            class: TransportErrorClass::Connect,
            detail: "connect failed".to_owned().into(),
        }
        .class(),
        ErrorClass::Transport
    );

    // Remote vs RateLimited on the leaf accessor directly.
    assert_eq!(api(500).class(), ErrorClass::Remote);
    assert_eq!(api(429).class(), ErrorClass::RateLimited);

    // Signing — surfaced from the signing edge.
    assert_eq!(
        SigningError::Signer {
            operation: "typed-data signature",
            message: "signer failed".to_owned().into(),
        }
        .class(),
        ErrorClass::Signing
    );

    // Cancelled — cooperative cancellation across every error type.
    assert_eq!(CoreError::Cancelled.class(), ErrorClass::Cancelled);
    assert_eq!(OrderbookError::Cancelled.class(), ErrorClass::Cancelled);
    assert_eq!(TradingError::Cancelled.class(), ErrorClass::Cancelled);

    // Internal — invariant or helper-contract violations.
    assert_eq!(
        CoreError::Serialization("decode failed".to_owned().into()).class(),
        ErrorClass::Internal
    );
}

#[test]
fn error_class_delegates_through_trading_and_facade() {
    // The composite TradingError delegates to the wrapped error, so a wrapped
    // 429 stays RateLimited rather than collapsing to a coarse bucket.
    assert_eq!(
        TradingError::Orderbook(api(429)).class(),
        ErrorClass::RateLimited
    );

    // SdkError delegates to each leaf accessor: the class is identical whether
    // a caller holds the facade error or the bare leaf error.
    let transport = OrderbookError::Transport {
        class: TransportErrorClass::Connect,
        detail: "connect failed".to_owned().into(),
    };
    let leaf_class = transport.class();
    assert_eq!(SdkError::from(transport).class(), leaf_class);
    assert_eq!(
        SdkError::from(TradingError::Cancelled).class(),
        ErrorClass::Cancelled
    );
}
