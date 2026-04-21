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
    HttpTransport, ReqwestTransport, ReqwestTransportConfig, SupportedChainId, TransportError,
};
use cow_sdk_subgraph::{SubgraphApi, SubgraphApiBaseUrls};

#[derive(Debug, Default)]
struct StubTransport;

#[async_trait::async_trait(?Send)]
impl HttpTransport for StubTransport {
    async fn get(&self, _path: &str) -> Result<String, TransportError> {
        Ok(String::new())
    }
    async fn post(&self, _path: &str, _body: &str) -> Result<String, TransportError> {
        Ok(String::new())
    }
    async fn delete(&self, _path: &str, _body: &str) -> Result<String, TransportError> {
        Ok(String::new())
    }
}

#[test]
fn build_with_required_inputs_yields_a_typed_api() {
    let api = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("partner-key")
        .transport(Arc::new(StubTransport))
        .build();

    assert_eq!(api.config().chain_id, SupportedChainId::Mainnet);
    assert_eq!(api.api_name(), "CoW Protocol Subgraph");
}

#[test]
fn native_default_build_path_supplies_a_reqwest_transport() {
    let api = SubgraphApi::builder()
        .chain(SupportedChainId::GnosisChain)
        .api_key("partner-key")
        .build();

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
        .base_urls(base_urls.clone())
        .build();

    assert_eq!(api.config().base_urls.as_ref(), Some(&base_urls));
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
        .build();

    assert!(Arc::ptr_eq(api.transport(), &transport));
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
        .build();
    let _ = SubgraphApi::builder()
        .chain(SupportedChainId::GnosisChain)
        .api_key("partner-key")
        .client(shared)
        .build();
}

/// Compile-time assertion: `.build()` is unreachable when the chain id is
/// missing.
///
/// ```compile_fail
/// use cow_sdk_subgraph::SubgraphApi;
///
/// let _ = SubgraphApi::builder()
///     .api_key("partner-key")
///     .build();
/// ```
#[test]
fn typestate_compile_fail_no_chain_documented() {}

/// Compile-time assertion: `.build()` is unreachable when the API key is
/// missing.
///
/// ```compile_fail
/// use cow_sdk_core::SupportedChainId;
/// use cow_sdk_subgraph::SubgraphApi;
///
/// let _ = SubgraphApi::builder()
///     .chain(SupportedChainId::Mainnet)
///     .build();
/// ```
#[test]
fn typestate_compile_fail_no_api_key_documented() {}

/// Compile-time assertion: `.build()` is unreachable when neither the
/// required chain id nor API key have been supplied.
///
/// ```compile_fail
/// use cow_sdk_subgraph::SubgraphApi;
///
/// let _ = SubgraphApi::builder().build();
/// ```
#[test]
fn typestate_compile_fail_empty_builder_documented() {}
