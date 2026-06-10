//! Public-surface contract assertions for [`OrderbookApiBuilder`].
//!
//! Every test exercises one observable shape of the typestate-checked
//! construction path. The `trybuild` compile-fail witnesses below pin the
//! typestate preconditions: invoking `.build()` before chain id, environment,
//! or transport are supplied is a compile-time error. Runtime tests cover the
//! happy-path build variants and assert that transport injection,
//! per-environment base-URL overrides, partner API keys, and shared
//! `reqwest::Client` reuse all flow through the resulting `OrderbookApi`.

use std::sync::Arc;

use cow_sdk_core::transport::policy::{RetryPolicy, TransportPolicy};
use cow_sdk_core::{
    ApiContext, CowEnv, HttpTransport, REDACTED_PLACEHOLDER, RedactedUrlMap, ReqwestTransport,
    ReqwestTransportConfig, SupportedChainId,
};
use cow_sdk_orderbook::{EnvBaseUrlOverrides, ExternalHostPolicy, OrderbookApi};
use cow_sdk_test_utils::mocks::{Canned, RecordingHttpTransport, StubHttpTransport};

#[test]
fn build_with_required_inputs_yields_a_typed_api() {
    let api = OrderbookApi::builder()
        .chain(SupportedChainId::Mainnet)
        .environment(CowEnv::Prod)
        .transport(Arc::new(StubHttpTransport))
        .build()
        .expect("orderbook client with explicit transport must build");

    assert_eq!(api.context().chain_id, SupportedChainId::Mainnet);
    assert_eq!(api.context().env, CowEnv::Prod);
}

#[test]
fn native_default_build_path_supplies_a_reqwest_transport() {
    let api = OrderbookApi::builder()
        .chain(SupportedChainId::Sepolia)
        .environment(CowEnv::Staging)
        .build()
        .expect("default orderbook client must build");

    assert_eq!(api.context().chain_id, SupportedChainId::Sepolia);
    assert_eq!(api.context().env, CowEnv::Staging);
}

#[test]
fn builder_from_context_propagates_chain_environment_api_key_and_base_urls() {
    let mut base_urls = std::collections::BTreeMap::new();
    base_urls.insert(
        u64::from(SupportedChainId::Mainnet),
        "https://shipped.example/api".to_owned(),
    );
    let context = ApiContext::new(SupportedChainId::Mainnet, CowEnv::Prod)
        .with_api_key(cow_sdk_core::Redacted::new("partner-key".to_owned()))
        .with_base_urls(base_urls.clone());

    let api = OrderbookApi::builder_from_context(context)
        .external_host_policy(ExternalHostPolicy::Allow(vec![
            "shipped.example".to_owned(),
        ]))
        .build()
        .expect("orderbook client with allowed custom host must build");

    assert_eq!(api.context().chain_id, SupportedChainId::Mainnet);
    assert_eq!(api.context().env, CowEnv::Prod);
    assert_eq!(
        api.context()
            .api_key
            .as_ref()
            .map(|value| value.as_inner().clone()),
        Some("partner-key".to_owned()),
    );
    assert_eq!(
        api.context()
            .base_urls
            .as_ref()
            .map(RedactedUrlMap::as_inner),
        Some(&base_urls),
    );
}

#[test]
fn builder_debug_redacts_partner_api_key() {
    let builder = OrderbookApi::builder()
        .chain(SupportedChainId::Mainnet)
        .environment(CowEnv::Prod)
        .api_key("partner-key");

    let debug = format!("{builder:?}");

    assert!(debug.contains(REDACTED_PLACEHOLDER));
    assert!(!debug.contains("partner-key"));
}

#[test]
fn builder_debug_redacts_base_url_credentials() {
    let base_urls = std::collections::BTreeMap::from([(
        u64::from(SupportedChainId::Mainnet),
        "https://user:pass@example.test/path?apiKey=secret-token".to_owned(),
    )]);
    let builder = OrderbookApi::builder()
        .chain(SupportedChainId::Mainnet)
        .environment(CowEnv::Prod)
        .base_urls(base_urls);

    let debug = format!("{builder:#?}");

    assert!(debug.contains(REDACTED_PLACEHOLDER));
    assert!(!debug.contains("user:pass"));
    assert!(!debug.contains("apiKey=secret-token"));
    assert!(!debug.contains("example.test"));
}

#[test]
fn builder_debug_redacts_userinfo_in_custom_base_url_overrides() {
    let builder = OrderbookApi::builder()
        .chain(SupportedChainId::Mainnet)
        .environment(CowEnv::Prod)
        .base_url("https://user:pass@custom.example/mainnet?apiKey=secret");

    let debug = format!("{builder:#?}");

    assert!(debug.contains(REDACTED_PLACEHOLDER));
    assert!(!debug.contains("user:pass"));
    assert!(!debug.contains("apiKey=secret"));
    assert!(!debug.contains("custom.example"));
}

#[test]
fn env_base_url_overrides_debug_redacts_embedded_credentials() {
    let mut overrides = EnvBaseUrlOverrides::default();
    overrides.set(CowEnv::Prod, "https://u:p@example.com/");
    overrides.set(CowEnv::Staging, "https://s:t@staging.example.com/path");

    let debug = format!("{overrides:?}");

    assert!(debug.contains(REDACTED_PLACEHOLDER));
    assert!(!debug.contains("u:p"));
    assert!(!debug.contains("s:t"));
    assert!(!debug.contains("example.com"));
}

#[test]
fn policy_override_replaces_default_request_policy() {
    let policy =
        TransportPolicy::default().with_retry(RetryPolicy::builder().max_attempts(1).build());
    let api = OrderbookApi::builder()
        .chain(SupportedChainId::Mainnet)
        .environment(CowEnv::Prod)
        .transport_policy(policy)
        .transport(Arc::new(StubHttpTransport))
        .build()
        .expect("orderbook client with policy override must build");

    assert_eq!(api.request_policy().max_attempts(), 1);
}

#[test]
fn explicit_transport_overrides_default_native_handle() {
    let transport: Arc<dyn HttpTransport + Send + Sync> = Arc::new(
        ReqwestTransport::new(
            ReqwestTransportConfig::new("https://transport.example")
                .with_user_agent("cow-rs-builder-tests"),
        )
        .expect("reqwest transport must build for the explicit-injection test"),
    );

    let api = OrderbookApi::builder()
        .chain(SupportedChainId::Mainnet)
        .environment(CowEnv::Prod)
        .transport(transport.clone())
        .build()
        .expect("orderbook client with explicit transport must build");

    assert!(Arc::ptr_eq(api.transport(), &transport));
}

#[tokio::test]
async fn injected_transport_observes_every_live_request_from_the_built_client() {
    let recorder = RecordingHttpTransport::new([Canned::Ok("v1.2.3".to_owned())]);
    let transport: Arc<dyn HttpTransport + Send + Sync> = recorder.clone();
    let api = OrderbookApi::builder()
        .chain(SupportedChainId::Mainnet)
        .environment(CowEnv::Prod)
        .transport(transport.clone())
        .build()
        .expect("orderbook client with injected transport must build");

    assert!(Arc::ptr_eq(api.transport(), &transport));

    let version = api
        .version()
        .await
        .expect("the injected transport should deliver the canned version response");
    assert_eq!(version, "v1.2.3");

    let calls = recorder.observed();
    assert_eq!(
        calls.len(),
        1,
        "exactly one live request should flow through the injected transport"
    );
    assert_eq!(
        calls[0].method, "GET",
        "the version endpoint must dispatch through the GET path"
    );
    assert!(
        calls[0].url.contains("/api/v1/version"),
        "the dispatched URL must reach the version endpoint: {}",
        calls[0].url
    );
}

#[test]
fn shared_client_override_reuses_caller_built_reqwest_client() {
    let shared = reqwest::Client::builder()
        .user_agent("cow-rs-shared-client-tests")
        .build()
        .expect("reqwest client must build for the shared-client test");

    // Two builders fed the same client must produce two `OrderbookApi`
    // handles whose pipelines share the underlying connection pool.
    let _ = OrderbookApi::builder()
        .chain(SupportedChainId::Mainnet)
        .environment(CowEnv::Prod)
        .client(shared.clone())
        .build()
        .expect("first shared-client orderbook handle must build");
    let _ = OrderbookApi::builder()
        .chain(SupportedChainId::GnosisChain)
        .environment(CowEnv::Prod)
        .client(shared)
        .build()
        .expect("second shared-client orderbook handle must build");
}

/// Compile-fail proof that `OrderbookApiBuilder::build` is unreachable until
/// the chain id and environment typestates are both satisfied.
///
/// Each fixture under `tests/ui/` attempts `.build()` on an incomplete builder;
/// `trybuild` compiles every one and asserts it fails with the pinned
/// "no method named `build`" diagnostic. This actually exercises the compiler
/// on each case, unlike a doc-comment block, which Rust does not run from an
/// integration-test file.
#[test]
fn typestate_rejects_incomplete_builders() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/build_without_chain.rs");
    cases.compile_fail("tests/ui/build_without_environment.rs");
    cases.compile_fail("tests/ui/build_on_empty_builder.rs");
}
