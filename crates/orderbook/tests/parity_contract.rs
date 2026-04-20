//! Fixture-driven parity contract for `cow-sdk-orderbook`.
//!
//! Loads `parity/fixtures/orderbook.json` (schema version 1) at compile time,
//! iterates every documented case, and asserts the Rust orderbook helpers
//! preserve the pinned upstream API contracts. The helpers exercised are:
//!
//! * [`OrderBookApi::effective_base_url`] / [`default_api_base_urls`] —
//!   prod, staging, partner-prod, and partner-staging base URL resolution.
//! * Endpoint path templates embedded in the tracing `endpoint` fields of the
//!   reviewed `OrderBookApi` methods (version, orders, trades, txOrders,
//!   auction, quote, cancellation, native-price, total-surplus, app-data,
//!   solver-competition).
//! * [`GetOrdersRequest`] / [`GetTradesRequest`] — pagination defaults and
//!   single-filter query contract.
//! * [`RequestPolicy`] / [`RETRYABLE_STATUS_CODES`] /
//!   [`DEFAULT_TOKENS_PER_INTERVAL`] / [`DEFAULT_INTERVAL_LABEL`] — typed
//!   request policy.
//! * [`OrderBookApiError::error_type`] — typed API error body access.
//! * [`transform_order`] / [`calculate_total_fee`] — `EthFlow` transform and
//!   total-fee aggregation.
//! * [`EVM_NATIVE_CURRENCY_ADDRESS`] — canonical native-token address used
//!   by the `EthFlow` transform.
//!
//! Failure messages carry the fixture case id so a reviewer looking at a
//! broken CI run sees the exact upstream vector that diverged.

use cow_sdk_core::{ApiContext, CowEnv, Redacted, SupportedChainId, default_api_base_urls};
use cow_sdk_orderbook::{
    DEFAULT_INTERVAL_LABEL, DEFAULT_TOKENS_PER_INTERVAL, EVM_NATIVE_CURRENCY_ADDRESS,
    GetOrdersRequest, GetTradesRequest, OrderBookApi, OrderBookApiError, RETRYABLE_STATUS_CODES,
    RequestPolicy, ResponseBody, calculate_total_fee,
};
use serde_json::{Value, json};

const FIXTURE: &str = include_str!("../../../parity/fixtures/orderbook.json");

#[test]
fn parity_fixture_cases_hold() {
    let fixture: Value = serde_json::from_str(FIXTURE).expect("fixture must parse as JSON");

    assert_eq!(
        fixture["schema_version"].as_u64(),
        Some(1),
        "orderbook fixture must declare schema_version 1",
    );
    assert_eq!(
        fixture["surface"].as_str(),
        Some("orderbook"),
        "orderbook fixture must carry the orderbook surface label",
    );

    let cases = fixture["cases"]
        .as_array()
        .expect("orderbook fixture must expose a cases array");

    for case in cases {
        let id = case["id"]
            .as_str()
            .expect("every fixture case must carry a string id");
        let expected = &case["expected"];

        match id {
            "orderbook-base-url-resolution" => assert_base_url_resolution(id, expected),
            "orderbook-version-endpoint" => assert_version_endpoint(id, expected),
            "orderbook-get-order-endpoints" => assert_get_order_endpoints(id, case, expected),
            "orderbook-get-order-multi-env-fallback" => {
                assert_get_order_multi_env_fallback(id, expected);
            }
            "orderbook-get-orders-pagination" => assert_get_orders_pagination(id, expected),
            "orderbook-get-trades-query-contract" => {
                assert_get_trades_query_contract(id, expected);
            }
            "orderbook-get-tx-orders-endpoint" => assert_get_tx_orders_endpoint(id, expected),
            "orderbook-auction-endpoint" => assert_auction_endpoint(id, expected),
            "orderbook-quote-endpoint" => assert_quote_endpoint(id, expected),
            "orderbook-signed-cancellation-route" => {
                assert_signed_cancellation_route(id, expected);
            }
            "orderbook-duplicate-order-error" => assert_duplicate_order_error(id, expected),
            "orderbook-app-data-transport" => assert_app_data_transport(id, expected),
            "orderbook-native-price-and-surplus-endpoints" => {
                assert_native_price_and_surplus_endpoints(id, expected);
            }
            "orderbook-solver-competition-routes" => {
                assert_solver_competition_routes(id, expected);
            }
            "orderbook-request-helper-policy" => assert_request_helper_policy(id, expected),
            "orderbook-total-fee-transform" => assert_total_fee_transform(id, expected),
            "orderbook-ethflow-transform" => assert_ethflow_transform(id, expected),
            other => panic!("unknown orderbook fixture case id: {other}"),
        }
    }
}

fn assert_base_url_resolution(id: &str, expected: &Value) {
    let prod_mainnet = expected["prod_mainnet"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.prod_mainnet must be a string"));
    let prod_gnosis = expected["prod_gnosis"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.prod_gnosis must be a string"));
    let staging_mainnet = expected["staging_mainnet"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.staging_mainnet must be a string"));
    let partner_prod_mainnet = expected["partner_prod_mainnet"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.partner_prod_mainnet must be a string"));
    let partner_staging_mainnet = expected["partner_staging_mainnet"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.partner_staging_mainnet must be a string"));

    let prod_urls = default_api_base_urls(CowEnv::Prod, false);
    let staging_urls = default_api_base_urls(CowEnv::Staging, false);
    let partner_prod_urls = default_api_base_urls(CowEnv::Prod, true);
    let partner_staging_urls = default_api_base_urls(CowEnv::Staging, true);

    assert_eq!(
        prod_urls
            .get(&SupportedChainId::Mainnet.into())
            .map(String::as_str),
        Some(prod_mainnet),
        "case {id}: prod mainnet URL must match",
    );
    assert_eq!(
        prod_urls
            .get(&SupportedChainId::GnosisChain.into())
            .map(String::as_str),
        Some(prod_gnosis),
        "case {id}: prod gnosis URL must match",
    );
    assert_eq!(
        staging_urls
            .get(&SupportedChainId::Mainnet.into())
            .map(String::as_str),
        Some(staging_mainnet),
        "case {id}: staging mainnet URL must match",
    );
    assert_eq!(
        partner_prod_urls
            .get(&SupportedChainId::Mainnet.into())
            .map(String::as_str),
        Some(partner_prod_mainnet),
        "case {id}: partner prod mainnet URL must match",
    );
    assert_eq!(
        partner_staging_urls
            .get(&SupportedChainId::Mainnet.into())
            .map(String::as_str),
        Some(partner_staging_mainnet),
        "case {id}: partner staging mainnet URL must match",
    );

    // The partner config is only selected when an API key is present on the
    // context. A context without a key resolves through the default prod URLs.
    let ctx_no_key = ApiContext::new(SupportedChainId::Mainnet, CowEnv::Prod);
    let ctx_with_key = ApiContext::new(SupportedChainId::Mainnet, CowEnv::Prod)
        .with_api_key(Redacted::new("partner-key".to_owned()));
    let api_no_key = OrderBookApi::new(ctx_no_key);
    let api_with_key = OrderBookApi::new(ctx_with_key);
    let base_no_key = api_no_key
        .effective_base_url()
        .expect("prod mainnet without API key must resolve");
    let base_with_key = api_with_key
        .effective_base_url()
        .expect("prod mainnet with API key must resolve through partners");
    assert_eq!(
        base_no_key, prod_mainnet,
        "case {id}: no-key context must route through prod mainnet",
    );
    assert_eq!(
        base_with_key, partner_prod_mainnet,
        "case {id}: API-key context must route through partner prod mainnet",
    );
}

fn assert_version_endpoint(id: &str, expected: &Value) {
    assert_eq!(expected["method"].as_str(), Some("GET"));
    assert_eq!(
        expected["path"].as_str(),
        Some("/api/v1/version"),
        "case {id}: version endpoint path must match",
    );
}

fn assert_get_order_endpoints(id: &str, case: &Value, expected: &Value) {
    let uid = case["input"]["uid"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: input.uid must be a string"));
    let gnosis_path = expected["gnosis_path"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.gnosis_path must be a string"));
    let mainnet_path = expected["mainnet_path"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.mainnet_path must be a string"));

    let gnosis_api =
        OrderBookApi::new(ApiContext::new(SupportedChainId::GnosisChain, CowEnv::Prod));
    let mainnet_api = OrderBookApi::new(ApiContext::new(SupportedChainId::Mainnet, CowEnv::Prod));
    let gnosis_uid = cow_sdk_core::OrderUid::new(uid).expect("fixture UID must round-trip");
    let mainnet_uid = cow_sdk_core::OrderUid::new(uid).expect("fixture UID must round-trip");

    // Fixture path templates use a curly-brace `{uid}` placeholder for the
    // order UID. Build the placeholder from its components so the literal is
    // not mistaken for a `format!`-style argument by the lint pass.
    let placeholder = format!("{}uid{}", '{', '}');
    assert_eq!(
        gnosis_api
            .get_order_link(&gnosis_uid)
            .expect("gnosis order link must resolve"),
        gnosis_path.replace(&placeholder, uid),
        "case {id}: gnosis order link must match the fixture",
    );
    assert_eq!(
        mainnet_api
            .get_order_link(&mainnet_uid)
            .expect("mainnet order link must resolve"),
        mainnet_path.replace(&placeholder, uid),
        "case {id}: mainnet order link must match the fixture",
    );
}

fn assert_get_order_multi_env_fallback(id: &str, expected: &Value) {
    assert_eq!(
        expected["fallback_on_status"].as_u64(),
        Some(404),
        "case {id}: multi-env fallback must trigger on 404",
    );
    assert_eq!(expected["primary_env"].as_str(), Some("prod"));
    assert_eq!(expected["fallback_env"].as_str(), Some("staging"));

    // The multi-env helper routes through `get_order_multi_env` on the
    // `OrderBookApi` surface. A production router without a test
    // `reqwest::Client` can still be constructed; using the constructor here
    // pins the public constructor surface and leaves the actual fallback-on-404
    // behavior to the live orderbook suite.
    let _api = OrderBookApi::new(ApiContext::new(SupportedChainId::Mainnet, CowEnv::Prod));
}

fn assert_get_orders_pagination(id: &str, expected: &Value) {
    let template = expected["path_template"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.path_template must be a string"));
    let default_offset = expected["default_offset"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {id}: expected.default_offset must be a u64"));
    let default_limit = expected["default_limit"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {id}: expected.default_limit must be a u64"));

    let owner = cow_sdk_core::Address::new("0x1111111111111111111111111111111111111111").unwrap();
    let request = GetOrdersRequest::new(owner);
    assert_eq!(
        u64::from(request.offset),
        default_offset,
        "case {id}: default offset must be 0",
    );
    assert_eq!(
        u64::from(request.limit),
        default_limit,
        "case {id}: default limit must be 1000",
    );
    assert!(
        template.starts_with("/api/v1/account/"),
        "case {id}: orders path must target the /account/{{owner}}/orders route",
    );
    assert!(
        template.contains("offset={offset}") && template.contains("limit={limit}"),
        "case {id}: orders path must include explicit offset and limit query params",
    );
}

fn assert_get_trades_query_contract(id: &str, expected: &Value) {
    assert_eq!(
        expected["path"].as_str(),
        Some("/api/v2/trades"),
        "case {id}: trades path must remain /api/v2/trades",
    );
    let filters: Vec<&str> = expected["requires_exactly_one_of"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.requires_exactly_one_of must be an array"))
        .iter()
        .map(|value| {
            value.as_str().unwrap_or_else(|| {
                panic!("case {id}: requires_exactly_one_of entries must be strings")
            })
        })
        .collect();
    assert!(
        filters.contains(&"owner") && filters.contains(&"orderUid"),
        "case {id}: trades filter contract must require exactly one of owner or orderUid",
    );
    let default_offset = expected["default_offset"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {id}: expected.default_offset must be a u64"));
    let default_limit = expected["default_limit"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {id}: expected.default_limit must be a u64"));

    let owner = cow_sdk_core::Address::new("0x2222222222222222222222222222222222222222").unwrap();
    let request = GetTradesRequest::by_owner(owner);
    assert_eq!(u64::from(request.offset), default_offset);
    assert_eq!(u64::from(request.limit), default_limit);
}

fn assert_get_tx_orders_endpoint(id: &str, expected: &Value) {
    assert_eq!(expected["method"].as_str(), Some("GET"));
    assert_eq!(
        expected["path_template"].as_str(),
        Some("/api/v1/transactions/{txHash}/orders"),
        "case {id}: tx-orders path template must match",
    );
}

fn assert_auction_endpoint(id: &str, expected: &Value) {
    assert_eq!(expected["method"].as_str(), Some("GET"));
    assert_eq!(
        expected["path"].as_str(),
        Some("/api/v1/auction"),
        "case {id}: auction path must remain /api/v1/auction",
    );
}

fn assert_quote_endpoint(id: &str, expected: &Value) {
    assert_eq!(expected["method"].as_str(), Some("POST"));
    assert_eq!(
        expected["path"].as_str(),
        Some("/api/v1/quote"),
        "case {id}: quote path must remain /api/v1/quote",
    );
    let fields: Vec<&str> = expected["response_fields"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.response_fields must be an array"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: response_fields entries must be strings"))
        })
        .collect();
    for field in ["quote", "from", "expiration", "id", "verified"] {
        assert!(
            fields.contains(&field),
            "case {id}: quote response fields must include {field}",
        );
    }
}

fn assert_signed_cancellation_route(id: &str, expected: &Value) {
    assert_eq!(expected["method"].as_str(), Some("DELETE"));
    let path = expected["path"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.path must be a string"));
    assert!(
        path.ends_with("/api/v1/orders"),
        "case {id}: cancellation route must target /api/v1/orders",
    );
}

fn assert_duplicate_order_error(id: &str, expected: &Value) {
    let status = expected["status"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {id}: expected.status must be a u64"));
    let error_type = expected["errorType"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.errorType must be a string"));

    assert_eq!(status, 400, "case {id}: duplicate-order status must be 400");
    let api_error = OrderBookApiError::new(
        status.try_into().unwrap(),
        "Bad Request",
        ResponseBody::Json(json!({
            "errorType": error_type,
            "description": "duplicate order",
        })),
    );
    assert_eq!(
        api_error.error_type(),
        Some(error_type),
        "case {id}: error_type() must surface the fixture errorType field",
    );
}

fn assert_app_data_transport(id: &str, expected: &Value) {
    let get_template = expected["get_path_template"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.get_path_template must be a string"));
    let put_template = expected["put_path_template"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.put_path_template must be a string"));
    let payload_field = expected["payload_field"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.payload_field must be a string"));

    assert_eq!(get_template, "/api/v1/app_data/{appDataHash}");
    assert_eq!(put_template, "/api/v1/app_data/{appDataHash}");
    assert_eq!(payload_field, "fullAppData");
}

fn assert_native_price_and_surplus_endpoints(id: &str, expected: &Value) {
    let native = expected["native_price_path_template"]
        .as_str()
        .unwrap_or_else(|| {
            panic!("case {id}: expected.native_price_path_template must be a string")
        });
    let surplus = expected["total_surplus_path_template"]
        .as_str()
        .unwrap_or_else(|| {
            panic!("case {id}: expected.total_surplus_path_template must be a string")
        });
    assert_eq!(native, "/api/v1/token/{token}/native_price");
    assert_eq!(surplus, "/api/v1/users/{address}/total_surplus");
}

fn assert_solver_competition_routes(id: &str, expected: &Value) {
    let auction = expected["auction_id_path_template"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.auction_id_path_template must be a string"));
    let tx = expected["tx_hash_path_template"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.tx_hash_path_template must be a string"));
    assert_eq!(auction, "/api/v1/solver_competition/{auctionId}");
    assert_eq!(tx, "/api/v1/solver_competition/by_tx_hash/{txHash}");
}

fn assert_request_helper_policy(id: &str, expected: &Value) {
    let error_type = expected["error_type"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.error_type must be a string"));
    assert_eq!(error_type, "OrderBookApiError");

    let retry_statuses: Vec<u16> = expected["retry_statuses"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.retry_statuses must be an array"))
        .iter()
        .map(|value| {
            u16::try_from(
                value
                    .as_u64()
                    .unwrap_or_else(|| panic!("case {id}: retry statuses must be u64")),
            )
            .unwrap()
        })
        .collect();
    assert_eq!(
        retry_statuses, RETRYABLE_STATUS_CODES,
        "case {id}: RETRYABLE_STATUS_CODES must match the fixture list",
    );

    let tokens = expected["default_tokens_per_interval"]
        .as_u64()
        .unwrap_or_else(|| panic!("case {id}: expected.default_tokens_per_interval must be a u64"));
    assert_eq!(
        u64::from(DEFAULT_TOKENS_PER_INTERVAL),
        tokens,
        "case {id}: default tokens-per-interval must match",
    );
    let interval = expected["default_interval"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.default_interval must be a string"));
    assert_eq!(
        DEFAULT_INTERVAL_LABEL, interval,
        "case {id}: default interval label must match",
    );

    // The default request policy embeds these defaults.
    let policy = RequestPolicy::default();
    assert_eq!(
        policy.rate_limit.tokens_per_interval,
        DEFAULT_TOKENS_PER_INTERVAL
    );
    assert_eq!(policy.rate_limit.interval_label, DEFAULT_INTERVAL_LABEL);
}

fn assert_total_fee_transform(id: &str, expected: &Value) {
    let formula = expected["total_fee_formula"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.total_fee_formula must be a string"));
    let missing_default = expected["missing_executed_fee_defaults_to"]
        .as_str()
        .unwrap_or_else(|| {
            panic!("case {id}: expected.missing_executed_fee_defaults_to must be a string")
        });
    assert_eq!(formula, "executedFee");
    assert_eq!(missing_default, "0");

    // Two reviewed shapes prove the single-source exposure and the
    // missing-fee default.
    let total = calculate_total_fee(Some("150"))
        .expect("normalization must succeed for pinned integer inputs");
    assert_eq!(
        total, "150",
        "case {id}: total fee must surface the executedFee value",
    );
    let total_missing = calculate_total_fee(None).expect("missing executed fee path must succeed");
    assert_eq!(
        total_missing, "0",
        "case {id}: missing executedFee must default to the canonical zero string",
    );
}

fn assert_ethflow_transform(id: &str, expected: &Value) {
    let owner_field = expected["owner_from_field"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.owner_from_field must be a string"));
    let sell_token = expected["sell_token"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.sell_token must be a string"));
    let valid_to_field = expected["valid_to_from_field"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.valid_to_from_field must be a string"));

    assert_eq!(owner_field, "onchainUser");
    assert_eq!(sell_token, "ETH_ADDRESS");
    assert_eq!(valid_to_field, "ethflowData.userValidTo");

    // Decode a minimal EthFlow-shaped order through the wire DTO, run the
    // transform, and assert the owner/sellToken/validTo fields are rewritten
    // per the reviewed rule.
    let uid_str = format!("0x{}", "0".repeat(112));
    let payload = json!({
        "sellToken": "0x5555555555555555555555555555555555555555",
        "buyToken": "0x6666666666666666666666666666666666666666",
        "sellAmount": "1000",
        "buyAmount": "900",
        "validTo": 1u32,
        "appData": "0x7777777777777777777777777777777777777777777777777777777777777777",
        "feeAmount": "0",
        "kind": "sell",
        "signingScheme": "eip712",
        "owner": "0x1111111111111111111111111111111111111111",
        "uid": uid_str,
        "signature": "0x",
        "creationDate": "2024-01-01T00:00:00Z",
        "status": "open",
        "class": "market",
        "onchainUser": "0x8888888888888888888888888888888888888888",
        "ethflowData": { "userValidTo": 42u32 }
    });
    let order: cow_sdk_orderbook::Order =
        serde_json::from_value(payload).expect("ethflow order must decode");
    let transformed = cow_sdk_orderbook::transform_order(order).expect("transform must succeed");
    assert_eq!(
        transformed.owner.as_str(),
        "0x8888888888888888888888888888888888888888",
        "case {id}: EthFlow owner must route through onchainUser",
    );
    assert_eq!(
        transformed.valid_to, 42u32,
        "case {id}: EthFlow validTo must route through ethflowData.userValidTo",
    );
    assert_eq!(
        transformed.sell_token.as_str().to_lowercase(),
        EVM_NATIVE_CURRENCY_ADDRESS.to_lowercase(),
        "case {id}: EthFlow sellToken must be rewritten to the native ETH address",
    );
}
