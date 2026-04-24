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
use std::sync::Mutex;
use std::time::Duration;

use cow_sdk_core::{
    HttpTransport, REDACTED_PLACEHOLDER, ReqwestTransport, ReqwestTransportConfig,
    SupportedChainId, TransportError,
};
use cow_sdk_subgraph::{SubgraphApi, SubgraphApiBaseUrls};

#[derive(Debug, Default)]
struct StubTransport;

#[async_trait::async_trait]
impl HttpTransport for StubTransport {
    async fn get(
        &self,
        _path: &str,
        _headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        Ok(String::new())
    }
    async fn post(
        &self,
        _path: &str,
        _body: &str,
        _headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        Ok(String::new())
    }
    async fn put(
        &self,
        _path: &str,
        _body: &str,
        _headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        Ok(String::new())
    }
    async fn delete(
        &self,
        _path: &str,
        _body: &str,
        _headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
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
fn builder_debug_redacts_partner_api_key() {
    let builder = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("partner-key");

    let debug = format!("{builder:?}");

    assert!(debug.contains(REDACTED_PLACEHOLDER));
    assert!(!debug.contains("partner-key"));
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

#[derive(Debug, Default)]
struct BuilderRecordingTransport {
    calls: Mutex<Vec<String>>,
    response: Mutex<String>,
}

impl BuilderRecordingTransport {
    fn with_response(response: &str) -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            response: Mutex::new(response.to_owned()),
        }
    }

    fn calls(&self) -> Vec<String> {
        self.calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

#[async_trait::async_trait]
impl HttpTransport for BuilderRecordingTransport {
    async fn get(
        &self,
        path: &str,
        _headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        self.calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(format!("GET {path}"));
        Ok(self
            .response
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone())
    }
    async fn post(
        &self,
        path: &str,
        _body: &str,
        _headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        self.calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(format!("POST {path}"));
        Ok(self
            .response
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone())
    }
    async fn put(
        &self,
        path: &str,
        _body: &str,
        _headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        self.calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(format!("PUT {path}"));
        Ok(self
            .response
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone())
    }
    async fn delete(
        &self,
        path: &str,
        _body: &str,
        _headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        self.calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(format!("DELETE {path}"));
        Ok(self
            .response
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone())
    }
}

#[tokio::test]
async fn injected_transport_observes_every_live_request_from_the_built_client() {
    let recorder = Arc::new(BuilderRecordingTransport::with_response(
        "{\"data\":{\"totals\":[{\"tokens\":\"1\",\"orders\":\"2\",\"traders\":\"3\",\"settlements\":\"4\"}]}}",
    ));
    let transport: Arc<dyn HttpTransport + Send + Sync> = recorder.clone();
    let overrides: SubgraphApiBaseUrls = std::iter::once((
        SupportedChainId::Mainnet,
        Some("https://builder-recording.example".to_owned()),
    ))
    .collect();
    let api = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("partner-key")
        .base_urls(overrides)
        .transport(transport.clone())
        .build();

    assert!(Arc::ptr_eq(api.transport(), &transport));

    let totals = api
        .get_totals()
        .await
        .expect("the injected transport should deliver the canned totals response");
    assert_eq!(totals.tokens, "1");

    let calls = recorder.calls();
    assert_eq!(
        calls.len(),
        1,
        "exactly one live request should flow through the injected transport"
    );
    assert!(
        calls[0].starts_with("POST "),
        "the totals query must dispatch through the POST path: {}",
        calls[0]
    );
    assert!(
        calls[0].contains("builder-recording.example"),
        "the dispatched URL must reach the injected base URL: {}",
        calls[0]
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
