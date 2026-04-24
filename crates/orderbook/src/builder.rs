//! Typestate builder for [`OrderBookApi`].
//!
//! [`OrderBookApiBuilder`] is the sole production construction path for an
//! [`OrderBookApi`]. The compiler enforces that the chain id, environment,
//! and HTTP transport are all supplied before [`OrderBookApiBuilder::build`]
//! becomes callable. Optional configuration — request policy, API key, and
//! per-environment base-URL overrides — is layered on through fluent methods
//! that do not affect the typestate.
//!
//! On native targets the builder also exposes [`OrderBookApiBuilder::build`]
//! against the typestate where transport is unset, defaulting the transport
//! to [`ReqwestTransport`](cow_sdk_core::ReqwestTransport) so the common
//! single-target consumer never has to wire a transport explicitly. On
//! `wasm32` targets the default-transport build path is unavailable: the
//! caller MUST supply a `FetchTransport` from `cow-sdk-transport-wasm`
//! through [`OrderBookApiBuilder::transport`] before `build` becomes
//! reachable.
//!
//! # Examples
//!
//! ```
//! use cow_sdk_core::{CowEnv, SupportedChainId};
//! use cow_sdk_orderbook::OrderBookApi;
//!
//! # #[cfg(not(target_arch = "wasm32"))]
//! # {
//! let api = OrderBookApi::builder()
//!     .chain(SupportedChainId::Mainnet)
//!     .environment(CowEnv::Prod)
//!     .build();
//! let _ = api;
//! # }
//! ```

use std::marker::PhantomData;
use std::sync::Arc;

use cow_sdk_core::{ApiBaseUrls, CowEnv, HttpTransport, Redacted, SupportedChainId};
#[cfg(not(target_arch = "wasm32"))]
use cow_sdk_core::{ReqwestTransport, ReqwestTransportConfig};
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Client;

use crate::api::OrderBookApi;
#[cfg(not(target_arch = "wasm32"))]
use crate::request::DEFAULT_ORDERBOOK_USER_AGENT;
use crate::request::{OrderBookTransportPolicy, RequestRateLimiter};
use crate::types::{ApiContext, EnvBaseUrlOverrides};

/// Typestate marker — chain id has not been supplied.
#[derive(Debug, Clone, Copy)]
pub struct ChainIdUnset;
/// Typestate marker — chain id has been supplied.
#[derive(Debug, Clone, Copy)]
pub struct ChainIdSet;

/// Typestate marker — environment has not been supplied.
#[derive(Debug, Clone, Copy)]
pub struct EnvUnset;
/// Typestate marker — environment has been supplied.
#[derive(Debug, Clone, Copy)]
pub struct EnvSet;

/// Typestate marker — transport has not been supplied.
#[derive(Debug, Clone, Copy)]
pub struct TransportUnset;
/// Typestate marker — transport has been supplied.
#[derive(Debug, Clone, Copy)]
pub struct TransportSet;

/// Typestate-checked builder for [`OrderBookApi`].
///
/// The four type parameters track which of the required inputs (chain id,
/// environment, transport) have been supplied. [`OrderBookApiBuilder::build`]
/// is implemented only against the typestates that satisfy the documented
/// preconditions, so calling it with any required field still unset is a
/// compile-time error rather than a runtime failure.
#[derive(Debug, Clone)]
pub struct OrderBookApiBuilder<
    ChainState = ChainIdUnset,
    EnvState = EnvUnset,
    TransportState = TransportUnset,
> {
    chain: Option<SupportedChainId>,
    env: Option<CowEnv>,
    transport: Option<Arc<dyn HttpTransport + Send + Sync>>,
    transport_policy: Option<OrderBookTransportPolicy>,
    api_key: Option<Redacted<String>>,
    base_urls: Option<ApiBaseUrls>,
    env_base_url_overrides: EnvBaseUrlOverrides,
    _phantom: PhantomData<(ChainState, EnvState, TransportState)>,
}

impl Default for OrderBookApiBuilder<ChainIdUnset, EnvUnset, TransportUnset> {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderBookApiBuilder<ChainIdUnset, EnvUnset, TransportUnset> {
    /// Creates a fresh builder with no required fields supplied.
    #[must_use]
    pub fn new() -> Self {
        Self {
            chain: None,
            env: None,
            transport: None,
            transport_policy: None,
            api_key: None,
            base_urls: None,
            env_base_url_overrides: EnvBaseUrlOverrides::default(),
            _phantom: PhantomData,
        }
    }

    /// Seeds the builder from a fully populated [`ApiContext`].
    ///
    /// Transitions the chain and environment typestates to
    /// [`ChainIdSet`] / [`EnvSet`] in one step and propagates the
    /// optional API key and base-URL map onto the builder.
    #[must_use]
    pub fn from_context(
        context: ApiContext,
    ) -> OrderBookApiBuilder<ChainIdSet, EnvSet, TransportUnset> {
        let mut builder = Self::new().chain(context.chain_id).environment(context.env);
        if let Some(api_key) = context.api_key {
            builder.api_key = Some(api_key);
        }
        if let Some(base_urls) = context.base_urls {
            builder = builder.base_urls(base_urls);
        }
        builder
    }
}

impl<E, T> OrderBookApiBuilder<ChainIdUnset, E, T> {
    /// Supplies the chain id for the orderbook context.
    ///
    /// Transitions the chain typestate from [`ChainIdUnset`] to [`ChainIdSet`].
    #[must_use]
    pub fn chain(self, chain: SupportedChainId) -> OrderBookApiBuilder<ChainIdSet, E, T> {
        OrderBookApiBuilder {
            chain: Some(chain),
            env: self.env,
            transport: self.transport,
            transport_policy: self.transport_policy,
            api_key: self.api_key,
            base_urls: self.base_urls,
            env_base_url_overrides: self.env_base_url_overrides,
            _phantom: PhantomData,
        }
    }
}

impl<C, T> OrderBookApiBuilder<C, EnvUnset, T> {
    /// Supplies the deployment environment for the orderbook context.
    ///
    /// Transitions the environment typestate from [`EnvUnset`] to [`EnvSet`].
    #[must_use]
    pub fn environment(self, env: CowEnv) -> OrderBookApiBuilder<C, EnvSet, T> {
        OrderBookApiBuilder {
            chain: self.chain,
            env: Some(env),
            transport: self.transport,
            transport_policy: self.transport_policy,
            api_key: self.api_key,
            base_urls: self.base_urls,
            env_base_url_overrides: self.env_base_url_overrides,
            _phantom: PhantomData,
        }
    }
}

impl<C, E> OrderBookApiBuilder<C, E, TransportUnset> {
    /// Supplies the [`HttpTransport`] dispatch seam.
    ///
    /// Transitions the transport typestate from [`TransportUnset`] to
    /// [`TransportSet`]. The transport is the runtime-neutral injection point
    /// for native and browser HTTP backends; downstream consumers compose the
    /// typed client around the same `Arc<dyn HttpTransport + Send + Sync>`
    /// regardless of target.
    #[must_use]
    pub fn transport(
        self,
        transport: Arc<dyn HttpTransport + Send + Sync>,
    ) -> OrderBookApiBuilder<C, E, TransportSet> {
        OrderBookApiBuilder {
            chain: self.chain,
            env: self.env,
            transport: Some(transport),
            transport_policy: self.transport_policy,
            api_key: self.api_key,
            base_urls: self.base_urls,
            env_base_url_overrides: self.env_base_url_overrides,
            _phantom: PhantomData,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<C, E> OrderBookApiBuilder<C, E, TransportUnset> {
    /// Reuses an externally-built [`reqwest::Client`] as the backing
    /// transport.
    ///
    /// Multi-chain consumers compose one shared [`reqwest::Client`] (with its
    /// TCP, TLS, and HTTP/2 connection cache) across every
    /// [`OrderBookApi`] they construct, which is the recommended pattern
    /// for production bots that issue requests on behalf of several chains
    /// or trading accounts. The shared client is wrapped into a
    /// [`ReqwestTransport`] so every live request still flows through the
    /// single `HttpTransport` dispatch seam; the transport resolves paths
    /// against the empty base URL so the orderbook request helpers keep
    /// building full URLs from the API context.
    #[must_use]
    pub fn client(self, client: Client) -> OrderBookApiBuilder<C, E, TransportSet> {
        let transport: Arc<dyn HttpTransport + Send + Sync> =
            Arc::new(ReqwestTransport::with_client(client, ""));
        self.transport(transport)
    }
}

impl<C, E, T> OrderBookApiBuilder<C, E, T> {
    /// Sets the request retry, rate-limit, and HTTP-client policy bundle.
    ///
    /// When this method is not called, [`OrderBookApiBuilder::build`] uses
    /// [`OrderBookTransportPolicy::default`] which preserves the documented
    /// rate-limit and retry behavior.
    #[must_use]
    pub fn policy(mut self, policy: OrderBookTransportPolicy) -> Self {
        self.transport_policy = Some(policy);
        self
    }

    /// Attaches a partner-route API key forwarded as the `X-API-Key` header.
    #[must_use]
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(Redacted::new(api_key.into()));
        self
    }

    /// Supplies an explicit per-chain base-URL map for the resolved API
    /// context.
    #[must_use]
    pub fn base_urls(mut self, base_urls: ApiBaseUrls) -> Self {
        self.base_urls = Some(base_urls);
        self
    }

    /// Adds a per-environment base-URL override that takes precedence over
    /// URLs resolved from the API context.
    #[must_use]
    pub fn env_base_url(mut self, env: CowEnv, base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        self.env_base_url_overrides
            .set(env, normalize_base_url(&base_url));
        self
    }

    /// Adds a base-URL override for the environment already supplied to the
    /// builder.
    ///
    /// Convenience over [`OrderBookApiBuilder::env_base_url`] when the caller
    /// has just configured the environment through
    /// [`OrderBookApiBuilder::environment`] or
    /// [`OrderBookApiBuilder::from_context`] and wants to anchor the override
    /// to the same environment.
    ///
    /// # Panics
    ///
    /// Panics when the environment has not been supplied to the builder.
    #[must_use]
    pub fn base_url(self, base_url: impl Into<String>) -> Self {
        let env = self
            .env
            .expect("base_url requires environment to be supplied first via `.environment(...)` or `.from_context(...)`");
        self.env_base_url(env, base_url)
    }

    fn finish(self, transport: Arc<dyn HttpTransport + Send + Sync>) -> OrderBookApi {
        let chain = self
            .chain
            .expect("typestate guarantees chain id is supplied at build time");
        let env = self
            .env
            .expect("typestate guarantees environment is supplied at build time");
        let transport_policy = self.transport_policy.unwrap_or_default();
        let rate_limiter = RequestRateLimiter::new(transport_policy.request_policy().rate_limit);
        let mut context = ApiContext::new(chain, env);
        if let Some(api_key) = self.api_key {
            context.api_key = Some(api_key);
        }
        if let Some(base_urls) = self.base_urls {
            context.base_urls = Some(base_urls);
        }
        OrderBookApi::from_parts(
            context,
            transport_policy,
            rate_limiter,
            self.env_base_url_overrides,
            transport,
        )
    }
}

impl OrderBookApiBuilder<ChainIdSet, EnvSet, TransportSet> {
    /// Builds the [`OrderBookApi`] with the supplied chain, environment, and
    /// transport.
    ///
    /// # Panics
    ///
    /// Panics only if the typestate invariant is violated by an
    /// unsupported transmute of the builder's marker types; the typestate
    /// guarantees the transport, chain id, and environment are all
    /// populated by the time this method is reachable, so the panic is
    /// not reachable from safe code.
    #[must_use]
    pub fn build(self) -> OrderBookApi {
        let transport = self
            .transport
            .clone()
            .expect("typestate guarantees a transport is supplied at build time");
        self.finish(transport)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl OrderBookApiBuilder<ChainIdSet, EnvSet, TransportUnset> {
    /// Builds the [`OrderBookApi`] with the supplied chain and environment,
    /// defaulting the transport to a native [`ReqwestTransport`] handle.
    ///
    /// This convenience build path is only available on non-`wasm32` targets;
    /// browser consumers must call
    /// [`OrderBookApiBuilder::transport`] with a `FetchTransport` before
    /// reaching `build`.
    ///
    /// # Panics
    ///
    /// Panics only if the validated user-agent for the default native
    /// [`ReqwestTransport`] cannot be encoded as an HTTP header value;
    /// the workspace-shipped default carries a header-safe user-agent
    /// literal so the panic is not reachable from safe code.
    #[must_use]
    pub fn build(self) -> OrderBookApi {
        let user_agent = self
            .transport_policy
            .as_ref()
            .map_or(DEFAULT_ORDERBOOK_USER_AGENT, |policy| {
                policy.client_policy().user_agent()
            })
            .to_owned();
        let timeout = self
            .transport_policy
            .as_ref()
            .and_then(|policy| policy.client_policy().timeout());
        let mut config = ReqwestTransportConfig::new(String::new()).with_user_agent(user_agent);
        if let Some(timeout) = timeout {
            config = config.with_timeout(timeout);
        }
        let transport = ReqwestTransport::new(config)
            .expect("default ReqwestTransport must build with the validated user-agent");
        self.finish(Arc::new(transport))
    }
}

fn normalize_base_url(base_url: &str) -> String {
    base_url.trim_end_matches('/').to_owned()
}
