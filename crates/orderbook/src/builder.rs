//! Typestate builder for [`OrderbookApi`].
//!
//! [`OrderbookApiBuilder`] is the sole production construction path for an
//! [`OrderbookApi`]. The compiler enforces that the chain id, environment,
//! and HTTP transport are all supplied before [`OrderbookApiBuilder::build`]
//! becomes callable. Optional configuration — request policy, API key, and
//! per-environment base-URL overrides — is layered on through fluent methods
//! that do not affect the typestate.
//!
//! The typestate markers carry the value they prove is present (chain id,
//! environment, transport), so the build terminals read the configured value
//! directly from the type-level marker instead of unwrapping an `Option`. A
//! misconstructed builder is a compile error, and the terminals contain no
//! typestate-guard `expect`.
//!
//! On native targets the builder also exposes [`OrderbookApiBuilder::build`]
//! against the typestate where transport is unset, defaulting the transport
//! to [`ReqwestTransport`](cow_sdk_core::ReqwestTransport) so the common
//! single-target consumer never has to wire a transport explicitly. On
//! `wasm32` targets the default-transport build path is unavailable: the
//! caller MUST supply a `FetchTransport` from `cow-sdk-transport-wasm`
//! through [`OrderbookApiBuilder::transport`] before `build` becomes
//! reachable.
//!
//! # Examples
//!
//! ```
//! use cow_sdk_core::{CowEnv, SupportedChainId};
//! use cow_sdk_orderbook::OrderbookApi;
//!
//! # #[cfg(not(target_arch = "wasm32"))]
//! # {
//! let orderbook = OrderbookApi::builder()
//!     .chain(SupportedChainId::Mainnet)
//!     .env(CowEnv::Prod)
//!     .build()
//!     .expect("orderbook client builds with canonical defaults");
//! let _ = orderbook;
//! # }
//! ```

use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use cow_sdk_core::transport::policy::DEFAULT_ORDERBOOK_USER_AGENT;
use cow_sdk_core::transport::policy::TransportPolicy;
use cow_sdk_core::{
    ApiBaseUrls, CowEnv, ExternalHostPolicy, HttpTransport, Redacted, SupportedChainId,
    canonical_orderbook_hosts, validate_external_service_url,
};
#[cfg(not(target_arch = "wasm32"))]
use cow_sdk_core::{ReqwestTransport, ReqwestTransportConfig};
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Client;

use crate::api::OrderbookApi;
use crate::error::OrderbookError;
use crate::types::{ApiContext, EnvBaseUrlOverrides};

/// Typestate marker — chain id has not been supplied.
#[derive(Debug, Clone, Copy)]
pub struct ChainIdUnset(());
/// Typestate marker carrying the supplied chain id.
#[derive(Debug, Clone, Copy)]
pub struct ChainIdSet(SupportedChainId);

/// Typestate marker — environment has not been supplied.
#[derive(Debug, Clone, Copy)]
pub struct EnvUnset(());
/// Typestate marker carrying the supplied environment.
#[derive(Debug, Clone, Copy)]
pub struct EnvSet(CowEnv);

/// Typestate marker — transport has not been supplied.
#[derive(Debug, Clone, Copy)]
pub struct TransportUnset(());
/// Typestate marker carrying the supplied HTTP transport.
#[derive(Debug, Clone)]
pub struct TransportSet(Arc<dyn HttpTransport + Send + Sync>);

/// Typestate-checked builder for [`OrderbookApi`].
///
/// The three type parameters track which of the required inputs (chain id,
/// environment, transport) have been supplied, and the "set" markers carry the
/// value. [`OrderbookApiBuilder::build`] is implemented only against the
/// typestates that satisfy the documented preconditions, so calling it with
/// any required field still unset is a compile-time error rather than a
/// runtime failure.
#[derive(Debug, Clone)]
pub struct OrderbookApiBuilder<
    ChainState = ChainIdUnset,
    EnvState = EnvUnset,
    TransportState = TransportUnset,
> {
    chain: ChainState,
    env: EnvState,
    transport: TransportState,
    transport_policy: Option<TransportPolicy>,
    api_key: Option<Redacted<String>>,
    base_urls: Option<ApiBaseUrls>,
    env_base_url_overrides: EnvBaseUrlOverrides,
    host_policy: ExternalHostPolicy,
}

impl Default for OrderbookApiBuilder<ChainIdUnset, EnvUnset, TransportUnset> {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderbookApiBuilder<ChainIdUnset, EnvUnset, TransportUnset> {
    /// Creates a fresh builder with no required fields supplied.
    #[must_use]
    pub fn new() -> Self {
        Self {
            chain: ChainIdUnset(()),
            env: EnvUnset(()),
            transport: TransportUnset(()),
            transport_policy: None,
            api_key: None,
            base_urls: None,
            env_base_url_overrides: EnvBaseUrlOverrides::default(),
            host_policy: ExternalHostPolicy::Default,
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
    ) -> OrderbookApiBuilder<ChainIdSet, EnvSet, TransportUnset> {
        let mut builder = Self::new().chain(context.chain_id).env(context.env);
        if let Some(api_key) = context.api_key {
            builder.api_key = Some(api_key);
        }
        if let Some(base_urls) = context.base_urls {
            builder = builder.base_urls(base_urls);
        }
        builder
    }
}

impl<E, T> OrderbookApiBuilder<ChainIdUnset, E, T> {
    /// Supplies the chain id for the orderbook context.
    ///
    /// Transitions the chain typestate from [`ChainIdUnset`] to [`ChainIdSet`],
    /// which carries the supplied chain id.
    #[must_use]
    pub fn chain(self, chain: SupportedChainId) -> OrderbookApiBuilder<ChainIdSet, E, T> {
        OrderbookApiBuilder {
            chain: ChainIdSet(chain),
            env: self.env,
            transport: self.transport,
            transport_policy: self.transport_policy,
            api_key: self.api_key,
            base_urls: self.base_urls,
            env_base_url_overrides: self.env_base_url_overrides,
            host_policy: self.host_policy,
        }
    }
}

impl<C, T> OrderbookApiBuilder<C, EnvUnset, T> {
    /// Supplies the deployment environment for the orderbook context.
    ///
    /// Transitions the environment typestate from [`EnvUnset`] to [`EnvSet`],
    /// which carries the supplied environment.
    #[must_use]
    pub fn env(self, env: CowEnv) -> OrderbookApiBuilder<C, EnvSet, T> {
        OrderbookApiBuilder {
            chain: self.chain,
            env: EnvSet(env),
            transport: self.transport,
            transport_policy: self.transport_policy,
            api_key: self.api_key,
            base_urls: self.base_urls,
            env_base_url_overrides: self.env_base_url_overrides,
            host_policy: self.host_policy,
        }
    }
}

impl<C, E> OrderbookApiBuilder<C, E, TransportUnset> {
    /// Supplies the [`HttpTransport`] dispatch seam.
    ///
    /// Transitions the transport typestate from [`TransportUnset`] to
    /// [`TransportSet`], which carries the supplied transport. The transport is
    /// the runtime-neutral injection point for native and browser HTTP
    /// backends; downstream consumers compose the typed client around the same
    /// `Arc<dyn HttpTransport + Send + Sync>` regardless of target.
    #[must_use]
    pub fn transport(
        self,
        transport: Arc<dyn HttpTransport + Send + Sync>,
    ) -> OrderbookApiBuilder<C, E, TransportSet> {
        OrderbookApiBuilder {
            chain: self.chain,
            env: self.env,
            transport: TransportSet(transport),
            transport_policy: self.transport_policy,
            api_key: self.api_key,
            base_urls: self.base_urls,
            env_base_url_overrides: self.env_base_url_overrides,
            host_policy: self.host_policy,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<C, E> OrderbookApiBuilder<C, E, TransportUnset> {
    /// Reuses an externally-built [`reqwest::Client`] as the backing
    /// transport.
    ///
    /// Multi-chain consumers compose one shared [`reqwest::Client`] (with its
    /// TCP, TLS, and HTTP/2 connection cache) across every
    /// [`OrderbookApi`] they construct, which is the recommended pattern
    /// for production bots that issue requests on behalf of several chains
    /// or trading accounts. The shared client is wrapped into a
    /// [`ReqwestTransport`] so every live request still flows through the
    /// single `HttpTransport` dispatch seam; the transport resolves paths
    /// against the empty base URL so the orderbook request helpers keep
    /// building full URLs from the API context.
    #[must_use]
    pub fn client(self, client: Client) -> OrderbookApiBuilder<C, E, TransportSet> {
        let transport: Arc<dyn HttpTransport + Send + Sync> =
            Arc::new(ReqwestTransport::with_client(client, ""));
        self.transport(transport)
    }
}

impl<C, E, T> OrderbookApiBuilder<C, E, T> {
    /// Sets the request retry, rate-limit, and HTTP-client policy bundle.
    ///
    /// When this method is not called, [`OrderbookApiBuilder::build`] uses
    /// [`TransportPolicy::default_orderbook`] which preserves the documented
    /// rate-limit and retry behavior.
    #[must_use]
    pub fn transport_policy(mut self, policy: TransportPolicy) -> Self {
        self.transport_policy = Some(policy);
        self
    }

    /// Attaches a partner-route API key forwarded as the `X-API-Key` header.
    #[must_use]
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(Redacted::new(api_key.into()));
        self
    }

    /// Sets the external host policy used to validate explicit orderbook
    /// service endpoint overrides.
    ///
    /// The default accepts only the SDK's canonical `CoW Protocol` orderbook
    /// hosts. Local fixtures should use [`ExternalHostPolicy::Test`], and
    /// private mirrors should use [`ExternalHostPolicy::Allow`] with the
    /// mirror host.
    #[must_use]
    pub fn external_host_policy(mut self, policy: ExternalHostPolicy) -> Self {
        self.host_policy = policy;
        self
    }

    /// Supplies an explicit per-chain base-URL map for the resolved API
    /// context.
    #[must_use]
    pub fn base_urls(mut self, base_urls: impl Into<ApiBaseUrls>) -> Self {
        self.base_urls = Some(base_urls.into());
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
}

impl<C, T> OrderbookApiBuilder<C, EnvSet, T> {
    /// Adds a base-URL override for the environment already supplied to the
    /// builder.
    ///
    /// Convenience over [`OrderbookApiBuilder::env_base_url`] that reuses the
    /// environment carried by the [`EnvSet`] typestate. The method is reachable
    /// only after the environment has been supplied through
    /// [`OrderbookApiBuilder::env`] or
    /// [`OrderbookApiBuilder::from_context`], so calling it before the
    /// environment is set is a compile error rather than a runtime panic.
    #[must_use]
    pub fn base_url(self, base_url: impl Into<String>) -> Self {
        let env = self.env.0;
        self.env_base_url(env, base_url)
    }
}

impl<T> OrderbookApiBuilder<ChainIdSet, EnvSet, T> {
    /// Finalizes the builder once a transport has been resolved.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] when explicit base-URL overrides fail the
    /// configured external host policy.
    fn finish(
        self,
        transport: Arc<dyn HttpTransport + Send + Sync>,
    ) -> Result<OrderbookApi, OrderbookError> {
        validate_orderbook_base_urls(
            self.base_urls.as_ref(),
            &self.env_base_url_overrides,
            &self.host_policy,
        )?;

        let chain = self.chain.0;
        let env = self.env.0;
        let transport_policy = self
            .transport_policy
            .unwrap_or_else(TransportPolicy::default_orderbook);
        let rate_limiter = transport_policy.rate_limit().clone();
        let mut context = ApiContext::new(chain, env);
        if let Some(api_key) = self.api_key {
            context.api_key = Some(api_key);
        }
        if let Some(base_urls) = self.base_urls {
            context.base_urls = Some(base_urls);
        }
        Ok(OrderbookApi::from_parts(
            context,
            transport_policy,
            rate_limiter,
            self.env_base_url_overrides,
            transport,
        ))
    }
}

impl OrderbookApiBuilder<ChainIdSet, EnvSet, TransportSet> {
    /// Builds the [`OrderbookApi`] with the supplied chain, environment, and
    /// transport.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] when explicit base-URL overrides fail the
    /// configured external host policy.
    pub fn build(self) -> Result<OrderbookApi, OrderbookError> {
        let transport = self.transport.0.clone();
        self.finish(transport)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl OrderbookApiBuilder<ChainIdSet, EnvSet, TransportUnset> {
    /// Builds the [`OrderbookApi`] with the supplied chain and environment,
    /// defaulting the transport to a native [`ReqwestTransport`] handle.
    ///
    /// This convenience build path is only available on non-`wasm32` targets;
    /// browser consumers must call
    /// [`OrderbookApiBuilder::transport`] with a `FetchTransport` before
    /// reaching `build`.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] when explicit base-URL overrides fail the
    /// configured external host policy, or when the configured transport policy
    /// yields a user-agent that cannot be encoded as an HTTP header value while
    /// constructing the default native [`ReqwestTransport`].
    pub fn build(self) -> Result<OrderbookApi, OrderbookError> {
        let user_agent = self
            .transport_policy
            .as_ref()
            .map_or(DEFAULT_ORDERBOOK_USER_AGENT, |policy| policy.user_agent())
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
        let transport = ReqwestTransport::new(config)?;
        self.finish(Arc::new(transport))
    }
}

fn validate_orderbook_base_urls(
    base_urls: Option<&ApiBaseUrls>,
    env_base_url_overrides: &EnvBaseUrlOverrides,
    policy: &ExternalHostPolicy,
) -> Result<(), OrderbookError> {
    if let Some(base_urls) = base_urls {
        for base_url in base_urls.as_inner().values() {
            validate_external_service_url(base_url, canonical_orderbook_hosts(), policy)?;
        }
    }

    for base_url in [
        env_base_url_overrides.prod.as_ref(),
        env_base_url_overrides.staging.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        validate_external_service_url(base_url.as_inner(), canonical_orderbook_hosts(), policy)?;
    }

    Ok(())
}

fn normalize_base_url(base_url: &str) -> String {
    base_url.trim_end_matches('/').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typestate_markers_are_sealed_against_external_construction() {
        // These constructors are visible only inside this module because each
        // marker's field is private; external crates cannot construct them, so
        // the typestate cannot be forged from outside the crate. The "set"
        // markers carry their value through the same private field.
        let _ = ChainIdUnset(());
        let _ = ChainIdSet(SupportedChainId::Mainnet);
        let _ = EnvUnset(());
        let _ = EnvSet(CowEnv::Prod);
        let _ = TransportUnset(());
    }
}
