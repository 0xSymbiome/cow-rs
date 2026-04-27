use cow_sdk_transport_wasm::{FetchTransport, FetchTransportConfig};
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn fetch_transport_config_debug_redacts_base_url() {
    let config = FetchTransportConfig::new("https://fetch.example/quote?api_key=secret");
    let rendered = format!("{config:?}");

    assert!(rendered.contains("[redacted]"));
    assert!(!rendered.contains("fetch.example"));
    assert!(!rendered.contains("api_key"));

    let transport = FetchTransport::new(&config);
    assert_eq!(
        transport.base_url(),
        "https://fetch.example/quote?api_key=secret"
    );
}
