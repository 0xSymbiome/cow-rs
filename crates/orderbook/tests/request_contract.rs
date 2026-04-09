use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use cow_sdk_orderbook::request::{
    OrderBookApiError, RequestPolicy, RequestRateLimiter, ResponseBody, ResponseEnvelope,
    execute_empty_with, execute_json_with,
};
use cow_sdk_orderbook::{
    DEFAULT_INTERVAL_LABEL, DEFAULT_MAX_ATTEMPTS, DEFAULT_TOKENS_PER_INTERVAL,
    INTERNAL_SERVER_ERROR, RETRYABLE_STATUS_CODES, TOO_MANY_REQUESTS,
};
use serde_json::json;

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
