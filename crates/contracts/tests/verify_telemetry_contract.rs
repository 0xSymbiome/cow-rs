#![cfg(feature = "tracing")]

mod common;

use std::sync::Mutex;

use common::MockProvider;
use cow_sdk_contracts::{
    ContractsError, Eip1271VerificationCache, Eip1271VerificationRequest,
    verify_eip1271_signature_cached,
};
use cow_sdk_core::{Address, Hash32, HexData};
use cow_sdk_test_utils::builders::address;
use cow_sdk_test_utils::trace::{CapturedEvent, CapturedSpan, TraceCapture};
use tracing::Level;

#[tokio::test(flavor = "current_thread")]
async fn verifier_emits_canonical_span_and_safe_miss_store_events() {
    let capture = TraceCapture::install();
    let provider = MockProvider::new();
    let verifier = address("0x9008D19f58AAbD9eD0D60971565AA8510560ab41");
    provider.set_code(Some("0x6001600055"));
    provider.set_response("\"0x1626ba7e\"");

    verify_eip1271_signature_cached(
        &provider,
        &verification_request(verifier, "11"),
        &TestCache::default(),
    )
    .await
    .expect("valid magic value should verify");

    let spans = capture.spans();
    let span = spans
        .iter()
        .find(|span| span.name() == "verify.eip1271")
        .unwrap_or_else(|| panic!("verify.eip1271 span must be emitted: {spans:#?}"));
    assert_eq!(span.target(), "cow_sdk::verify_eip1271");
    let verifier_hex = verifier.to_hex_string();
    assert_eq!(span.field("verifier"), Some(verifier_hex.as_str()));
    assert_no_forbidden_fields(span);

    let events = capture.events();
    assert_cache_event(&events, "miss", None);
    assert_cache_event(&events, "store", Some("valid"));
    assert_events_have_no_forbidden_fields(&events);
}

#[tokio::test(flavor = "current_thread")]
async fn verifier_emits_hit_event_without_reaching_provider() {
    let capture = TraceCapture::install();
    let provider = MockProvider::new();
    let cache = TestCache::with_valid();

    verify_eip1271_signature_cached(
        &provider,
        &verification_request(address("0x1111111111111111111111111111111111111111"), "22"),
        &cache,
    )
    .await
    .expect("cached valid probe should verify without provider I/O");

    assert!(
        provider.calls.borrow().is_empty(),
        "cache hits must not perform a provider call"
    );

    let events = capture.events();
    assert_cache_event(&events, "hit", Some("valid"));
    assert_events_have_no_forbidden_fields(&events);
}

#[tokio::test(flavor = "current_thread")]
async fn verifier_emits_skip_event_for_non_cacheable_errors() {
    let capture = TraceCapture::install();
    let provider = MockProvider::new();
    let cache = TestCache::default();
    provider.set_code(Some("0x6001600055"));
    provider.set_response("{\"unexpected\":true}");

    let error = verify_eip1271_signature_cached(
        &provider,
        &verification_request(address("0x2222222222222222222222222222222222222222"), "33"),
        &cache,
    )
    .await
    .expect_err("malformed verifier response should fail closed");

    assert!(matches!(
        error,
        ContractsError::MalformedEip1271Response { .. }
    ));
    assert_eq!(
        cache.records(),
        0,
        "malformed responses must not be recorded"
    );

    let events = capture.events();
    assert_cache_event(&events, "miss", None);
    assert_cache_event(&events, "skip", Some("error"));
    assert_events_have_no_forbidden_fields(&events);
}

fn verification_request(verifier: Address, digest_byte: &str) -> Eip1271VerificationRequest {
    Eip1271VerificationRequest::new(
        verifier,
        Hash32::new(format!("0x{}", digest_byte.repeat(32))).expect("digest fixture must be valid"),
        HexData::new("0x1234").expect("signature fixture must be valid hex"),
    )
}

fn assert_cache_event(
    events: &[CapturedEvent],
    cache_status: &str,
    verification_result: Option<&str>,
) {
    assert!(
        events.iter().any(|event| {
            event.level() == Level::DEBUG
                && event.target() == "cow_sdk::verify_eip1271"
                && event.field("cache_status") == Some(cache_status)
                && event.field("verification_result") == verification_result
        }),
        "missing cache event status={cache_status:?} result={verification_result:?}: {events:#?}"
    );
}

fn assert_no_forbidden_fields(span: &CapturedSpan) {
    for field in [
        "signature",
        "digest",
        "chain_id",
        "provider_url",
        "response_body",
        "raw_digest",
    ] {
        assert_eq!(
            span.field(field),
            None,
            "contracts span must not record forbidden field {field}"
        );
    }
}

fn assert_events_have_no_forbidden_fields(events: &[CapturedEvent]) {
    for event in events {
        for field in [
            "signature",
            "digest",
            "provider_url",
            "response_body",
            "raw_digest",
        ] {
            assert_eq!(
                event.field(field),
                None,
                "contracts telemetry event must not record forbidden field {field}: {event:#?}"
            );
        }
    }
}

#[derive(Default)]
struct TestCache {
    contains_valid: Mutex<bool>,
    records: Mutex<usize>,
}

impl TestCache {
    fn with_valid() -> Self {
        Self {
            contains_valid: Mutex::new(true),
            records: Mutex::default(),
        }
    }

    fn records(&self) -> usize {
        *self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl Eip1271VerificationCache for TestCache {
    fn contains_valid(
        &self,
        _verifier: Address,
        _digest: [u8; 32],
        _signature_hash: [u8; 32],
    ) -> bool {
        *self
            .contains_valid
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn record_valid(&self, _verifier: Address, _digest: [u8; 32], _signature_hash: [u8; 32]) {
        *self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) += 1;
    }
}
