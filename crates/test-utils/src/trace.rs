//! A scoped tracing subscriber for asserting on emitted spans and events.
//!
//! [`TraceCapture::install`] registers a thread-local subscriber (kept alive by
//! the returned guard) that records every span and event, including names,
//! levels, `&'static str` targets, fields, and the parent-span stack, so tests
//! can assert on structured tracing output without a real collector.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Record};
use tracing::subscriber::Interest;
use tracing::{Event, Id, Level, Metadata, Subscriber};
use tracing_core::span::Current;

/// A scoped tracing subscriber that records spans and events for assertions.
///
/// The capturing subscriber is installed as the current thread's default for as
/// long as the returned value is held; dropping it restores the previous
/// subscriber.
pub struct TraceCapture {
    state: Arc<CaptureState>,
    #[cfg(not(target_arch = "wasm32"))]
    _guard: Option<tracing::dispatcher::DefaultGuard>,
}

impl TraceCapture {
    fn new_state_dispatch() -> (Arc<CaptureState>, tracing::Dispatch) {
        let state = Arc::new(CaptureState::default());
        let subscriber = CapturingSubscriber {
            state: state.clone(),
            next_id: AtomicU64::new(1),
        };
        (state, tracing::Dispatch::new(subscriber))
    }

    /// Installs the capturing subscriber as the current thread's scoped default
    /// and returns a handle that restores the previous subscriber when dropped.
    ///
    /// Use this when the spans and events under test are emitted on the calling
    /// thread; separate instances can coexist across tests in one binary. For
    /// output emitted from another thread (for example deep inside an async
    /// transport), use [`TraceCapture::install_global`]. Scoped thread-local
    /// subscribers require `std`, which is unavailable on `wasm32`, so the
    /// `wasm32` build installs globally instead.
    #[cfg(not(target_arch = "wasm32"))]
    #[must_use]
    pub fn install() -> Self {
        let (state, dispatch) = Self::new_state_dispatch();
        let guard = tracing::dispatcher::set_default(&dispatch);
        Self {
            state,
            _guard: Some(guard),
        }
    }

    /// On `wasm32`, scoped thread-local subscribers are unavailable, so
    /// installation is always global; see [`TraceCapture::install_global`].
    ///
    /// # Panics
    /// Panics if a global default subscriber has already been installed for the
    /// test binary.
    #[cfg(target_arch = "wasm32")]
    #[must_use]
    pub fn install() -> Self {
        Self::install_global()
    }

    /// Installs the capturing subscriber as the global default. Use this when
    /// the spans or events under test are emitted from a thread other than the
    /// installing one (a scoped thread-local subscriber would miss them).
    ///
    /// # Panics
    /// Panics if a global default subscriber has already been installed for the
    /// test binary; the global default can be set only once.
    #[must_use]
    pub fn install_global() -> Self {
        let (state, dispatch) = Self::new_state_dispatch();
        tracing::dispatcher::set_global_default(dispatch)
            .expect("a single global tracing subscriber is installed per test binary");
        Self {
            state,
            #[cfg(not(target_arch = "wasm32"))]
            _guard: None,
        }
    }

    /// Returns every captured event, in emission order.
    #[must_use]
    pub fn events(&self) -> Vec<CapturedEvent> {
        self.state
            .events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Returns every captured span, ordered by creation.
    #[must_use]
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

#[derive(Default)]
struct CaptureState {
    events: Mutex<Vec<CapturedEvent>>,
    spans: Mutex<BTreeMap<u64, CapturedSpan>>,
    span_metadata: Mutex<BTreeMap<u64, &'static Metadata<'static>>>,
    stack: Mutex<Vec<(Id, &'static Metadata<'static>)>>,
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
        self.state
            .span_metadata
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(id, attributes.metadata());
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

    fn enter(&self, span: &Id) {
        let metadata = self
            .state
            .span_metadata
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&span.clone().into_u64())
            .copied();
        let Some(metadata) = metadata else {
            return;
        };
        self.state
            .stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push((span.clone(), metadata));
    }

    fn exit(&self, span: &Id) {
        let mut stack = self
            .state
            .stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if stack.last().map(|(candidate, _)| candidate) == Some(span) {
            stack.pop();
        } else if let Some(index) = stack.iter().rposition(|(candidate, _)| candidate == span) {
            stack.remove(index);
        }
    }

    fn current_span(&self) -> Current {
        self.state
            .stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .last()
            .map_or_else(Current::none, |(id, metadata)| {
                Current::new(id.clone(), metadata)
            })
    }
}

/// A captured tracing event.
#[derive(Clone, Debug)]
pub struct CapturedEvent {
    level: Level,
    target: &'static str,
    fields: BTreeMap<String, String>,
}

impl CapturedEvent {
    /// The event's level.
    #[must_use]
    pub const fn level(&self) -> Level {
        self.level
    }

    /// The event's `&'static str` target.
    #[must_use]
    pub const fn target(&self) -> &str {
        self.target
    }

    /// The recorded value of `name`, if the event carried that field.
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields.get(name).map(String::as_str)
    }
}

/// A captured tracing span.
#[derive(Clone, Debug)]
pub struct CapturedSpan {
    name: String,
    target: &'static str,
    fields: BTreeMap<String, String>,
}

impl CapturedSpan {
    /// The span's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The span's `&'static str` target.
    #[must_use]
    pub const fn target(&self) -> &str {
        self.target
    }

    /// The recorded value of `name`, if the span carried that field.
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields.get(name).map(String::as_str)
    }

    /// The names of every field the span recorded, sorted.
    #[must_use]
    pub fn field_names(&self) -> Vec<&str> {
        self.fields.keys().map(String::as_str).collect()
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

#[cfg(test)]
mod tests {
    use super::TraceCapture;

    #[test]
    fn captures_spans_events_fields_and_targets() {
        let capture = TraceCapture::install();
        {
            let span = tracing::info_span!(target: "smoke::span", "smoke_span", answer = 42);
            let _entered = span.enter();
            tracing::warn!(target: "smoke::event", detail = "boom", "message");
        }

        let spans = capture.spans();
        let span = spans
            .iter()
            .find(|span| span.name() == "smoke_span")
            .expect("the installed subscriber must capture the span");
        assert_eq!(span.target(), "smoke::span");
        assert_eq!(span.field("answer"), Some("42"));
        assert!(span.field_names().contains(&"answer"));

        let events = capture.events();
        let event = events
            .iter()
            .find(|event| event.target() == "smoke::event")
            .expect("the installed subscriber must capture the event");
        assert_eq!(event.level(), tracing::Level::WARN);
        assert_eq!(event.field("detail"), Some("boom"));
    }
}
