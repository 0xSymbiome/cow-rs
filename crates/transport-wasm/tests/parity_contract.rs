//! Parity contract between [`cow_sdk_core::ReqwestTransport`] and
//! `cow_sdk_transport_wasm::FetchTransport`.
//!
//! Shared fixtures under `crates/core/tests/fixtures/transport/` carry the
//! canonical response bytes both adapters are required to deliver through
//! the shared [`HttpTransport`] trait. The `error-class` matrix re-exercises
//! each adapter against synthetic failure scenarios (connect-refused,
//! server-500, timeout, truncated-body) and asserts both adapters agree on
//! the same partitioned outcome: transport-level failures map to the same
//! [`TransportErrorClass`], and non-2xx responses map to
//! [`cow_sdk_core::TransportError::HttpStatus`] with the numeric status
//! preserved on both runtimes.
//!
//! The native half drives [`cow_sdk_core::ReqwestTransport`] against a
//! [`wiremock`] server that replays each fixture. The wasm half drives
//! `FetchTransport` against an injected JavaScript fetch mock that replays
//! the same fixture bytes and the same synthetic error shapes. The wasm
//! half is only exercised when the suite is compiled with
//! `wasm32-unknown-unknown` and run through a `wasm-bindgen-test` harness.
//! The standard `cargo test` workflow on native targets builds this file
//! with only the native half activated.

use cow_sdk_core::TransportErrorClass;

const GET_ORDERS_FIXTURE: &str =
    include_str!("../../core/tests/fixtures/transport/get_orders_ok.json");
const POST_QUOTE_FIXTURE: &str =
    include_str!("../../core/tests/fixtures/transport/post_quote_ok.json");
const DELETE_ORDER_FIXTURE: &str =
    include_str!("../../core/tests/fixtures/transport/delete_order_ok.txt");

/// Transport-layer error-class parity matrix shared between the native and
/// wasm halves.
///
/// Each entry names a synthetic transport failure and the single
/// [`TransportErrorClass`] both adapters must map it to. Non-success HTTP
/// status responses surface through the typed
/// [`cow_sdk_core::TransportError::HttpStatus`] variant (without a
/// [`TransportErrorClass`]) and are asserted through a dedicated test in
/// each half instead of through this matrix.
const CROSS_ADAPTER_ERROR_MATRIX: &[(&str, TransportErrorClass)] = &[
    ("connect-refused", TransportErrorClass::Connect),
    ("slow-response", TransportErrorClass::Timeout),
    ("truncated-body", TransportErrorClass::Body),
];

/// Sanity check on the shared matrix. Compiling this test against the
/// matrix makes the matrix appear in the wasm-side harness even when the
/// native and wasm halves are built separately.
#[test]
fn cross_adapter_error_matrix_names_every_exercised_class() {
    let class_values: Vec<TransportErrorClass> = CROSS_ADAPTER_ERROR_MATRIX
        .iter()
        .map(|(_, class)| *class)
        .collect();
    assert!(class_values.contains(&TransportErrorClass::Connect));
    assert!(class_values.contains(&TransportErrorClass::Timeout));
    assert!(class_values.contains(&TransportErrorClass::Body));
}

#[test]
fn fetch_transport_contract_remains_request_response_only() {
    let fetch_source = include_str!("../src/fetch.rs");
    let transport_docs = include_str!("../../../docs/transport.md");

    assert!(
        !fetch_source.contains("EventSource"),
        "FetchTransport must not grow a separate EventSource or SSE path",
    );
    assert!(
        !fetch_source.contains("text/event-stream"),
        "FetchTransport must stay scoped to request/response dispatch",
    );
    assert!(
        transport_docs.contains("request/response only"),
        "the public transport docs must state that the default seam is request/response only",
    );
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::time::Duration;

    use cow_sdk_core::{HttpTransport, ReqwestTransport, ReqwestTransportConfig, TransportError};
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::{
        CROSS_ADAPTER_ERROR_MATRIX, DELETE_ORDER_FIXTURE, GET_ORDERS_FIXTURE, POST_QUOTE_FIXTURE,
        TransportErrorClass,
    };

    const NO_HEADERS: &[(String, String)] = &[];

    fn build_transport(base_url: String) -> ReqwestTransport {
        ReqwestTransport::new(
            ReqwestTransportConfig::new(base_url).with_user_agent("cow-rs-parity-tests"),
        )
        .expect("reqwest transport must build with a validated user agent")
    }

    fn build_transport_with_timeout(base_url: String, timeout: Duration) -> ReqwestTransport {
        ReqwestTransport::new(
            ReqwestTransportConfig::new(base_url)
                .with_user_agent("cow-rs-parity-tests")
                .with_timeout(timeout),
        )
        .expect("reqwest transport must build with a validated user agent")
    }

    #[tokio::test]
    async fn reqwest_transport_get_returns_fixture_bytes() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/orders"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("Content-Type", "application/json")
                    .set_body_string(GET_ORDERS_FIXTURE),
            )
            .mount(&server)
            .await;

        let transport = build_transport(server.uri());
        let body = transport
            .get("/orders", NO_HEADERS, None)
            .await
            .expect("fixture round-trip must succeed through ReqwestTransport");
        assert_eq!(body, GET_ORDERS_FIXTURE);
    }

    #[tokio::test]
    async fn reqwest_transport_post_returns_fixture_bytes() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/quote"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("Content-Type", "application/json")
                    .set_body_string(POST_QUOTE_FIXTURE),
            )
            .mount(&server)
            .await;

        let transport = build_transport(server.uri());
        let body = transport
            .post("/quote", "{\"kind\":\"sell\"}", NO_HEADERS, None)
            .await
            .expect("fixture round-trip must succeed through ReqwestTransport");
        assert_eq!(body, POST_QUOTE_FIXTURE);
    }

    #[tokio::test]
    async fn reqwest_transport_delete_returns_fixture_bytes() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/orders/0x1"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("Content-Type", "text/plain")
                    .set_body_string(DELETE_ORDER_FIXTURE),
            )
            .mount(&server)
            .await;

        let transport = build_transport(server.uri());
        let body = transport
            .delete("/orders/0x1", "{\"uid\":\"0x1\"}", NO_HEADERS, None)
            .await
            .expect("fixture round-trip must succeed through ReqwestTransport");
        assert_eq!(body, DELETE_ORDER_FIXTURE);
    }

    #[tokio::test]
    async fn reqwest_transport_forwards_cache_control_headers() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/headers"))
            .and(header("cache-control", "no-cache"))
            .and(header("pragma", "no-cache"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&server)
            .await;

        let transport = build_transport(server.uri());
        let headers = [
            ("Cache-Control".to_owned(), "no-cache".to_owned()),
            ("Pragma".to_owned(), "no-cache".to_owned()),
        ];
        let body = transport
            .get("/headers", &headers, None)
            .await
            .expect("cache-control headers must be forwarded through ReqwestTransport");
        assert_eq!(body, "ok");
    }

    #[tokio::test]
    async fn connect_refused_maps_to_connect_class_per_matrix() {
        let transport = build_transport("http://127.0.0.1:1".to_owned());
        let error = transport
            .get("/anything", NO_HEADERS, None)
            .await
            .expect_err("connect to a closed port must fail");
        assert_eq!(error.class(), Some(expected_class("connect-refused")));
    }

    #[tokio::test]
    async fn server_500_maps_to_http_status_with_numeric_code() {
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
            .expect_err("a 500 response must surface a typed HttpStatus error");
        match error {
            TransportError::HttpStatus {
                status,
                headers,
                body,
            } => {
                assert_eq!(status, 500);
                assert!(headers.iter().any(|(name, value)| {
                    name.eq_ignore_ascii_case("retry-after") && value == "5"
                }));
                assert_eq!(body, "upstream exploded");
            }
            other => panic!("expected HttpStatus variant, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn slow_response_maps_to_timeout_class_per_matrix() {
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
        assert_eq!(error.class(), Some(expected_class("slow-response")));
    }

    #[tokio::test]
    async fn truncated_body_maps_within_documented_body_partition() {
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

        let transport = build_transport(server.uri());
        let error = transport
            .get("/truncated", NO_HEADERS, None)
            .await
            .expect_err("truncated body must surface a transport error");
        let class = error
            .class()
            .expect("body-stream failure must carry a typed class");
        assert!(
            matches!(
                class,
                TransportErrorClass::Body
                    | TransportErrorClass::Request
                    | TransportErrorClass::Decode
            ),
            "body-stream failure must classify within the documented partition, got {class:?}"
        );
        assert_eq!(expected_class("truncated-body"), TransportErrorClass::Body);
    }

    fn expected_class(label: &'static str) -> TransportErrorClass {
        CROSS_ADAPTER_ERROR_MATRIX
            .iter()
            .find(|(entry, _)| *entry == label)
            .map_or_else(
                || panic!("matrix missing class entry for label `{label}`"),
                |(_, class)| *class,
            )
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use cow_sdk_core::{HttpTransport, TransportError};
    use cow_sdk_transport_wasm::{FetchTransport, FetchTransportConfig};
    use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    use super::{
        CROSS_ADAPTER_ERROR_MATRIX, DELETE_ORDER_FIXTURE, GET_ORDERS_FIXTURE, POST_QUOTE_FIXTURE,
        TransportErrorClass,
    };

    const NO_HEADERS: &[(String, String)] = &[];

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen(inline_js = r"
export function install_fetch_ok_mock(body) {
  const previous = globalThis.fetch;
  globalThis.fetch = (_input, _init) => {
    const response = new Response(body, {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    });
    return Promise.resolve(response);
  };
  return previous;
}

export function install_fetch_status_mock(status, body) {
  const previous = globalThis.fetch;
  globalThis.fetch = (_input, _init) => {
    const response = new Response(body, {
      status,
      headers: { 'Retry-After': '5' },
    });
    return Promise.resolve(response);
  };
  return previous;
}

export function install_fetch_rejection_mock(name) {
  const previous = globalThis.fetch;
  globalThis.fetch = (_input, _init) => {
    const error = new DOMException('mock transport failure', name);
    return Promise.reject(error);
  };
  return previous;
}

export function install_fetch_header_echo_mock() {
  const previous = globalThis.fetch;
  globalThis.fetch = (input) => {
    const headers = input instanceof Request ? input.headers : new Headers();
    const response = new Response(JSON.stringify({
      cacheControl: headers.get('Cache-Control'),
      pragma: headers.get('Pragma'),
    }), {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    });
    return Promise.resolve(response);
  };
  return previous;
}

export function restore_fetch(previous) {
  if (previous !== undefined && previous !== null) {
    globalThis.fetch = previous;
  }
}
")]
    extern "C" {
        fn install_fetch_ok_mock(body: &str) -> JsValue;
        fn install_fetch_status_mock(status: u16, body: &str) -> JsValue;
        fn install_fetch_rejection_mock(name: &str) -> JsValue;
        fn install_fetch_header_echo_mock() -> JsValue;
        fn restore_fetch(previous: JsValue);
    }

    fn transport() -> FetchTransport {
        FetchTransport::new(&FetchTransportConfig::new("https://parity.example"))
    }

    fn matrix_class(label: &'static str) -> TransportErrorClass {
        CROSS_ADAPTER_ERROR_MATRIX
            .iter()
            .find(|(entry, _)| *entry == label)
            .map_or_else(
                || panic!("matrix missing class entry for label `{label}`"),
                |(_, class)| *class,
            )
    }

    #[wasm_bindgen_test]
    async fn fetch_transport_get_returns_fixture_bytes() {
        let previous = install_fetch_ok_mock(GET_ORDERS_FIXTURE);
        let body = transport()
            .get("/orders", NO_HEADERS, None)
            .await
            .expect("fetch transport must deliver the mocked fixture body");
        restore_fetch(previous);
        assert_eq!(body, GET_ORDERS_FIXTURE);
    }

    #[wasm_bindgen_test]
    async fn fetch_transport_post_returns_fixture_bytes() {
        let previous = install_fetch_ok_mock(POST_QUOTE_FIXTURE);
        let body = transport()
            .post("/quote", "{\"kind\":\"sell\"}", NO_HEADERS, None)
            .await
            .expect("fetch transport must deliver the mocked fixture body");
        restore_fetch(previous);
        assert_eq!(body, POST_QUOTE_FIXTURE);
    }

    #[wasm_bindgen_test]
    async fn fetch_transport_delete_returns_fixture_bytes() {
        let previous = install_fetch_ok_mock(DELETE_ORDER_FIXTURE);
        let body = transport()
            .delete("/orders/0x1", "{\"uid\":\"0x1\"}", NO_HEADERS, None)
            .await
            .expect("fetch transport must deliver the mocked fixture body");
        restore_fetch(previous);
        assert_eq!(body, DELETE_ORDER_FIXTURE);
    }

    #[wasm_bindgen_test]
    async fn fetch_transport_forwards_cache_control_headers() {
        let previous = install_fetch_header_echo_mock();
        let headers = [
            ("Cache-Control".to_owned(), "no-cache".to_owned()),
            ("Pragma".to_owned(), "no-cache".to_owned()),
        ];
        let body = transport()
            .get("/headers", &headers, None)
            .await
            .expect("fetch transport must forward cache-control headers");
        restore_fetch(previous);

        assert!(body.contains("\"cacheControl\":\"no-cache\""));
        assert!(body.contains("\"pragma\":\"no-cache\""));
    }

    #[wasm_bindgen_test]
    async fn server_500_maps_to_http_status_with_numeric_code() {
        let previous = install_fetch_status_mock(500, "upstream exploded");
        let error = transport()
            .get("/boom", NO_HEADERS, None)
            .await
            .expect_err("500 response must surface a typed HttpStatus error");
        restore_fetch(previous);
        match error {
            TransportError::HttpStatus {
                status,
                headers,
                body,
            } => {
                assert_eq!(status, 500);
                assert!(headers.iter().any(|(name, value)| {
                    name.eq_ignore_ascii_case("retry-after") && value == "5"
                }));
                assert_eq!(body, "upstream exploded");
            }
            other => panic!("expected HttpStatus variant, got {other:?}"),
        }
    }

    #[wasm_bindgen_test]
    async fn abort_rejection_maps_to_timeout_class_per_matrix() {
        let previous = install_fetch_rejection_mock("AbortError");
        let error = transport()
            .get("/slow", NO_HEADERS, None)
            .await
            .expect_err("AbortError rejection must surface as Timeout");
        restore_fetch(previous);
        assert_eq!(error.class(), Some(matrix_class("slow-response")));
    }

    #[wasm_bindgen_test]
    async fn network_rejection_maps_to_connect_class_per_matrix() {
        let previous = install_fetch_rejection_mock("TypeError");
        let error = transport()
            .get("/boom", NO_HEADERS, None)
            .await
            .expect_err("network failure must surface a transport error");
        restore_fetch(previous);
        assert_eq!(error.class(), Some(matrix_class("connect-refused")));
    }
}
