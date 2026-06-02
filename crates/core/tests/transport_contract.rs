#![cfg(not(target_arch = "wasm32"))]

use std::time::Duration;

use cow_sdk_core::config::{DEFAULT_TCP_KEEPALIVE, DEFAULT_USER_AGENT};
use cow_sdk_core::transport::{classify_reqwest_error, sanitize_public_base_url};
use cow_sdk_core::{
    HttpTransport, ReqwestTransport, ReqwestTransportConfig, TransportError, TransportErrorClass,
};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const NO_HEADERS: &[(String, String)] = &[];

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

#[test]
fn reqwest_transport_config_defaults_match_services_aligned_policy() {
    let config = ReqwestTransportConfig::new("https://transport.example");

    assert_eq!(config.user_agent(), DEFAULT_USER_AGENT);
    assert_eq!(config.tcp_keepalive(), DEFAULT_TCP_KEEPALIVE);
}

#[test]
fn sanitize_public_base_url_strips_private_url_parts() {
    let sanitized =
        sanitize_public_base_url("https://api.example.com/path/to/resource?api_key=secret#frag");

    assert_eq!(sanitized, "https://api.example.com");
    assert!(!sanitized.contains("path"));
    assert!(!sanitized.contains("api_key"));
    assert!(!sanitized.contains("secret"));
    assert!(!sanitized.contains("frag"));
}

#[test]
fn sanitize_public_base_url_strips_userinfo_credentials() {
    let sanitized = sanitize_public_base_url("https://user:pass@api.example.com/orders");

    assert_eq!(sanitized, "https://api.example.com");
    assert!(!sanitized.contains("user"));
    assert!(!sanitized.contains("pass"));
}

#[test]
fn sanitize_public_base_url_strips_userinfo_without_password() {
    let sanitized = sanitize_public_base_url("https://user@api.example.com/orders");

    assert_eq!(sanitized, "https://api.example.com");
    assert!(!sanitized.contains("user@"));
}

#[tokio::test]
async fn default_config_sends_services_aligned_user_agent() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/headers"))
        .and(wiremock::matchers::header("user-agent", DEFAULT_USER_AGENT))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;

    let transport = ReqwestTransport::new(ReqwestTransportConfig::new(server.uri()))
        .expect("default reqwest transport must build with the SDK user-agent");
    let body = transport
        .get("/headers", NO_HEADERS, None)
        .await
        .expect("default user-agent must be sent on requests");

    assert_eq!(body, "ok");
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
        .get("/orders", NO_HEADERS, None)
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
        .post("/quote", "{\"kind\":\"sell\"}", NO_HEADERS, None)
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
        .delete("/orders", "{\"uid\":\"0x1\"}", NO_HEADERS, None)
        .await
        .expect("delete round-trip must succeed against the mock server");

    assert_eq!(body, "deleted");
}

#[tokio::test]
async fn status_error_maps_to_http_status_variant_without_exposing_url() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/boom"))
        .respond_with(
            ResponseTemplate::new(500)
                .insert_header("Retry-After", "5")
                .set_body_string("upstream exploded"),
        )
        .mount(&server)
        .await;

    let transport = build_transport(server.uri());
    let error = transport
        .get("/boom", NO_HEADERS, None)
        .await
        .expect_err("non-2xx response must surface as a TransportError");

    match &error {
        TransportError::HttpStatus {
            status,
            headers,
            body,
        } => {
            assert_eq!(*status, 500);
            assert!(headers.iter().any(|(name, value)| {
                name.eq_ignore_ascii_case("retry-after") && value.as_inner() == "5"
            }));
            assert_eq!(body.as_inner(), "upstream exploded");
        }
        other => panic!("expected HttpStatus variant, got {other:?}"),
    }
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
        .get("/slow", NO_HEADERS, None)
        .await
        .expect_err("slow response must exceed the configured timeout");

    assert_eq!(error.class(), Some(TransportErrorClass::Timeout));
}

#[tokio::test]
async fn per_call_timeout_overrides_constructor_default() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/slow"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(5)))
        .mount(&server)
        .await;

    let transport = build_transport(server.uri());
    let error = transport
        .get("/slow", NO_HEADERS, Some(Duration::from_millis(100)))
        .await
        .expect_err("the per-call timeout must override the default");

    assert_eq!(error.class(), Some(TransportErrorClass::Timeout));
}

#[tokio::test]
async fn per_call_headers_reach_the_remote_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/headers"))
        .and(wiremock::matchers::header("x-api-key", "partner-value"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;

    let transport = build_transport(server.uri());
    let headers = [("x-api-key".to_owned(), "partner-value".to_owned())];
    let body = transport
        .get("/headers", &headers, None)
        .await
        .expect("per-call headers must be forwarded to the endpoint");

    assert_eq!(body, "ok");
}

#[tokio::test]
async fn connect_failure_maps_to_connect_class() {
    // Port 1 on localhost is effectively guaranteed to be closed in the test
    // environment, so the connect attempt fails and the adapter classifies
    // the failure through `is_connect`.
    let transport = build_transport("http://127.0.0.1:1".to_owned());
    let error = transport
        .get("/anything", NO_HEADERS, None)
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
        .get("/loop", NO_HEADERS, None)
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
        message: "bad input".to_owned().into(),
    };
    assert!(error.class().is_none());
}

#[test]
fn http_status_error_surfaces_without_class_but_preserves_status_and_body() {
    let error = TransportError::HttpStatus {
        status: 418,
        headers: vec![("Retry-After".to_owned(), "5".to_owned().into())],
        body: "I am a teapot".to_owned().into(),
    };
    assert!(error.class().is_none());
    match error {
        TransportError::HttpStatus {
            status,
            headers,
            body,
        } => {
            assert_eq!(status, 418);
            assert_eq!(headers[0].0, "Retry-After");
            assert_eq!(headers[0].1.as_inner(), "5");
            assert_eq!(body.as_inner(), "I am a teapot");
        }
        _ => panic!("constructed variant must survive the round-trip"),
    }
}

#[tokio::test]
async fn reqwest_transport_is_dyn_compatible_behind_arc() {
    use std::sync::Arc;

    let transport: Arc<dyn HttpTransport> = Arc::new(
        ReqwestTransport::new(
            ReqwestTransportConfig::new("http://127.0.0.1:1").with_user_agent("cow-rs-tests"),
        )
        .expect("reqwest transport must build"),
    );
    // Exercise the dyn dispatch path: the call must fail since port 1 is
    // closed, but the trait-object invocation itself must compile.
    let error = transport
        .get("/path", NO_HEADERS, None)
        .await
        .expect_err("connect to a closed port must fail through the trait-object dispatch");
    assert!(error.class().is_some());
}

#[tokio::test]
async fn connect_failure_through_invalid_url_classifies_as_builder() {
    // An invalid URL bypasses the resolver and forces `reqwest` to surface
    // a builder-layer error at request construction time. The host is
    // syntactically malformed (the bracketed token is not a valid IPv6
    // literal), so no real network traffic is attempted at any layer.
    let client = reqwest::Client::new();
    let builder_error = client
        .request(reqwest::Method::GET, "https://[invalid ipv6]/")
        .build()
        .expect_err("malformed URL must produce a builder-layer reqwest error");

    let transport_error = classify_reqwest_error(builder_error);
    assert_eq!(transport_error.class(), Some(TransportErrorClass::Builder));
}

#[test]
fn transport_error_class_labels_cover_every_documented_variant() {
    assert_eq!(TransportErrorClass::Timeout.as_str(), "timeout");
    assert_eq!(TransportErrorClass::Connect.as_str(), "connect");
    assert_eq!(TransportErrorClass::Redirect.as_str(), "redirect");
    assert_eq!(TransportErrorClass::Decode.as_str(), "decode");
    assert_eq!(TransportErrorClass::Body.as_str(), "body");
    assert_eq!(TransportErrorClass::Builder.as_str(), "builder");
    assert_eq!(TransportErrorClass::Request.as_str(), "request");
    assert_eq!(TransportErrorClass::Status.as_str(), "status");
    assert_eq!(TransportErrorClass::Upgrade.as_str(), "upgrade");
    assert_eq!(
        TransportErrorClass::ResponseTooLarge.as_str(),
        "response_too_large"
    );
    assert_eq!(TransportErrorClass::Other.as_str(), "other");
}

fn build_transport_with_cap(base_url: String, max_response_bytes: usize) -> ReqwestTransport {
    ReqwestTransport::new(
        ReqwestTransportConfig::new(base_url)
            .with_user_agent("cow-rs-tests")
            .with_max_response_bytes(max_response_bytes),
    )
    .expect("reqwest client construction must succeed with a validated user agent")
}

#[tokio::test]
async fn response_within_cap_is_returned_intact() {
    let server = MockServer::start().await;
    let body = "x".repeat(1024);
    Mock::given(method("GET"))
        .and(path("/ok"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body.clone()))
        .mount(&server)
        .await;

    let transport = build_transport_with_cap(server.uri(), 4096);
    let received = transport
        .get("/ok", NO_HEADERS, None)
        .await
        .expect("a body within the cap must be returned");
    assert_eq!(received, body);
}

#[tokio::test]
async fn response_exceeding_cap_is_rejected_as_response_too_large() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/big"))
        .respond_with(ResponseTemplate::new(200).set_body_string("x".repeat(8192)))
        .mount(&server)
        .await;

    let transport = build_transport_with_cap(server.uri(), 4096);
    let error = transport
        .get("/big", NO_HEADERS, None)
        .await
        .expect_err("a body over the cap must be rejected");
    assert_eq!(error.class(), Some(TransportErrorClass::ResponseTooLarge));
}

#[tokio::test]
async fn response_exactly_at_cap_is_accepted_and_one_over_is_rejected() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/exact"))
        .respond_with(ResponseTemplate::new(200).set_body_string("y".repeat(2048)))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/over"))
        .respond_with(ResponseTemplate::new(200).set_body_string("y".repeat(2049)))
        .mount(&server)
        .await;

    let transport = build_transport_with_cap(server.uri(), 2048);
    let at_cap = transport
        .get("/exact", NO_HEADERS, None)
        .await
        .expect("a body exactly at the cap must be accepted");
    assert_eq!(at_cap.len(), 2048);

    let over = transport
        .get("/over", NO_HEADERS, None)
        .await
        .expect_err("a body one byte over the cap must be rejected");
    assert_eq!(over.class(), Some(TransportErrorClass::ResponseTooLarge));
}

#[tokio::test]
async fn oversized_error_status_body_is_rejected_as_response_too_large() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/err"))
        .respond_with(ResponseTemplate::new(500).set_body_string("e".repeat(8192)))
        .mount(&server)
        .await;

    let transport = build_transport_with_cap(server.uri(), 4096);
    let error = transport
        .get("/err", NO_HEADERS, None)
        .await
        .expect_err("an oversized error body must be rejected before it is buffered");
    // An oversized error body is refused rather than surfaced as an HttpStatus
    // that carries the full body through the typed error channel.
    assert_eq!(error.class(), Some(TransportErrorClass::ResponseTooLarge));
}

#[tokio::test]
async fn non_utf8_body_is_decoded_lossily_without_a_cap_layer_error() {
    let server = MockServer::start().await;
    // 0xFF is not valid UTF-8; the lossy decode replaces it rather than
    // erroring, matching the prior text-based read behavior.
    Mock::given(method("GET"))
        .and(path("/binary"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![0xff, 0xfe, 0xfd]))
        .mount(&server)
        .await;

    let transport = build_transport_with_cap(server.uri(), 4096);
    let received = transport
        .get("/binary", NO_HEADERS, None)
        .await
        .expect("a non-UTF-8 body must decode lossily, not error");
    assert!(received.contains('\u{FFFD}'));
}

#[tokio::test]
async fn gzip_bomb_is_rejected_on_decompressed_size() {
    use std::io::Write as _;

    use flate2::{Compression, write::GzEncoder};

    // A small compressed body that decompresses far past the cap. reqwest
    // decompresses before yielding chunks, so the cap observes the
    // decompressed size and rejects the bomb rather than the compressed size.
    let decompressed = vec![0u8; 1024 * 1024];
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(&decompressed)
        .expect("gzip encoding must succeed");
    let compressed = encoder.finish().expect("gzip finalize must succeed");
    assert!(
        compressed.len() < 64 * 1024,
        "the compressed bomb must be far smaller than its decompressed size"
    );

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/bomb"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-encoding", "gzip")
                .set_body_bytes(compressed),
        )
        .mount(&server)
        .await;

    let transport = build_transport_with_cap(server.uri(), 64 * 1024);
    let error = transport
        .get("/bomb", NO_HEADERS, None)
        .await
        .expect_err("a decompression bomb must be rejected on its decompressed size");
    assert_eq!(error.class(), Some(TransportErrorClass::ResponseTooLarge));
}

#[cfg(feature = "tracing")]
mod tracing_contract {
    use super::*;

    use cow_sdk_test_utils::trace::TraceCapture;

    #[tokio::test(flavor = "current_thread")]
    async fn reqwest_dispatch_emits_one_path_only_transport_span_with_body_sizes() {
        let capture = TraceCapture::install_global();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/quote"))
            .respond_with(ResponseTemplate::new(200).set_body_string("quoted"))
            .mount(&server)
            .await;

        let transport = ReqwestTransport::new(ReqwestTransportConfig::new(String::new()))
            .expect("default reqwest transport must build");
        let body = "{\"kind\":\"sell\"}";
        let server_uri = server.uri();
        let server_authority = server_uri.trim_start_matches("http://");
        let url_with_userinfo = format!("{server_uri}/quote?api_key=secret").replacen(
            "http://",
            "http://user:pass@",
            1,
        );
        let response = transport
            .post(&url_with_userinfo, body, NO_HEADERS, None)
            .await
            .expect("mocked POST must succeed");

        assert_eq!(response, "quoted");

        let spans = capture.spans();
        let expected_bytes_sent = body.len().to_string();
        let expected_bytes_received = "quoted".len().to_string();
        let transport_spans: Vec<_> = spans
            .iter()
            .filter(|span| {
                span.name() == "transport.dispatch"
                    && span.field("method") == Some("POST")
                    && span.field("endpoint") == Some("/quote")
                    && span.field("bytes_sent") == Some(expected_bytes_sent.as_str())
                    && span.field("bytes_received") == Some(expected_bytes_received.as_str())
            })
            .collect();
        assert_eq!(
            transport_spans.len(),
            1,
            "one matching transport span must be emitted for this dispatch: {spans:#?}"
        );
        let span = transport_spans[0];
        assert_eq!(span.field("method"), Some("POST"));
        assert_eq!(span.field("endpoint"), Some("/quote"));
        assert_eq!(span.field("bytes_sent"), Some(expected_bytes_sent.as_str()));
        assert_eq!(
            span.field("bytes_received"),
            Some(expected_bytes_received.as_str())
        );

        let endpoint = span
            .field("endpoint")
            .expect("endpoint field must be present");
        assert!(!endpoint.contains("user"));
        assert!(!endpoint.contains("pass"));
        assert!(!endpoint.contains("api_key"));
        assert!(!endpoint.contains(server_authority));
    }
}
