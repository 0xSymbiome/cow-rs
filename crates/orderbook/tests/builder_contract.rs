//! Public-surface contract assertions for [`OrderBookApiBuilder`].
//!
//! Every test exercises one observable shape of the typestate-checked
//! construction path. Inline `compile_fail` doctests pin the typestate
//! preconditions: invoking `.build()` before chain id, environment, or
//! transport are supplied is a compile-time error. Runtime tests cover the
//! happy-path build variants and assert that transport injection,
//! per-environment base-URL overrides, partner API keys, and shared
//! `reqwest::Client` reuse all flow through the resulting `OrderBookApi`.

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use cow_sdk_core::{
    ApiContext, CowEnv, HttpTransport, ReqwestTransport, ReqwestTransportConfig, SupportedChainId,
    TransportError,
};
use cow_sdk_orderbook::{OrderBookApi, OrderBookTransportPolicy, RequestPolicy};

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
    let api = OrderBookApi::builder()
        .chain(SupportedChainId::Mainnet)
        .environment(CowEnv::Prod)
        .transport(Arc::new(StubTransport))
        .build();

    assert_eq!(api.context().chain_id, SupportedChainId::Mainnet);
    assert_eq!(api.context().env, CowEnv::Prod);
}

#[test]
fn native_default_build_path_supplies_a_reqwest_transport() {
    let api = OrderBookApi::builder()
        .chain(SupportedChainId::Sepolia)
        .environment(CowEnv::Staging)
        .build();

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

    let api = OrderBookApi::builder_from_context(context).build();

    assert_eq!(api.context().chain_id, SupportedChainId::Mainnet);
    assert_eq!(api.context().env, CowEnv::Prod);
    assert_eq!(
        api.context()
            .api_key
            .as_ref()
            .map(|value| value.as_inner().clone()),
        Some("partner-key".to_owned()),
    );
    assert_eq!(api.context().base_urls.as_ref(), Some(&base_urls));
}

#[test]
fn policy_override_replaces_default_request_policy() {
    let policy = OrderBookTransportPolicy::default().with_request_policy(RequestPolicy::new(
        1,
        cow_sdk_orderbook::request::RateLimitSettings::default(),
    ));
    let api = OrderBookApi::builder()
        .chain(SupportedChainId::Mainnet)
        .environment(CowEnv::Prod)
        .policy(policy)
        .transport(Arc::new(StubTransport))
        .build();

    assert_eq!(api.request_policy().max_attempts, 1);
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

    let api = OrderBookApi::builder()
        .chain(SupportedChainId::Mainnet)
        .environment(CowEnv::Prod)
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
    let recorder = Arc::new(BuilderRecordingTransport::with_response("v1.2.3"));
    let transport: Arc<dyn HttpTransport + Send + Sync> = recorder.clone();
    let api = OrderBookApi::builder()
        .chain(SupportedChainId::Mainnet)
        .environment(CowEnv::Prod)
        .transport(transport.clone())
        .build();

    assert!(Arc::ptr_eq(api.transport(), &transport));

    let version = api
        .get_version()
        .await
        .expect("the injected transport should deliver the canned version response");
    assert_eq!(version, "v1.2.3");

    let calls = recorder.calls();
    assert_eq!(
        calls.len(),
        1,
        "exactly one live request should flow through the injected transport"
    );
    assert!(
        calls[0].starts_with("GET "),
        "the version endpoint must dispatch through the GET path: {}",
        calls[0]
    );
    assert!(
        calls[0].contains("/api/v1/version"),
        "the dispatched URL must reach the version endpoint: {}",
        calls[0]
    );
}

#[test]
fn shared_client_override_reuses_caller_built_reqwest_client() {
    let shared = reqwest::Client::builder()
        .user_agent("cow-rs-shared-client-tests")
        .build()
        .expect("reqwest client must build for the shared-client test");

    // Two builders fed the same client must produce two `OrderBookApi`
    // handles whose pipelines share the underlying connection pool.
    let _ = OrderBookApi::builder()
        .chain(SupportedChainId::Mainnet)
        .environment(CowEnv::Prod)
        .client(shared.clone())
        .build();
    let _ = OrderBookApi::builder()
        .chain(SupportedChainId::GnosisChain)
        .environment(CowEnv::Prod)
        .client(shared)
        .build();
}

/// Compile-time assertion: `.build()` is unreachable when the chain id is
/// missing.
///
/// ```compile_fail
/// use cow_sdk_orderbook::OrderBookApi;
/// use cow_sdk_core::CowEnv;
///
/// let _ = OrderBookApi::builder()
///     .environment(CowEnv::Prod)
///     .build();
/// ```
#[test]
fn typestate_compile_fail_no_chain_documented() {}

/// Compile-time assertion: `.build()` is unreachable when the environment
/// is missing.
///
/// ```compile_fail
/// use cow_sdk_orderbook::OrderBookApi;
/// use cow_sdk_core::SupportedChainId;
///
/// let _ = OrderBookApi::builder()
///     .chain(SupportedChainId::Mainnet)
///     .build();
/// ```
#[test]
fn typestate_compile_fail_no_environment_documented() {}

/// Compile-time assertion: `.build()` is unreachable when neither the
/// required chain id nor environment have been supplied.
///
/// ```compile_fail
/// use cow_sdk_orderbook::OrderBookApi;
///
/// let _ = OrderBookApi::builder().build();
/// ```
#[test]
fn typestate_compile_fail_empty_builder_documented() {}
