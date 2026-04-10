use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use std::time::{Duration, Instant};

use cow_sdk_orderbook::request::{
    FetchParams, HttpMethod, OrderBookApiError, RateLimitSettings, RequestPolicy,
    RequestRateLimiter, ResponseBody, ResponseEnvelope, execute_empty_with, execute_json_with,
    request_empty, request_json, request_text,
};
use cow_sdk_orderbook::{
    DEFAULT_INTERVAL_LABEL, DEFAULT_MAX_ATTEMPTS, DEFAULT_TOKENS_PER_INTERVAL,
    INTERNAL_SERVER_ERROR, RETRYABLE_STATUS_CODES, TOO_MANY_REQUESTS,
};
use reqwest::{
    Client,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use serde_json::json;
use wiremock::{
    Mock, MockServer, Request, ResponseTemplate,
    matchers::{header, method, path},
};

#[tokio::test]
async fn request_policy_defaults_match_fixture_contract() {
    let policy = RequestPolicy::default();

    assert_eq!(policy.max_attempts, DEFAULT_MAX_ATTEMPTS);
    assert_eq!(
        policy.rate_limit.tokens_per_interval,
        DEFAULT_TOKENS_PER_INTERVAL
    );
    assert_eq!(policy.rate_limit.interval_label, DEFAULT_INTERVAL_LABEL);
    assert_eq!(RETRYABLE_STATUS_CODES, [408, 425, 429, 500, 502, 503, 504]);
    assert!(policy.should_retry_status(TOO_MANY_REQUESTS));
    assert!(!policy.should_retry_status(400));
}

#[tokio::test]
async fn execute_json_with_retries_transient_statuses_until_success() {
    let policy = RequestPolicy::default();
    let limiter = RequestRateLimiter::new(policy.rate_limit);
    let attempts = Arc::new(AtomicUsize::new(0));

    let result: serde_json::Value = execute_json_with(&policy, &limiter, {
        let attempts = attempts.clone();
        move || {
            let attempts = attempts.clone();
            async move {
                let current = attempts.fetch_add(1, Ordering::SeqCst);
                if current < 2 {
                    Ok(ResponseEnvelope::json(
                        INTERNAL_SERVER_ERROR,
                        json!({
                            "errorType": "InternalServerError",
                            "description": "retry me"
                        }),
                    ))
                } else {
                    Ok(ResponseEnvelope::json(200, json!({ "ok": true })))
                }
            }
        }
    })
    .await
    .expect("third attempt should succeed");

    assert_eq!(attempts.load(Ordering::SeqCst), 3);
    assert_eq!(result["ok"], json!(true));
}

#[tokio::test]
async fn request_json_retries_429_and_preserves_headers_on_each_attempt() {
    let server = MockServer::start().await;
    let attempts = Arc::new(AtomicUsize::new(0));

    Mock::given(method("GET"))
        .and(path("/api/v1/retry"))
        .and(header("x-api-key", "secret"))
        .respond_with({
            let attempts = attempts.clone();
            move |_request: &Request| {
                let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                if attempt == 0 {
                    ResponseTemplate::new(TOO_MANY_REQUESTS).set_body_json(json!({
                        "errorType": "RateLimited",
                        "description": "retry after rate limit"
                    }))
                } else {
                    ResponseTemplate::new(200).set_body_json(json!({ "ok": true }))
                }
            }
        })
        .expect(2)
        .mount(&server)
        .await;

    let policy = RequestPolicy {
        max_attempts: 3,
        ..RequestPolicy::default()
    };
    let limiter = RequestRateLimiter::new(policy.rate_limit);
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("x-api-key"),
        HeaderValue::from_static("secret"),
    );

    let result: serde_json::Value = request_json(
        &Client::new(),
        &server.uri(),
        &FetchParams::new("/api/v1/retry", HttpMethod::Get),
        &policy,
        &limiter,
        Some(headers),
    )
    .await
    .expect("second request should succeed after retry");

    assert_eq!(result["ok"], json!(true));
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn execute_json_with_stops_on_non_retryable_api_error_and_preserves_body() {
    let policy = RequestPolicy::default();
    let limiter = RequestRateLimiter::new(policy.rate_limit);
    let attempts = Arc::new(AtomicUsize::new(0));

    let error = execute_json_with::<serde_json::Value, _, _>(&policy, &limiter, {
        let attempts = attempts.clone();
        move || {
            let attempts = attempts.clone();
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Ok(ResponseEnvelope::json(
                    400,
                    json!({
                        "errorType": "DuplicateOrder",
                        "description": "order already exists"
                    }),
                ))
            }
        }
    })
    .await
    .expect_err("400 duplicate order should not be retried");

    match error {
        cow_sdk_orderbook::OrderbookError::Api(api_error) => {
            assert_eq!(api_error.status, 400);
            assert_eq!(api_error.error_type(), Some("DuplicateOrder"));
            assert_eq!(attempts.load(Ordering::SeqCst), 1);
        }
        other => panic!("expected API error, got {other:?}"),
    }
}

#[tokio::test]
async fn execute_empty_with_allows_204_without_body() {
    let policy = RequestPolicy::default();
    let limiter = RequestRateLimiter::new(policy.rate_limit);

    execute_empty_with(&policy, &limiter, || async {
        Ok(ResponseEnvelope::empty(204))
    })
    .await
    .expect("204 response should be accepted");
}

#[tokio::test]
async fn request_text_and_empty_share_the_request_builder_and_success_path() {
    let server = MockServer::start().await;
    let observed_accepts = Arc::new(Mutex::new(Vec::new()));

    Mock::given(method("GET"))
        .and(path("/api/v1/version"))
        .respond_with({
            let observed_accepts = observed_accepts.clone();
            move |request: &Request| {
                if let Some(value) = request
                    .headers
                    .get("accept")
                    .and_then(|value| value.to_str().ok())
                {
                    observed_accepts
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .push(value.to_owned());
                }
                ResponseTemplate::new(200).set_body_string("v1.2.3")
            }
        })
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/orders"))
        .respond_with({
            let observed_accepts = observed_accepts.clone();
            move |request: &Request| {
                if let Some(value) = request
                    .headers
                    .get("accept")
                    .and_then(|value| value.to_str().ok())
                {
                    observed_accepts
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .push(value.to_owned());
                }
                ResponseTemplate::new(204)
            }
        })
        .expect(1)
        .mount(&server)
        .await;

    let policy = RequestPolicy::default();
    let limiter = RequestRateLimiter::new(policy.rate_limit);

    let version = request_text(
        &Client::new(),
        &server.uri(),
        &FetchParams::new("/api/v1/version", HttpMethod::Get),
        &policy,
        &limiter,
        None,
    )
    .await
    .expect("text response should decode");
    request_empty(
        &Client::new(),
        &server.uri(),
        &FetchParams::new("/api/v1/orders", HttpMethod::Delete),
        &policy,
        &limiter,
        None,
    )
    .await
    .expect("empty response should decode");

    assert_eq!(version, "v1.2.3");
    let observed = observed_accepts
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    assert!(observed.contains(&"text/plain, application/json".to_owned()));
    assert!(observed.contains(&"application/json".to_owned()));
}

#[tokio::test]
async fn rate_limiter_spaces_requests_after_token_budget_is_consumed() {
    let interval = Duration::from_millis(40);
    let policy = RequestPolicy {
        max_attempts: 1,
        rate_limit: RateLimitSettings {
            tokens_per_interval: 1,
            interval,
            interval_label: "test",
        },
    };
    let limiter = RequestRateLimiter::new(policy.rate_limit);

    execute_empty_with(&policy, &limiter, || async {
        Ok(ResponseEnvelope::empty(204))
    })
    .await
    .expect("first token should be available immediately");

    let started = Instant::now();
    execute_empty_with(&policy, &limiter, || async {
        Ok(ResponseEnvelope::empty(204))
    })
    .await
    .expect("second request should wait for the next token window");

    assert!(
        started.elapsed() >= Duration::from_millis(20),
        "second request should be delayed by the shared limiter"
    );
}

#[test]
fn typed_api_error_exposes_json_body_and_error_type() {
    let error = OrderBookApiError::new(
        400,
        "Bad Request",
        ResponseBody::Json(json!({
            "errorType": "DuplicateOrder",
            "description": "duplicate order"
        })),
    );

    assert_eq!(error.status, 400);
    assert_eq!(error.error_type(), Some("DuplicateOrder"));
}
