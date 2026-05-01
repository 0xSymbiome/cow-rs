mod common;

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use std::time::{Duration, Instant};

#[cfg(feature = "tracing")]
use cow_sdk_core::Amount;
use cow_sdk_core::{
    Cancellable, HttpClientPolicy, HttpTransport, ReqwestTransport, ReqwestTransportConfig,
};
use cow_sdk_orderbook::error::classify_reqwest_error;
use cow_sdk_orderbook::request::{
    DEFAULT_ORDERBOOK_USER_AGENT, FetchParams, HttpMethod, JitterStrategy, OrderBookApiError,
    OrderBookTransportPolicy, RateLimitSettings, RequestPolicy, RequestRateLimiter, ResponseBody,
    ResponseEnvelope, execute_empty_with, execute_json_with, request_empty, request_json,
    request_text,
};
use cow_sdk_orderbook::{
    CowEnv, DEFAULT_INTERVAL_LABEL, DEFAULT_MAX_ATTEMPTS, DEFAULT_TOKENS_PER_INTERVAL,
    INTERNAL_SERVER_ERROR, OrderbookError, RETRYABLE_STATUS_CODES, SupportedChainId,
    TOO_MANY_REQUESTS,
};
#[cfg(feature = "tracing")]
use cow_sdk_orderbook::{OrderCreation, OrderQuoteRequest, QuoteSide, SigningScheme};
use proptest::prelude::*;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

fn build_shared_transport() -> Arc<dyn HttpTransport + Send + Sync> {
    Arc::new(
        ReqwestTransport::new(
            ReqwestTransportConfig::new(String::new()).with_user_agent("cow-rs-request-tests"),
        )
        .expect("reqwest transport must build for the request-helper tests"),
    )
}
use serde_json::json;
use tokio::time::{sleep, timeout};
use wiremock::{
    Mock, MockServer, Request, ResponseTemplate,
    matchers::{header, method, path},
};

#[cfg(feature = "tracing")]
use crate::common::{
    build_orderbook_api_with_base_url, sample_buy_token, sample_order_uid, sample_owner,
    sample_quote_response_json, sample_signature,
};
use crate::common::{build_orderbook_api_with_policy, default_context};

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
    let policy = RequestPolicy::default().with_jitter(JitterStrategy::none());

    assert_eq!(policy.backoff_delay(1), Duration::from_millis(50));
    assert_eq!(policy.backoff_delay(2), Duration::from_millis(100));
    assert_eq!(policy.backoff_delay(3), Duration::from_millis(200));
    assert_eq!(policy.backoff_delay(7), Duration::from_millis(3200));
    assert_eq!(policy.backoff_delay(8), Duration::from_millis(3200));
}

proptest! {
    #[test]
    fn seeded_jitter_decorrelates_parallel_retry_waits(seed in any::<u64>(), attempt_index in 1usize..=7) {
        let base_policy = RequestPolicy::new(3, RateLimitSettings::default())
            .with_jitter(JitterStrategy::none());
        let base = base_policy.backoff_delay(attempt_index);
        let policy = RequestPolicy::new(3, RateLimitSettings::default())
            .with_jitter(JitterStrategy::decorrelated_from_seed(seed));
        let first_policy = policy.clone();
        let second_policy = policy;

        let first = first_policy.backoff_delay(attempt_index);
        let second = second_policy.backoff_delay(attempt_index);

        prop_assert_ne!(first, second);
        prop_assert!(first >= base);
        prop_assert!(second >= base);
        prop_assert!(first <= base.saturating_add(base / 2));
        prop_assert!(second <= base.saturating_add(base / 2));
    }
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

    let policy = RequestPolicy::new(3, RateLimitSettings::default());
    let limiter = RequestRateLimiter::new(policy.rate_limit);
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("x-api-key"),
        HeaderValue::from_static("secret"),
    );

    let result: serde_json::Value = request_json(
        &build_shared_transport(),
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

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn retry_after_backoff_wait_can_be_cancelled_before_next_attempt() {
    let server = MockServer::start().await;
    let attempts = Arc::new(AtomicUsize::new(0));

    Mock::given(method("GET"))
        .and(path("/api/v1/version"))
        .respond_with({
            let attempts = attempts.clone();
            move |_request: &Request| {
                let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                if attempt == 0 {
                    ResponseTemplate::new(TOO_MANY_REQUESTS)
                        .insert_header("Retry-After", "30")
                        .set_body_json(json!({
                            "errorType": "RateLimited",
                            "description": "retry after cancellation boundary"
                        }))
                } else {
                    ResponseTemplate::new(200).set_body_string("v1.2.3")
                }
            }
        })
        .mount(&server)
        .await;

    let transport_policy = OrderBookTransportPolicy::default().with_request_policy(
        RequestPolicy::new(2, RateLimitSettings::default()).with_jitter(JitterStrategy::none()),
    );
    let api = build_orderbook_api_with_policy(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        transport_policy,
    )
    .with_env_base_url(CowEnv::Prod, server.uri());
    let token = cow_sdk_core::CancellationToken::new();
    let token_for_call = token.clone();

    let call = api.get_version().cancel_with(&token_for_call);
    let trigger = async {
        for _ in 0..100 {
            if attempts.load(Ordering::SeqCst) == 1 {
                break;
            }
            tokio::task::yield_now().await;
        }
        assert_eq!(
            attempts.load(Ordering::SeqCst),
            1,
            "the first retryable response must be observed before cancellation"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
        token.cancel();
    };

    let (result, ()) = tokio::join!(call, trigger);

    assert!(matches!(result, Err(OrderbookError::Cancelled)));
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        1,
        "cancelling during Retry-After backoff must not dispatch a second attempt"
    );
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
                        "errorType": "DuplicatedOrder",
                        "description": "order already exists"
                    }),
                ))
            }
        }
    })
    .await
    .expect_err("400 duplicate order should not be retried");

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
            assert_eq!(attempts.load(Ordering::SeqCst), 1);
        }
        other => panic!("expected Rejected, got {other:?}"),
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
        &build_shared_transport(),
        &server.uri(),
        &FetchParams::new("/api/v1/version", HttpMethod::Get),
        &policy,
        &limiter,
        None,
    )
    .await
    .expect("text response should decode");
    request_empty(
        &build_shared_transport(),
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
    let policy = RequestPolicy::new(1, RateLimitSettings::new(1, interval, "test"));
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
    let policy = RequestPolicy::new(1, RateLimitSettings::new(1, interval, "test"));
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
    let policy = RequestPolicy::new(1, RateLimitSettings::new(1, interval, "test"));
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
fn typed_api_error_preserves_status_body_and_message() {
    let error = OrderBookApiError::new(
        400,
        "Bad Request",
        ResponseBody::Json(json!({
            "errorType": "DuplicatedOrder",
            "description": "order already exists"
        })),
    );

    assert_eq!(error.status, 400);
    assert!(
        matches!(&error.body, ResponseBody::Json(_)),
        "typed body must be preserved verbatim"
    );
    assert!(
        error.to_string().contains("order already exists"),
        "Display must surface the description from the envelope",
    );
}

#[test]
fn json_envelope_classifies_to_typed_rejection_through_from_api_error() {
    let api_error = OrderBookApiError::new(
        400,
        "Bad Request",
        ResponseBody::Json(json!({
            "errorType": "DuplicatedOrder",
            "description": "order already exists"
        })),
    );

    match cow_sdk_orderbook::OrderbookError::from(api_error) {
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

    let policy = RequestPolicy::new(1, RateLimitSettings::default());
    let limiter = RequestRateLimiter::new(policy.rate_limit);

    let error = request_json::<serde_json::Value>(
        &build_shared_transport(),
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
            assert!(!message.to_string().is_empty());
        }
        other => panic!("expected serialization error, got {other:?}"),
    }
}

#[tokio::test]
async fn retryable_api_error_does_not_retry_past_the_final_attempt() {
    let policy = RequestPolicy::new(1, RateLimitSettings::default());
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
        cow_sdk_orderbook::OrderbookError::Rejected {
            status,
            rejection,
            source,
        } => {
            assert_eq!(status.as_u16(), INTERNAL_SERVER_ERROR);
            assert_eq!(
                rejection,
                cow_sdk_orderbook::OrderbookRejection::InternalServerError,
                "the final retryable status must not degrade into a transport error",
            );
            assert_eq!(source.status, INTERNAL_SERVER_ERROR);
        }
        other => panic!("expected Rejected, got {other:?}"),
    }
}

#[tokio::test]
async fn transport_errors_delay_between_retryable_attempts() {
    let policy = RequestPolicy::new(2, RateLimitSettings::default());
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
                Err((
                    cow_sdk_core::TransportErrorClass::Other,
                    "temporary network outage".to_owned(),
                ))
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
        cow_sdk_orderbook::OrderbookError::Transport { ref detail, .. }
            if detail == "temporary network outage"
    ));
}

#[tokio::test]
async fn final_transport_error_returns_without_sleeping_again() {
    let policy = RequestPolicy::new(1, RateLimitSettings::default());
    let limiter = RequestRateLimiter::new(policy.rate_limit);

    timeout(
        Duration::from_millis(35),
        execute_empty_with(&policy, &limiter, || async {
            Err((
                cow_sdk_core::TransportErrorClass::Other,
                "single-attempt transport failure".to_owned(),
            ))
        }),
    )
    .await
    .expect("terminal transport failures must not sleep after the last attempt")
    .expect_err("the single transport failure should still surface");
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
#[ignore = "nightly deterministic retry / timeout soak"]
async fn retry_timeout_soak_exercises_deterministic_waveforms() {
    let policy =
        RequestPolicy::new(3, RateLimitSettings::default()).with_jitter(JitterStrategy::none());

    for round in 0..32usize {
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
                                "description": format!("retry soak round {round}")
                            }),
                        ))
                    } else {
                        Ok(ResponseEnvelope::json(200, &json!({ "round": round })))
                    }
                }
            }
        })
        .await
        .expect("third deterministic retry attempt should succeed");

        assert_eq!(result["round"], json!(round));
        assert_eq!(attempts.load(Ordering::SeqCst), 3);

        let limiter = RequestRateLimiter::new(policy.rate_limit);
        let timeout_attempts = Arc::new(AtomicUsize::new(0));
        let error = execute_empty_with(&policy, &limiter, {
            let timeout_attempts = timeout_attempts.clone();
            move || {
                let timeout_attempts = timeout_attempts.clone();
                async move {
                    timeout_attempts.fetch_add(1, Ordering::SeqCst);
                    Err((
                        cow_sdk_core::TransportErrorClass::Timeout,
                        format!("deterministic timeout soak round {round}"),
                    ))
                }
            }
        })
        .await
        .expect_err("timeout waveform should exhaust deterministic retry attempts");

        assert_eq!(timeout_attempts.load(Ordering::SeqCst), 3);
        assert!(matches!(
            error,
            cow_sdk_orderbook::OrderbookError::Transport {
                class: cow_sdk_core::TransportErrorClass::Timeout,
                ..
            }
        ));
    }
}

#[tokio::test]
async fn api_errors_keep_empty_bodies_empty_even_outside_204_successes() {
    let policy = RequestPolicy::new(1, RateLimitSettings::default());
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
    let policy = RequestPolicy::new(1, RateLimitSettings::default());
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

#[tokio::test]
async fn reqwest_error_classification_strips_url_query_and_host() {
    let secret_host = "invalid-orderbook-host-for-redaction-regression.test";
    let secret_key = "super-secret-api-key-should-never-leak";
    let url = format!("http://{secret_host}/v1/auction?api_key={secret_key}");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(200))
        .build()
        .expect("test client must construct");
    let raw_error = client
        .get(&url)
        .send()
        .await
        .expect_err("unreachable host must produce a reqwest error");

    let (class, detail) = classify_reqwest_error(raw_error);
    assert!(
        !detail.contains(secret_host),
        "classified transport error must strip the host: {detail}"
    );
    assert!(
        !detail.contains(secret_key),
        "classified transport error must strip query-string secrets: {detail}"
    );
    assert!(
        !detail.contains("api_key"),
        "classified transport error must strip query parameter names: {detail}"
    );
    assert!(
        !detail.contains("http://"),
        "classified transport error must not include the URL scheme prefix: {detail}"
    );
    let class_prefix = class.as_str();
    assert!(
        [
            "timeout", "connect", "redirect", "decode", "body", "builder", "request", "status",
            "other"
        ]
        .contains(&class_prefix),
        "classification prefix must come from the documented reqwest is_* set, got {class_prefix}"
    );
}

#[test]
fn orderbook_transport_error_from_conversion_classifies_without_url_exposure() {
    // Construct a builder-time reqwest error so conversion path is exercised
    // deterministically without requiring the network.
    let builder_error = reqwest::Url::parse("not a url").unwrap_err();
    let message = format!("builder-error-fixture: {builder_error}");
    // Route a synthetic reqwest error through OrderbookError by triggering an
    // invalid URL inside reqwest::Client::get so the redaction path is covered.
    let client = reqwest::Client::new();
    let err = client
        .request(reqwest::Method::GET, "http://[invalid ipv6]/")
        .build()
        .expect_err("malformed URL must produce a builder-layer reqwest error");

    let orderbook_err: OrderbookError = err.into();
    let rendered = format!("{orderbook_err}");
    assert!(
        !rendered.contains("invalid ipv6"),
        "converted orderbook error must not expose URL fragments: {rendered} ({message})"
    );
    match &orderbook_err {
        OrderbookError::Transport { detail, .. } => {
            assert!(
                !detail.contains("http://"),
                "wrapped transport detail must not include the URL scheme prefix: {detail}"
            );
        }
        OrderbookError::Serialization(inner) => {
            let body = inner.to_string();
            assert!(
                !body.contains("http://"),
                "wrapped serialization body must not include the URL scheme prefix: {body}"
            );
        }
        other => panic!("expected Transport or Serialization variant, got {other:?}"),
    }
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
    use tracing_core::span::Current;

    #[tokio::test(flavor = "current_thread")]
    async fn execute_with_emits_retry_events_with_status_and_transport_error_fields() {
        let capture = TraceCapture::install();
        let policy =
            RequestPolicy::new(2, RateLimitSettings::default()).with_jitter(JitterStrategy::none());
        let limiter = RequestRateLimiter::new(policy.rate_limit);
        let attempts = Arc::new(AtomicUsize::new(0));

        let result: serde_json::Value = execute_json_with(&policy, &limiter, {
            let attempts = attempts.clone();
            move || {
                let attempts = attempts.clone();
                async move {
                    let current = attempts.fetch_add(1, Ordering::SeqCst);
                    if current == 0 {
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
        .expect("second attempt should succeed");

        assert_eq!(result["ok"], json!(true));

        let transport_limiter = RequestRateLimiter::new(policy.rate_limit);
        let _ = execute_empty_with(&policy, &transport_limiter, || async {
            Err((
                cow_sdk_core::TransportErrorClass::Other,
                "temporary transport failure".to_owned(),
            ))
        })
        .await
        .expect_err("transport errors should exhaust attempts");

        let events = capture.events();
        assert!(
            events.iter().any(|event| {
                event.level == Level::DEBUG
                    && event.field("attempt_index") == Some("1")
                    && event.field("status") == Some("500")
                    && event.field("backoff_ms") == Some("50")
            }),
            "retry event must carry status and backoff fields: {events:#?}"
        );
        assert!(
            events.iter().any(|event| {
                event.level == Level::DEBUG
                    && event.field("attempt_index") == Some("1")
                    && event.field("transport_error_class") == Some("other")
                    && event.field("backoff_ms") == Some("50")
            }),
            "retry event must carry transport error class and backoff fields: {events:#?}"
        );
        assert!(
            events.iter().any(|event| {
                event.level == Level::WARN
                    && event.field("attempt_index") == Some("2")
                    && event.field("transport_error_class") == Some("other")
                    && event.field("backoff_ms") == Some("0")
            }),
            "final retry failure must warn with terminal attempt fields: {events:#?}"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn send_order_span_records_quote_id_attempts_and_status() {
        let capture = TraceCapture::install();
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/quote"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_quote_response_json()))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/api/v1/orders"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_order_uid().as_str()))
            .mount(&server)
            .await;

        let api = build_orderbook_api_with_base_url(
            default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
            server.uri(),
        );
        let quote = api
            .get_quote(&OrderQuoteRequest::new(
                sample_owner(),
                sample_buy_token(),
                sample_owner(),
                QuoteSide::sell(Amount::new("1000000").expect("test amount literal is valid")),
            ))
            .await
            .expect("quote should succeed");
        let quote_id = quote.id.expect("fixture carries a quote id");
        let order = OrderCreation::from_quote(
            &quote.quote,
            sample_owner(),
            None,
            SigningScheme::Eip712,
            sample_signature(),
        )
        .with_quote_id(quote_id);

        let uid = api
            .send_order(&order)
            .await
            .expect("order submission should succeed");

        assert_eq!(uid.as_str(), sample_order_uid().as_str());
        let spans = capture.spans();
        assert!(
            spans.iter().any(|span| {
                span.name == "send_order"
                    && span.field("quote_id") == Some("42")
                    && span.field("attempts") == Some("1")
                    && span.field("status") == Some("200")
            }),
            "send_order span must carry populated quote_id, attempts, and status fields: {spans:#?}"
        );
        assert!(
            spans.iter().any(|span| {
                span.name == "get_quote"
                    && span.field("quote_id") == Some("42")
                    && span.field("attempts") == Some("1")
                    && span.field("status") == Some("200")
            }),
            "quote span must carry populated quote_id, attempts, and status fields: {spans:#?}"
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

        fn spans(&self) -> Vec<CapturedSpan> {
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

    #[derive(Clone, Debug)]
    struct CapturedEvent {
        level: Level,
        fields: BTreeMap<String, String>,
    }

    impl CapturedEvent {
        fn field(&self, name: &str) -> Option<&str> {
            self.fields.get(name).map(String::as_str)
        }
    }

    #[derive(Clone, Debug)]
    struct CapturedSpan {
        name: String,
        fields: BTreeMap<String, String>,
    }

    impl CapturedSpan {
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
