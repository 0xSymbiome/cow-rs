#![cfg(feature = "tracing")]
//! Telemetry contract for the IPFS app-data read path.
//!
//! Every `fetch_doc_*` entry point funnels through the shared
//! `fetch_doc_from_cid_with_policy` leaf, so each fetch path emits exactly one
//! span. The span records the requested `cid` and a stable `endpoint` label
//! only; the configured gateway read base URI is never recorded.

mod common;

use async_trait::async_trait;
use cow_sdk_app_data::{
    AppDataError, IpfsFetchTransport, fetch_doc_from_app_data_hex, fetch_doc_from_cid,
};
use cow_sdk_test_utils::trace::TraceCapture;

use crate::common::{APP_DATA_HEX, APP_DATA_STRING, CID};

struct StubTransport;

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl IpfsFetchTransport for StubTransport {
    async fn get(&self, _uri: &str) -> Result<String, AppDataError> {
        Ok(APP_DATA_STRING.to_owned())
    }
}

#[tokio::test(flavor = "current_thread")]
async fn fetch_doc_from_cid_emits_one_span_with_cid_and_endpoint() {
    let capture = TraceCapture::install();

    fetch_doc_from_cid(CID, &StubTransport, None)
        .await
        .expect("stub transport returns valid app-data json");

    let spans = capture.spans();
    let fetch_spans: Vec<_> = spans
        .iter()
        .filter(|span| span.field("endpoint") == Some("app_data.fetch_doc_from_cid"))
        .collect();
    assert_eq!(
        fetch_spans.len(),
        1,
        "a fetch emits exactly one fetch span: {spans:#?}"
    );

    let span = fetch_spans[0];
    assert_eq!(span.field("cid"), Some(CID));
    // The gateway read base URI is never recorded.
    for forbidden in [
        "uri",
        "read_base_uri",
        "read_uri",
        "url",
        "transport",
        "policy",
    ] {
        assert!(
            span.field(forbidden).is_none(),
            "fetch span must not record {forbidden}: {span:#?}"
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn fetch_doc_from_app_data_hex_funnels_through_one_shared_leaf_span() {
    let capture = TraceCapture::install();

    fetch_doc_from_app_data_hex(APP_DATA_HEX, &StubTransport, None)
        .await
        .expect("stub transport returns valid app-data json");

    let spans = capture.spans();
    let fetch_span_count = spans
        .iter()
        .filter(|span| span.field("endpoint") == Some("app_data.fetch_doc_from_cid"))
        .count();
    assert_eq!(
        fetch_span_count, 1,
        "the hex path delegates through one fetch span, not nested duplicates: {spans:#?}"
    );
}
