//! Parity contract between [`cow_sdk_core::ReqwestTransport`] and
//! `cow_sdk_transport_wasm::FetchTransport`.
//!
//! Shared fixtures under `crates/core/tests/fixtures/transport/` carry the
//! canonical response bytes both adapters are expected to return end-to-end.
//! The native half of the suite drives [`cow_sdk_core::ReqwestTransport`]
//! against a [`wiremock`] server that replays each fixture and asserts the
//! transport returns the fixture body byte-for-byte. The wasm32 half drives
//! `FetchTransport` against an injected JavaScript mock that replays the
//! same fixture and makes the same assertion. The two halves together
//! establish that callers observe identical [`String`] outputs no matter
//! which adapter handles the request.
//!
//! The wasm half is only exercised when the suite is compiled with
//! `wasm32-unknown-unknown` and run through a `wasm-bindgen-test` harness.
//! The standard `cargo test` workflow on native targets builds this file
//! with only the native half activated.

const GET_ORDERS_FIXTURE: &str =
    include_str!("../../core/tests/fixtures/transport/get_orders_ok.json");
const POST_QUOTE_FIXTURE: &str =
    include_str!("../../core/tests/fixtures/transport/post_quote_ok.json");
const DELETE_ORDER_FIXTURE: &str =
    include_str!("../../core/tests/fixtures/transport/delete_order_ok.txt");

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use cow_sdk_core::{HttpTransport, ReqwestTransport, ReqwestTransportConfig};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::{DELETE_ORDER_FIXTURE, GET_ORDERS_FIXTURE, POST_QUOTE_FIXTURE};

    fn build_transport(base_url: String) -> ReqwestTransport {
        ReqwestTransport::new(
            ReqwestTransportConfig::new(base_url).with_user_agent("cow-rs-parity-tests"),
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
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use cow_sdk_core::HttpTransport;
    use cow_sdk_transport_wasm::{FetchTransport, FetchTransportConfig};
    use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    use super::{DELETE_ORDER_FIXTURE, GET_ORDERS_FIXTURE, POST_QUOTE_FIXTURE};

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen(inline_js = r"
export function install_fetch_mock(body) {
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

export function restore_fetch(previous) {
  if (previous !== undefined && previous !== null) {
    globalThis.fetch = previous;
  }
}
")]
    extern "C" {
        fn install_fetch_mock(body: &str) -> JsValue;
        fn restore_fetch(previous: JsValue);
    }

    fn transport() -> FetchTransport {
        FetchTransport::new(&FetchTransportConfig::new("https://parity.example"))
    }

    #[wasm_bindgen_test]
    async fn fetch_transport_get_returns_fixture_bytes() {
        let previous = install_fetch_mock(GET_ORDERS_FIXTURE);
        let body = transport()
            .get("/orders")
            .await
            .expect("fetch transport must deliver the mocked fixture body");
        restore_fetch(previous);
        assert_eq!(body, GET_ORDERS_FIXTURE);
    }

    #[wasm_bindgen_test]
    async fn fetch_transport_post_returns_fixture_bytes() {
        let previous = install_fetch_mock(POST_QUOTE_FIXTURE);
        let body = transport()
            .post("/quote", "{\"kind\":\"sell\"}")
            .await
            .expect("fetch transport must deliver the mocked fixture body");
        restore_fetch(previous);
        assert_eq!(body, POST_QUOTE_FIXTURE);
    }

    #[wasm_bindgen_test]
    async fn fetch_transport_delete_returns_fixture_bytes() {
        let previous = install_fetch_mock(DELETE_ORDER_FIXTURE);
        let body = transport()
            .delete("/orders/0x1", "{\"uid\":\"0x1\"}")
            .await
            .expect("fetch transport must deliver the mocked fixture body");
        restore_fetch(previous);
        assert_eq!(body, DELETE_ORDER_FIXTURE);
    }
}
