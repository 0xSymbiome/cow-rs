//! Typestate builder for [`SubgraphApi`].
//!
//! [`SubgraphApiBuilder`] is the sole production construction path for a
//! [`SubgraphApi`]. The compiler enforces that the chain id, partner API
//! key, and HTTP transport are all supplied before
//! [`SubgraphApiBuilder::build`] becomes callable. Optional configuration —
//! transport policy and per-chain base-URL overrides — is layered on
//! through fluent methods that do not affect the typestate.
//!
//! The builder also exposes [`SubgraphApiBuilder::build`] against the
//! typestate where transport is unset, defaulting the transport per target:
//! [`ReqwestTransport`](cow_sdk_core::ReqwestTransport) on native targets and
//! `FetchTransport` from `cow-sdk-core` (the realm's global
//! `fetch`) on `wasm32`, so the common consumer never has to wire a
//! transport explicitly on either target. Consumers that need a custom
//! backend keep the explicit [`SubgraphApiBuilder::transport`] seam.
//!
//! # Examples
//!
//! ```
//! use cow_sdk_core::SupportedChainId;
//! use cow_sdk_subgraph::SubgraphApi;
//!
//! # #[cfg(not(target_arch = "wasm32"))]
//! # {
//! let subgraph = SubgraphApi::builder()
//!     .chain(SupportedChainId::Mainnet)
//!     .api_key("partner-graph-api-key")
//!     .build()
//!     .expect("subgraph client builds with canonical defaults");
//! let _ = subgraph;
//! # }
//! ```

use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use cow_sdk_core::transport::policy::DEFAULT_SUBGRAPH_USER_AGENT;
use cow_sdk_core::transport::policy::TransportPolicy;
use cow_sdk_core::{
    ExternalHostPolicy, HttpTransport, Redacted, SupportedChainId, canonical_subgraph_hosts,
    validate_external_service_url,
};
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use cow_sdk_core::{FetchTransport, FetchTransportConfig};
#[cfg(not(target_arch = "wasm32"))]
use cow_sdk_core::{ReqwestTransport, ReqwestTransportConfig, TransportError, TransportErrorClass};
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Client;

use crate::api::{SubgraphApi, SubgraphApiBaseUrls, SubgraphConfig, build_prod_config};
use crate::error::SubgraphError;

/// Typestate marker — chain id has not been supplied.
#[derive(Debug, Clone, Copy)]
pub struct ChainIdUnset(());
/// Typestate marker carrying the supplied chain id.
#[derive(Debug, Clone, Copy)]
pub struct ChainIdSet(SupportedChainId);

/// Typestate marker — API key has not been supplied.
#[derive(Debug, Clone, Copy)]
pub struct ApiKeyUnset(());
/// Typestate marker carrying the supplied partner Graph API key.
#[derive(Debug, Clone)]
pub struct ApiKeySet(Redacted<String>);

/// Typestate marker — transport has not been supplied.
#[derive(Debug, Clone, Copy)]
pub struct TransportUnset(());
/// Typestate marker carrying the supplied HTTP transport.
#[derive(Debug, Clone)]
pub struct TransportSet(Arc<dyn HttpTransport + Send + Sync>);

/// Typestate-checked builder for [`SubgraphApi`].
///
/// The three type parameters track which of the required inputs (chain id,
/// API key, transport) have been supplied. [`SubgraphApiBuilder::build`]
/// is implemented only against the typestates that satisfy the documented
/// preconditions, so calling it with any required field still unset is a
/// compile-time error rather than a runtime failure.
#[derive(Debug, Clone)]
pub struct SubgraphApiBuilder<
    ChainState = ChainIdUnset,
    ApiKeyState = ApiKeyUnset,
    TransportState = TransportUnset,
> {
    chain: ChainState,
    api_key: ApiKeyState,
    transport: TransportState,
    transport_policy: Option<TransportPolicy>,
    base_urls: Option<SubgraphApiBaseUrls>,
    host_policy: ExternalHostPolicy,
}

impl Default for SubgraphApiBuilder<ChainIdUnset, ApiKeyUnset, TransportUnset> {
    fn default() -> Self {
        Self::new()
    }
}

impl SubgraphApiBuilder<ChainIdUnset, ApiKeyUnset, TransportUnset> {
    /// Creates a fresh builder with no required fields supplied.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            chain: ChainIdUnset(()),
            api_key: ApiKeyUnset(()),
            transport: TransportUnset(()),
            transport_policy: None,
            base_urls: None,
            host_policy: ExternalHostPolicy::Default,
        }
    }
}

impl<A, T> SubgraphApiBuilder<ChainIdUnset, A, T> {
    /// Supplies the chain id for the subgraph context.
    ///
    /// Transitions the chain typestate from [`ChainIdUnset`] to [`ChainIdSet`].
    #[must_use]
    pub fn chain(self, chain: SupportedChainId) -> SubgraphApiBuilder<ChainIdSet, A, T> {
        SubgraphApiBuilder {
            chain: ChainIdSet(chain),
            api_key: self.api_key,
            transport: self.transport,
            transport_policy: self.transport_policy,
            base_urls: self.base_urls,
            host_policy: self.host_policy,
        }
    }
}

impl<C, T> SubgraphApiBuilder<C, ApiKeyUnset, T> {
    /// Supplies the partner Graph API key used for production routing.
    ///
    /// Transitions the API-key typestate from [`ApiKeyUnset`] to
    /// [`ApiKeySet`]. The key is wrapped in
    /// [`cow_sdk_core::Redacted`] before storage so it is never emitted
    /// through debug, display, or serialized output.
    #[must_use]
    pub fn api_key(self, api_key: impl Into<String>) -> SubgraphApiBuilder<C, ApiKeySet, T> {
        SubgraphApiBuilder {
            chain: self.chain,
            api_key: ApiKeySet(Redacted::new(api_key.into())),
            transport: self.transport,
            transport_policy: self.transport_policy,
            base_urls: self.base_urls,
            host_policy: self.host_policy,
        }
    }
}

impl<C, A> SubgraphApiBuilder<C, A, TransportUnset> {
    /// Supplies the [`HttpTransport`] dispatch seam.
    ///
    /// Transitions the transport typestate from [`TransportUnset`] to
    /// [`TransportSet`]. The transport is the runtime-neutral injection
    /// point for native and browser HTTP backends.
    #[must_use]
    pub fn transport(
        self,
        transport: Arc<dyn HttpTransport + Send + Sync>,
    ) -> SubgraphApiBuilder<C, A, TransportSet> {
        SubgraphApiBuilder {
            chain: self.chain,
            api_key: self.api_key,
            transport: TransportSet(transport),
            transport_policy: self.transport_policy,
            base_urls: self.base_urls,
            host_policy: self.host_policy,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<C, A> SubgraphApiBuilder<C, A, TransportUnset> {
    /// Reuses an externally-built [`reqwest::Client`] as the backing
    /// transport.
    ///
    /// Multi-chain consumers compose one shared [`reqwest::Client`] (with
    /// its TCP, TLS, and HTTP/2 connection cache) across every
    /// [`SubgraphApi`] they construct. The shared client is wrapped into a
    /// [`ReqwestTransport`] so every live request still flows through the
    /// single `HttpTransport` dispatch seam; the transport resolves paths
    /// against the empty base URL so the subgraph request pipeline keeps
    /// building full URLs from the API-key-derived routing map.
    #[must_use]
    pub fn client(self, client: Client) -> SubgraphApiBuilder<C, A, TransportSet> {
        let transport: Arc<dyn HttpTransport + Send + Sync> =
            Arc::new(ReqwestTransport::with_client(client, ""));
        self.transport(transport)
    }
}

impl<C, A, T> SubgraphApiBuilder<C, A, T> {
    /// Sets the request retry, rate-limit, and HTTP-client policy bundle.
    ///
    /// When this method is not called, [`SubgraphApiBuilder::build`] uses
    /// [`TransportPolicy::default_subgraph`] which preserves the
    /// documented default behavior.
    #[must_use]
    pub fn transport_policy(mut self, policy: TransportPolicy) -> Self {
        self.transport_policy = Some(policy);
        self
    }

    /// Sets the external host policy used to validate explicit subgraph
    /// service endpoint overrides.
    ///
    /// The default accepts only the SDK's canonical The Graph gateway host.
    /// Local fixtures should use [`ExternalHostPolicy::Test`], and private
    /// mirrors should use [`ExternalHostPolicy::Allow`] with the mirror host.
    #[must_use]
    pub fn external_host_policy(mut self, policy: ExternalHostPolicy) -> Self {
        self.host_policy = policy;
        self
    }

    /// Supplies an explicit per-chain base-URL map.
    ///
    /// Each entry overrides the production routing derived from the
    /// supplied API key for the corresponding chain. A `None` value
    /// marks the chain as unsupported on the resulting client.
    #[must_use]
    pub fn base_urls(mut self, base_urls: impl Into<SubgraphApiBaseUrls>) -> Self {
        self.base_urls = Some(base_urls.into());
        self
    }
}

impl<T> SubgraphApiBuilder<ChainIdSet, ApiKeySet, T> {
    /// Finalizes the builder once a transport has been resolved.
    ///
    /// The chain id and API key are read directly from the data-carrying
    /// [`ChainIdSet`] / [`ApiKeySet`] typestate markers, so finalization
    /// performs no `Option` unwrap and contains no typestate-guard panic.
    ///
    /// # Errors
    ///
    /// Returns [`SubgraphError`] when explicit base-URL overrides fail the
    /// configured external host policy.
    fn finish(
        self,
        transport: Arc<dyn HttpTransport + Send + Sync>,
    ) -> Result<SubgraphApi, SubgraphError> {
        validate_subgraph_base_urls(self.base_urls.as_ref(), &self.host_policy)?;

        let chain = self.chain.0;
        let api_key = self.api_key.0;
        let transport_policy = self
            .transport_policy
            .unwrap_or_else(TransportPolicy::default_subgraph);
        let rate_limiter = transport_policy.rate_limit().clone();
        let prod_config = build_prod_config();
        let config = SubgraphConfig {
            chain_id: chain,
            base_urls: self.base_urls,
        };
        Ok(SubgraphApi::from_parts(
            config,
            api_key,
            prod_config,
            transport_policy,
            rate_limiter,
            transport,
        ))
    }
}

impl SubgraphApiBuilder<ChainIdSet, ApiKeySet, TransportSet> {
    /// Builds the [`SubgraphApi`] with the supplied chain, API key, and
    /// transport.
    ///
    /// # Errors
    ///
    /// Returns [`SubgraphError`] when explicit base-URL overrides fail the
    /// configured external host policy.
    pub fn build(self) -> Result<SubgraphApi, SubgraphError> {
        let transport = self.transport.0.clone();
        self.finish(transport)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl SubgraphApiBuilder<ChainIdSet, ApiKeySet, TransportUnset> {
    /// Builds the [`SubgraphApi`] with the supplied chain and API key,
    /// defaulting the transport to a native [`ReqwestTransport`] handle.
    ///
    /// On `wasm32` targets the same terminal defaults to the browser
    /// `FetchTransport` instead, so the zero-config construction path is
    /// available on every target.
    ///
    /// # Errors
    ///
    /// Returns [`SubgraphError`] when explicit base-URL overrides fail the
    /// configured external host policy, or
    /// [`SubgraphError::TransportConfiguration`] when the configured transport
    /// policy yields a user-agent that cannot be encoded as an HTTP header
    /// value while constructing the default native [`ReqwestTransport`].
    pub fn build(self) -> Result<SubgraphApi, SubgraphError> {
        let user_agent = self
            .transport_policy
            .as_ref()
            .map_or(DEFAULT_SUBGRAPH_USER_AGENT, |policy| policy.user_agent())
            .to_owned();
        let timeout = self
            .transport_policy
            .as_ref()
            .and_then(TransportPolicy::timeout);
        let max_response_bytes = self
            .transport_policy
            .as_ref()
            .map(|policy| policy.client_policy().max_response_bytes());
        let mut config = ReqwestTransportConfig::new(String::new()).with_user_agent(user_agent);
        if let Some(timeout) = timeout {
            config = config.with_timeout(timeout);
        }
        if let Some(max_response_bytes) = max_response_bytes {
            config = config.with_max_response_bytes(max_response_bytes);
        }
        let transport =
            ReqwestTransport::new(config).map_err(subgraph_transport_configuration_error)?;
        self.finish(Arc::new(transport))
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
impl SubgraphApiBuilder<ChainIdSet, ApiKeySet, TransportUnset> {
    /// Builds the [`SubgraphApi`] with the supplied chain and API key,
    /// defaulting the transport to the browser [`FetchTransport`] backed by
    /// the realm's global `fetch`.
    ///
    /// The default mirrors the native [`ReqwestTransport`] terminal: the
    /// configured transport policy's timeout and response-byte cap are
    /// applied to the transport. The policy's user-agent is deliberately not
    /// applied — `User-Agent` is a forbidden request header for browser
    /// `fetch`, so the runtime's own value is sent instead.
    ///
    /// # Errors
    ///
    /// Returns [`SubgraphError`] when explicit base-URL overrides fail the
    /// configured external host policy.
    pub fn build(self) -> Result<SubgraphApi, SubgraphError> {
        let timeout = self
            .transport_policy
            .as_ref()
            .and_then(TransportPolicy::timeout);
        let max_response_bytes = self
            .transport_policy
            .as_ref()
            .map(|policy| policy.client_policy().max_response_bytes());
        let mut config = FetchTransportConfig::new(String::new());
        if let Some(timeout) = timeout {
            config = config.with_timeout(timeout);
        }
        if let Some(max_response_bytes) = max_response_bytes {
            config = config.with_max_response_bytes(max_response_bytes);
        }
        let transport = FetchTransport::new(&config);
        self.finish(Arc::new(transport))
    }
}

/// Maps a transport-construction failure from the default native
/// [`ReqwestTransport`] onto the context-free
/// [`SubgraphError::TransportConfiguration`] variant.
///
/// The default-transport build path runs before any request is assembled, so
/// no per-request context is available to populate the context-carrying
/// [`SubgraphError::Transport`] variant; the redacted transport detail is
/// carried through unchanged so the workspace redaction posture is preserved
/// (ADR 0025).
#[cfg(not(target_arch = "wasm32"))]
fn subgraph_transport_configuration_error(error: TransportError) -> SubgraphError {
    let (class, details) = match error {
        TransportError::Configuration { message } => (TransportErrorClass::Builder, message),
        TransportError::Transport { class, detail } => (class, detail),
        TransportError::HttpStatus { status, .. } => (
            TransportErrorClass::Status,
            Redacted::new(format!("transport returned HTTP status {status}")),
        ),
        _ => (
            TransportErrorClass::Other,
            Redacted::new("transport configuration error".to_owned()),
        ),
    };
    SubgraphError::TransportConfiguration { class, details }
}

fn validate_subgraph_base_urls(
    base_urls: Option<&SubgraphApiBaseUrls>,
    policy: &ExternalHostPolicy,
) -> Result<(), SubgraphError> {
    if let Some(base_urls) = base_urls {
        for base_url in base_urls.as_inner().values().flatten() {
            validate_external_service_url(base_url, canonical_subgraph_hosts(), policy)?;
        }
    }

    Ok(())
}
