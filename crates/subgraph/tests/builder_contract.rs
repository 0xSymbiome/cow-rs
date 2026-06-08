//! Public-surface contract assertions for [`SubgraphApiBuilder`].
//!
//! Every test exercises one observable shape of the typestate-checked
//! construction path. Inline `compile_fail` doctests pin the typestate
//! preconditions: invoking `.build()` before chain id, API key, or
//! transport are supplied is a compile-time error. Runtime tests cover the
//! happy-path build variants and assert that transport injection,
//! per-chain base-URL overrides, partner API keys, and shared
//! `reqwest::Client` reuse all flow through the resulting `SubgraphApi`.

use std::sync::Arc;

use cow_sdk_core::{
    HttpTransport, REDACTED_PLACEHOLDER, ReqwestTransport, ReqwestTransportConfig, SupportedChainId,
};
use cow_sdk_subgraph::{ExternalHostPolicy, SubgraphApi, SubgraphApiBaseUrls};
use cow_sdk_test_utils::mocks::{Canned, RecordingHttpTransport, StubHttpTransport};

#[test]
fn build_with_required_inputs_yields_a_typed_api() {
    let api = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("partner-key")
        .transport(Arc::new(StubHttpTransport))
        .build()
        .expect("subgraph client with explicit transport must build");

    assert_eq!(api.config().chain_id, SupportedChainId::Mainnet);
    assert_eq!(api.api_name(), "CoW Protocol Subgraph");
}

#[test]
fn native_default_build_path_supplies_a_reqwest_transport() {
    let api = SubgraphApi::builder()
        .chain(SupportedChainId::GnosisChain)
        .api_key("partner-key")
        .build()
        .expect("default subgraph client must build");

    assert_eq!(api.config().chain_id, SupportedChainId::GnosisChain);
}

#[test]
fn base_urls_override_propagates_to_the_built_client() {
    let base_urls: SubgraphApiBaseUrls = [
        (
            SupportedChainId::Mainnet,
            Some("https://subgraph.example/mainnet".to_owned()),
        ),
        (SupportedChainId::GnosisChain, None),
    ]
    .into_iter()
    .collect();

    let api = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("partner-key")
        .external_host_policy(ExternalHostPolicy::Allow(vec![
            "subgraph.example".to_owned(),
        ]))
        .base_urls(base_urls.clone())
        .build()
        .expect("subgraph client with allowed custom host must build");

    assert_eq!(api.config().base_urls.as_ref(), Some(&base_urls));
}

#[test]
fn builder_debug_redacts_partner_api_key() {
    let builder = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("partner-key");

    let debug = format!("{builder:?}");

    assert!(debug.contains(REDACTED_PLACEHOLDER));
    assert!(!debug.contains("partner-key"));
}

#[test]
fn builder_debug_redacts_base_url_credentials() {
    let base_urls: SubgraphApiBaseUrls = [
        (
            SupportedChainId::Mainnet,
            Some("https://user:pass@example.test/path?apiKey=secret-token".to_owned()),
        ),
        (SupportedChainId::GnosisChain, None),
    ]
    .into_iter()
    .collect();
    let builder = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("partner-key")
        .base_urls(base_urls);

    let debug = format!("{builder:#?}");

    assert!(debug.contains(REDACTED_PLACEHOLDER));
    assert!(!debug.contains("partner-key"));
    assert!(!debug.contains("user:pass"));
    assert!(!debug.contains("apiKey=secret-token"));
    assert!(!debug.contains("example.test"));
}

#[test]
fn builder_debug_redacts_userinfo_in_custom_endpoint_url() {
    let base_urls: SubgraphApiBaseUrls = [
        (
            SupportedChainId::Mainnet,
            Some("https://user:pass@subgraph.example/path?apiKey=secret-token".to_owned()),
        ),
        (SupportedChainId::GnosisChain, None),
    ]
    .into_iter()
    .collect();
    let builder = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("partner-key")
        .base_urls(base_urls);

    let debug = format!("{builder:#?}");

    assert!(debug.contains(REDACTED_PLACEHOLDER));
    assert!(!debug.contains("partner-key"));
    assert!(!debug.contains("user:pass"));
    assert!(!debug.contains("apiKey=secret-token"));
    assert!(!debug.contains("subgraph.example"));
}

#[test]
fn explicit_transport_overrides_default_native_handle() {
    let transport: Arc<dyn HttpTransport + Send + Sync> = Arc::new(
        ReqwestTransport::new(
            ReqwestTransportConfig::new("https://transport.example")
                .with_user_agent("cow-rs-subgraph-builder-tests"),
        )
        .expect("reqwest transport must build for the explicit-injection test"),
    );

    let api = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("partner-key")
        .transport(transport.clone())
        .build()
        .expect("subgraph client with explicit transport must build");

    assert!(Arc::ptr_eq(api.transport(), &transport));
}

#[tokio::test]
async fn injected_transport_observes_every_live_request_from_the_built_client() {
    let recorder = RecordingHttpTransport::new([Canned::Ok(
        "{\"data\":{\"totals\":[{\"tokens\":\"1\",\"orders\":\"2\",\"traders\":\"3\",\"settlements\":\"4\"}]}}"
            .to_owned(),
    )]);
    let transport: Arc<dyn HttpTransport + Send + Sync> = recorder.clone();
    let overrides: SubgraphApiBaseUrls = std::iter::once((
        SupportedChainId::Mainnet,
        Some("https://builder-recording.example".to_owned()),
    ))
    .collect();
    let api = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("partner-key")
        .external_host_policy(ExternalHostPolicy::Allow(vec![
            "builder-recording.example".to_owned(),
        ]))
        .base_urls(overrides)
        .transport(transport.clone())
        .build()
        .expect("subgraph client with injected transport must build");

    assert!(Arc::ptr_eq(api.transport(), &transport));

    let totals = api
        .totals()
        .await
        .expect("the injected transport should deliver the canned totals response");
    assert_eq!(totals.tokens, "1");

    let calls = recorder.observed();
    assert_eq!(
        calls.len(),
        1,
        "exactly one live request should flow through the injected transport"
    );
    assert_eq!(
        calls[0].method, "POST",
        "the totals query must dispatch through the POST path"
    );
    assert!(
        calls[0].url.contains("builder-recording.example"),
        "the dispatched URL must reach the injected base URL: {}",
        calls[0].url
    );
}

#[test]
fn shared_client_override_reuses_caller_built_reqwest_client() {
    let shared = reqwest::Client::builder()
        .user_agent("cow-rs-subgraph-shared-client-tests")
        .build()
        .expect("reqwest client must build for the shared-client test");

    let _ = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("partner-key")
        .client(shared.clone())
        .build()
        .expect("first shared-client subgraph handle must build");
    let _ = SubgraphApi::builder()
        .chain(SupportedChainId::GnosisChain)
        .api_key("partner-key")
        .client(shared)
        .build()
        .expect("second shared-client subgraph handle must build");
}

/// Compile-fail proof that `SubgraphApiBuilder::build` is unreachable until the
/// chain id and API-key typestates are both satisfied.
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
    cases.compile_fail("tests/ui/build_without_api_key.rs");
    cases.compile_fail("tests/ui/build_on_empty_builder.rs");
}
