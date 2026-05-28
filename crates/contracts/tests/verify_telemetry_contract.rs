#![cfg(feature = "tracing")]
#![allow(
    clippy::missing_const_for_fn,
    reason = "test helpers capture runtime subscriber state"
)]

mod common;

use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use common::MockProvider;
use cow_sdk_contracts::{
    ContractsError, Eip1271VerificationCache, Eip1271VerificationRequest,
    verify_eip1271_signature_cached,
};
use cow_sdk_core::{Address, Hash32, HexData};
use tracing::{
    Event, Id, Level, Metadata, Subscriber,
    field::{Field, Visit},
    span::{Attributes, Record},
    subscriber::Interest,
};

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

fn address(value: &str) -> Address {
    Address::new(value).expect("address fixture must be valid")
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

struct TraceCapture {
    state: Arc<CaptureState>,
    _guard: tracing::dispatcher::DefaultGuard,
}

impl TraceCapture {
    fn install() -> Self {
        let state = Arc::new(CaptureState::default());
        let subscriber = CapturingSubscriber {
            state: state.clone(),
            next_id: AtomicU64::new(1),
        };
        let dispatch = tracing::Dispatch::new(subscriber);
        let guard = tracing::dispatcher::set_default(&dispatch);
        Self {
            state,
            _guard: guard,
        }
    }

    fn spans(&self) -> Vec<CapturedSpan> {
        self.state
            .spans
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .cloned()
            .collect()
    }

    fn events(&self) -> Vec<CapturedEvent> {
        self.state
            .events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

#[derive(Default)]
struct CaptureState {
    spans: Mutex<BTreeMap<u64, CapturedSpan>>,
    events: Mutex<Vec<CapturedEvent>>,
}

struct CapturingSubscriber {
    state: Arc<CaptureState>,
    next_id: AtomicU64,
}

impl Subscriber for CapturingSubscriber {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    fn register_callsite(&self, _metadata: &'static Metadata<'static>) -> Interest {
        Interest::always()
    }

    fn new_span(&self, attributes: &Attributes<'_>) -> Id {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
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
                    target: attributes.metadata().target(),
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

    fn event(&self, event: &Event<'_>) {
        let mut fields = FieldMap::default();
        event.record(&mut fields);
        self.state
            .events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(CapturedEvent {
                level: *event.metadata().level(),
                target: event.metadata().target(),
                fields: fields.0,
            });
    }

    fn enter(&self, _span: &Id) {}

    fn exit(&self, _span: &Id) {}
}

#[derive(Clone, Debug)]
struct CapturedSpan {
    name: String,
    target: &'static str,
    fields: BTreeMap<String, String>,
}

impl CapturedSpan {
    fn name(&self) -> &str {
        &self.name
    }

    fn target(&self) -> &str {
        self.target
    }

    fn field(&self, name: &str) -> Option<&str> {
        self.fields.get(name).map(String::as_str)
    }
}

#[derive(Clone, Debug)]
struct CapturedEvent {
    level: Level,
    target: &'static str,
    fields: BTreeMap<String, String>,
}

impl CapturedEvent {
    fn level(&self) -> Level {
        self.level
    }

    fn target(&self) -> &str {
        self.target
    }

    fn field(&self, name: &str) -> Option<&str> {
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
