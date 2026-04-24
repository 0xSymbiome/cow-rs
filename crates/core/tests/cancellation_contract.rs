//! Contract coverage for the [`Cancellable`] combinator on `cow-sdk-core`.
//!
//! These scenarios lock the observable behaviour of the combinator: the
//! biased cancellation poll, the pass-through on a quiescent token, drop
//! semantics when the wrapper is discarded, and the generic composition
//! shape consumers rely on.

#![allow(
    clippy::missing_const_for_fn,
    reason = "pedantic lints acceptable in test helper code"
)]

use core::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use cow_sdk_core::{Cancellable, CancellationToken, CoreError};

#[tokio::test]
async fn pre_cancelled_token_resolves_to_cancelled_error() {
    let token = CancellationToken::new();
    token.cancel();

    let inner = async { Ok::<(), CoreError>(()) };
    let error = inner
        .cancel_with(&token)
        .await
        .expect_err("fired token must short-circuit the inner future");

    assert!(matches!(error, CoreError::Cancelled));
}

#[tokio::test]
async fn inner_future_passes_through_when_token_stays_quiescent() {
    let token = CancellationToken::new();
    let inner = async { Ok::<u32, CoreError>(42) };

    let value = inner
        .cancel_with(&token)
        .await
        .expect("inner must resolve when the token never fires");

    assert_eq!(value, 42);
}

#[tokio::test]
async fn dropping_the_wrapper_drops_the_inner_future() {
    struct DropMarker(Arc<AtomicBool>);

    impl Drop for DropMarker {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    let dropped = Arc::new(AtomicBool::new(false));
    let marker = DropMarker(Arc::clone(&dropped));
    let token = CancellationToken::new();

    let inner = async move {
        // Keep the drop marker alive until the wrapper is discarded.
        let _keep = marker;
        core::future::pending::<Result<(), CoreError>>().await
    };

    let wrapped = inner.cancel_with(&token);
    drop(wrapped);

    assert!(
        dropped.load(Ordering::SeqCst),
        "the wrapper must drop the inner future when it is itself dropped",
    );
}

#[tokio::test]
async fn blanket_impl_composes_generic_futures() {
    async fn run_cancellable<F, T>(future: F, token: &CancellationToken) -> Result<T, CoreError>
    where
        F: Future<Output = Result<T, CoreError>>,
    {
        future.cancel_with(token).await
    }

    let token = CancellationToken::new();
    let value = run_cancellable(async { Ok::<u32, CoreError>(7) }, &token)
        .await
        .expect("quiescent token must let the inner resolve");
    assert_eq!(value, 7);

    token.cancel();
    let error = run_cancellable(async { Ok::<u32, CoreError>(7) }, &token)
        .await
        .expect_err("fired token must short-circuit through the generic adapter");
    assert!(matches!(error, CoreError::Cancelled));
}

#[cfg(feature = "tracing")]
mod tracing_contract {
    use std::{
        collections::BTreeMap,
        sync::{
            Arc, Mutex,
            atomic::{AtomicU64, Ordering},
        },
    };

    use super::*;
    use tracing::{
        Event, Id, Level, Metadata, Subscriber,
        field::{Field, Visit},
        span::{Attributes, Record},
        subscriber::Interest,
    };

    #[tokio::test(flavor = "current_thread")]
    async fn cancellation_branch_emits_warn_event_with_target_and_field() {
        let capture = TraceCapture::install();
        let token = CancellationToken::new();
        token.cancel();

        let error = async { Ok::<(), CoreError>(()) }
            .cancel_with(&token)
            .await
            .expect_err("fired token must produce a cancellation error");

        assert!(matches!(error, CoreError::Cancelled));
        let events = capture.events();
        assert!(
            events.iter().any(|event| {
                event.level == Level::WARN
                    && event.target == "cow_sdk::cancel"
                    && event.field("cancelled") == Some("true")
            }),
            "cancellation must emit a warn event with target and cancelled field: {events:#?}",
        );
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

        fn new_span(&self, _attributes: &Attributes<'_>) -> Id {
            Id::from_u64(self.next_id.fetch_add(1, Ordering::SeqCst))
        }

        fn record(&self, _span: &Id, _values: &Record<'_>) {}

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
    struct CapturedEvent {
        level: Level,
        target: &'static str,
        fields: BTreeMap<String, String>,
    }

    impl CapturedEvent {
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
}
