#![cfg(all(target_arch = "wasm32", feature = "tracing"))]
//! Telemetry contract for the wasm export surface.
//!
//! Every instrumented export emits one span carrying a stable
//! `wasm.<area>.<method>` endpoint label. This contract exercises a
//! representative IPFS read export; the full endpoint scheme is documented in
//! `docs/observability.md`.

mod common;

use cow_sdk_js::exports::IpfsClient;
use cow_sdk_test_utils::trace::TraceCapture;
use js_sys::Function;
use wasm_bindgen_test::*;

use crate::common::{APP_DATA_CONTENT, CID_APP_DATA, ipfs_config};

wasm_bindgen_test_configure!(run_in_browser);

fn app_data_fetch_callback() -> Function {
    Function::new_with_args(
        "request",
        &format!(
            "return {{ status: 200, headers: {{}}, body: '{}' }};",
            APP_DATA_CONTENT.replace('\'', "\\'")
        ),
    )
}

#[wasm_bindgen_test]
async fn ipfs_fetch_export_emits_one_endpoint_labelled_span() {
    let capture = TraceCapture::install();

    let client = IpfsClient::new(ipfs_config(
        Some("https://ipfs.example.test/ipfs"),
        Some(500),
        &app_data_fetch_callback(),
    ))
    .expect("ipfs client config is valid");
    client
        .fetch_app_data_from_cid(CID_APP_DATA.to_owned(), None)
        .await
        .expect("the stub gateway returns valid app-data json");

    let spans = capture.spans();
    let matching = spans
        .iter()
        .filter(|span| span.field("endpoint") == Some("wasm.ipfs.fetch_app_data_from_cid"))
        .count();
    assert_eq!(
        matching, 1,
        "the wasm ipfs fetch export emits exactly one endpoint-labelled span"
    );
}
