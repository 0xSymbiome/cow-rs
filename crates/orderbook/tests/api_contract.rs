mod common;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cow_sdk_core::transport::policy::{
    DEFAULT_MAX_ATTEMPTS, DEFAULT_ORDERBOOK_USER_AGENT, TransportPolicy,
};
use cow_sdk_core::{
    Amount, AppDataHash, CoreError, DEFAULT_HTTP_TIMEOUT, HttpClientPolicy, ValidationError,
};
use cow_sdk_orderbook::{
    ApiContextOverride, AppDataObject, CowEnv, HashMismatchStage, OrderCancellations,
    OrderCreation, OrderQuoteSide, OrderStatus, OrderbookError, OrdersQuery, SigningScheme,
    SolverCompetitionResponse, SupportedChainId, TradesQuery,
};
use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_partial_json, method, path, query_param},
};

use crate::common::{
    SAMPLE_UPLOAD_BODY, build_orderbook_api, build_orderbook_api_with_base_url,
    build_orderbook_api_with_policy, build_orderbook_api_with_shared_client, default_context,
    limiter, retry_policy, sample_app_data_hash, sample_order_json, sample_order_uid, sample_owner,
    sample_quote_response_json, sample_signature, sample_trade_json, sample_tx_hash,
    sample_upload_body_hash,
};

#[tokio::test]
async fn version_endpoint_matches_transport_contract() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string("v1.2.3"))
        .mount(&server)
        .await;

    let api = build_orderbook_api_with_base_url(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        server.uri(),
    );

    let version = api.version().await.expect("version request should succeed");

    assert_eq!(version, "v1.2.3");
}

#[test]
fn default_transport_policy_is_explicit_and_stable() {
    let api = build_orderbook_api(default_context(SupportedChainId::GnosisChain, CowEnv::Prod));
    let policy = api.transport_policy();

    assert_eq!(policy.client_policy().timeout(), Some(DEFAULT_HTTP_TIMEOUT));
    assert_eq!(
        policy.client_policy().user_agent(),
        DEFAULT_ORDERBOOK_USER_AGENT
    );
    assert_eq!(policy.retry().max_attempts(), DEFAULT_MAX_ATTEMPTS);
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

    let api = build_orderbook_api(default_context(SupportedChainId::GnosisChain, CowEnv::Prod))
        .with_context_override(
            ApiContextOverride::new()
                .with_base_urls(base_urls)
                .with_api_key("partner-key".to_owned().into()),
        );

    let version = api
        .version()
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
    let api = build_orderbook_api(default_context(SupportedChainId::GnosisChain, CowEnv::Prod))
        .with_context_override(
            ApiContextOverride::new().with_api_key("partner\r\nkey".to_owned().into()),
        );

    let error = api
        .version()
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

    let api = build_orderbook_api(default_context(SupportedChainId::GnosisChain, CowEnv::Prod))
        .with_context_override(ApiContextOverride::new().with_base_urls(context_base_urls))
        .with_env_base_url(CowEnv::Prod, "https://override.example/xdai/");

    assert_eq!(
        api.order_link(&uid)
            .expect("explicit env override should win"),
        format!(
            "https://override.example/xdai/api/v1/orders/{}",
            uid.to_hex_string()
        )
    );
}

#[test]
fn api_debug_redacts_context_base_url_credentials() {
    let base_urls = std::collections::BTreeMap::from([(
        u64::from(SupportedChainId::GnosisChain),
        "https://user:pass@example.test/path?apiKey=secret-token".to_owned(),
    )]);
    let api = build_orderbook_api(default_context(SupportedChainId::GnosisChain, CowEnv::Prod))
        .with_context_override(ApiContextOverride::new().with_base_urls(base_urls));

    let debug = format!("{api:#?}");

    assert!(debug.contains(cow_sdk_core::REDACTED_PLACEHOLDER));
    assert!(!debug.contains("user:pass"));
    assert!(!debug.contains("apiKey=secret-token"));
    assert!(!debug.contains("example.test"));
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

    let transport_policy = TransportPolicy::default()
        .with_client_policy(
            HttpClientPolicy::new("custom-orderbook-client/9.9.9")
                .expect("custom header must be valid")
                .without_timeout(),
        )
        .with_retry(retry_policy(1));
    let api = build_orderbook_api_with_policy(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        transport_policy,
    )
    .with_env_base_url(CowEnv::Prod, server.uri());

    let version = api
        .version()
        .await
        .expect("custom client policy should succeed");

    assert_eq!(version, "v9.9.9");
    assert_eq!(api.client_policy().timeout(), None);
    assert_eq!(api.retry_policy().max_attempts(), 1);
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

    let transport_policy = TransportPolicy::default()
        .with_retry(retry_policy(1))
        .with_rate_limit(limiter(1, Duration::from_millis(60), "test"));
    let api = build_orderbook_api_with_policy(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        transport_policy,
    )
    .with_env_base_url(CowEnv::Prod, server.uri());

    let sibling = api.clone();
    let (first, second) = tokio::join!(api.version(), sibling.version());

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

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn service_unavailable_retry_after_header_delays_retry_for_at_least_server_cooldown() {
    let server = MockServer::start().await;
    let arrivals = Arc::new(Mutex::new(Vec::new()));

    Mock::given(method("GET"))
        .and(path("/api/v1/version"))
        .respond_with({
            let arrivals = arrivals.clone();
            move |_request: &wiremock::Request| {
                let mut arrivals = arrivals
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                arrivals.push(Instant::now());

                if arrivals.len() == 1 {
                    ResponseTemplate::new(503)
                        .insert_header("Retry-After", "5")
                        .set_body_json(json!({
                            "errorType": "InternalServerError",
                            "description": "retry after cooldown",
                        }))
                } else {
                    ResponseTemplate::new(200).set_body_string("v1.2.3")
                }
            }
        })
        .expect(2)
        .mount(&server)
        .await;

    let policy = TransportPolicy::default().with_retry(retry_policy(2));
    let api = build_orderbook_api_with_policy(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        policy,
    )
    .with_env_base_url(CowEnv::Prod, server.uri());

    let version = api
        .version()
        .await
        .expect("retry-after response should be retried after the server cooldown");

    assert_eq!(version, "v1.2.3");
    let arrivals = arrivals
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert_eq!(arrivals.len(), 2);
    assert!(
        arrivals[1].duration_since(arrivals[0]) >= Duration::from_secs(5),
        "Retry-After: 5 must delay the next attempt by at least five seconds"
    );
}

#[test]
fn order_link_uses_chain_aware_urls_for_gnosis_and_mainnet() {
    let uid = sample_order_uid();
    let gnosis = build_orderbook_api(default_context(SupportedChainId::GnosisChain, CowEnv::Prod));
    let mainnet = gnosis
        .clone()
        .with_context_override(ApiContextOverride::new().with_chain_id(SupportedChainId::Mainnet));

    assert_eq!(
        gnosis
            .order_link(&uid)
            .expect("gnosis order link should resolve"),
        format!(
            "https://api.cow.fi/xdai/api/v1/orders/{}",
            uid.to_hex_string()
        )
    );
    assert_eq!(
        mainnet
            .order_link(&uid)
            .expect("mainnet order link should resolve"),
        format!(
            "https://api.cow.fi/mainnet/api/v1/orders/{}",
            uid.to_hex_string()
        )
    );
}

#[tokio::test]
async fn get_orders_uses_default_pagination_and_transforms_orders() {
    let server = MockServer::start().await;
    let uid = sample_order_uid();
    Mock::given(method("GET"))
        .and(path(format!(
            "/api/v1/account/{}/orders",
            sample_owner().to_hex_string()
        )))
        .and(query_param("offset", "0"))
        .and(query_param("limit", "1000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![sample_order_json(&uid)]))
        .mount(&server)
        .await;

    let api = build_orderbook_api_with_base_url(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        server.uri(),
    );

    let orders = api
        .orders(&OrdersQuery::new(sample_owner()))
        .await
        .expect("orders request should succeed");

    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].uid, uid);
    assert_eq!(
        orders[0].total_fee,
        Amount::new("20").expect("test amount literal must be valid")
    );
}

#[tokio::test]
async fn account_orders_pagination_boundary_table() {
    for (offset, limit) in [(0, 1), (1, 1000), (u32::MAX - 1, u32::MAX)] {
        let server = MockServer::start().await;
        let uid = sample_order_uid();
        Mock::given(method("GET"))
            .and(path(format!(
                "/api/v1/account/{}/orders",
                sample_owner().to_hex_string()
            )))
            .and(query_param("offset", offset.to_string()))
            .and(query_param("limit", limit.to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_json(vec![sample_order_json(&uid)]))
            .mount(&server)
            .await;

        let api = build_orderbook_api_with_base_url(
            default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
            server.uri(),
        );
        let request = OrdersQuery::new(sample_owner())
            .with_offset(offset)
            .with_limit(limit);

        let orders = api
            .orders(&request)
            .await
            .unwrap_or_else(|error| panic!("pagination case {offset}/{limit} failed: {error}"));

        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].uid, uid);
    }
}

#[tokio::test]
async fn get_trades_requires_owner_xor_order_uid_and_keeps_default_pagination() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2/trades"))
        .and(query_param("owner", sample_owner().to_hex_string()))
        .and(query_param("offset", "0"))
        .and(query_param("limit", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![sample_trade_json()]))
        .mount(&server)
        .await;

    let api = build_orderbook_api_with_base_url(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        server.uri(),
    );
    let trades = api
        .trades(&TradesQuery::by_owner(sample_owner()))
        .await
        .expect("trade request should succeed");

    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].tx_hash, Some(sample_tx_hash()));

    let invalid = api
        .trades(&TradesQuery::new(
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
            "errorType": "DuplicatedOrder",
            "description": "order already exists"
        })))
        .mount(&server)
        .await;

    let api = build_orderbook_api_with_base_url(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        server.uri(),
    );
    let quote = api
        .quote(&cow_sdk_orderbook::OrderQuoteRequest::new(
            sample_owner(),
            crate::common::sample_buy_token(),
            sample_owner(),
            OrderQuoteSide::sell(
                // The canned quote response echoes a fixed leg of
                // sellAmount 1000 + feeAmount 10, so the request asks for the
                // same before-fee total and `ensure_matches` reconciles it.
                Amount::new("1010").expect("test amount literal must be valid"),
            ),
        ))
        .await
        .expect("quote should succeed");

    let order = OrderCreation::from_quote(
        &quote,
        sample_owner(),
        None,
        SigningScheme::Eip712,
        sample_signature(),
    );

    let error = api
        .send_order(&order)
        .await
        .expect_err("duplicate order should surface API error");

    match error {
        cow_sdk_orderbook::OrderbookError::Rejected {
            status,
            rejection,
            source,
        } => {
            assert_eq!(status.as_u16(), 400);
            assert_eq!(
                rejection,
                cow_sdk_orderbook::OrderbookRejection::DuplicatedOrder
            );
            assert_eq!(source.status, 400);
        }
        other => panic!("expected Rejected, got {other:?}"),
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

    let api = build_orderbook_api_with_base_url(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        server.uri(),
    );
    let cancellation =
        OrderCancellations::new(vec![sample_order_uid()], sample_signature().to_owned());

    api.send_cancellations(&cancellation)
        .await
        .expect("signed cancellation should succeed");
}

#[tokio::test]
async fn order_lookup_falls_back_to_staging_only_on_404() {
    let prod = MockServer::start().await;
    let staging = MockServer::start().await;
    let uid = sample_order_uid();

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/orders/{}", uid.to_hex_string())))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "errorType": "NotFound",
            "description": "missing in prod"
        })))
        .mount(&prod)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/orders/{}", uid.to_hex_string())))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_order_json(&uid)))
        .mount(&staging)
        .await;

    let api = build_orderbook_api(default_context(SupportedChainId::GnosisChain, CowEnv::Prod))
        .with_env_base_url(CowEnv::Prod, prod.uri())
        .with_env_base_url(CowEnv::Staging, staging.uri());

    let order = api
        .order_multi_env(&uid)
        .await
        .expect("staging fallback should succeed");

    assert_eq!(order.uid, uid);
    assert_eq!(order.status, OrderStatus::Open);
}

#[tokio::test]
async fn app_data_transport_helpers_use_get_and_put_hash_routes() {
    let server = MockServer::start().await;
    let get_hash = sample_app_data_hash();
    let upload_hash = sample_upload_body_hash();

    Mock::given(method("GET"))
        .and(path(format!(
            "/api/v1/app_data/{}",
            get_hash.to_hex_string()
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "fullAppData": "{\"metadata\":true}"
        })))
        .mount(&server)
        .await;
    // The orderbook responds to a successful PUT with the bare hex-encoded
    // hash as the JSON body; AppDataHash#[serde(transparent)] decodes the
    // form directly without an envelope.
    Mock::given(method("PUT"))
        .and(path(format!(
            "/api/v1/app_data/{}",
            upload_hash.to_hex_string()
        )))
        .and(body_partial_json(
            json!({ "fullAppData": SAMPLE_UPLOAD_BODY }),
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!(upload_hash.to_hex_string())))
        .mount(&server)
        .await;

    let api = build_orderbook_api_with_base_url(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        server.uri(),
    );

    let downloaded: AppDataObject = api
        .app_data(&get_hash)
        .await
        .expect("app-data fetch should succeed");
    api.upload_app_data(&upload_hash, SAMPLE_UPLOAD_BODY)
        .await
        .expect("app-data upload should succeed");

    assert_eq!(downloaded.full_app_data, "{\"metadata\":true}");
}

#[tokio::test]
async fn upload_app_data_rejects_client_precheck_mismatch_without_network() {
    let server = MockServer::start().await;

    // No mock mounted: any dispatch to this server returns 404 / wiremock-no-match,
    // which the SDK would surface as a non-Cancelled error. The test passes only
    // if the precheck short-circuits before the network call.
    let api = build_orderbook_api_with_base_url(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        server.uri(),
    );

    let supplied = AppDataHash::ZERO;
    let body = SAMPLE_UPLOAD_BODY;
    let expected_observed = AppDataHash::from_full_app_data(body);

    let error = api
        .upload_app_data(&supplied, body)
        .await
        .expect_err("client precheck must reject the mismatched hash");

    match error {
        OrderbookError::AppDataHashMismatch {
            expected,
            observed,
            stage,
        } => {
            assert_eq!(expected, supplied);
            assert_eq!(observed, expected_observed);
            assert_eq!(stage, HashMismatchStage::ClientPrecheck);
        }
        other => panic!("expected client-precheck mismatch, got {other:?}"),
    }

    assert!(
        server
            .received_requests()
            .await
            .unwrap_or_default()
            .is_empty(),
        "client precheck must not dispatch a network request",
    );
}

#[tokio::test]
async fn upload_app_data_rejects_server_echo_mismatch() {
    let server = MockServer::start().await;
    let body = SAMPLE_UPLOAD_BODY;
    let supplied = AppDataHash::from_full_app_data(body);
    let returned = AppDataHash::ZERO;

    Mock::given(method("PUT"))
        .and(path(format!(
            "/api/v1/app_data/{}",
            supplied.to_hex_string()
        )))
        .and(body_partial_json(json!({ "fullAppData": body })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!(returned.to_hex_string())))
        .mount(&server)
        .await;

    let api = build_orderbook_api_with_base_url(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        server.uri(),
    );

    let error = api
        .upload_app_data(&supplied, body)
        .await
        .expect_err("server-echo verification must reject the disagreeing hash");

    match error {
        OrderbookError::AppDataHashMismatch {
            expected,
            observed,
            stage,
        } => {
            assert_eq!(expected, supplied);
            assert_eq!(observed, returned);
            assert_eq!(stage, HashMismatchStage::ServerEcho);
        }
        other => panic!("expected server-echo mismatch, got {other:?}"),
    }
}

#[tokio::test]
async fn upload_app_data_accepts_status_200_for_already_existing_documents() {
    // The orderbook returns HTTP 200 with the same hash payload when the
    // document is already registered; HTTP 201 is the "newly stored" case.
    // The SDK must treat both as success when the server echoes the
    // caller-supplied hash.
    let server = MockServer::start().await;
    let body = SAMPLE_UPLOAD_BODY;
    let supplied = AppDataHash::from_full_app_data(body);

    Mock::given(method("PUT"))
        .and(path(format!(
            "/api/v1/app_data/{}",
            supplied.to_hex_string()
        )))
        .and(body_partial_json(json!({ "fullAppData": body })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(supplied.to_hex_string())))
        .mount(&server)
        .await;

    let api = build_orderbook_api_with_base_url(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        server.uri(),
    );

    api.upload_app_data(&supplied, body)
        .await
        .expect("already-existing upload (200) must succeed");
}

#[tokio::test]
async fn native_price_surplus_and_solver_competition_routes_are_covered() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!(
            "/api/v1/token/{}/native_price",
            sample_owner().to_hex_string()
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "price": 0.0004 })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!(
            "/api/v1/users/{}/total_surplus",
            sample_owner().to_hex_string()
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "totalSurplus": "100000000"
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v2/solver_competition/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "auctionId": 7,
            "auctionStartBlock": 100,
            "auctionDeadlineBlock": 110,
            "transactionHashes": [],
            "referenceScores": {},
            "auction": { "orders": [], "prices": {} },
            "solutions": []
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!(
            "/api/v2/solver_competition/by_tx_hash/{}",
            sample_tx_hash().to_hex_string()
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "auctionId": 8,
            "auctionStartBlock": 200,
            "auctionDeadlineBlock": 210,
            "transactionHashes": [],
            "referenceScores": {},
            "auction": { "orders": [], "prices": {} },
            "solutions": []
        })))
        .mount(&server)
        .await;

    let api = build_orderbook_api_with_base_url(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        server.uri(),
    );

    let native_price = api
        .native_price(&sample_owner())
        .await
        .expect("native price request should succeed");
    let surplus = api
        .total_surplus(&sample_owner())
        .await
        .expect("surplus request should succeed");
    let by_auction = api
        .solver_competition(7)
        .await
        .expect("competition by auction id should succeed");
    let by_tx = api
        .solver_competition_by_tx_hash(&sample_tx_hash())
        .await
        .expect("competition by tx hash should succeed");

    assert!((native_price.price - 0.0004).abs() < 1.0e-12);
    assert_eq!(
        surplus.total_surplus,
        Some(Amount::new("100000000").expect("test amount literal must be valid"))
    );
    assert_eq!(by_auction.auction_id, 7);
    assert_eq!(by_tx.auction_id, 8);
}

#[test]
fn solver_competition_response_decodes_typed_with_reference_scores_and_orders() {
    let uid = "0x59920c85de0162e9e55df8d396e75f3b6b7c2dfdb535f03e5c807731c31585eaff714b8b0e2700303ec912bd40496c3997ceea2b616d6710";
    let solver = "0x2222222222222222222222222222222222222222";
    let token = "0x0000000000000000000000000000000000000001";
    let tx = "0x3333333333333333333333333333333333333333333333333333333333333333";
    let body = json!({
        "auctionId": 13_036_993,
        "auctionStartBlock": 25_203_423,
        "auctionDeadlineBlock": 25_203_426,
        "transactionHashes": [tx],
        "referenceScores": { solver: "15047858248418147" },
        "auction": { "orders": [uid], "prices": { token: "1000000000000000000" } },
        "solutions": [{
            "solverAddress": solver,
            "score": "15047858248418147",
            "ranking": 1,
            "clearingPrices": { token: "8" },
            "orders": [{
                "id": uid,
                "sellAmount": "1000000000000",
                "buyAmount": "999764982430588460321926",
                "buyToken": token,
                "sellToken": solver
            }],
            "isWinner": true,
            "filteredOut": false,
            "referenceScore": "15047858248418147",
            "txHash": tx
        }]
    });

    let parsed: SolverCompetitionResponse =
        serde_json::from_value(body.clone()).expect("v2 solver-competition body must decode");

    assert_eq!(parsed.auction_id, 13_036_993);
    assert_eq!(
        parsed.reference_scores.len(),
        1,
        "reference scores must be captured rather than dropped"
    );
    assert_eq!(parsed.auction.orders.len(), 1);
    assert_eq!(parsed.auction.prices.len(), 1);
    let solution = parsed.solutions.first().expect("one solution expected");
    assert!(solution.is_winner);
    assert_eq!(solution.ranking, 1);
    let order = solution
        .orders
        .first()
        .expect("per-solution touched orders must be captured rather than dropped");
    assert!(
        order.buy_token.is_some() && order.sell_token.is_some(),
        "touched-order token addresses must be captured",
    );
    assert!(solution.reference_score.is_some());
    assert!(solution.tx_hash.is_some());

    // Unknown future fields must not break decoding (no deny_unknown_fields).
    let mut forward = body;
    forward["cipFutureField"] = json!(true);
    forward["solutions"][0]["newSolutionField"] = json!(123);
    serde_json::from_value::<SolverCompetitionResponse>(forward)
        .expect("unknown future fields must be tolerated");

    // Required scalars are non-optional: a missing one fails loudly.
    let incomplete = json!({ "auctionId": 1, "auctionStartBlock": 1, "auction": {} });
    assert!(
        serde_json::from_value::<SolverCompetitionResponse>(incomplete).is_err(),
        "a missing required field must error rather than silently default",
    );
}

#[tokio::test]
async fn get_order_status_route_is_typed() {
    let server = MockServer::start().await;
    let uid = sample_order_uid();
    Mock::given(method("GET"))
        .and(path(format!(
            "/api/v1/orders/{}/status",
            uid.to_hex_string()
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "type": "open",
            "value": null
        })))
        .mount(&server)
        .await;

    let api = build_orderbook_api_with_base_url(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        server.uri(),
    );
    let status = api
        .order_competition_status(&uid)
        .await
        .expect("status request should succeed");

    assert_eq!(
        status.kind,
        cow_sdk_orderbook::CompetitionOrderStatusKind::Open
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
    let first_api = build_orderbook_api_with_shared_client(
        shared.clone(),
        cow_sdk_core::ApiContext::new(SupportedChainId::Mainnet, CowEnv::Prod)
            .with_base_urls(first_base_urls),
    );

    let second_base_urls = std::collections::BTreeMap::from([(
        u64::from(SupportedChainId::GnosisChain),
        format!("{}/", second.uri()),
    )]);
    let second_api = build_orderbook_api_with_shared_client(
        shared,
        cow_sdk_core::ApiContext::new(SupportedChainId::GnosisChain, CowEnv::Prod)
            .with_base_urls(second_base_urls),
    );

    let first_version = first_api
        .version()
        .await
        .expect("first shared-client request must succeed");
    let second_version = second_api
        .version()
        .await
        .expect("second shared-client request must succeed");

    assert_eq!(first_version, "shared-client-first");
    assert_eq!(second_version, "shared-client-second");
}

mod recording_transport {
    use std::sync::Arc;

    use cow_sdk_core::transport::policy::TransportPolicy;
    use cow_sdk_core::{Amount, ApiContext, HttpTransport, SupportedChainId};
    use cow_sdk_orderbook::{
        CowEnv, OrderCancellations, OrderCreation, OrderQuoteSide, OrderbookApi, OrderbookError,
        OrderbookRejection, SigningScheme,
    };
    use cow_sdk_test_utils::mocks::{Canned, RecordingHttpTransport};

    use crate::common::{
        sample_buy_token, sample_order_uid, sample_owner, sample_quote_response_json,
        sample_signature,
    };
    use crate::retry_policy;

    fn api_with_recorder(recorder: Arc<RecordingHttpTransport>) -> OrderbookApi {
        api_with_recorder_and_policy(recorder, TransportPolicy::default())
    }

    fn api_with_recorder_and_policy(
        recorder: Arc<RecordingHttpTransport>,
        policy: TransportPolicy,
    ) -> OrderbookApi {
        let context = ApiContext::new(SupportedChainId::Mainnet, CowEnv::Prod);
        OrderbookApi::builder_from_context(context)
            .transport_policy(policy)
            .transport(recorder as Arc<dyn HttpTransport + Send + Sync>)
            .build()
            .expect("orderbook client with injected transport must build")
    }

    #[tokio::test]
    async fn orderbook_get_order_dispatches_through_injected_transport() {
        let uid = sample_order_uid();
        let order_json = crate::common::sample_order_json(&uid);
        let recorder = RecordingHttpTransport::new([Canned::Ok(order_json.to_string())]);
        let api = api_with_recorder(recorder.clone());

        let order = api
            .order(&uid)
            .await
            .expect("order lookup must succeed through the injected transport");

        assert_eq!(order.uid, uid);
        let calls = recorder.observed();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].method, "GET");
        assert!(calls[0].url.contains(uid.to_hex_string().as_str()));
        assert!(calls[0].body.is_empty());
        assert!(calls[0].has_timeout);
    }

    #[tokio::test]
    async fn orderbook_send_order_dispatches_through_injected_transport() {
        let recorder = RecordingHttpTransport::new([
            Canned::Ok(sample_quote_response_json().to_string()),
            Canned::Ok(format!("\"{}\"", sample_order_uid().to_hex_string())),
        ]);
        let api = api_with_recorder(recorder.clone());

        let quote = api
            .quote(&cow_sdk_orderbook::OrderQuoteRequest::new(
                sample_owner(),
                sample_buy_token(),
                sample_owner(),
                OrderQuoteSide::sell(
                    // The canned quote response echoes a fixed leg of
                    // sellAmount 1000 + feeAmount 10, so the request asks for the
                    // same before-fee total and `ensure_matches` reconciles it.
                    Amount::new("1010").expect("test amount literal must be valid"),
                ),
            ))
            .await
            .expect("quote request must succeed through the injected transport");

        let order = OrderCreation::from_quote(
            &quote,
            sample_owner(),
            None,
            SigningScheme::Eip712,
            sample_signature(),
        );
        let uid = api
            .send_order(&order)
            .await
            .expect("send_order must succeed through the injected transport");

        assert_eq!(uid.to_hex_string(), sample_order_uid().to_hex_string());
        let calls = recorder.observed();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].method, "POST");
        assert!(calls[0].url.contains("/api/v1/quote"));
        assert_eq!(calls[1].method, "POST");
        assert!(calls[1].url.contains("/api/v1/orders"));
        assert!(
            calls[1].body.contains("sellToken"),
            "the POST body must carry a serialized OrderCreation: {}",
            calls[1].body
        );
    }

    #[tokio::test]
    async fn orderbook_delete_cancellation_dispatches_through_injected_transport() {
        let recorder = RecordingHttpTransport::new([Canned::Ok(String::new())]);
        let api = api_with_recorder(recorder.clone());
        let cancellation =
            OrderCancellations::new(vec![sample_order_uid()], sample_signature().to_owned());

        api.send_cancellations(&cancellation)
            .await
            .expect("signed cancellation must succeed through the injected transport");

        let calls = recorder.observed();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].method, "DELETE");
        assert!(calls[0].url.contains("/api/v1/orders"));
        assert!(
            calls[0].body.contains("signature"),
            "DELETE body must carry a signed cancellation envelope: {}",
            calls[0].body
        );
    }

    #[tokio::test]
    async fn orderbook_rate_limit_and_backoff_still_apply_through_injected_transport() {
        let recorder = RecordingHttpTransport::new([
            Canned::HttpStatus {
                status: 503,
                headers: Vec::new(),
                body: "{\"errorType\":\"InternalServerError\",\"description\":\"try again\"}"
                    .to_owned(),
            },
            Canned::HttpStatus {
                status: 503,
                headers: Vec::new(),
                body: "{\"errorType\":\"InternalServerError\",\"description\":\"try again\"}"
                    .to_owned(),
            },
            Canned::Ok("v1.2.3".to_owned()),
        ]);
        let policy = TransportPolicy::default().with_retry(retry_policy(5));
        let api = api_with_recorder_and_policy(recorder.clone(), policy);

        let version = api
            .version()
            .await
            .expect("the third attempt must succeed after the retry loop");
        assert_eq!(version, "v1.2.3");

        let calls = recorder.observed();
        assert_eq!(
            calls.len(),
            3,
            "the backoff wrapper must retry transient 503 responses through the injected transport"
        );
    }

    #[tokio::test]
    async fn orderbook_non_2xx_surfaces_as_http_status_error_through_injected_transport() {
        let recorder = RecordingHttpTransport::new([Canned::HttpStatus {
            status: 400,
            headers: Vec::new(),
            body: "{\"errorType\":\"DuplicatedOrder\",\"description\":\"order already exists\"}"
                .to_owned(),
        }]);
        let policy = TransportPolicy::default().with_retry(retry_policy(1));
        let api = api_with_recorder_and_policy(recorder.clone(), policy);

        let error = api
            .version()
            .await
            .expect_err("non-2xx response must surface through the typed error channel");
        match error {
            OrderbookError::Rejected {
                status, rejection, ..
            } => {
                assert_eq!(status.as_u16(), 400);
                assert_eq!(rejection, OrderbookRejection::DuplicatedOrder);
            }
            OrderbookError::Api(envelope) => {
                assert_eq!(envelope.status, 400);
            }
            other => panic!("expected Rejected or Api error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn orderbook_non_2xx_in_the_ok_channel_is_normalized_onto_the_error_path() {
        // A misbehaving custom transport that returns a non-2xx status through
        // the `Ok` success channel instead of `TransportError::HttpStatus`. The
        // orderbook normalizes it onto the same typed error path, so a wrong
        // status delivered on the success channel can never be mistaken for a
        // 2xx response and retry classification stays uniform.
        let recorder = RecordingHttpTransport::new([Canned::Success {
            status: 400,
            headers: vec![("content-type".to_owned(), "application/json".to_owned())],
            body: "{\"errorType\":\"DuplicatedOrder\",\"description\":\"order already exists\"}"
                .to_owned(),
        }]);
        let policy = TransportPolicy::default().with_retry(retry_policy(1));
        let api = api_with_recorder_and_policy(recorder.clone(), policy);

        let error = api
            .version()
            .await
            .expect_err("a non-2xx status in the Ok channel must surface as a typed error");
        match error {
            OrderbookError::Rejected {
                status, rejection, ..
            } => {
                assert_eq!(status.as_u16(), 400);
                assert_eq!(rejection, OrderbookRejection::DuplicatedOrder);
            }
            OrderbookError::Api(envelope) => {
                assert_eq!(envelope.status, 400);
            }
            other => panic!("expected Rejected or Api error, got {other:?}"),
        }
    }
}
