//! Typestate builder for [`SubgraphApi`].
//!
//! [`SubgraphApiBuilder`] is the sole production construction path for a
//! [`SubgraphApi`]. The compiler enforces that the chain id, partner API
//! key, and HTTP transport are all supplied before
//! [`SubgraphApiBuilder::build`] becomes callable. Optional configuration —
//! transport policy and per-chain base-URL overrides — is layered on
//! through fluent methods that do not affect the typestate.
//!
//! On native targets the builder also exposes [`SubgraphApiBuilder::build`]
//! against the typestate where transport is unset, defaulting the transport
//! to [`ReqwestTransport`](cow_sdk_core::ReqwestTransport) so the common
//! single-target consumer never has to wire a transport explicitly. On
//! `wasm32` targets the default-transport build path is unavailable: the
//! caller MUST supply a `FetchTransport` from `cow-sdk-transport-wasm`
//! through [`SubgraphApiBuilder::transport`] before
//! [`build`](Self::build) becomes reachable.
//!
//! # Examples
//!
//! ```
//! use cow_sdk_core::SupportedChainId;
//! use cow_sdk_subgraph::SubgraphApi;
//!
//! # #[cfg(not(target_arch = "wasm32"))]
//! # {
//! let api = SubgraphApi::builder()
//!     .chain(SupportedChainId::Mainnet)
//!     .api_key("partner-graph-api-key")
//!     .build();
//! let _ = api;
//! # }
//! ```

use std::marker::PhantomData;
use std::sync::Arc;

use cow_sdk_core::{HttpTransport, Redacted, SupportedChainId};
#[cfg(not(target_arch = "wasm32"))]
use cow_sdk_core::{ReqwestTransport, ReqwestTransportConfig};
use reqwest::Client;

#[cfg(not(target_arch = "wasm32"))]
use crate::api::DEFAULT_SUBGRAPH_USER_AGENT;
use crate::api::{
    SubgraphApi, SubgraphApiBaseUrls, SubgraphConfig, SubgraphTransportPolicy, build_prod_config,
};

/// Typestate marker — chain id has not been supplied.
#[derive(Debug, Clone, Copy)]
pub struct ChainIdUnset;
/// Typestate marker — chain id has been supplied.
#[derive(Debug, Clone, Copy)]
pub struct ChainIdSet;

/// Typestate marker — API key has not been supplied.
#[derive(Debug, Clone, Copy)]
pub struct ApiKeyUnset;
/// Typestate marker — API key has been supplied.
#[derive(Debug, Clone, Copy)]
pub struct ApiKeySet;

/// Typestate marker — transport has not been supplied.
#[derive(Debug, Clone, Copy)]
pub struct TransportUnset;
/// Typestate marker — transport has been supplied.
#[derive(Debug, Clone, Copy)]
pub struct TransportSet;

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
    chain: Option<SupportedChainId>,
    api_key: Option<String>,
    transport: Option<Arc<dyn HttpTransport + Send + Sync>>,
    transport_policy: Option<SubgraphTransportPolicy>,
    base_urls: Option<SubgraphApiBaseUrls>,
    client: Option<Client>,
    _phantom: PhantomData<(ChainState, ApiKeyState, TransportState)>,
}

impl Default for SubgraphApiBuilder<ChainIdUnset, ApiKeyUnset, TransportUnset> {
    fn default() -> Self {
        Self::new()
    }
}

impl SubgraphApiBuilder<ChainIdUnset, ApiKeyUnset, TransportUnset> {
    /// Creates a fresh builder with no required fields supplied.
    #[must_use]
    pub fn new() -> Self {
        Self {
            chain: None,
            api_key: None,
            transport: None,
            transport_policy: None,
            base_urls: None,
            client: None,
            _phantom: PhantomData,
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
            chain: Some(chain),
            api_key: self.api_key,
            transport: self.transport,
            transport_policy: self.transport_policy,
            base_urls: self.base_urls,
            client: self.client,
            _phantom: PhantomData,
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
            api_key: Some(api_key.into()),
            transport: self.transport,
            transport_policy: self.transport_policy,
            base_urls: self.base_urls,
            client: self.client,
            _phantom: PhantomData,
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
            transport: Some(transport),
            transport_policy: self.transport_policy,
            base_urls: self.base_urls,
            client: self.client,
            _phantom: PhantomData,
        }
    }
}

impl<C, A, T> SubgraphApiBuilder<C, A, T> {
    /// Sets the request retry, rate-limit, and HTTP-client policy bundle.
    ///
    /// When this method is not called, [`SubgraphApiBuilder::build`] uses
    /// [`SubgraphTransportPolicy::default`] which preserves the
    /// documented default behavior.
    #[must_use]
    pub fn policy(mut self, policy: SubgraphTransportPolicy) -> Self {
        self.transport_policy = Some(policy);
        self
    }

    /// Supplies an explicit per-chain base-URL map.
    ///
    /// Each entry overrides the production routing derived from the
    /// supplied API key for the corresponding chain. A `None` value
    /// marks the chain as unsupported on the resulting client.
    #[must_use]
    pub fn base_urls(mut self, base_urls: SubgraphApiBaseUrls) -> Self {
        self.base_urls = Some(base_urls);
        self
    }

    /// Reuses an externally-built [`reqwest::Client`] for the request
    /// pipeline.
    ///
    /// Multi-chain consumers compose one shared `reqwest::Client` (with
    /// its TCP, TLS, and HTTP/2 connection cache) across every
    /// [`SubgraphApi`] they construct. When no shared client is
    /// supplied, the builder constructs a fresh one from the active
    /// [`SubgraphTransportPolicy`].
    #[must_use]
    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    fn finish(self, transport: Arc<dyn HttpTransport + Send + Sync>) -> SubgraphApi {
        let chain = self
            .chain
            .expect("typestate guarantees chain id is supplied at build time");
        let api_key = self
            .api_key
            .expect("typestate guarantees api key is supplied at build time");
        let transport_policy = self.transport_policy.unwrap_or_default();
        let built_client = build_subgraph_client(&transport_policy);
        let client = self.client.unwrap_or(built_client);
        let api_key = Redacted::new(api_key);
        let prod_config = build_prod_config();
        let config = SubgraphConfig {
            chain_id: chain,
            base_urls: self.base_urls,
        };
        SubgraphApi::from_parts(
            client,
            config,
            api_key,
            prod_config,
            transport_policy,
            transport,
        )
    }
}

impl SubgraphApiBuilder<ChainIdSet, ApiKeySet, TransportSet> {
    /// Builds the [`SubgraphApi`] with the supplied chain, API key, and
    /// transport.
    ///
    /// # Panics
    ///
    /// Panics only if the typestate invariant is violated by an
    /// unsupported transmute of the builder's marker types; the
    /// typestate guarantees the transport, chain id, and API key are
    /// all populated by the time this method is reachable, so the panic
    /// is not reachable from safe code.
    #[must_use]
    pub fn build(self) -> SubgraphApi {
        let transport = self
            .transport
            .clone()
            .expect("typestate guarantees a transport is supplied at build time");
        self.finish(transport)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl SubgraphApiBuilder<ChainIdSet, ApiKeySet, TransportUnset> {
    /// Builds the [`SubgraphApi`] with the supplied chain and API key,
    /// defaulting the transport to a native [`ReqwestTransport`] handle.
    ///
    /// This convenience build path is only available on non-`wasm32`
    /// targets; browser consumers must call
    /// [`SubgraphApiBuilder::transport`] with a `FetchTransport` before
    /// reaching [`build`](Self::build).
    ///
    /// # Panics
    ///
    /// Panics only if the validated user-agent for the default native
    /// [`ReqwestTransport`] cannot be encoded as an HTTP header value;
    /// the workspace-shipped default carries a header-safe user-agent
    /// literal so the panic is not reachable from safe code.
    #[must_use]
    pub fn build(self) -> SubgraphApi {
        let user_agent = self
            .transport_policy
            .as_ref()
            .map_or(DEFAULT_SUBGRAPH_USER_AGENT, |policy| {
                policy.client_policy().user_agent()
            })
            .to_owned();
        let config = ReqwestTransportConfig::new(String::new()).with_user_agent(user_agent);
        let transport = ReqwestTransport::new(config)
            .expect("default ReqwestTransport must build with the validated user-agent");
        self.finish(Arc::new(transport))
    }
}

fn build_subgraph_client(policy: &SubgraphTransportPolicy) -> Client {
    Client::builder()
        .user_agent(policy.client_policy().user_agent().to_owned())
        .build()
        .expect("validated subgraph client policy must remain buildable")
}
