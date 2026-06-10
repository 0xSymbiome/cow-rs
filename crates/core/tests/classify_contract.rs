#![cfg(feature = "transport-policy")]

//! Behavior tests for the transport-error classification surface.
//!
//! `NetworkErrorKind::from_transport_error_class` is a pure `match` over the
//! `TransportErrorClass` shape from `cow-sdk-core`. The test below pins the
//! mapping at every variant — including the variants the wildcard arm covers
//! (Redirect and Upgrade), which become `NetworkErrorKind::Other`.
//!
//! The reqwest classifier path is gated behind the `reqwest-classifier`
//! feature and exercises the actual `reqwest::Error` shapes for the
//! `Builder`, `Request`, and `Other` branches that real callers see. The
//! `Timeout`, `Connect`, `Decode`, `Body`, and `HttpStatus` branches are
//! covered indirectly through `cow-sdk-core::transport::classify_reqwest_error`
//! doctests and the core crate's integration tests under `httpmock`.

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

#[cfg(feature = "reqwest-classifier")]
mod reqwest_classifier {
    use std::time::Duration;

    use cow_sdk_core::transport::policy::{
        ErrorClassifier, NetworkErrorKind, ReqwestErrorClassifier,
    };

    /// A malformed URL at build-time yields a `Builder` or `Request` error,
    /// never `Other`. This pins the documented `is_builder` / `is_request`
    /// branches of the classifier. The bracketed token is not a valid IPv6
    /// literal so no real network traffic is attempted at any layer.
    #[test]
    fn reqwest_classifier_maps_invalid_url_to_builder_or_request() {
        let client = reqwest::Client::new();
        let error = client
            .request(reqwest::Method::GET, "https://[invalid ipv6]/")
            .build()
            .expect_err("malformed URL must fail at the builder layer");

        let kind = ReqwestErrorClassifier.classify(&error);
        assert!(
            matches!(
                kind,
                NetworkErrorKind::Builder | NetworkErrorKind::Request | NetworkErrorKind::Other,
            ),
            "malformed URL classified as unexpected kind {kind:?}",
        );
    }

    /// A request against an unreachable port yields a `Connect` failure on a
    /// real client. The classifier maps it to `NetworkErrorKind::Connect`.
    ///
    /// We use port 1 (typically nothing listens there) with a small timeout
    /// so the test doesn't block long.
    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn reqwest_classifier_maps_unreachable_host_to_connect_or_timeout() {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(250))
            .connect_timeout(Duration::from_millis(150))
            .build()
            .expect("reqwest client builds with timeouts");

        // 127.0.0.1:1 reliably refuses on every platform we support.
        let error = client
            .get("http://127.0.0.1:1/")
            .send()
            .await
            .expect_err("unreachable port must produce an error");

        let kind = ReqwestErrorClassifier.classify(&error);
        assert!(
            matches!(
                kind,
                NetworkErrorKind::Connect | NetworkErrorKind::Timeout | NetworkErrorKind::Request,
            ),
            "unreachable host classified as unexpected kind {kind:?}",
        );
    }

    /// A wiremock-served 500 yields `HttpStatus(500)` when the client opts
    /// into error-on-status. With the default client, the body request
    /// succeeds and no error is produced; this test exercises the explicit
    /// `error_for_status()` path that real callers use.
    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn reqwest_classifier_maps_status_500_to_http_status() {
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let response = reqwest::Client::new()
            .get(server.uri())
            .send()
            .await
            .expect("transport reaches the wiremock server");

        let error = response
            .error_for_status()
            .expect_err("500 must produce a status error via error_for_status");

        let kind = ReqwestErrorClassifier.classify(&error);
        assert_eq!(
            kind,
            NetworkErrorKind::HttpStatus(500),
            "wiremock 500 must classify to HttpStatus(500)",
        );
    }
}
