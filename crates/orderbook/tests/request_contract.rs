mod common;
use common::{limiter, retry_policy};

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use std::time::{Duration, Instant};

#[cfg(feature = "tracing")]
use cow_sdk_core::Amount;
use cow_sdk_core::{Cancellable, HttpTransport, ReqwestTransport, ReqwestTransportConfig};
use cow_sdk_orderbook::OrderbookError;
use cow_sdk_orderbook::error::classify_reqwest_error;
use cow_sdk_orderbook::request::{
    FetchParams, HttpMethod, OrderbookApiError, ResponseBody, ResponseEnvelope, execute_empty_with,
    execute_json_with, request_empty, request_json, request_text,
};
#[cfg(feature = "tracing")]
use cow_sdk_orderbook::{CowEnv, SupportedChainId};
#[cfg(feature = "tracing")]
use cow_sdk_orderbook::{OrderCreation, OrderQuoteRequest, OrderQuoteSide, SigningScheme};
use cow_sdk_core::transport::policy::{
    INTERNAL_SERVER_ERROR, JitterStrategy, RequestRateLimiter, RetryPolicy, TOO_MANY_REQUESTS,
};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

fn build_shared_transport() -> Arc<dyn HttpTransport + Send + Sync> {
    Arc::new(
        ReqwestTransport::new(
            ReqwestTransportConfig::new(String::new()).with_user_agent("cow-rs-request-tests"),
        )
        .expect("reqwest transport must build for the request-helper tests"),
    )
}

const fn retry_policy_no_jitter(max_attempts: usize) -> RetryPolicy {
    RetryPolicy::builder()
        .max_attempts(max_attempts)
        .jitter(JitterStrategy::none())
        .build()
}

fn default_limiter() -> RequestRateLimiter {
    RequestRateLimiter::default_orderbook()
}

use serde_json::json;
use tokio::sync::Notify;
use tokio::time::timeout;
use wiremock::{
    Mock, MockServer, Request, ResponseTemplate,
    matchers::{header, method, path},
};

#[cfg(feature = "tracing")]
use crate::common::{
    build_orderbook_api_with_base_url, default_context, sample_buy_token, sample_order_uid,
    sample_owner, sample_quote_response_json, sample_signature,
};

#[tokio::test]
async fn execute_json_with_retries_transient_statuses_until_success() {
    let policy = RetryPolicy::default();
    let limiter = default_limiter();
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

    let policy = retry_policy(3);
    let limiter = default_limiter();
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

#[tokio::test]
async fn retry_after_backoff_wait_can_be_cancelled_before_next_attempt() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let first_retryable_response = Arc::new(Notify::new());

    let retry = RetryPolicy::builder()
        .max_attempts(2)
        .base_delay(Duration::from_millis(250))
        .max_delay(Duration::from_millis(250))
        .jitter(JitterStrategy::none())
        .build();
    let limiter = default_limiter();
    let token = cow_sdk_core::CancellationToken::new();
    let token_for_call = token.clone();
    let attempts_for_call = attempts.clone();
    let first_retryable_response_for_call = first_retryable_response.clone();

    let call = tokio::spawn(async move {
        execute_json_with::<serde_json::Value, _, _>(&retry, &limiter, move || {
            let attempts = attempts_for_call.clone();
            let first_retryable_response = first_retryable_response_for_call.clone();
            async move {
                let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                if attempt == 0 {
                    first_retryable_response.notify_one();
                    Ok(ResponseEnvelope::json(
                        TOO_MANY_REQUESTS,
                        &json!({
                            "errorType": "RateLimited",
                            "description": "retry after cancellation boundary"
                        }),
                    ))
                } else {
                    Ok(ResponseEnvelope::json(200, &json!({ "version": "v1.2.3" })))
                }
            }
        })
        .cancel_with(&token_for_call)
        .await
    });
    first_retryable_response.notified().await;
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        1,
        "the first retryable response must be observed before cancellation"
    );
    token.cancel();
    let result = call.await.expect("request task must not panic");

    assert!(
        matches!(result, Err(OrderbookError::Cancelled)),
        "expected cancellation during retry backoff, got {result:?}"
    );
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        1,
        "cancelling during Retry-After backoff must not dispatch a second attempt"
    );
}

#[tokio::test]
async fn execute_json_with_stops_on_non_retryable_api_error_and_preserves_body() {
    let policy = RetryPolicy::default();
    let limiter = default_limiter();
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
    let policy = RetryPolicy::default();
    let limiter = default_limiter();

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

    let policy = RetryPolicy::default();
    let limiter = default_limiter();

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
    let policy = retry_policy(1);
    let limiter = limiter(1, interval, "test");

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
    let policy = retry_policy(1);
    let limiter = limiter(1, interval, "test");
    let arrivals = Arc::new(Mutex::new(Vec::new()));

    let spawn_attempt =
        |policy: RetryPolicy, limiter: RequestRateLimiter, arrivals: Arc<Mutex<Vec<Instant>>>| {
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

#[test]
fn typed_api_error_preserves_status_body_and_message() {
    let error = OrderbookApiError::new(
        400,
        "Bad Request",
        ResponseBody::Json(json!({
            "errorType": "DuplicatedOrder",
            "description": "order already exists"
        })),
    );

    assert_eq!(error.status, 400);
    assert!(
        matches!(error.body.as_inner(), ResponseBody::Json(_)),
        "typed body must be preserved verbatim"
    );
    assert!(
        error.to_string().contains("[redacted]"),
        "Display must redact the description from the envelope",
    );
}

#[test]
fn json_envelope_classifies_to_typed_rejection_through_from_api_error() {
    let api_error = OrderbookApiError::new(
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

#[tokio::test]
async fn request_json_surfaces_malformed_success_payloads() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/malformed"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not-json"))
        .mount(&server)
        .await;

    let policy = retry_policy(1);
    let limiter = default_limiter();

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
        cow_sdk_orderbook::OrderbookError::Serialization { category, .. } => {
            assert!(
                matches!(category, "syntax" | "data" | "eof" | "io"),
                "serialization category must be a known tag, got {category:?}",
            );
        }
        other => panic!("expected serialization error, got {other:?}"),
    }
}

#[tokio::test]
async fn retryable_api_error_does_not_retry_past_the_final_attempt() {
    let policy = retry_policy(1);
    let limiter = default_limiter();
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
    let policy = retry_policy(2);
    let limiter = default_limiter();
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
            if detail.as_inner() == "temporary network outage"
    ));
}

#[tokio::test]
async fn final_transport_error_returns_without_sleeping_again() {
    let policy = retry_policy(1);
    let limiter = default_limiter();

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
    let policy = retry_policy_no_jitter(3);

    for round in 0..32usize {
        let limiter = default_limiter();
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

        let limiter = default_limiter();
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
    let policy = retry_policy(1);
    let limiter = default_limiter();

    let error = execute_json_with::<serde_json::Value, _, _>(&policy, &limiter, || async {
        Ok(ResponseEnvelope::empty(INTERNAL_SERVER_ERROR))
    })
    .await
    .expect_err("empty error bodies must remain typed as empty");

    match error {
        cow_sdk_orderbook::OrderbookError::Api(api_error) => {
            assert_eq!(api_error.status, INTERNAL_SERVER_ERROR);
            assert_eq!(api_error.body.as_inner(), &ResponseBody::Empty);
        }
        other => panic!("expected API error, got {other:?}"),
    }
}

#[tokio::test]
async fn api_errors_keep_plain_text_payloads_out_of_the_json_decoder() {
    let policy = retry_policy(1);
    let limiter = default_limiter();

    let error = execute_json_with::<serde_json::Value, _, _>(&policy, &limiter, || async {
        Ok(ResponseEnvelope::text(400, "plain-text upstream failure"))
    })
    .await
    .expect_err("plain-text API errors must not be treated as malformed JSON");

    match error {
        cow_sdk_orderbook::OrderbookError::Api(api_error) => {
            assert_eq!(
                api_error.body.as_inner(),
                &ResponseBody::Text("plain-text upstream failure".to_owned())
            );
        }
        other => panic!("expected API error, got {other:?}"),
    }
}

/// Regression coverage for the services 422 plain-text rejection emitted by
/// axum's default `Json<OrderQuoteRequest>` extractor on shape failure (for
/// example a request body that is missing the `from` field). The exact wire
/// text is pinned so a future axum or services update that changes the
/// formatting will fail this test rather than silently regress the
/// captured-on-Text contract.
#[tokio::test]
async fn quote_422_axum_json_rejection_plain_text_is_captured_as_text() {
    const SERVICES_422_PLAIN_TEXT: &str = "Failed to deserialize the JSON body into the target type: missing field `from` \
         at line 1 column 2";

    let policy = retry_policy(1);
    let limiter = default_limiter();

    let error = execute_json_with::<serde_json::Value, _, _>(&policy, &limiter, || async {
        Ok(ResponseEnvelope::text(422, SERVICES_422_PLAIN_TEXT))
    })
    .await
    .expect_err("services 422 plain-text rejection must surface through OrderbookError::Api");

    match error {
        cow_sdk_orderbook::OrderbookError::Api(api_error) => {
            assert_eq!(api_error.status, 422);
            assert_eq!(
                api_error.body.as_inner(),
                &ResponseBody::Text(SERVICES_422_PLAIN_TEXT.to_owned()),
                "the original axum/serde rejection text must be preserved verbatim on the \
                 structured body so consumers can extract it through `body.as_inner()`",
            );
        }
        other => panic!("expected OrderbookError::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn reqwest_error_classification_strips_url_query_and_host() {
    // Synthetic redaction fixture values. The host is a `.test` reserved
    // TLD per RFC 6761 (never resolves) and the token is a deterministic
    // sentinel used only to verify the classifier strips identifying
    // payload before returning. No real network traffic is intended.
    let unreachable_host = "invalid-orderbook-host-for-redaction-regression.test";
    let redaction_fixture_token = "redaction-regression-fixture-token-0001";
    let url = format!("https://{unreachable_host}/v1/auction?api_key={redaction_fixture_token}");

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
        !detail.contains(unreachable_host),
        "classified transport error must strip the host: {detail}"
    );
    assert!(
        !detail.contains(redaction_fixture_token),
        "classified transport error must strip query-string payload: {detail}"
    );
    assert!(
        !detail.contains("api_key"),
        "classified transport error must strip query parameter names: {detail}"
    );
    assert!(
        !detail.contains("https://"),
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
    // invalid URL inside reqwest::Client::get so the redaction path is
    // covered. The bracketed token is not a valid IPv6 literal so no real
    // network traffic is attempted at any layer.
    let client = reqwest::Client::new();
    let err = client
        .request(reqwest::Method::GET, "https://[invalid ipv6]/")
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
                !detail.as_inner().contains("http://"),
                "wrapped transport detail must not include the URL scheme prefix: {detail}"
            );
        }
        OrderbookError::Serialization { .. } => {
            assert!(
                !rendered.contains("http://"),
                "serialization diagnostic must not include the URL scheme prefix: {rendered}"
            );
        }
        other => panic!("expected Transport or Serialization variant, got {other:?}"),
    }
}

#[cfg(feature = "tracing")]
mod tracing_contract {
    use std::sync::{Arc, atomic::Ordering};

    use super::*;
    use tracing::Level;

    use cow_sdk_test_utils::trace::TraceCapture;

    #[tokio::test(flavor = "current_thread")]
    async fn execute_with_emits_retry_events_with_status_and_transport_error_fields() {
        let capture = TraceCapture::install();
        let policy = retry_policy_no_jitter(2);
        let limiter = default_limiter();
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

        let transport_limiter = default_limiter();
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
                event.level() == Level::DEBUG
                    && event.field("attempt_index") == Some("1")
                    && event.field("status") == Some("500")
                    && event.field("backoff_ms") == Some("50")
            }),
            "retry event must carry status and backoff fields: {events:#?}"
        );
        assert!(
            events.iter().any(|event| {
                event.level() == Level::DEBUG
                    && event.field("attempt_index") == Some("1")
                    && event.field("transport_error_class") == Some("other")
                    && event.field("backoff_ms") == Some("50")
            }),
            "retry event must carry transport error class and backoff fields: {events:#?}"
        );
        assert!(
            events.iter().any(|event| {
                event.level() == Level::WARN
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
            .respond_with(
                ResponseTemplate::new(200).set_body_json(sample_order_uid().to_hex_string()),
            )
            .mount(&server)
            .await;

        let api = build_orderbook_api_with_base_url(
            default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
            server.uri(),
        );
        let quote = api
            .quote(&OrderQuoteRequest::new(
                sample_owner(),
                sample_buy_token(),
                sample_owner(),
                OrderQuoteSide::sell(Amount::new("1000000").expect("test amount literal is valid")),
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

        assert_eq!(uid.to_hex_string(), sample_order_uid().to_hex_string());
        let spans = capture.spans();
        assert!(
            spans.iter().any(|span| {
                span.name() == "send_order"
                    && span.field("quote_id") == Some("42")
                    && span.field("attempts") == Some("1")
                    && span.field("status") == Some("200")
            }),
            "send_order span must carry populated quote_id, attempts, and status fields: {spans:#?}"
        );
        assert!(
            spans.iter().any(|span| {
                span.name() == "quote"
                    && span.field("quote_id") == Some("42")
                    && span.field("attempts") == Some("1")
                    && span.field("status") == Some("200")
            }),
            "quote span must carry populated quote_id, attempts, and status fields: {spans:#?}"
        );
    }
}
