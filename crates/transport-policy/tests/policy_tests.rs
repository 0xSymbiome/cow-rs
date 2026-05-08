use std::time::{Duration, SystemTime};

use cow_sdk_core::{CancellationToken, DEFAULT_HTTP_TIMEOUT};
use cow_sdk_transport_policy::{
    DEFAULT_ORDERBOOK_USER_AGENT, DEFAULT_SUBGRAPH_USER_AGENT, JitterStrategy, LimiterScope,
    RETRYABLE_STATUSES, RequestRateLimiter, RetryPolicy, TransportPolicy, is_retryable_status,
    sleep,
};
#[cfg(feature = "reqwest-classifier")]
use cow_sdk_transport_policy::{ErrorClassifier, NetworkErrorKind, ReqwestErrorClassifier};
use proptest::prelude::*;
use url::Url;

#[test]
fn prop_tpp_001_default_orderbook_transport_policy_is_stable() {
    let policy = TransportPolicy::default_orderbook();

    assert_eq!(policy.user_agent(), DEFAULT_ORDERBOOK_USER_AGENT);
    assert_eq!(policy.timeout(), Some(DEFAULT_HTTP_TIMEOUT));
    assert_eq!(policy.retry().max_attempts(), 10);
    assert_eq!(policy.rate_limit().tokens_per_interval(), 5);
    assert_eq!(policy.rate_limit().scope(), LimiterScope::PerHost);
}

#[test]
fn prop_tpp_002_default_subgraph_transport_policy_is_stable() {
    let policy = TransportPolicy::default_subgraph();

    assert_eq!(policy.user_agent(), DEFAULT_SUBGRAPH_USER_AGENT);
    assert_eq!(policy.timeout(), Some(DEFAULT_HTTP_TIMEOUT));
    assert_eq!(policy.retry().max_attempts(), 10);
    assert_eq!(policy.rate_limit().tokens_per_interval(), 5);
    assert_eq!(policy.rate_limit().scope(), LimiterScope::PerHost);
}

#[test]
fn prop_tpp_003_no_retry_policy_is_idempotent() {
    let first = RetryPolicy::no_retry();
    let second = RetryPolicy::no_retry();

    assert_eq!(first, second);
    assert_eq!(first.max_attempts(), 1);
    assert_eq!(first.delay_for_attempt(1), Duration::from_millis(50));
}

#[test]
fn prop_tpp_004_decorrelated_jitter_is_bounded_by_max_delay() {
    let policy = RetryPolicy::builder()
        .jitter(JitterStrategy::decorrelated_from_seed(42))
        .max_delay(Duration::from_millis(250))
        .build();

    for attempt in 1..=32 {
        assert!(policy.delay_for_attempt(attempt) <= Duration::from_millis(250));
    }
}

#[test]
fn prop_tpp_005_request_rate_limiter_uses_host_keys_for_per_host_scope() {
    let limiter = RequestRateLimiter::builder()
        .scope(LimiterScope::PerHost)
        .build();
    let first = Url::parse("https://api.cow.fi/mainnet/api/v1/orders").unwrap();
    let second = Url::parse("https://api.cow.fi/mainnet/api/v1/version").unwrap();
    let third = Url::parse("https://barn.api.cow.fi/xdai/api/v1/orders").unwrap();

    assert_eq!(limiter.key_for_url(&first), limiter.key_for_url(&second));
    assert_ne!(limiter.key_for_url(&first), limiter.key_for_url(&third));
}

#[cfg(feature = "reqwest-classifier")]
#[test]
fn prop_tpp_006_reqwest_error_classifier_is_total() {
    let client = reqwest::Client::new();
    let error = client
        .request(reqwest::Method::GET, "http://[invalid ipv6]/")
        .build()
        .expect_err("malformed URL must fail at the builder layer");

    let kind = ReqwestErrorClassifier.classify(&error);
    assert!(matches!(
        kind,
        NetworkErrorKind::Builder | NetworkErrorKind::Request | NetworkErrorKind::Other
    ));
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn prop_tpp_007_cross_target_sleep_accuracy_is_bounded() {
    let start = std::time::Instant::now();
    sleep(Duration::from_millis(5)).await;
    assert!(start.elapsed() < Duration::from_millis(250));
}

#[test]
fn prop_tpp_008_retryable_status_list_is_complete() {
    assert_eq!(RETRYABLE_STATUSES, [408, 425, 429, 500, 502, 503, 504]);
    for status in RETRYABLE_STATUSES {
        assert!(is_retryable_status(status));
    }
    assert!(!is_retryable_status(400));
    assert!(!is_retryable_status(404));
}

proptest! {
    #[test]
    fn retry_backoff_is_monotonic(attempt in 1_usize..20) {
        let policy = RetryPolicy::builder()
            .jitter(JitterStrategy::decorrelated_from_seed(7))
            .build();
        prop_assert!(policy.delay_for_attempt(attempt + 1) >= policy.delay_for_attempt(attempt));
    }

    #[test]
    fn full_jitter_stays_inside_delay_window(attempt in 1_usize..20, seed in any::<u64>()) {
        let policy = RetryPolicy::builder()
            .jitter(JitterStrategy::full_from_seed(seed))
            .build();
        prop_assert!(policy.delay_for_attempt(attempt) <= policy.max_delay());
    }

    #[test]
    fn status_retryability_matches_public_list(status in 100_u16..600) {
        prop_assert_eq!(is_retryable_status(status), RETRYABLE_STATUSES.contains(&status));
    }
}

#[tokio::test]
async fn rate_limiter_can_be_cancelled_while_waiting() {
    let limiter = RequestRateLimiter::builder()
        .tokens_per_interval(1)
        .interval(Duration::from_secs(60))
        .build();
    let url = Url::parse("https://api.cow.fi/mainnet/api/v1/orders").unwrap();
    let token = CancellationToken::new();

    limiter.acquire(&url, &token).await.unwrap();
    token.cancel();

    let result = limiter.acquire(&url, &token).await;
    assert!(result.is_err());
}

#[test]
fn retry_after_only_affects_rate_limit_and_unavailable_statuses() {
    let policy = RetryPolicy::builder()
        .jitter(JitterStrategy::none())
        .build();
    let headers = vec![("Retry-After".to_owned(), "5".to_owned())];
    let now = SystemTime::UNIX_EPOCH;

    assert_eq!(
        policy.delay_for_status(1, 429, &headers, now),
        Duration::from_secs(5)
    );
    assert_eq!(
        policy.delay_for_status(1, 503, &headers, now),
        Duration::from_secs(5)
    );
    assert_eq!(
        policy.delay_for_status(1, 500, &headers, now),
        Duration::from_millis(50)
    );
}
