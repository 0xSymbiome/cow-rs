#![cfg(feature = "tracing")]

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use cow_sdk_core::{HttpTransport, ReqwestTransport, ReqwestTransportConfig};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use cow_sdk_test_utils::trace::TraceCapture;

    const NO_HEADERS: &[(String, String)] = &[];

    #[tokio::test(flavor = "current_thread")]
    async fn reqwest_dispatch_span_matches_transport_shape() {
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
        let response = transport
            .post(
                &format!("{server_uri}/quote?api_key=secret"),
                body,
                NO_HEADERS,
                None,
            )
            .await
            .expect("mocked reqwest POST must succeed");

        assert_eq!(response.body(), "quoted");
        assert_transport_span(
            &capture,
            "POST",
            None,
            body.len(),
            "quoted".len(),
            server_authority,
        );
    }

    fn assert_transport_span(
        capture: &TraceCapture,
        method: &str,
        chain: Option<&str>,
        bytes_sent: usize,
        bytes_received: usize,
        forbidden_endpoint_fragment: &str,
    ) {
        let spans = capture.spans();
        let transport_spans: Vec<_> = spans
            .iter()
            .filter(|span| span.name() == "transport.dispatch")
            .collect();
        assert_eq!(
            transport_spans.len(),
            1,
            "one transport span must be emitted per dispatch: {spans:#?}"
        );
        let span = transport_spans[0];
        let expected_bytes_sent = bytes_sent.to_string();
        let expected_bytes_received = bytes_received.to_string();
        assert_eq!(span.field("method"), Some(method));
        assert_eq!(span.field("endpoint"), Some("/quote"));
        assert_eq!(span.field("chain"), chain);
        assert_eq!(span.field("bytes_sent"), Some(expected_bytes_sent.as_str()));
        assert_eq!(
            span.field("bytes_received"),
            Some(expected_bytes_received.as_str())
        );

        let endpoint = span
            .field("endpoint")
            .expect("endpoint field must be present");
        assert!(!endpoint.contains(forbidden_endpoint_fragment));
        assert!(!endpoint.contains("api_key"));
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::time::Duration;

    use cow_sdk_core::{FetchTransport, FetchTransportConfig};
    use cow_sdk_core::{HttpTransport, TransportError, TransportErrorClass};
    use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    use cow_sdk_test_utils::trace::TraceCapture;

    const NO_HEADERS: &[(String, String)] = &[];

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen(inline_js = r"
export function install_fetch_ok_mock(body) {
  const previous = globalThis.fetch;
  globalThis.fetch = (_input, _init) => Promise.resolve(new Response(body, {
    status: 200,
    headers: { 'Content-Type': 'text/plain' },
  }));
  return previous;
}

export function install_fetch_stalled_body_mock() {
  const previous = globalThis.fetch;
  globalThis.fetch = (input, init) => {
    const signal = init && init.signal ? init.signal :
      input instanceof Request ? input.signal : undefined;
    const response = new Response('', {
      status: 200,
      headers: { 'Content-Type': 'text/plain' },
    });
    Object.defineProperty(response, 'text', {
      value: () => new Promise((_resolve, reject) => {
        const abortBody = () => reject(new DOMException('aborted', 'AbortError'));
        if (signal) {
          if (signal.aborted) {
            abortBody();
          } else {
            signal.addEventListener('abort', abortBody, { once: true });
          }
        }
      }),
    });
    return Promise.resolve(response);
  };
  return previous;
}

export function install_fetch_status_mock(status) {
  const previous = globalThis.fetch;
  globalThis.fetch = (_input, _init) => Promise.resolve(new Response('error', {
    status,
    headers: { 'Content-Type': 'text/plain' },
  }));
  return previous;
}

export function install_fetch_reject_mock() {
  const previous = globalThis.fetch;
  globalThis.fetch = (_input, _init) =>
    Promise.reject(new DOMException('network down', 'NetworkError'));
  return previous;
}

export function install_timer_counter() {
  const original_set = globalThis.setTimeout;
  const original_clear = globalThis.clearTimeout;
  const counter = { active: 0, peak: 0 };
  globalThis.__cow_timer_counter = counter;
  globalThis.setTimeout = (cb, ms, ...args) => {
    const id = original_set(() => {
      counter.active = Math.max(0, counter.active - 1);
      cb(...args);
    }, ms);
    counter.active += 1;
    counter.peak = Math.max(counter.peak, counter.active);
    return id;
  };
  globalThis.clearTimeout = (id) => {
    counter.active = Math.max(0, counter.active - 1);
    return original_clear(id);
  };
  return { original_set, original_clear };
}

export function restore_timer_counter(saved) {
  globalThis.setTimeout = saved.original_set;
  globalThis.clearTimeout = saved.original_clear;
  delete globalThis.__cow_timer_counter;
}

export function active_timer_count() {
  const counter = globalThis.__cow_timer_counter;
  return counter ? counter.active : 0;
}

export function restore_fetch(previous) {
  if (previous !== undefined && previous !== null) {
    globalThis.fetch = previous;
  }
}
")]
    extern "C" {
        fn install_fetch_ok_mock(body: &str) -> JsValue;
        fn install_fetch_stalled_body_mock() -> JsValue;
        fn install_fetch_status_mock(status: u16) -> JsValue;
        fn install_fetch_reject_mock() -> JsValue;
        fn install_timer_counter() -> JsValue;
        fn restore_timer_counter(saved: JsValue);
        fn active_timer_count() -> u32;
        fn restore_fetch(previous: JsValue);
    }

    #[wasm_bindgen_test]
    async fn fetch_dispatch_span_matches_transport_shape() {
        let capture = TraceCapture::install_global();
        let previous = install_fetch_ok_mock("quoted");
        let transport = FetchTransport::new(&FetchTransportConfig::new("https://fetch.example"));
        let body = "{\"kind\":\"sell\"}";
        let response = transport
            .post(
                "https://fetch.example/quote?api_key=secret",
                body,
                NO_HEADERS,
                None,
            )
            .await
            .expect("mocked fetch POST must succeed");
        restore_fetch(previous);

        assert_eq!(response.body(), "quoted");
        assert_transport_span(&capture, "POST", Some("wasm32"), body.len(), "quoted".len());
    }

    #[wasm_bindgen_test]
    async fn timeout_covers_response_body_read_when_body_stalls() {
        let previous = install_fetch_stalled_body_mock();
        let transport = FetchTransport::new(&FetchTransportConfig::new("https://fetch.example"));
        let result = transport
            .get(
                "/stalled-body",
                NO_HEADERS,
                Some(Duration::from_millis(100)),
            )
            .await;
        restore_fetch(previous);

        let error = result.expect_err("stalled body must surface as a timeout");
        match error {
            TransportError::Transport { class, .. } => {
                assert_eq!(class, TransportErrorClass::Timeout);
            }
            other => panic!("expected Transport(Timeout), got {other:?}"),
        }
    }

    #[wasm_bindgen_test]
    async fn timeout_clears_timer_on_successful_body_completion() {
        let saved_counter = install_timer_counter();
        let previous_fetch = install_fetch_ok_mock("ok");
        let transport = FetchTransport::new(&FetchTransportConfig::new("https://fetch.example"));

        for _ in 0..100 {
            transport
                .get("/ok", NO_HEADERS, Some(Duration::from_secs(5)))
                .await
                .expect("ok response");
        }

        let active = active_timer_count();
        restore_fetch(previous_fetch);
        restore_timer_counter(saved_counter);
        assert_eq!(active, 0, "no timers may leak after successful requests");
    }

    #[wasm_bindgen_test]
    async fn timeout_clears_timer_on_http_status_error_path() {
        let saved_counter = install_timer_counter();
        let previous_fetch = install_fetch_status_mock(503);
        let transport = FetchTransport::new(&FetchTransportConfig::new("https://fetch.example"));

        let result = transport
            .get("/status", NO_HEADERS, Some(Duration::from_secs(5)))
            .await;
        let active = active_timer_count();
        restore_fetch(previous_fetch);
        restore_timer_counter(saved_counter);

        match result {
            Err(TransportError::HttpStatus { status, .. }) => assert_eq!(status, 503),
            other => panic!("expected HttpStatus(503), got {other:?}"),
        }
        assert_eq!(active, 0, "HTTP-status exit paths must clear the timer");
    }

    #[wasm_bindgen_test]
    async fn timeout_clears_timer_on_network_error_path() {
        let saved_counter = install_timer_counter();
        let previous_fetch = install_fetch_reject_mock();
        let transport = FetchTransport::new(&FetchTransportConfig::new("https://fetch.example"));

        let result = transport
            .get("/network", NO_HEADERS, Some(Duration::from_secs(5)))
            .await;
        let active = active_timer_count();
        restore_fetch(previous_fetch);
        restore_timer_counter(saved_counter);

        match result {
            Err(TransportError::Transport { class, .. }) => {
                assert_eq!(class, TransportErrorClass::Connect);
            }
            other => panic!("expected Transport(Connect), got {other:?}"),
        }
        assert_eq!(active, 0, "network-error exit path must clear the timer");
    }

    #[wasm_bindgen_test]
    async fn timeout_clears_timer_on_abort_error_rejection() {
        let saved_counter = install_timer_counter();
        let previous_fetch = install_fetch_stalled_body_mock();
        let transport = FetchTransport::new(&FetchTransportConfig::new("https://fetch.example"));

        let result = transport
            .get("/abort", NO_HEADERS, Some(Duration::from_millis(1)))
            .await;
        let active = active_timer_count();
        restore_fetch(previous_fetch);
        restore_timer_counter(saved_counter);

        assert!(matches!(
            result,
            Err(TransportError::Transport {
                class: TransportErrorClass::Timeout,
                ..
            })
        ));
        assert_eq!(active, 0, "abort-error exit path must clear the timer");
    }

    fn assert_transport_span(
        capture: &TraceCapture,
        method: &str,
        chain: Option<&str>,
        bytes_sent: usize,
        bytes_received: usize,
    ) {
        let spans = capture.spans();
        let transport_spans: Vec<_> = spans
            .iter()
            .filter(|span| span.name() == "transport.dispatch")
            .collect();
        assert_eq!(
            transport_spans.len(),
            1,
            "one transport span must be emitted per dispatch: {spans:#?}"
        );
        let span = transport_spans[0];
        let expected_bytes_sent = bytes_sent.to_string();
        let expected_bytes_received = bytes_received.to_string();
        assert_eq!(span.field("method"), Some(method));
        assert_eq!(span.field("endpoint"), Some("/quote"));
        assert_eq!(span.field("chain"), chain);
        assert_eq!(span.field("bytes_sent"), Some(expected_bytes_sent.as_str()));
        assert_eq!(
            span.field("bytes_received"),
            Some(expected_bytes_received.as_str())
        );

        let endpoint = span
            .field("endpoint")
            .expect("endpoint field must be present");
        assert!(!endpoint.contains("fetch.example"));
        assert!(!endpoint.contains("api_key"));
        assert_eq!(
            span.field_names(),
            vec![
                "bytes_received",
                "bytes_sent",
                "chain",
                "endpoint",
                "method"
            ],
            "browser transport spans must not grow undocumented fields"
        );
    }
}
