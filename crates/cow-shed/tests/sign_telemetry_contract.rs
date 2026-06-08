#![cfg(feature = "tracing")]
//! Telemetry contract for the COW Shed signing entry point.
//!
//! `CowShedHooks::sign` is the crate's one signer-mediated async method. It
//! emits a single `sign` span carrying the chain, version, and a stable
//! `endpoint` label, and records no signer, signature, owner, nonce, or
//! payload material.

use alloy_primitives::{Address, B256, Bytes, U256};
use cow_sdk_core::SupportedChainId;
use cow_sdk_cow_shed::{Call, CowShedHooks, CowShedVersion};
use cow_sdk_test_utils::mocks::RecordingSigner;
use cow_sdk_test_utils::trace::TraceCapture;

#[tokio::test(flavor = "current_thread")]
async fn sign_emits_one_span_with_chain_version_and_endpoint() {
    let capture = TraceCapture::install();
    let signer = RecordingSigner::new();
    let hooks = CowShedHooks::new(SupportedChainId::Mainnet).with_version(CowShedVersion::V1_0_1);
    let calls = [Call::new(Address::ZERO, U256::ZERO, Bytes::new())];

    hooks
        .sign(&signer, &calls, B256::ZERO, U256::MAX)
        .await
        .expect("the recording signer yields a parseable recoverable signature");

    let spans = capture.spans();
    let sign_spans: Vec<_> = spans.iter().filter(|span| span.name() == "sign").collect();
    assert_eq!(
        sign_spans.len(),
        1,
        "sign must emit exactly one span: {spans:#?}"
    );

    let span = sign_spans[0];
    assert_eq!(span.field("endpoint"), Some("cow_shed.sign"));
    assert_eq!(span.field("version"), Some("V1_0_1"));
    assert!(
        span.field("chain").is_some(),
        "sign span must record the chain: {span:#?}"
    );
    // No signer, signature, owner, nonce, deadline, or payload material is
    // recorded; the owner is resolved inside the call and never surfaced.
    for forbidden in [
        "signer",
        "signature",
        "owner",
        "nonce",
        "deadline",
        "payload",
    ] {
        assert!(
            span.field(forbidden).is_none(),
            "sign span must not record {forbidden}: {span:#?}"
        );
    }
}
