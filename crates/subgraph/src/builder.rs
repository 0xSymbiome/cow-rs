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
//! through [`SubgraphApiBuilder::transport`] before `build` becomes
//! reachable.
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
//!     .build()
//!     .expect("subgraph client builds with canonical defaults");
//! let _ = api;
//! # }
//! ```

use std::marker::PhantomData;
use std::sync::Arc;

use cow_sdk_core::{
    ExternalHostPolicy, HttpTransport, Redacted, SupportedChainId, canonical_subgraph_hosts,
    validate_external_service_url,
};
#[cfg(not(target_arch = "wasm32"))]
use cow_sdk_core::{ReqwestTransport, ReqwestTransportConfig};
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Client;

#[cfg(not(target_arch = "wasm32"))]
use crate::api::DEFAULT_SUBGRAPH_USER_AGENT;
use crate::api::{
    SubgraphApi, SubgraphApiBaseUrls, SubgraphConfig, SubgraphTransportPolicy, build_prod_config,
};
use crate::error::SubgraphError;

/// Typestate marker — chain id has not been supplied.
#[derive(Debug, Clone, Copy)]
pub struct ChainIdUnset(());
/// Typestate marker — chain id has been supplied.
#[derive(Debug, Clone, Copy)]
pub struct ChainIdSet(());

/// Typestate marker — API key has not been supplied.
#[derive(Debug, Clone, Copy)]
pub struct ApiKeyUnset(());
/// Typestate marker — API key has been supplied.
#[derive(Debug, Clone, Copy)]
pub struct ApiKeySet(());

/// Typestate marker — transport has not been supplied.
#[derive(Debug, Clone, Copy)]
pub struct TransportUnset(());
/// Typestate marker — transport has been supplied.
#[derive(Debug, Clone, Copy)]
pub struct TransportSet(());

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
    api_key: Option<Redacted<String>>,
    transport: Option<Arc<dyn HttpTransport + Send + Sync>>,
    transport_policy: Option<SubgraphTransportPolicy>,
    base_urls: Option<SubgraphApiBaseUrls>,
    host_policy: ExternalHostPolicy,
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
            host_policy: ExternalHostPolicy::Default,
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
            host_policy: self.host_policy,
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
            api_key: Some(Redacted::new(api_key.into())),
            transport: self.transport,
            transport_policy: self.transport_policy,
            base_urls: self.base_urls,
            host_policy: self.host_policy,
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
            host_policy: self.host_policy,
            _phantom: PhantomData,
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
    /// [`SubgraphTransportPolicy::default`] which preserves the
    /// documented default behavior.
    #[must_use]
    pub fn policy(mut self, policy: SubgraphTransportPolicy) -> Self {
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
    pub fn with_external_host_policy(mut self, policy: ExternalHostPolicy) -> Self {
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

    /// Finalizes the builder once a transport has been selected.
    ///
    /// # Errors
    ///
    /// Returns [`SubgraphError`] when explicit base-URL overrides fail the
    /// configured external host policy.
    ///
    /// # Panics
    ///
    /// Panics only if the typestate marker invariants are bypassed and the
    /// chain id or API key was not supplied before finalization.
    fn finish(
        self,
        transport: Arc<dyn HttpTransport + Send + Sync>,
    ) -> Result<SubgraphApi, SubgraphError> {
        validate_subgraph_base_urls(self.base_urls.as_ref(), &self.host_policy)?;

        let chain = self
            .chain
            // SAFETY: finish is reached only by typestate build paths that set
            // the chain marker.
            .expect("typestate guarantees chain id is supplied at build time");
        let api_key = self
            .api_key
            // SAFETY: finish is reached only by typestate build paths that set
            // the API-key marker.
            .expect("typestate guarantees api key is supplied at build time");
        let transport_policy = self.transport_policy.unwrap_or_default();
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
    ///
    /// # Panics
    ///
    /// Panics only if the typestate marker is bypassed and the required
    /// transport is missing at build time.
    pub fn build(self) -> Result<SubgraphApi, SubgraphError> {
        let transport = self
            .transport
            .clone()
            // SAFETY: this impl is only available for the TransportSet
            // typestate, so a missing transport means the marker invariant was
            // bypassed.
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
    /// reaching `build`.
    ///
    /// # Errors
    ///
    /// Returns [`SubgraphError`] when explicit base-URL overrides fail the
    /// configured external host policy.
    ///
    /// # Panics
    ///
    /// Panics only if the validated user-agent for the default native
    /// [`ReqwestTransport`] cannot be encoded as an HTTP header value;
    /// the workspace-shipped default carries a header-safe user-agent
    /// literal so the panic is not reachable from safe code.
    pub fn build(self) -> Result<SubgraphApi, SubgraphError> {
        let user_agent = self
            .transport_policy
            .as_ref()
            .map_or(DEFAULT_SUBGRAPH_USER_AGENT, |policy| {
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
            // SAFETY: the default user-agent comes from a validated static
            // literal or from an existing HttpClientPolicy.
            .expect("default ReqwestTransport must build with the validated user-agent");
        self.finish(Arc::new(transport))
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typestate_markers_are_sealed_against_external_construction() {
        // These constructors are visible only inside this module because the
        // tuple field is private; external callers cannot write `Marker(())`.
        let _ = ChainIdUnset(());
        let _ = ChainIdSet(());
        let _ = ApiKeyUnset(());
        let _ = ApiKeySet(());
        let _ = TransportUnset(());
        let _ = TransportSet(());
    }
}
