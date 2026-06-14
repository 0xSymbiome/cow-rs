#![cfg(feature = "transport-policy")]

//! Behavior tests for the transport-error classification surface.
//!
//! `NetworkErrorKind::from_transport_error_class` is a pure `match` over the
//! `TransportErrorClass` shape from `cow-sdk-core`. The test below pins the
//! mapping at every variant — including the variants the wildcard arm covers
//! (Redirect and Upgrade), which become `NetworkErrorKind::Other`.

use cow_sdk_core::TransportErrorClass;
use cow_sdk_core::transport::policy::NetworkErrorKind;

#[test]
fn network_error_kind_mapping_round_trip_is_total() {
    let cases = [
        (TransportErrorClass::Timeout, NetworkErrorKind::Timeout),
        (TransportErrorClass::Connect, NetworkErrorKind::Connect),
        (TransportErrorClass::Decode, NetworkErrorKind::Decode),
        (TransportErrorClass::Body, NetworkErrorKind::Decode),
        (TransportErrorClass::Status, NetworkErrorKind::HttpStatus(0)),
        (TransportErrorClass::Request, NetworkErrorKind::Request),
        (TransportErrorClass::Builder, NetworkErrorKind::Builder),
        (
            TransportErrorClass::ResponseTooLarge,
            NetworkErrorKind::ResponseTooLarge,
        ),
        // Wildcard `_` arm: every other class becomes `Other`.
        (TransportErrorClass::Redirect, NetworkErrorKind::Other),
        (TransportErrorClass::Upgrade, NetworkErrorKind::Other),
    ];

    for (class, expected_kind) in cases {
        let mapped = NetworkErrorKind::from_transport_error_class(class);
        assert_eq!(
            mapped, expected_kind,
            "TransportErrorClass::{class:?} must map to {expected_kind:?}",
        );
    }
}

#[test]
fn response_too_large_is_never_retried() {
    // Retrying an over-cap response is futile and would re-download up to the
    // limit on every attempt, so the deterministic ResponseTooLarge outcome
    // must be classified non-retryable.
    let policy = cow_sdk_core::transport::policy::RetryPolicy::default();
    assert!(!policy.should_retry_network(NetworkErrorKind::ResponseTooLarge));
}
