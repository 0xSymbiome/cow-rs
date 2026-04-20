mod common;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cow_sdk_core::{CoreError, DEFAULT_HTTP_TIMEOUT, HttpClientPolicy, ValidationError};
use cow_sdk_orderbook::{
    ApiContextOverride, AppDataObject, CowEnv, DEFAULT_MAX_ATTEMPTS, DEFAULT_ORDERBOOK_USER_AGENT,
    GetOrdersRequest, GetTradesRequest, OrderBookApi, OrderBookTransportPolicy, OrderCancellations,
    OrderCreation, OrderStatus, QuoteSide, RequestPolicy, SigningScheme, SupportedChainId,
};
use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_partial_json, method, path, query_param},
};

use crate::common::{
    default_context, sample_app_data_hash, sample_order_json, sample_order_uid, sample_owner,
    sample_quote_response_json, sample_signature, sample_trade_json, sample_tx_hash,
};

#[tokio::test]
async fn version_endpoint_matches_transport_contract() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string("v1.2.3"))
        .mount(&server)
        .await;

    let api = OrderBookApi::new_with_base_url(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        server.uri(),
    );

    let version = api
        .get_version()
        .await
        .expect("version request should succeed");

    assert_eq!(version, "v1.2.3");
}

#[test]
fn default_transport_policy_is_explicit_and_stable() {
    let api = OrderBookApi::new(default_context(SupportedChainId::GnosisChain, CowEnv::Prod));
    let policy = api.transport_policy();

    assert_eq!(policy.client_policy().timeout(), Some(DEFAULT_HTTP_TIMEOUT));
    assert_eq!(
        policy.client_policy().user_agent(),
        DEFAULT_ORDERBOOK_USER_AGENT
    );
    assert_eq!(policy.request_policy().max_attempts, DEFAULT_MAX_ATTEMPTS);
}

#[tokio::test]
async fn context_override_applies_base_urls_and_api_key_to_requests() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/version"))
        .and(wiremock::matchers::header("x-api-key", "partner-key"))
        .respond_with(ResponseTemplate::new(200).set_body_string("v1.2.3"))
        .mount(&server)
        .await;

    let base_urls = std::collections::BTreeMap::from([(
        u64::from(SupportedChainId::GnosisChain),
        format!("{}/", server.uri()),
    )]);

    let api = OrderBookApi::new(default_context(SupportedChainId::GnosisChain, CowEnv::Prod))
        .with_context_override(
            ApiContextOverride::new()
                .with_base_urls(base_urls)
                .with_api_key("partner-key".to_owned().into()),
        );

    let version = api
        .get_version()
        .await
        .expect("context override request should succeed");

    assert_eq!(version, "v1.2.3");
    assert_eq!(
        api.context()
            .api_key
            .as_ref()
            .map(|value| value.as_inner().as_str()),
        Some("partner-key")
    );
}

#[tokio::test]
async fn invalid_partner_api_key_fails_before_transport() {
    let api = OrderBookApi::new(default_context(SupportedChainId::GnosisChain, CowEnv::Prod))
        .with_context_override(
            ApiContextOverride::new().with_api_key("partner\r\nkey".to_owned().into()),
        );

    let error = api
        .get_version()
        .await
        .expect_err("invalid API key must fail before request transport");

    assert!(matches!(
        error,
        cow_sdk_orderbook::OrderbookError::Core(CoreError::Validation(
            ValidationError::InvalidHttpHeaderValue { field: "api_key" }
        ))
    ));
}

#[test]
fn explicit_env_base_url_override_precedes_context_base_urls() {
    let uid = sample_order_uid();
    let context_base_urls = std::collections::BTreeMap::from([(
        u64::from(SupportedChainId::GnosisChain),
        "https://context.example/xdai/".to_owned(),
    )]);

    let api = OrderBookApi::new(default_context(SupportedChainId::GnosisChain, CowEnv::Prod))
        .with_context_override(ApiContextOverride::new().with_base_urls(context_base_urls))
        .with_env_base_url(CowEnv::Prod, "https://override.example/xdai/");

    assert_eq!(
        api.get_order_link(&uid)
            .expect("explicit env override should win"),
        format!(
            "https://override.example/xdai/api/v1/orders/{}",
            uid.as_str()
        )
    );
}

#[tokio::test]
async fn transport_policy_override_rebuilds_client_with_custom_user_agent() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/version"))
        .and(wiremock::matchers::header(
            "user-agent",
            "custom-orderbook-client/9.9.9",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_string("v9.9.9"))
        .mount(&server)
        .await;

    let transport_policy = OrderBookTransportPolicy::default()
        .with_client_policy(
            HttpClientPolicy::new("custom-orderbook-client/9.9.9")
                .expect("custom header must be valid")
                .without_timeout(),
        )
        .with_request_policy(RequestPolicy {
            max_attempts: 1,
            ..RequestPolicy::default()
        });
    let api = OrderBookApi::new_with_transport_policy(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        transport_policy,
    )
    .with_env_base_url(CowEnv::Prod, server.uri());

    let version = api
        .get_version()
        .await
        .expect("custom client policy should succeed");

    assert_eq!(version, "v9.9.9");
    assert_eq!(api.client_policy().timeout(), None);
    assert_eq!(api.request_policy().max_attempts, 1);
}

#[tokio::test]
async fn cloned_clients_share_the_same_instance_scoped_rate_limiter() {
    let server = MockServer::start().await;
    let arrivals = Arc::new(Mutex::new(Vec::new()));

    Mock::given(method("GET"))
        .and(path("/api/v1/version"))
        .respond_with({
            let arrivals = arrivals.clone();
            move |_request: &wiremock::Request| {
                arrivals
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(Instant::now());
                ResponseTemplate::new(200).set_body_string("v1.2.3")
            }
        })
        .expect(2)
        .mount(&server)
        .await;

    let transport_policy = OrderBookTransportPolicy::default().with_request_policy(RequestPolicy {
        max_attempts: 1,
        rate_limit: cow_sdk_orderbook::request::RateLimitSettings {
            tokens_per_interval: 1,
            interval: Duration::from_millis(60),
            interval_label: "test",
        },
    });
    let api = OrderBookApi::new_with_transport_policy(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        transport_policy,
    )
    .with_env_base_url(CowEnv::Prod, server.uri());

    let sibling = api.clone();
    let (first, second) = tokio::join!(api.get_version(), sibling.get_version());

    assert_eq!(
        first.expect("first version request should succeed"),
        "v1.2.3"
    );
    assert_eq!(
        second.expect("second version request should succeed"),
        "v1.2.3"
    );

    let arrivals = arrivals
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert_eq!(arrivals.len(), 2);
    assert!(
        arrivals[1].duration_since(arrivals[0]) >= Duration::from_millis(30),
        "cloned clients should share one limiter instance"
    );
}

#[test]
fn order_link_uses_chain_aware_urls_for_gnosis_and_mainnet() {
    let uid = sample_order_uid();
    let gnosis = OrderBookApi::new(default_context(SupportedChainId::GnosisChain, CowEnv::Prod));
    let mainnet = gnosis
        .clone()
        .with_context_override(ApiContextOverride::new().with_chain_id(SupportedChainId::Mainnet));

    assert_eq!(
        gnosis
            .get_order_link(&uid)
            .expect("gnosis order link should resolve"),
        format!("https://api.cow.fi/xdai/api/v1/orders/{}", uid.as_str())
    );
    assert_eq!(
        mainnet
            .get_order_link(&uid)
            .expect("mainnet order link should resolve"),
        format!("https://api.cow.fi/mainnet/api/v1/orders/{}", uid.as_str())
    );
}

#[tokio::test]
async fn get_orders_uses_default_pagination_and_transforms_orders() {
    let server = MockServer::start().await;
    let uid = sample_order_uid();
    Mock::given(method("GET"))
        .and(path(format!(
            "/api/v1/account/{}/orders",
            sample_owner().as_str()
        )))
        .and(query_param("offset", "0"))
        .and(query_param("limit", "1000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![sample_order_json(&uid)]))
        .mount(&server)
        .await;

    let api = OrderBookApi::new_with_base_url(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        server.uri(),
    );

    let orders = api
        .get_orders(&GetOrdersRequest::new(sample_owner()))
        .await
        .expect("orders request should succeed");

    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].uid, uid);
    assert_eq!(orders[0].total_fee, "20");
}

#[tokio::test]
async fn get_trades_requires_owner_xor_order_uid_and_keeps_default_pagination() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2/trades"))
        .and(query_param("owner", sample_owner().as_str()))
        .and(query_param("offset", "0"))
        .and(query_param("limit", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![sample_trade_json()]))
        .mount(&server)
        .await;

    let api = OrderBookApi::new_with_base_url(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        server.uri(),
    );
    let trades = api
        .get_trades(&GetTradesRequest::by_owner(sample_owner()))
        .await
        .expect("trade request should succeed");

    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].transaction_hash, sample_tx_hash());

    let invalid = api
        .get_trades(&GetTradesRequest::new(
            Some(sample_owner()),
            Some(sample_order_uid()),
        ))
        .await
        .expect_err("owner+uid request must fail before transport");

    match invalid {
        cow_sdk_orderbook::OrderbookError::InvalidTradesQuery { reason, .. } => {
            assert!(reason.to_string().contains("exactly one"));
        }
        other => panic!("expected InvalidTradesQuery, got {other:?}"),
    }
}

#[tokio::test]
async fn get_quote_and_send_order_cover_quote_and_duplicate_order_paths() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/quote"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_quote_response_json()))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/orders"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "errorType": "DuplicateOrder",
            "description": "order already exists"
        })))
        .mount(&server)
        .await;

    let api = OrderBookApi::new_with_base_url(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        server.uri(),
    );
    let quote = api
        .get_quote(&cow_sdk_orderbook::OrderQuoteRequest::new(
            sample_owner(),
            crate::common::sample_buy_token(),
            sample_owner(),
            QuoteSide::sell("1000000"),
        ))
        .await
        .expect("quote should succeed");

    let order = OrderCreation::from_quote(
        &quote.quote,
        sample_owner(),
        None,
        SigningScheme::Eip712,
        sample_signature(),
    )
    .with_quote_id(quote.id.expect("fixture includes quote id"));

    let error = api
        .send_order(&order)
        .await
        .expect_err("duplicate order should surface API error");

    match error {
        cow_sdk_orderbook::OrderbookError::Api(api_error) => {
            assert_eq!(api_error.status, 400);
            assert_eq!(api_error.error_type(), Some("DuplicateOrder"));
        }
        other => panic!("expected API error, got {other:?}"),
    }
}

#[tokio::test]
async fn signed_cancellations_use_delete_orders_route() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/orders"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let api = OrderBookApi::new_with_base_url(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        server.uri(),
    );
    let cancellation =
        OrderCancellations::new(vec![sample_order_uid()], sample_signature().to_owned());

    api.send_signed_order_cancellations(&cancellation)
        .await
        .expect("signed cancellation should succeed");
}

#[tokio::test]
async fn order_lookup_falls_back_to_staging_only_on_404() {
    let prod = MockServer::start().await;
    let staging = MockServer::start().await;
    let uid = sample_order_uid();

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/orders/{}", uid.as_str())))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "errorType": "NotFound",
            "description": "missing in prod"
        })))
        .mount(&prod)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/orders/{}", uid.as_str())))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_order_json(&uid)))
        .mount(&staging)
        .await;

    let api = OrderBookApi::new(default_context(SupportedChainId::GnosisChain, CowEnv::Prod))
        .with_env_base_url(CowEnv::Prod, prod.uri())
        .with_env_base_url(CowEnv::Staging, staging.uri());

    let order = api
        .get_order_multi_env(&uid)
        .await
        .expect("staging fallback should succeed");

    assert_eq!(order.uid, uid);
    assert_eq!(order.status, OrderStatus::Open);
}

#[tokio::test]
async fn app_data_transport_helpers_use_get_and_put_hash_routes() {
    let server = MockServer::start().await;
    let hash = sample_app_data_hash();

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/app_data/{}", hash.as_str())))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "fullAppData": "{\"metadata\":true}"
        })))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path(format!("/api/v1/app_data/{}", hash.as_str())))
        .and(body_partial_json(
            json!({ "fullAppData": "{\"metadata\":true}" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "fullAppData": "{\"metadata\":true}"
        })))
        .mount(&server)
        .await;

    let api = OrderBookApi::new_with_base_url(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        server.uri(),
    );

    let downloaded: AppDataObject = api
        .get_app_data(&hash)
        .await
        .expect("app-data fetch should succeed");
    let uploaded = api
        .upload_app_data(&hash, "{\"metadata\":true}")
        .await
        .expect("app-data upload should succeed");

    assert_eq!(downloaded.full_app_data, "{\"metadata\":true}");
    assert_eq!(uploaded.full_app_data, "{\"metadata\":true}");
}

#[tokio::test]
async fn native_price_surplus_solver_competition_and_auction_routes_are_covered() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!(
            "/api/v1/token/{}/native_price",
            sample_owner().as_str()
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "price": 0.0004 })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!(
            "/api/v1/users/{}/total_surplus",
            sample_owner().as_str()
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "totalSurplus": "100000000"
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/solver_competition/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "auctionId": 7
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!(
            "/api/v1/solver_competition/by_tx_hash/{}",
            sample_tx_hash()
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "auctionId": 8
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/auction"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 1,
            "block": 100,
            "orders": [],
            "prices": {}
        })))
        .mount(&server)
        .await;

    let api = OrderBookApi::new_with_base_url(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        server.uri(),
    );

    let native_price = api
        .get_native_price(&sample_owner())
        .await
        .expect("native price request should succeed");
    let surplus = api
        .get_total_surplus(&sample_owner())
        .await
        .expect("surplus request should succeed");
    let by_auction = api
        .get_solver_competition_by_auction_id(7)
        .await
        .expect("competition by auction id should succeed");
    let by_tx = api
        .get_solver_competition_by_tx_hash(sample_tx_hash())
        .await
        .expect("competition by tx hash should succeed");
    let auction = api
        .get_auction()
        .await
        .expect("auction request should succeed");

    assert!((native_price.price - 0.0004).abs() < 1.0e-12);
    assert_eq!(surplus.total_surplus, "100000000");
    assert_eq!(by_auction.auction_id, Some(7));
    assert_eq!(by_tx.auction_id, Some(8));
    assert_eq!(auction.id, Some(1));
}

#[tokio::test]
async fn get_order_status_route_is_typed() {
    let server = MockServer::start().await;
    let uid = sample_order_uid();
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/orders/{}/status", uid.as_str())))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "type": "open",
            "value": null
        })))
        .mount(&server)
        .await;

    let api = OrderBookApi::new_with_base_url(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        server.uri(),
    );
    let status = api
        .get_order_competition_status(&uid)
        .await
        .expect("status request should succeed");

    assert_eq!(
        status.kind,
        cow_sdk_orderbook::CompetitionOrderStatusKind::Open
    );
}

#[tokio::test]
async fn get_version_returns_cancelled_when_combinator_token_fires_before_send() {
    use cow_sdk_core::Cancellable;

    let api = OrderBookApi::new(default_context(SupportedChainId::Mainnet, CowEnv::Prod));
    let token = cow_sdk_core::CancellationToken::new();
    token.cancel();

    let error = api
        .get_version()
        .cancel_with(&token)
        .await
        .expect_err("pre-cancelled token must produce a Cancelled error");
    assert!(matches!(
        error,
        cow_sdk_orderbook::OrderbookError::Cancelled
    ));
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn get_version_combinator_aborts_an_in_flight_request() {
    use cow_sdk_core::Cancellable;

    struct DropSpy(Arc<AtomicBool>);

    impl Drop for DropSpy {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/version"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("v1.0.0")
                .set_delay(Duration::from_secs(30)),
        )
        .mount(&server)
        .await;

    let api = OrderBookApi::new_with_base_url(
        default_context(SupportedChainId::Mainnet, CowEnv::Prod),
        server.uri(),
    );
    let token = cow_sdk_core::CancellationToken::new();
    let token_for_task = token.clone();
    let dropped = Arc::new(AtomicBool::new(false));
    let spy = DropSpy(Arc::clone(&dropped));

    let started = Instant::now();
    let task = tokio::spawn(async move {
        let _spy = spy;
        api.get_version().cancel_with(&token_for_task).await
    });

    tokio::time::sleep(Duration::from_millis(50)).await;
    token.cancel();

    let result = task.await.expect("cancellation task should not panic");
    let elapsed = started.elapsed();

    assert!(matches!(
        result,
        Err(cow_sdk_orderbook::OrderbookError::Cancelled)
    ));
    assert!(
        elapsed < Duration::from_secs(5),
        "cancellation must drop the in-flight future within the request deadline; elapsed = {elapsed:?}"
    );
    assert!(
        dropped.load(Ordering::SeqCst),
        "the inner request future must be dropped when the cancellation token fires"
    );
}

#[tokio::test]
async fn shared_client_fans_requests_across_multiple_orderbook_instances() {
    let first = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string("shared-client-first"))
        .expect(1)
        .mount(&first)
        .await;

    let second = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string("shared-client-second"))
        .expect(1)
        .mount(&second)
        .await;

    let shared = reqwest::Client::builder()
        .user_agent(DEFAULT_ORDERBOOK_USER_AGENT)
        .build()
        .expect("reqwest client must build for the shared-client regression test");

    let first_base_urls = std::collections::BTreeMap::from([(
        u64::from(SupportedChainId::Mainnet),
        format!("{}/", first.uri()),
    )]);
    let first_api = OrderBookApi::from_shared_client(
        shared.clone(),
        cow_sdk_core::ApiContext::new(SupportedChainId::Mainnet, CowEnv::Prod)
            .with_base_urls(first_base_urls),
    );

    let second_base_urls = std::collections::BTreeMap::from([(
        u64::from(SupportedChainId::GnosisChain),
        format!("{}/", second.uri()),
    )]);
    let second_api = OrderBookApi::from_shared_client(
        shared,
        cow_sdk_core::ApiContext::new(SupportedChainId::GnosisChain, CowEnv::Prod)
            .with_base_urls(second_base_urls),
    );

    let first_version = first_api
        .get_version()
        .await
        .expect("first shared-client request must succeed");
    let second_version = second_api
        .get_version()
        .await
        .expect("second shared-client request must succeed");

    assert_eq!(first_version, "shared-client-first");
    assert_eq!(second_version, "shared-client-second");
}
