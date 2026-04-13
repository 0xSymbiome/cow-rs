use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use std::time::{Duration, Instant};

use cow_sdk_core::HttpClientPolicy;
use cow_sdk_orderbook::request::{
    DEFAULT_ORDERBOOK_USER_AGENT, FetchParams, HttpMethod, OrderBookApiError,
    OrderBookTransportPolicy, RateLimitSettings, RequestPolicy, RequestRateLimiter, ResponseBody,
    ResponseEnvelope, execute_empty_with, execute_json_with, request_empty, request_json,
    request_text,
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
use tokio::time::{sleep, timeout};
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

#[test]
fn request_policy_backoff_is_exponential_and_caps_growth() {
    let policy = RequestPolicy::default();

    assert_eq!(policy.backoff_delay(1), Duration::from_millis(50));
    assert_eq!(policy.backoff_delay(2), Duration::from_millis(100));
    assert_eq!(policy.backoff_delay(3), Duration::from_millis(200));
    assert_eq!(policy.backoff_delay(7), Duration::from_millis(3200));
    assert_eq!(policy.backoff_delay(8), Duration::from_millis(3200));
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
                        &json!({
                            "errorType": "InternalServerError",
                            "description": "retry me"
                        }),
                    ))
                } else {
                    Ok(ResponseEnvelope::json(200, &json!({ "ok": true })))
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
                    &json!({
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
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
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
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
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
        .unwrap_or_else(std::sync::PoisonError::into_inner)
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

#[tokio::test]
async fn concurrent_attempts_share_limiter_state_across_clones() {
    let interval = Duration::from_millis(60);
    let policy = RequestPolicy {
        max_attempts: 1,
        rate_limit: RateLimitSettings {
            tokens_per_interval: 1,
            interval,
            interval_label: "test",
        },
    };
    let limiter = RequestRateLimiter::new(policy.rate_limit);
    let arrivals = Arc::new(Mutex::new(Vec::new()));

    let spawn_attempt =
        |policy: RequestPolicy, limiter: RequestRateLimiter, arrivals: Arc<Mutex<Vec<Instant>>>| {
            tokio::spawn(async move {
                execute_empty_with(&policy, &limiter, || {
                    let arrivals = arrivals.clone();
                    async move {
                        arrivals
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .push(Instant::now());
                        Ok(ResponseEnvelope::empty(204))
                    }
                })
                .await
            })
        };

    let first = spawn_attempt(policy.clone(), limiter.clone(), arrivals.clone());
    let second = spawn_attempt(policy.clone(), limiter.clone(), arrivals.clone());

    first
        .await
        .expect("first task should join cleanly")
        .expect("first attempt should succeed");
    second
        .await
        .expect("second task should join cleanly")
        .expect("second attempt should succeed");

    let arrivals = arrivals
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert_eq!(arrivals.len(), 2);
    assert!(
        arrivals[1].duration_since(arrivals[0]) >= interval / 2,
        "shared limiter should delay concurrent attempts behind the consumed token"
    );
}

#[tokio::test]
async fn cancelling_waiting_attempt_keeps_limiter_reusable() {
    let interval = Duration::from_millis(60);
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

    let waiting_policy = policy.clone();
    let waiting_limiter = limiter.clone();
    let waiting = tokio::spawn(async move {
        execute_empty_with(&waiting_policy, &waiting_limiter, || async {
            Ok(ResponseEnvelope::empty(204))
        })
        .await
    });

    sleep(Duration::from_millis(5)).await;
    waiting.abort();

    let aborted = waiting
        .await
        .expect_err("aborted waiter should surface cancellation");
    assert!(aborted.is_cancelled());

    sleep(interval + Duration::from_millis(5)).await;

    timeout(
        Duration::from_millis(200),
        execute_empty_with(&policy, &limiter, || async {
            Ok(ResponseEnvelope::empty(204))
        }),
    )
    .await
    .expect("reused limiter should not hang after waiter cancellation")
    .expect("reused limiter should grant the next token");
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

#[test]
fn transport_policy_wraps_validated_shared_client_policy() {
    let custom = HttpClientPolicy::new("custom-orderbook-test/1.0.0")
        .expect("custom user-agent should be valid")
        .without_timeout();
    let policy = OrderBookTransportPolicy::default().with_client_policy(custom.clone());

    assert_eq!(policy.client_policy(), &custom);
    assert_eq!(policy.client_policy().timeout(), None);
    assert_eq!(
        OrderBookTransportPolicy::default()
            .client_policy()
            .user_agent(),
        DEFAULT_ORDERBOOK_USER_AGENT
    );
}

#[test]
fn shared_http_client_policy_rejects_invalid_user_agents() {
    let error = HttpClientPolicy::new("bad\r\nagent").expect_err("CRLF must be rejected");
    assert_eq!(
        error.to_string(),
        "user_agent must be a valid HTTP header value"
    );
}

#[tokio::test]
async fn request_json_surfaces_malformed_success_payloads() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/malformed"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not-json"))
        .mount(&server)
        .await;

    let policy = RequestPolicy {
        max_attempts: 1,
        ..RequestPolicy::default()
    };
    let limiter = RequestRateLimiter::new(policy.rate_limit);

    let error = request_json::<serde_json::Value>(
        &Client::new(),
        &server.uri(),
        &FetchParams::new("/api/v1/malformed", HttpMethod::Get),
        &policy,
        &limiter,
        None,
    )
    .await
    .expect_err("malformed success payload should fail");

    match error {
        cow_sdk_orderbook::OrderbookError::Serialization(message) => {
            assert!(!message.is_empty());
        }
        other => panic!("expected serialization error, got {other:?}"),
    }
}

#[tokio::test]
async fn retryable_api_error_does_not_retry_past_the_final_attempt() {
    let policy = RequestPolicy {
        max_attempts: 1,
        ..RequestPolicy::default()
    };
    let limiter = RequestRateLimiter::new(policy.rate_limit);
    let attempts = Arc::new(AtomicUsize::new(0));

    let error = execute_json_with::<serde_json::Value, _, _>(&policy, &limiter, {
        let attempts = attempts.clone();
        move || {
            let attempts = attempts.clone();
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Ok(ResponseEnvelope::json(
                    INTERNAL_SERVER_ERROR,
                    &json!({
                        "errorType": "InternalServerError",
                        "description": "last attempt must surface the API error"
                    }),
                ))
            }
        }
    })
    .await
    .expect_err("the last retryable status must remain an API error");

    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    match error {
        cow_sdk_orderbook::OrderbookError::Api(api_error) => {
            assert_eq!(api_error.status, INTERNAL_SERVER_ERROR);
            assert_eq!(
                api_error.error_type(),
                Some("InternalServerError"),
                "the final retryable status must not degrade into a transport error"
            );
        }
        other => panic!("expected API error, got {other:?}"),
    }
}

#[tokio::test]
async fn transport_errors_delay_between_retryable_attempts() {
    let policy = RequestPolicy {
        max_attempts: 2,
        ..RequestPolicy::default()
    };
    let limiter = RequestRateLimiter::new(policy.rate_limit);
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempt_times = Arc::new(Mutex::new(Vec::new()));

    let error = execute_empty_with(&policy, &limiter, {
        let attempts = attempts.clone();
        let attempt_times = attempt_times.clone();
        move || {
            let attempts = attempts.clone();
            let attempt_times = attempt_times.clone();
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                attempt_times
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(Instant::now());
                Err("temporary network outage".to_owned())
            }
        }
    })
    .await
    .expect_err("transport failures should still surface after retry");

    assert_eq!(attempts.load(Ordering::SeqCst), 2);
    let attempt_times = attempt_times
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert_eq!(attempt_times.len(), 2);
    assert!(
        attempt_times[1].duration_since(attempt_times[0]) >= Duration::from_millis(40),
        "retryable transport failures must delay before the next attempt, not only after the final failure"
    );
    assert!(matches!(
        error,
        cow_sdk_orderbook::OrderbookError::Transport(message)
            if message == "temporary network outage"
    ));
}

#[tokio::test]
async fn final_transport_error_returns_without_sleeping_again() {
    let policy = RequestPolicy {
        max_attempts: 1,
        ..RequestPolicy::default()
    };
    let limiter = RequestRateLimiter::new(policy.rate_limit);

    timeout(
        Duration::from_millis(35),
        execute_empty_with(&policy, &limiter, || async {
            Err("single-attempt transport failure".to_owned())
        }),
    )
    .await
    .expect("terminal transport failures must not sleep after the last attempt")
    .expect_err("the single transport failure should still surface");
}

#[tokio::test]
async fn api_errors_keep_empty_bodies_empty_even_outside_204_successes() {
    let policy = RequestPolicy {
        max_attempts: 1,
        ..RequestPolicy::default()
    };
    let limiter = RequestRateLimiter::new(policy.rate_limit);

    let error = execute_json_with::<serde_json::Value, _, _>(&policy, &limiter, || async {
        Ok(ResponseEnvelope::empty(INTERNAL_SERVER_ERROR))
    })
    .await
    .expect_err("empty error bodies must remain typed as empty");

    match error {
        cow_sdk_orderbook::OrderbookError::Api(api_error) => {
            assert_eq!(api_error.status, INTERNAL_SERVER_ERROR);
            assert_eq!(api_error.body, ResponseBody::Empty);
        }
        other => panic!("expected API error, got {other:?}"),
    }
}

#[tokio::test]
async fn api_errors_keep_plain_text_payloads_out_of_the_json_decoder() {
    let policy = RequestPolicy {
        max_attempts: 1,
        ..RequestPolicy::default()
    };
    let limiter = RequestRateLimiter::new(policy.rate_limit);

    let error = execute_json_with::<serde_json::Value, _, _>(&policy, &limiter, || async {
        Ok(ResponseEnvelope::text(400, "plain-text upstream failure"))
    })
    .await
    .expect_err("plain-text API errors must not be treated as malformed JSON");

    match error {
        cow_sdk_orderbook::OrderbookError::Api(api_error) => {
            assert_eq!(
                api_error.body,
                ResponseBody::Text("plain-text upstream failure".to_owned())
            );
        }
        other => panic!("expected API error, got {other:?}"),
    }
}
