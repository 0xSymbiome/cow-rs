#![allow(
    clippy::missing_const_for_fn,
    clippy::too_many_lines,
    clippy::type_complexity,
    reason = "table-driven cancellation tests keep shared harness code close to the cases"
)]

mod common;

use core::future::Future;
use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use cow_sdk_core::{Amount, Cancellable, CancellationToken};
use cow_sdk_orderbook::{
    AppDataObject, Auction, CompetitionOrderStatus, CowEnv, GetOrdersRequest, GetTradesRequest,
    NativePriceResponse, Order, OrderBookApi, OrderCancellations, OrderCreation, OrderQuoteRequest,
    OrderQuoteResponse, OrderUid, QuoteSide, SigningScheme, SolverCompetitionResponse,
    SupportedChainId, TotalSurplus, Trade,
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

use crate::common::{
    build_orderbook_api, build_orderbook_api_with_base_url, default_context, sample_app_data_hash,
    sample_order_uid, sample_owner, sample_quote_response_json, sample_signature, sample_tx_hash,
};

type CaseFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), cow_sdk_orderbook::OrderbookError>> + 'a>>;

struct CancellationCase {
    method_name: &'static str,
    http_method: &'static str,
    path: fn() -> String,
    invoke: for<'a> fn(&'a OrderBookApi) -> CaseFuture<'a>,
}

const TESTED_METHODS: &[CancellationCase] = &[
    CancellationCase {
        method_name: "get_quote",
        http_method: "POST",
        path: path_quote,
        invoke: invoke_get_quote,
    },
    CancellationCase {
        method_name: "send_order",
        http_method: "POST",
        path: path_orders,
        invoke: invoke_send_order,
    },
    CancellationCase {
        method_name: "send_signed_order_cancellations",
        http_method: "DELETE",
        path: path_orders,
        invoke: invoke_send_signed_order_cancellations,
    },
    CancellationCase {
        method_name: "get_order",
        http_method: "GET",
        path: path_order,
        invoke: invoke_get_order,
    },
    CancellationCase {
        method_name: "get_order_multi_env",
        http_method: "GET",
        path: path_order,
        invoke: invoke_get_order_multi_env,
    },
    CancellationCase {
        method_name: "get_orders",
        http_method: "GET",
        path: path_account_orders,
        invoke: invoke_get_orders,
    },
    CancellationCase {
        method_name: "get_tx_orders",
        http_method: "GET",
        path: path_tx_orders,
        invoke: invoke_get_tx_orders,
    },
    CancellationCase {
        method_name: "get_trades",
        http_method: "GET",
        path: path_trades,
        invoke: invoke_get_trades,
    },
    CancellationCase {
        method_name: "get_order_competition_status",
        http_method: "GET",
        path: path_order_status,
        invoke: invoke_get_order_competition_status,
    },
    CancellationCase {
        method_name: "get_native_price",
        http_method: "GET",
        path: path_native_price,
        invoke: invoke_get_native_price,
    },
    CancellationCase {
        method_name: "get_total_surplus",
        http_method: "GET",
        path: path_total_surplus,
        invoke: invoke_get_total_surplus,
    },
    CancellationCase {
        method_name: "get_app_data",
        http_method: "GET",
        path: path_app_data,
        invoke: invoke_get_app_data,
    },
    CancellationCase {
        method_name: "upload_app_data",
        http_method: "PUT",
        path: path_app_data,
        invoke: invoke_upload_app_data,
    },
    CancellationCase {
        method_name: "get_solver_competition_by_auction_id",
        http_method: "GET",
        path: path_solver_competition_by_auction_id,
        invoke: invoke_get_solver_competition_by_auction_id,
    },
    CancellationCase {
        method_name: "get_solver_competition_by_tx_hash",
        http_method: "GET",
        path: path_solver_competition_by_tx_hash,
        invoke: invoke_get_solver_competition_by_tx_hash,
    },
    CancellationCase {
        method_name: "get_latest_solver_competition",
        http_method: "GET",
        path: path_latest_solver_competition,
        invoke: invoke_get_latest_solver_competition,
    },
    CancellationCase {
        method_name: "get_auction",
        http_method: "GET",
        path: path_auction,
        invoke: invoke_get_auction,
    },
];

#[tokio::test]
async fn every_remaining_orderbook_method_returns_cancelled_when_token_is_pre_cancelled() {
    for case in TESTED_METHODS {
        let api = build_orderbook_api(default_context(SupportedChainId::Mainnet, CowEnv::Prod));
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
            matches!(error, cow_sdk_orderbook::OrderbookError::Cancelled),
            "{} must lift pre-cancelled tokens into OrderbookError::Cancelled, got {error:?}",
            case.method_name,
        );
    }
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn every_remaining_orderbook_method_aborts_an_in_flight_request() {
    for case in TESTED_METHODS {
        let server = MockServer::start().await;
        mount_slow_response(case, &server).await;
        let api = build_orderbook_api_with_base_url(
            default_context(SupportedChainId::Mainnet, CowEnv::Prod),
            server.uri(),
        );
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
            matches!(result, Err(cow_sdk_orderbook::OrderbookError::Cancelled)),
            "{} must lift in-flight aborts into OrderbookError::Cancelled, got {result:?}",
            case.method_name,
        );
        assert!(
            elapsed < Duration::from_secs(5),
            "{} must abort before the slow response deadline; elapsed = {elapsed:?}",
            case.method_name,
        );
        assert!(
            dropped.load(Ordering::SeqCst),
            "{} must drop the inner request future when the token fires",
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

async fn mount_slow_response(case: &CancellationCase, server: &MockServer) {
    Mock::given(method(case.http_method))
        .and(path((case.path)()))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "cancel": "before decode" }))
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
            "slow orderbook mock did not observe the request before cancellation"
        );
        tokio::task::yield_now().await;
    }
}

fn path_quote() -> String {
    "/api/v1/quote".to_owned()
}

fn path_orders() -> String {
    "/api/v1/orders".to_owned()
}

fn path_order() -> String {
    format!("/api/v1/orders/{}", sample_order_uid().as_str())
}

fn path_account_orders() -> String {
    format!("/api/v1/account/{}/orders", sample_owner().as_str())
}

fn path_tx_orders() -> String {
    format!("/api/v1/transactions/{}/orders", sample_tx_hash())
}

fn path_trades() -> String {
    "/api/v2/trades".to_owned()
}

fn path_order_status() -> String {
    format!("/api/v1/orders/{}/status", sample_order_uid().as_str())
}

fn path_native_price() -> String {
    format!("/api/v1/token/{}/native_price", sample_owner().as_str())
}

fn path_total_surplus() -> String {
    format!("/api/v1/users/{}/total_surplus", sample_owner().as_str())
}

fn path_app_data() -> String {
    format!("/api/v1/app_data/{}", sample_app_data_hash().as_str())
}

fn path_solver_competition_by_auction_id() -> String {
    "/api/v1/solver_competition/7".to_owned()
}

fn path_solver_competition_by_tx_hash() -> String {
    format!("/api/v1/solver_competition/by_tx_hash/{}", sample_tx_hash())
}

fn path_latest_solver_competition() -> String {
    "/api/v1/solver_competition/latest".to_owned()
}

fn path_auction() -> String {
    "/api/v1/auction".to_owned()
}

fn invoke_get_quote(api: &OrderBookApi) -> CaseFuture<'_> {
    Box::pin(async move {
        let request = quote_request();
        api.get_quote(&request)
            .await
            .map(|_: OrderQuoteResponse| ())
    })
}

fn invoke_send_order(api: &OrderBookApi) -> CaseFuture<'_> {
    Box::pin(async move {
        let order = order_creation();
        api.send_order(&order).await.map(|_: OrderUid| ())
    })
}

fn invoke_send_signed_order_cancellations(api: &OrderBookApi) -> CaseFuture<'_> {
    Box::pin(async move {
        let cancellation =
            OrderCancellations::new(vec![sample_order_uid()], sample_signature().to_owned());
        api.send_signed_order_cancellations(&cancellation).await
    })
}

fn invoke_get_order(api: &OrderBookApi) -> CaseFuture<'_> {
    Box::pin(async move { api.get_order(&sample_order_uid()).await.map(|_: Order| ()) })
}

fn invoke_get_order_multi_env(api: &OrderBookApi) -> CaseFuture<'_> {
    Box::pin(async move {
        api.get_order_multi_env(&sample_order_uid())
            .await
            .map(|_: Order| ())
    })
}

fn invoke_get_orders(api: &OrderBookApi) -> CaseFuture<'_> {
    Box::pin(async move {
        let request = GetOrdersRequest::new(sample_owner());
        api.get_orders(&request).await.map(|_: Vec<Order>| ())
    })
}

fn invoke_get_tx_orders(api: &OrderBookApi) -> CaseFuture<'_> {
    Box::pin(async move {
        api.get_tx_orders(sample_tx_hash())
            .await
            .map(|_: Vec<Order>| ())
    })
}

fn invoke_get_trades(api: &OrderBookApi) -> CaseFuture<'_> {
    Box::pin(async move {
        let request = GetTradesRequest::by_owner(sample_owner());
        api.get_trades(&request).await.map(|_: Vec<Trade>| ())
    })
}

fn invoke_get_order_competition_status(api: &OrderBookApi) -> CaseFuture<'_> {
    Box::pin(async move {
        api.get_order_competition_status(&sample_order_uid())
            .await
            .map(|_: CompetitionOrderStatus| ())
    })
}

fn invoke_get_native_price(api: &OrderBookApi) -> CaseFuture<'_> {
    Box::pin(async move {
        api.get_native_price(&sample_owner())
            .await
            .map(|_: NativePriceResponse| ())
    })
}

fn invoke_get_total_surplus(api: &OrderBookApi) -> CaseFuture<'_> {
    Box::pin(async move {
        api.get_total_surplus(&sample_owner())
            .await
            .map(|_: TotalSurplus| ())
    })
}

fn invoke_get_app_data(api: &OrderBookApi) -> CaseFuture<'_> {
    Box::pin(async move {
        api.get_app_data(&sample_app_data_hash())
            .await
            .map(|_: AppDataObject| ())
    })
}

fn invoke_upload_app_data(api: &OrderBookApi) -> CaseFuture<'_> {
    Box::pin(async move {
        api.upload_app_data(&sample_app_data_hash(), "{\"metadata\":true}")
            .await
            .map(|_: AppDataObject| ())
    })
}

fn invoke_get_solver_competition_by_auction_id(api: &OrderBookApi) -> CaseFuture<'_> {
    Box::pin(async move {
        api.get_solver_competition_by_auction_id(7)
            .await
            .map(|_: SolverCompetitionResponse| ())
    })
}

fn invoke_get_solver_competition_by_tx_hash(api: &OrderBookApi) -> CaseFuture<'_> {
    Box::pin(async move {
        api.get_solver_competition_by_tx_hash(sample_tx_hash())
            .await
            .map(|_: SolverCompetitionResponse| ())
    })
}

fn invoke_get_latest_solver_competition(api: &OrderBookApi) -> CaseFuture<'_> {
    Box::pin(async move {
        api.get_latest_solver_competition()
            .await
            .map(|_: SolverCompetitionResponse| ())
    })
}

fn invoke_get_auction(api: &OrderBookApi) -> CaseFuture<'_> {
    Box::pin(async move { api.get_auction().await.map(|_: Auction| ()) })
}

fn quote_request() -> OrderQuoteRequest {
    OrderQuoteRequest::new(
        sample_owner(),
        crate::common::sample_buy_token(),
        sample_owner(),
        QuoteSide::sell(Amount::new("1000000").expect("test amount literal must be valid")),
    )
}

fn order_creation() -> OrderCreation {
    let quote: OrderQuoteResponse = serde_json::from_value(sample_quote_response_json())
        .expect("quote fixture must deserialize for order creation");
    OrderCreation::from_quote(
        &quote.quote,
        sample_owner(),
        None,
        SigningScheme::Eip712,
        sample_signature(),
    )
    .with_quote_id(quote.id.expect("quote fixture includes quote id"))
}
