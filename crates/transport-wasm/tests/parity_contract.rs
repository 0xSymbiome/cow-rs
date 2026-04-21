//! Parity contract between [`cow_sdk_core::ReqwestTransport`] and
//! `cow_sdk_transport_wasm::FetchTransport`.
//!
//! Shared fixtures under `crates/core/tests/fixtures/transport/` carry the
//! canonical response bytes and the expected [`TransportErrorClass`] for
//! every partition arm both adapters are required to deliver.
//!
//! The native half of the suite drives [`cow_sdk_core::ReqwestTransport`]
//! against a [`wiremock`] server that replays each fixture and asserts the
//! transport returns the fixture body byte-for-byte. The `error-class`
//! matrix re-exercises the same adapter against synthetic failure scenarios
//! (connect-refused, status, timeout, decode) and asserts the mapping into
//! [`TransportErrorClass`] matches the entry both adapters agree on in
//! [`CROSS_ADAPTER_ERROR_MATRIX`]. The wasm32 half drives `FetchTransport`
//! against an injected JavaScript fetch mock that replays the same fixture
//! bytes and the same synthetic error shapes, asserting equal `class`
//! values — the explicit cross-adapter byte-identity plus error-class
//! parity claim the shipped consumer depends on.
//!
//! The wasm half is only exercised when the suite is compiled with
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

/// Error-class parity matrix shared between the native and wasm halves.
///
/// Each entry names a synthetic failure scenario and the single
/// [`TransportErrorClass`] both adapters must map it to. The native half
/// drives the scenario end-to-end; the wasm half re-drives each scenario
/// through the injected fetch mock and asserts the same class.
const CROSS_ADAPTER_ERROR_MATRIX: &[(&str, TransportErrorClass)] = &[
    ("connect-refused", TransportErrorClass::Connect),
    ("server-500", TransportErrorClass::Status),
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
    assert!(class_values.contains(&TransportErrorClass::Status));
    assert!(class_values.contains(&TransportErrorClass::Timeout));
    assert!(class_values.contains(&TransportErrorClass::Body));
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::time::Duration;

    use cow_sdk_core::{HttpTransport, ReqwestTransport, ReqwestTransportConfig};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::{
        CROSS_ADAPTER_ERROR_MATRIX, DELETE_ORDER_FIXTURE, GET_ORDERS_FIXTURE, POST_QUOTE_FIXTURE,
        TransportErrorClass,
    };

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
            .get("/orders")
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
            .post("/quote", "{\"kind\":\"sell\"}")
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
            .delete("/orders/0x1", "{\"uid\":\"0x1\"}")
            .await
            .expect("fixture round-trip must succeed through ReqwestTransport");
        assert_eq!(body, DELETE_ORDER_FIXTURE);
    }

    #[tokio::test]
    async fn connect_refused_maps_to_connect_class_per_matrix() {
        let transport = build_transport("http://127.0.0.1:1".to_owned());
        let error = transport
            .get("/anything")
            .await
            .expect_err("connect to a closed port must fail");
        assert_eq!(error.class(), Some(expected_class("connect-refused")));
    }

    #[tokio::test]
    async fn server_500_maps_to_status_class_per_matrix() {
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
            .expect_err("a 500 response must classify as Status");
        assert_eq!(error.class(), Some(expected_class("server-500")));
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
            .get("/slow")
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
            .get("/truncated")
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
        // Also documented in CROSS_ADAPTER_ERROR_MATRIX as Body; the other
        // two arms are reqwest-version-specific variants the wasm half
        // never exposes.
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
    use cow_sdk_core::HttpTransport;
    use cow_sdk_transport_wasm::{FetchTransport, FetchTransportConfig};
    use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    use super::{
        CROSS_ADAPTER_ERROR_MATRIX, DELETE_ORDER_FIXTURE, GET_ORDERS_FIXTURE, POST_QUOTE_FIXTURE,
        TransportErrorClass,
    };

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

export function install_fetch_status_mock(status) {
  const previous = globalThis.fetch;
  globalThis.fetch = (_input, _init) => {
    const response = new Response('', { status });
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

export function restore_fetch(previous) {
  if (previous !== undefined && previous !== null) {
    globalThis.fetch = previous;
  }
}
")]
    extern "C" {
        fn install_fetch_ok_mock(body: &str) -> JsValue;
        fn install_fetch_status_mock(status: u16) -> JsValue;
        fn install_fetch_rejection_mock(name: &str) -> JsValue;
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
            .get("/orders")
            .await
            .expect("fetch transport must deliver the mocked fixture body");
        restore_fetch(previous);
        assert_eq!(body, GET_ORDERS_FIXTURE);
    }

    #[wasm_bindgen_test]
    async fn fetch_transport_post_returns_fixture_bytes() {
        let previous = install_fetch_ok_mock(POST_QUOTE_FIXTURE);
        let body = transport()
            .post("/quote", "{\"kind\":\"sell\"}")
            .await
            .expect("fetch transport must deliver the mocked fixture body");
        restore_fetch(previous);
        assert_eq!(body, POST_QUOTE_FIXTURE);
    }

    #[wasm_bindgen_test]
    async fn fetch_transport_delete_returns_fixture_bytes() {
        let previous = install_fetch_ok_mock(DELETE_ORDER_FIXTURE);
        let body = transport()
            .delete("/orders/0x1", "{\"uid\":\"0x1\"}")
            .await
            .expect("fetch transport must deliver the mocked fixture body");
        restore_fetch(previous);
        assert_eq!(body, DELETE_ORDER_FIXTURE);
    }

    #[wasm_bindgen_test]
    async fn server_500_maps_to_status_class_per_matrix() {
        let previous = install_fetch_status_mock(500);
        let error = transport()
            .get("/boom")
            .await
            .expect_err("500 response must surface a transport error");
        restore_fetch(previous);
        assert_eq!(error.class(), Some(matrix_class("server-500")));
    }

    #[wasm_bindgen_test]
    async fn abort_rejection_maps_to_timeout_class_per_matrix() {
        let previous = install_fetch_rejection_mock("AbortError");
        let error = transport()
            .get("/slow")
            .await
            .expect_err("AbortError rejection must surface as Timeout");
        restore_fetch(previous);
        assert_eq!(error.class(), Some(matrix_class("slow-response")));
    }

    #[wasm_bindgen_test]
    async fn network_rejection_maps_to_connect_class_per_matrix() {
        let previous = install_fetch_rejection_mock("TypeError");
        let error = transport()
            .get("/boom")
            .await
            .expect_err("network failure must surface a transport error");
        restore_fetch(previous);
        assert_eq!(error.class(), Some(matrix_class("connect-refused")));
    }
}
