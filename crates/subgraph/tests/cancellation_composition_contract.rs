#![allow(
    clippy::missing_const_for_fn,
    clippy::too_many_lines,
    clippy::type_complexity,
    reason = "table-driven cancellation tests keep shared harness code close to the cases"
)]

use core::future::Future;
use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use cow_sdk_core::{Cancellable, CancellationToken, SupportedChainId};
use cow_sdk_subgraph::{
    LastDaysVolumeResponse, LastHoursVolumeResponse, SubgraphApi, SubgraphConfigOverride,
    SubgraphError, SubgraphQueryRequest, Total,
};
use serde_json::Value;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

mod common;

type CaseFuture<'a> = Pin<Box<dyn Future<Output = Result<(), SubgraphError>> + 'a>>;

struct CancellationCase {
    method_name: &'static str,
    invoke: for<'a> fn(&'a SubgraphApi) -> CaseFuture<'a>,
}

const TESTED_METHODS: &[CancellationCase] = &[
    CancellationCase {
        method_name: "get_totals_with_config",
        invoke: invoke_get_totals_with_config,
    },
    CancellationCase {
        method_name: "get_last_days_volume",
        invoke: invoke_get_last_days_volume,
    },
    CancellationCase {
        method_name: "get_last_days_volume_with_config",
        invoke: invoke_get_last_days_volume_with_config,
    },
    CancellationCase {
        method_name: "get_last_hours_volume",
        invoke: invoke_get_last_hours_volume,
    },
    CancellationCase {
        method_name: "get_last_hours_volume_with_config",
        invoke: invoke_get_last_hours_volume_with_config,
    },
    CancellationCase {
        method_name: "run_query",
        invoke: invoke_run_query,
    },
    CancellationCase {
        method_name: "run_query_with_config",
        invoke: invoke_run_query_with_config,
    },
];

#[tokio::test]
async fn every_remaining_subgraph_method_returns_cancelled_when_token_is_pre_cancelled() {
    for case in TESTED_METHODS {
        let api = SubgraphApi::builder()
            .chain(SupportedChainId::Mainnet)
            .api_key("FakeApiKey")
            .build()
            .expect("default subgraph client must build");
        let token = CancellationToken::new();
        token.cancel();

        let error = match (case.invoke)(&api).cancel_with(&token).await {
            Ok(()) => panic!(
                "{} must return an error for the pre-cancelled token branch",
                case.method_name,
            ),
            Err(error) => error,
        };

        assert!(
            matches!(error, SubgraphError::Cancelled),
            "{} must lift pre-cancelled tokens into SubgraphError::Cancelled, got {error:?}",
            case.method_name,
        );
    }
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn every_remaining_subgraph_method_aborts_an_in_flight_request() {
    for case in TESTED_METHODS {
        let server = MockServer::start().await;
        mount_slow_subgraph_response(&server).await;
        let api = common::loopback_client_no_timeout(server.uri());
        let token = CancellationToken::new();
        let token_for_call = token.clone();
        let dropped = Arc::new(AtomicBool::new(false));
        let spy = DropSpy(Arc::clone(&dropped));

        let started = Instant::now();
        let call = async {
            let _spy = spy;
            (case.invoke)(&api).cancel_with(&token_for_call).await
        };
        let trigger = async {
            wait_until_request_is_in_flight(&server).await;
            token.cancel();
        };

        let (result, ()) = tokio::join!(call, trigger);
        let elapsed = started.elapsed();

        assert!(
            matches!(result, Err(SubgraphError::Cancelled)),
            "{} must lift in-flight aborts into SubgraphError::Cancelled, got {result:?}",
            case.method_name,
        );
        assert!(
            elapsed < Duration::from_secs(5),
            "{} must abort before the slow response deadline; elapsed = {elapsed:?}",
            case.method_name,
        );
        assert!(
            dropped.load(Ordering::SeqCst),
            "{} must drop the inner subgraph request future when the token fires",
            case.method_name,
        );
    }
}

struct DropSpy(Arc<AtomicBool>);

impl Drop for DropSpy {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

async fn mount_slow_subgraph_response(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "data": null }))
                .set_delay(Duration::from_secs(30)),
        )
        .mount(server)
        .await;
}

async fn wait_until_request_is_in_flight(server: &MockServer) {
    // The enclosing test runs under `tokio::test(flavor = "current_thread",
    // start_paused = true)`. With paused tokio time, the reqwest client's
    // HTTP transmission and wiremock's request ingestion proceed on the
    // real wall clock while every tokio sleep on the test's runtime
    // auto-advances when no other task is pending. A pure `yield_now`
    // poll therefore burns through its iteration budget faster than the
    // operating-system scheduler can deliver the request bytes on a busy
    // CI runner, which is why this helper previously flaked under heavy
    // load on every test target. The new shape couples the poll to a
    // real-time deadline measured from `std::time::Instant`, which the
    // paused tokio clock cannot accelerate, and yields the runtime
    // between polls so wiremock's task on the same single-threaded
    // executor can drain the inbound queue.
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let requests = server
            .received_requests()
            .await
            .expect("wiremock should expose received requests");
        if !requests.is_empty() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "slow subgraph mock did not observe the request before cancellation"
        );
        tokio::task::yield_now().await;
    }
}

fn invoke_get_totals_with_config(api: &SubgraphApi) -> CaseFuture<'_> {
    Box::pin(async move {
        api.get_totals_with_config(SubgraphConfigOverride::default())
            .await
            .map(|_: Total| ())
    })
}

fn invoke_get_last_days_volume(api: &SubgraphApi) -> CaseFuture<'_> {
    Box::pin(async move {
        api.get_last_days_volume(7)
            .await
            .map(|_: LastDaysVolumeResponse| ())
    })
}

fn invoke_get_last_days_volume_with_config(api: &SubgraphApi) -> CaseFuture<'_> {
    Box::pin(async move {
        api.get_last_days_volume_with_config(7, SubgraphConfigOverride::default())
            .await
            .map(|_: LastDaysVolumeResponse| ())
    })
}

fn invoke_get_last_hours_volume(api: &SubgraphApi) -> CaseFuture<'_> {
    Box::pin(async move {
        api.get_last_hours_volume(24)
            .await
            .map(|_: LastHoursVolumeResponse| ())
    })
}

fn invoke_get_last_hours_volume_with_config(api: &SubgraphApi) -> CaseFuture<'_> {
    Box::pin(async move {
        api.get_last_hours_volume_with_config(24, SubgraphConfigOverride::default())
            .await
            .map(|_: LastHoursVolumeResponse| ())
    })
}

fn invoke_run_query(api: &SubgraphApi) -> CaseFuture<'_> {
    Box::pin(async move {
        api.run_query::<Value, _>(
            SubgraphQueryRequest::new("query CancellationProbe { totals { orders } }")
                .with_operation_name("CancellationProbe"),
        )
        .await
        .map(|_: Value| ())
    })
}

fn invoke_run_query_with_config(api: &SubgraphApi) -> CaseFuture<'_> {
    Box::pin(async move {
        api.run_query_with_config::<Value, _>(
            SubgraphQueryRequest::new("query CancellationProbe { totals { orders } }")
                .with_operation_name("CancellationProbe"),
            SubgraphConfigOverride::default(),
        )
        .await
        .map(|_: Value| ())
    })
}
