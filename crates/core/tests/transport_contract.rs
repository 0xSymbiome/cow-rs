#![cfg(not(target_arch = "wasm32"))]

use std::time::Duration;

use cow_sdk_core::transport::classify_reqwest_error;
use cow_sdk_core::{
    HttpTransport, ReqwestTransport, ReqwestTransportConfig, TransportError, TransportErrorClass,
};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn build_transport(base_url: String) -> ReqwestTransport {
    ReqwestTransport::new(ReqwestTransportConfig::new(base_url).with_user_agent("cow-rs-tests"))
        .expect("reqwest client construction must succeed with a validated user agent")
}

fn build_transport_with_timeout(base_url: String, timeout: Duration) -> ReqwestTransport {
    ReqwestTransport::new(
        ReqwestTransportConfig::new(base_url)
            .with_user_agent("cow-rs-tests")
            .with_timeout(timeout),
    )
    .expect("reqwest client construction must succeed with a validated user agent")
}

#[tokio::test]
async fn get_round_trip_returns_response_body() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/orders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "ok": true })))
        .mount(&server)
        .await;

    let transport = build_transport(server.uri());
    let body = transport
        .get("/orders")
        .await
        .expect("get round-trip must succeed against the mock server");

    assert_eq!(body, "{\"ok\":true}");
}

#[tokio::test]
async fn post_round_trip_forwards_body_and_returns_response_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/quote"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "quoteId": 42 })))
        .mount(&server)
        .await;

    let transport = build_transport(server.uri());
    let body = transport
        .post("/quote", "{\"kind\":\"sell\"}")
        .await
        .expect("post round-trip must succeed against the mock server");

    assert_eq!(body, "{\"quoteId\":42}");
}

#[tokio::test]
async fn delete_round_trip_forwards_body_and_returns_response_body() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/orders"))
        .respond_with(ResponseTemplate::new(200).set_body_string("deleted"))
        .mount(&server)
        .await;

    let transport = build_transport(server.uri());
    let body = transport
        .delete("/orders", "{\"uid\":\"0x1\"}")
        .await
        .expect("delete round-trip must succeed against the mock server");

    assert_eq!(body, "deleted");
}

#[tokio::test]
async fn status_error_maps_to_status_class_without_exposing_url() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/boom"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let transport = build_transport(server.uri());
    let error = transport
        .get("/boom")
        .await
        .expect_err("non-2xx response must surface as a TransportError");

    let Some(class) = error.class() else {
        panic!("status failure must carry a transport-error class: {error:?}");
    };
    assert_eq!(class, TransportErrorClass::Status);
    let rendered = format!("{error}");
    assert!(
        !rendered.contains(server.uri().trim_start_matches("http://")),
        "rendered error must not include the server URL: {rendered}"
    );
}

#[tokio::test]
async fn timeout_maps_to_timeout_class() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/slow"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(5)))
        .mount(&server)
        .await;

    let transport = build_transport_with_timeout(server.uri(), Duration::from_millis(100));
    let error = transport
        .get("/slow")
        .await
        .expect_err("slow response must exceed the configured timeout");

    assert_eq!(error.class(), Some(TransportErrorClass::Timeout));
}

#[tokio::test]
async fn connect_failure_maps_to_connect_class() {
    // Port 1 on localhost is effectively guaranteed to be closed in the test
    // environment, so the connect attempt fails and the adapter classifies
    // the failure through `is_connect`.
    let transport = build_transport("http://127.0.0.1:1".to_owned());
    let error = transport
        .get("/anything")
        .await
        .expect_err("connect to a closed port must fail");

    assert_eq!(error.class(), Some(TransportErrorClass::Connect));
}

#[tokio::test]
async fn redirect_loop_maps_to_redirect_class() {
    let server = MockServer::start().await;
    let loop_url = format!("{}/loop", server.uri());
    Mock::given(method("GET"))
        .and(path("/loop"))
        .respond_with(ResponseTemplate::new(302).insert_header("Location", loop_url.as_str()))
        .mount(&server)
        .await;

    let transport = build_transport(server.uri());
    let error = transport
        .get("/loop")
        .await
        .expect_err("self-redirecting endpoint must exhaust the redirect policy");

    assert_eq!(error.class(), Some(TransportErrorClass::Redirect));
}

#[tokio::test]
async fn decode_failure_classifies_through_exposed_helper() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/malformed"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
        .mount(&server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/malformed", server.uri()))
        .send()
        .await
        .expect("mock server must return a response body");
    let decode_error = response
        .json::<serde_json::Value>()
        .await
        .expect_err("parsing invalid JSON must produce a decode-class reqwest error");

    let transport_error = classify_reqwest_error(decode_error);
    assert_eq!(transport_error.class(), Some(TransportErrorClass::Decode));
    let rendered = format!("{transport_error}");
    assert!(
        !rendered.contains(server.uri().trim_start_matches("http://")),
        "decode error must strip the attached URL: {rendered}"
    );
}

#[tokio::test]
async fn body_stream_failure_classifies_through_exposed_helper() {
    // A server that closes the connection mid-response produces a reqwest
    // error that exercises the body-stream classification arm. Depending on
    // the hyper version, the failure may surface at `.send()` (Request class)
    // or at `.text()` (Body class). Both arms are documented members of the
    // partition and pass through `without_url` in the public helper.
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/truncated"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Length", "1024")
                .set_body_string("short"),
        )
        .mount(&server)
        .await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("client must build for the body-stream scenario");
    let url = format!("{}/truncated", server.uri());
    let transport_error = match client.get(&url).send().await {
        Ok(response) => {
            let body_error = response
                .text()
                .await
                .expect_err("truncated response body must surface a reqwest error");
            classify_reqwest_error(body_error)
        }
        Err(error) => classify_reqwest_error(error),
    };

    let class = transport_error
        .class()
        .expect("body-stream failure must carry a typed class");
    assert!(
        matches!(
            class,
            TransportErrorClass::Body | TransportErrorClass::Request | TransportErrorClass::Decode
        ),
        "body-stream failure must classify within the documented category set, got {class:?}"
    );
    let rendered = format!("{transport_error}");
    assert!(
        !rendered.contains(server.uri().trim_start_matches("http://")),
        "body-stream error must strip the attached URL: {rendered}"
    );
}

#[test]
fn configuration_error_surfaces_without_class() {
    let error = TransportError::Configuration {
        message: "bad input".to_owned(),
    };
    assert!(error.class().is_none());
}
