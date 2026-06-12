#![cfg(feature = "transport-policy")]

use std::time::{Duration, SystemTime};

use cow_sdk_core::transport::policy::{
    DEFAULT_IPFS_USER_AGENT, DEFAULT_ORDERBOOK_USER_AGENT, DEFAULT_SUBGRAPH_USER_AGENT,
    DEFAULT_TRADING_USER_AGENT, JitterStrategy, LimiterScope, NetworkErrorKind, RETRYABLE_STATUSES,
    RequestRateLimiter, RequestRateLimiterBuilder, RetryPolicy, RetryPolicyBuilder,
    TransportPolicy, TransportPolicyBuilder, is_retryable_status, sleep,
};
#[cfg(feature = "reqwest-classifier")]
use cow_sdk_core::transport::policy::{ErrorClassifier, ReqwestErrorClassifier};
use cow_sdk_core::{CancellationToken, DEFAULT_HTTP_TIMEOUT, HttpClientPolicy, ValidationError};
use proptest::prelude::*;
use url::Url;

#[test]
fn default_orderbook_transport_policy_is_stable() {
    let policy = TransportPolicy::default_orderbook();

    assert_eq!(policy.user_agent(), DEFAULT_ORDERBOOK_USER_AGENT);
    assert_eq!(policy.timeout(), Some(DEFAULT_HTTP_TIMEOUT));
    assert_eq!(policy.retry().max_attempts(), 10);
    assert_eq!(policy.rate_limit().tokens_per_interval(), 5);
    assert_eq!(policy.rate_limit().scope(), LimiterScope::PerHost);
}

#[test]
fn default_subgraph_transport_policy_is_stable() {
    let policy = TransportPolicy::default_subgraph();

    assert_eq!(policy.user_agent(), DEFAULT_SUBGRAPH_USER_AGENT);
    assert_eq!(policy.timeout(), Some(DEFAULT_HTTP_TIMEOUT));
    assert_eq!(policy.retry().max_attempts(), 10);
    assert_eq!(policy.rate_limit().tokens_per_interval(), 5);
    assert_eq!(policy.rate_limit().scope(), LimiterScope::PerHost);
}

#[test]
fn default_policies_carry_per_client_response_byte_caps() {
    use cow_sdk_core::DEFAULT_MAX_RESPONSE_BYTES;
    use cow_sdk_core::transport::policy::{IPFS_MAX_RESPONSE_BYTES, SUBGRAPH_MAX_RESPONSE_BYTES};

    assert_eq!(
        TransportPolicy::default_orderbook()
            .client_policy()
            .max_response_bytes(),
        DEFAULT_MAX_RESPONSE_BYTES
    );
    assert_eq!(
        TransportPolicy::default_trading()
            .client_policy()
            .max_response_bytes(),
        DEFAULT_MAX_RESPONSE_BYTES
    );
    assert_eq!(
        TransportPolicy::default_subgraph()
            .client_policy()
            .max_response_bytes(),
        SUBGRAPH_MAX_RESPONSE_BYTES
    );
    assert_eq!(
        TransportPolicy::default_ipfs()
            .client_policy()
            .max_response_bytes(),
        IPFS_MAX_RESPONSE_BYTES
    );
    // The untrusted-gateway caps are deliberately tighter than the
    // trusted-orderbook default; pin the ordering at compile time.
    const {
        assert!(SUBGRAPH_MAX_RESPONSE_BYTES < DEFAULT_MAX_RESPONSE_BYTES);
        assert!(IPFS_MAX_RESPONSE_BYTES < SUBGRAPH_MAX_RESPONSE_BYTES);
    }
}

#[test]
fn no_retry_policy_is_idempotent() {
    let first = RetryPolicy::no_retry();
    let second = RetryPolicy::no_retry();

    assert_eq!(first, second);
    assert_eq!(first.max_attempts(), 1);
    assert_eq!(first.delay_for_attempt(1), Duration::from_millis(50));
}

#[test]
fn decorrelated_jitter_is_bounded_by_max_delay() {
    let policy = RetryPolicy::builder()
        .jitter(JitterStrategy::decorrelated_from_seed(42))
        .max_delay(Duration::from_millis(250))
        .build();

    for attempt in 1..=32 {
        assert!(policy.delay_for_attempt(attempt) <= Duration::from_millis(250));
    }
}

#[test]
fn request_rate_limiter_uses_host_keys_for_per_host_scope() {
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
fn reqwest_error_classifier_is_total() {
    // The bracketed token is not a valid IPv6 literal so the URL fails at
    // the builder layer and no real network traffic is attempted.
    let client = reqwest::Client::new();
    let error = client
        .request(reqwest::Method::GET, "https://[invalid ipv6]/")
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
async fn cross_target_sleep_accuracy_is_bounded() {
    let start = std::time::Instant::now();
    sleep(Duration::from_millis(5)).await;
    assert!(start.elapsed() < Duration::from_millis(250));
}

#[test]
fn retryable_status_list_is_complete() {
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

// -------------------------------------------------------------------------
// TransportPolicy constructors and builder coverage (PROP-TPP-009..015).
// -------------------------------------------------------------------------

#[test]
fn explicit_constructor_disables_tracing_and_preserves_parts() {
    let client = HttpClientPolicy::new(DEFAULT_ORDERBOOK_USER_AGENT).expect("static UA validates");
    let retry = RetryPolicy::no_retry();
    let rate_limit = RequestRateLimiter::unlimited();

    let policy = TransportPolicy::new(client.clone(), retry.clone(), rate_limit.clone());

    assert!(
        !policy.tracing_enabled(),
        "explicit `new` must default tracing to disabled",
    );
    assert_eq!(policy.client_policy(), &client);
    assert_eq!(policy.retry(), &retry);
    assert_eq!(policy.rate_limit(), &rate_limit);
}

#[test]
fn default_trading_uses_trading_user_agent_and_orderbook_limiter() {
    let policy = TransportPolicy::default_trading();

    assert_eq!(policy.user_agent(), DEFAULT_TRADING_USER_AGENT);
    assert_eq!(policy.timeout(), Some(DEFAULT_HTTP_TIMEOUT));
    assert_eq!(policy.retry().max_attempts(), 10);
    assert_eq!(policy.rate_limit().tokens_per_interval(), 5);
    assert_eq!(policy.rate_limit().scope(), LimiterScope::PerHost);
    assert!(!policy.tracing_enabled());
}

#[test]
fn default_ipfs_disables_retry_and_timeout_and_uses_unlimited_limiter() {
    let policy = TransportPolicy::default_ipfs();

    assert_eq!(policy.user_agent(), DEFAULT_IPFS_USER_AGENT);
    assert_eq!(
        policy.timeout(),
        None,
        "IPFS default disables the request timeout",
    );
    assert_eq!(
        policy.retry().max_attempts(),
        1,
        "IPFS default performs only the first attempt",
    );
    assert_eq!(
        policy.rate_limit().tokens_per_interval(),
        0,
        "IPFS default uses the unlimited limiter (zero tokens short-circuits)",
    );
    assert!(!policy.tracing_enabled());
}

#[test]
fn with_setters_replace_only_their_targeted_field() {
    let base = TransportPolicy::default_orderbook();

    // with_client_policy replaces only the client.
    let other_client =
        HttpClientPolicy::new(DEFAULT_SUBGRAPH_USER_AGENT).expect("static UA validates");
    let mutated = base.clone().with_client_policy(other_client.clone());
    assert_eq!(mutated.client_policy(), &other_client);
    assert_eq!(mutated.retry(), base.retry());
    assert_eq!(mutated.rate_limit(), base.rate_limit());

    // with_retry replaces only the retry policy.
    let other_retry = RetryPolicy::no_retry();
    let mutated = base.clone().with_retry(other_retry.clone());
    assert_eq!(mutated.retry(), &other_retry);
    assert_eq!(mutated.client_policy(), base.client_policy());
    assert_eq!(mutated.rate_limit(), base.rate_limit());

    // with_rate_limit replaces only the limiter.
    let other_limit = RequestRateLimiter::unlimited();
    let mutated = base.clone().with_rate_limit(other_limit.clone());
    assert_eq!(mutated.rate_limit(), &other_limit);
    assert_eq!(mutated.client_policy(), base.client_policy());
    assert_eq!(mutated.retry(), base.retry());

    // with_tracing_enabled toggles only the tracing flag.
    let mutated = base.clone().with_tracing_enabled(true);
    assert!(mutated.tracing_enabled());
    assert_eq!(mutated.client_policy(), base.client_policy());
    assert_eq!(mutated.retry(), base.retry());
    assert_eq!(mutated.rate_limit(), base.rate_limit());
}

#[test]
fn user_agent_validation_error_surfaces_directly() {
    // ASCII control character (0x7F is DEL) is rejected by `HttpClientPolicy`
    // header validation; the builder surfaces the core `ValidationError`
    // directly now that no build-error wrapper sits in front of it.
    let error = TransportPolicyBuilder::new()
        .user_agent("\u{007F}invalid")
        .expect_err("control character user-agent must fail validation");

    assert!(
        matches!(
            error,
            ValidationError::InvalidHttpHeaderValue {
                field: "user_agent"
            }
        ),
        "control character UA must surface as the header-value validation error; got {error:?}",
    );
    // Display renders the underlying validation message.
    assert!(
        !error.to_string().is_empty(),
        "ValidationError Display must render a message",
    );
}

#[test]
fn builder_round_trip_preserves_every_setter() {
    use cow_sdk_core::transport::policy::IPFS_MAX_RESPONSE_BYTES;

    let custom_retry = RetryPolicyBuilder::new()
        .max_attempts(3)
        .base_delay(Duration::from_millis(25))
        .max_delay(Duration::from_millis(1_000))
        .jitter(JitterStrategy::none())
        .build();
    let custom_limit = RequestRateLimiterBuilder::new()
        .tokens_per_interval(7)
        .interval(Duration::from_secs(2))
        .interval_label("two-seconds")
        .scope(LimiterScope::Global)
        .build();

    let policy = TransportPolicyBuilder::new()
        .timeout(Duration::from_secs(13))
        .user_agent("cow-sdk-test/0.0.1")
        .expect("ASCII user-agent validates")
        .retry(custom_retry.clone())
        .rate_limit(custom_limit.clone())
        .tracing_enabled(true)
        .build();

    assert_eq!(policy.user_agent(), "cow-sdk-test/0.0.1");
    assert_eq!(policy.timeout(), Some(Duration::from_secs(13)));
    assert_eq!(policy.retry(), &custom_retry);
    assert_eq!(policy.rate_limit(), &custom_limit);
    assert!(policy.tracing_enabled());

    // A caller-set client policy is refined in place: `user_agent` and
    // `timeout` preserve every other client-policy field, so a tightened
    // ADR 0055 response-byte cap and a deliberately disabled timeout survive
    // the refinement instead of resetting to the workspace defaults.
    let hardened = HttpClientPolicy::new("partner-bot/1.0")
        .expect("static UA validates")
        .without_timeout()
        .with_max_response_bytes(IPFS_MAX_RESPONSE_BYTES);

    let after_user_agent = TransportPolicyBuilder::new()
        .client_policy(hardened.clone())
        .user_agent("partner-bot/2.0")
        .expect("ASCII user-agent validates")
        .build();
    assert_eq!(
        after_user_agent.client_policy().max_response_bytes(),
        IPFS_MAX_RESPONSE_BYTES,
        "user_agent must not reset the caller's response-byte cap",
    );
    assert_eq!(
        after_user_agent.timeout(),
        None,
        "user_agent must not re-arm a deliberately disabled timeout",
    );
    assert_eq!(after_user_agent.user_agent(), "partner-bot/2.0");

    let after_timeout = TransportPolicyBuilder::new()
        .client_policy(hardened)
        .timeout(Duration::from_secs(3))
        .build();
    assert_eq!(
        after_timeout.client_policy().max_response_bytes(),
        IPFS_MAX_RESPONSE_BYTES,
        "timeout must not reset the caller's response-byte cap",
    );
    assert_eq!(after_timeout.user_agent(), "partner-bot/1.0");
    assert_eq!(after_timeout.timeout(), Some(Duration::from_secs(3)));
}

#[test]
fn builder_defaults_to_orderbook_user_agent_when_unset() {
    let policy = TransportPolicyBuilder::default().build();

    assert_eq!(policy.user_agent(), DEFAULT_ORDERBOOK_USER_AGENT);
    assert_eq!(policy.timeout(), Some(DEFAULT_HTTP_TIMEOUT));
    assert!(!policy.tracing_enabled());
    // The seeded, otherwise-untouched builder is field-identical to the
    // documented orderbook default — including the response-byte cap.
    assert_eq!(policy, TransportPolicy::default_orderbook());
}

// -------------------------------------------------------------------------
// JitterStrategy coverage (PROP-TPP-016..020).
// -------------------------------------------------------------------------

#[test]
fn none_jitter_returns_capped_base_delay_unchanged() {
    let policy = RetryPolicy::builder()
        .jitter(JitterStrategy::none())
        .base_delay(Duration::from_millis(50))
        .max_delay(Duration::from_millis(3_200))
        .build();

    // attempt 1 is exactly base_delay; attempt 7 is capped at max_delay.
    assert_eq!(policy.delay_for_attempt(1), Duration::from_millis(50));
    assert_eq!(policy.delay_for_attempt(7), Duration::from_millis(3_200));
    // attempt 20 stays clamped at max_delay (the saturating-shift path).
    assert_eq!(policy.delay_for_attempt(20), Duration::from_millis(3_200));
}

#[test]
fn full_jitter_time_seeded_constructor_does_not_panic() {
    // The time-seeded constructor is non-deterministic; we only smoke-test
    // that it returns the `Full` variant and that applying it to a retry
    // policy yields delays within the documented `[0, max_delay]` window.
    let strategy = JitterStrategy::full();
    assert!(matches!(strategy, JitterStrategy::Full { .. }));

    let policy = RetryPolicy::builder()
        .jitter(strategy)
        .max_delay(Duration::from_millis(200))
        .build();
    for attempt in 1_usize..=8 {
        assert!(policy.delay_for_attempt(attempt) <= Duration::from_millis(200));
    }
}

#[test]
fn equal_jitter_returns_at_least_half_capped_base_delay() {
    // Equal-jitter preserves half the base delay deterministically, then
    // adds up to half from a seed-derived offset. With base_delay 100 and
    // a max_delay >= base_delay, the result is bounded `[50, 100]`.
    let policy = RetryPolicy::builder()
        .jitter(JitterStrategy::equal_from_seed(0xABCD_EF01))
        .base_delay(Duration::from_millis(100))
        .max_delay(Duration::from_millis(100))
        .build();

    for attempt in 1_usize..=8 {
        let delay = policy.delay_for_attempt(attempt);
        // The base_backoff clamp pins the input to 100ms once attempt >= 1,
        // so equal-jitter is bounded [50, 100]ms.
        assert!(
            delay >= Duration::from_millis(50),
            "equal jitter must preserve at least half the base delay at attempt {attempt}: {delay:?}",
        );
        assert!(
            delay <= Duration::from_millis(100),
            "equal jitter must not exceed max_delay at attempt {attempt}: {delay:?}",
        );
    }
}

#[test]
fn default_jitter_strategy_is_decorrelated_variant() {
    assert!(matches!(
        JitterStrategy::default(),
        JitterStrategy::Decorrelated { .. },
    ));
    assert!(matches!(
        JitterStrategy::decorrelated(),
        JitterStrategy::Decorrelated { .. },
    ));
}

#[test]
fn zero_base_delay_returns_zero_across_every_strategy() {
    for strategy in [
        JitterStrategy::none(),
        JitterStrategy::full_from_seed(1),
        JitterStrategy::equal_from_seed(2),
        JitterStrategy::decorrelated_from_seed(3),
    ] {
        let policy = RetryPolicy::builder()
            .jitter(strategy)
            .base_delay(Duration::ZERO)
            .max_delay(Duration::ZERO)
            .build();
        for attempt in 1_usize..=8 {
            assert_eq!(
                policy.delay_for_attempt(attempt),
                Duration::ZERO,
                "zero-window strategy {strategy:?} must yield zero at attempt {attempt}",
            );
        }
    }
}

// -------------------------------------------------------------------------
// RequestRateLimiter coverage (PROP-TPP-021..025).
// -------------------------------------------------------------------------

#[tokio::test]
async fn unlimited_rate_limiter_never_delays_or_errors() {
    let limiter = RequestRateLimiter::unlimited();
    let url = Url::parse("https://api.cow.fi/mainnet/api/v1/orders").unwrap();
    let token = CancellationToken::new();

    let start = std::time::Instant::now();
    for _ in 0..100 {
        limiter
            .acquire(&url, &token)
            .await
            .expect("unlimited limiter always succeeds");
    }
    // 100 token-less acquires should complete near-instantly. Generous
    // upper bound to tolerate slow CI.
    assert!(
        start.elapsed() < Duration::from_millis(500),
        "unlimited limiter must not introduce measurable delay; elapsed = {:?}",
        start.elapsed(),
    );
}

#[test]
fn global_scope_uses_constant_key_regardless_of_host() {
    let limiter = RequestRateLimiter::builder()
        .scope(LimiterScope::Global)
        .build();
    let host_a = Url::parse("https://api.cow.fi/mainnet/api/v1/orders").unwrap();
    let host_b = Url::parse("https://barn.api.cow.fi/xdai/api/v1/version").unwrap();
    let host_c = Url::parse("https://thegraph.com/foo/bar").unwrap();

    let key_a = limiter.key_for_url(&host_a);
    let key_b = limiter.key_for_url(&host_b);
    let key_c = limiter.key_for_url(&host_c);

    assert_eq!(key_a, "global");
    assert_eq!(key_a, key_b);
    assert_eq!(key_a, key_c);
}

#[tokio::test]
async fn acquire_global_shares_one_bucket_across_calls() {
    // tokens_per_interval = 2 with a long interval; two acquire_global
    // calls must both succeed without waiting.
    let limiter = RequestRateLimiter::builder()
        .tokens_per_interval(2)
        .interval(Duration::from_secs(60))
        .scope(LimiterScope::Global)
        .build();
    let token = CancellationToken::new();

    let start = std::time::Instant::now();
    limiter.acquire_global(&token).await.unwrap();
    limiter.acquire_global(&token).await.unwrap();
    // Both fit in the initial budget; no waiting expected.
    assert!(start.elapsed() < Duration::from_millis(250));
}

#[tokio::test]
async fn pre_cancelled_token_returns_cancelled_immediately() {
    let limiter = RequestRateLimiter::builder()
        .tokens_per_interval(1)
        .interval(Duration::from_secs(60))
        .build();
    let url = Url::parse("https://api.cow.fi/mainnet/api/v1/orders").unwrap();
    let token = CancellationToken::new();

    // Drain the single token first.
    limiter.acquire(&url, &token).await.unwrap();
    // Cancel before the next acquire even starts.
    token.cancel();

    let start = std::time::Instant::now();
    let result = limiter.acquire(&url, &token).await;
    assert!(result.is_err(), "pre-cancelled token must short-circuit");
    // Pre-cancellation must NOT cause us to sleep for the 60s interval.
    assert!(
        start.elapsed() < Duration::from_millis(250),
        "pre-cancelled token must return immediately; elapsed = {:?}",
        start.elapsed(),
    );
}

#[test]
fn rate_limiter_builder_round_trip_preserves_setters() {
    let limiter = RequestRateLimiterBuilder::new()
        .tokens_per_interval(11)
        .interval(Duration::from_millis(750))
        .interval_label("custom-interval")
        .scope(LimiterScope::Global)
        .build();

    assert_eq!(limiter.tokens_per_interval(), 11);
    assert_eq!(limiter.interval(), Duration::from_millis(750));
    assert_eq!(limiter.interval_label(), "custom-interval");
    assert_eq!(limiter.scope(), LimiterScope::Global);

    // Default forwards to new().
    let default_builder = RequestRateLimiterBuilder::default();
    let new_builder = RequestRateLimiterBuilder::new();
    assert_eq!(default_builder, new_builder);
}

// -------------------------------------------------------------------------
// RetryPolicy coverage (PROP-TPP-026..032).
// -------------------------------------------------------------------------

#[test]
fn retry_policy_new_sets_max_attempts_and_keeps_defaults() {
    let policy = RetryPolicy::new(5);
    assert_eq!(policy.max_attempts(), 5);
    assert_eq!(policy.base_delay(), Duration::from_millis(50));
    assert_eq!(policy.max_delay(), Duration::from_millis(3_200));
    // `new` builds via the const builder which seeds `JitterStrategy::None`.
    assert!(matches!(policy.jitter(), JitterStrategy::None));
}

#[test]
fn with_jitter_replaces_only_the_jitter_field() {
    let base = RetryPolicy::default();
    let mutated = base.clone().with_jitter(JitterStrategy::equal_from_seed(7));

    assert!(matches!(
        mutated.jitter(),
        JitterStrategy::Equal { seed: 7 }
    ));
    assert_eq!(mutated.max_attempts(), base.max_attempts());
    assert_eq!(mutated.base_delay(), base.base_delay());
    assert_eq!(mutated.max_delay(), base.max_delay());
}

#[test]
fn should_retry_status_matches_the_public_retryable_list() {
    let policy = RetryPolicy::default();
    for status in RETRYABLE_STATUSES {
        assert!(
            policy.should_retry_status(status),
            "retryable status {status} must opt in",
        );
    }
    for status in [200_u16, 204, 301, 400, 401, 403, 404, 410, 418, 501] {
        assert!(
            !policy.should_retry_status(status),
            "non-retryable status {status} must opt out",
        );
    }
}

#[test]
fn should_retry_network_only_retries_documented_kinds() {
    let policy = RetryPolicy::default();
    // Documented retryable kinds.
    assert!(policy.should_retry_network(NetworkErrorKind::Timeout));
    assert!(policy.should_retry_network(NetworkErrorKind::Connect));
    assert!(policy.should_retry_network(NetworkErrorKind::Request));
    assert!(policy.should_retry_network(NetworkErrorKind::Other));
    // Non-retryable kinds — protocol or local errors that retrying cannot fix.
    assert!(!policy.should_retry_network(NetworkErrorKind::Decode));
    assert!(!policy.should_retry_network(NetworkErrorKind::Builder));
    assert!(!policy.should_retry_network(NetworkErrorKind::Cancelled));
    assert!(!policy.should_retry_network(NetworkErrorKind::HttpStatus(500)));
    assert!(!policy.should_retry_network(NetworkErrorKind::HttpStatus(429)));
}

#[test]
fn base_backoff_clamps_to_max_delay_across_attempt_range() {
    let policy = RetryPolicy::builder()
        .base_delay(Duration::from_millis(50))
        .max_delay(Duration::from_millis(200))
        .jitter(JitterStrategy::none())
        .build();

    // Attempt 1: 50ms × 2^0 = 50ms.
    assert_eq!(policy.delay_for_attempt(1), Duration::from_millis(50));
    // Attempt 2: 50ms × 2^1 = 100ms.
    assert_eq!(policy.delay_for_attempt(2), Duration::from_millis(100));
    // Attempt 3: 50ms × 2^2 = 200ms (exactly at the cap).
    assert_eq!(policy.delay_for_attempt(3), Duration::from_millis(200));
    // Attempt 7: 50ms × 2^6 = 3200ms but clamped to 200ms.
    assert_eq!(policy.delay_for_attempt(7), Duration::from_millis(200));
    // Attempt 20: exponent clamped to 6 internally; result still capped.
    assert_eq!(policy.delay_for_attempt(20), Duration::from_millis(200));
}

#[test]
fn retry_after_helper_is_case_insensitive() {
    let policy = RetryPolicy::builder()
        .jitter(JitterStrategy::none())
        .build();
    let now = SystemTime::UNIX_EPOCH;

    for header_name in ["Retry-After", "retry-after", "RETRY-AFTER", "Retry-after"] {
        let headers = vec![(header_name.to_owned(), "5".to_owned())];
        assert_eq!(
            policy.delay_for_status(1, 429, &headers, now),
            Duration::from_secs(5),
            "{header_name} must be honored case-insensitively",
        );
    }

    // No Retry-After header: falls back to the regular backoff.
    let headers = vec![("X-Other".to_owned(), "ignored".to_owned())];
    assert_eq!(
        policy.delay_for_status(1, 429, &headers, now),
        Duration::from_millis(50),
    );

    // For non-retry-after-eligible statuses, the header is ignored.
    let headers = vec![("Retry-After".to_owned(), "60".to_owned())];
    assert_eq!(
        policy.delay_for_status(1, 500, &headers, now),
        Duration::from_millis(50),
    );
}

#[test]
fn retry_builder_round_trip_and_zero_attempts_clamps_to_one() {
    let custom_jitter = JitterStrategy::full_from_seed(42);
    let policy = RetryPolicyBuilder::new()
        .max_attempts(4)
        .base_delay(Duration::from_millis(75))
        .max_delay(Duration::from_millis(5_000))
        .jitter(custom_jitter)
        .build();

    assert_eq!(policy.max_attempts(), 4);
    assert_eq!(policy.base_delay(), Duration::from_millis(75));
    assert_eq!(policy.max_delay(), Duration::from_millis(5_000));
    assert!(matches!(policy.jitter(), JitterStrategy::Full { seed: 42 }));

    // `max_attempts(0)` is silently promoted to 1 to avoid infinite-zero loops.
    let zero_attempts = RetryPolicyBuilder::new().max_attempts(0).build();
    assert_eq!(zero_attempts.max_attempts(), 1);

    // Default forwards to new().
    let default_builder = RetryPolicyBuilder::default();
    let new_builder = RetryPolicyBuilder::new();
    assert_eq!(default_builder, new_builder);
}
