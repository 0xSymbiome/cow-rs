#![cfg(feature = "tracing")]

mod capture {
    use std::{
        collections::BTreeMap,
        sync::{Arc, Mutex},
    };

    use tracing::{
        Event, Id, Metadata, Subscriber,
        field::{Field, Visit},
        span::{Attributes, Record},
        subscriber::Interest,
    };

    pub struct TraceCapture {
        state: Arc<CaptureState>,
    }

    impl TraceCapture {
        pub fn install() -> Self {
            let state = Arc::new(CaptureState::default());
            let subscriber = CapturingSubscriber {
                state: state.clone(),
            };
            let dispatch = tracing::Dispatch::new(subscriber);
            tracing::dispatcher::set_global_default(dispatch)
                .expect("transport tracing contract installs one subscriber per test binary");
            Self { state }
        }

        pub fn spans(&self) -> Vec<CapturedSpan> {
            self.state
                .spans
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .values()
                .cloned()
                .collect()
        }
    }

    struct CaptureState {
        next_id: Mutex<u64>,
        spans: Mutex<BTreeMap<u64, CapturedSpan>>,
    }

    impl Default for CaptureState {
        fn default() -> Self {
            Self {
                next_id: Mutex::new(1),
                spans: Mutex::default(),
            }
        }
    }

    struct CapturingSubscriber {
        state: Arc<CaptureState>,
    }

    impl Subscriber for CapturingSubscriber {
        fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
            true
        }

        fn register_callsite(&self, _metadata: &'static Metadata<'static>) -> Interest {
            Interest::always()
        }

        fn new_span(&self, attributes: &Attributes<'_>) -> Id {
            let id = next_span_id(&self.state);
            let mut fields = FieldMap::default();
            attributes.record(&mut fields);
            self.state
                .spans
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(
                    id,
                    CapturedSpan {
                        name: attributes.metadata().name().to_owned(),
                        fields: fields.0,
                    },
                );
            Id::from_u64(id)
        }

        fn record(&self, span: &Id, values: &Record<'_>) {
            let mut fields = FieldMap::default();
            values.record(&mut fields);
            let mut spans = self
                .state
                .spans
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(span) = spans.get_mut(&span.clone().into_u64()) {
                span.fields.extend(fields.0);
            }
        }

        fn record_follows_from(&self, _span: &Id, _follows: &Id) {}

        fn event(&self, _event: &Event<'_>) {}

        fn enter(&self, _span: &Id) {}

        fn exit(&self, _span: &Id) {}
    }

    fn next_span_id(state: &CaptureState) -> u64 {
        let mut next_id = state
            .next_id
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let id = *next_id;
        *next_id += 1;
        id
    }

    #[derive(Clone, Debug)]
    pub struct CapturedSpan {
        name: String,
        fields: BTreeMap<String, String>,
    }

    impl CapturedSpan {
        pub fn name(&self) -> &str {
            &self.name
        }

        #[cfg(target_arch = "wasm32")]
        pub fn field_names(&self) -> Vec<&str> {
            self.fields.keys().map(String::as_str).collect()
        }

        pub fn field(&self, name: &str) -> Option<&str> {
            self.fields.get(name).map(String::as_str)
        }
    }

    #[derive(Default)]
    struct FieldMap(BTreeMap<String, String>);

    impl FieldMap {
        fn record_value(&mut self, field: &Field, value: String) {
            self.0.insert(field.name().to_owned(), value);
        }
    }

    impl Visit for FieldMap {
        fn record_i64(&mut self, field: &Field, value: i64) {
            self.record_value(field, value.to_string());
        }

        fn record_u64(&mut self, field: &Field, value: u64) {
            self.record_value(field, value.to_string());
        }

        fn record_bool(&mut self, field: &Field, value: bool) {
            self.record_value(field, value.to_string());
        }

        fn record_str(&mut self, field: &Field, value: &str) {
            self.record_value(field, value.to_owned());
        }

        fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
            self.record_value(field, format!("{value:?}"));
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use cow_sdk_core::{HttpTransport, ReqwestTransport, ReqwestTransportConfig};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::capture::TraceCapture;

    const NO_HEADERS: &[(String, String)] = &[];

    #[tokio::test(flavor = "current_thread")]
    async fn reqwest_dispatch_span_matches_transport_shape() {
        let capture = TraceCapture::install();
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

        assert_eq!(response, "quoted");
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
    use cow_sdk_core::HttpTransport;
    use cow_sdk_transport_wasm::{FetchTransport, FetchTransportConfig};
    use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    use super::capture::TraceCapture;

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

export function restore_fetch(previous) {
  if (previous !== undefined && previous !== null) {
    globalThis.fetch = previous;
  }
}
")]
    extern "C" {
        fn install_fetch_ok_mock(body: &str) -> JsValue;
        fn restore_fetch(previous: JsValue);
    }

    #[wasm_bindgen_test]
    async fn fetch_dispatch_span_matches_transport_shape() {
        let capture = TraceCapture::install();
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

        assert_eq!(response, "quoted");
        assert_transport_span(&capture, "POST", Some("wasm32"), body.len(), "quoted".len());
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
