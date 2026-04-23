//! Typed subgraph client configuration and query execution.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use cow_sdk_core::{
    HttpClientPolicy, HttpTransport, Redacted, SupportedChainId, TransportError,
    TransportErrorClass,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use crate::{
    builder::{ApiKeyUnset, ChainIdUnset, SubgraphApiBuilder, TransportUnset},
    error::{SubgraphError, SubgraphGraphQlError, SubgraphRequestErrorContext},
    queries::{LAST_DAYS_VOLUME_QUERY, LAST_HOURS_VOLUME_QUERY, TOTALS_QUERY},
    types::{
        LastDaysVolumeResponse, LastHoursVolumeResponse, SubgraphQueryRequest, Total,
        TotalsResponse,
    },
};

const SUBGRAPH_BASE_URL: &str = "https://gateway.thegraph.com/api/";
const REDACTED_API_KEY_SEGMENT: &str = "<redacted>";
const CUSTOM_OVERRIDE_ROUTE_IDENTITY: &str = "<custom override>";

/// Human-readable name for the `CoW` Protocol subgraph service.
pub const API_NAME: &str = "CoW Protocol Subgraph";
/// Default user-agent used by the subgraph client.
pub const DEFAULT_SUBGRAPH_USER_AGENT: &str =
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Base-URL overrides keyed by chain id.
///
/// A `Some(url)` entry enables that chain and routes requests to `url`. A
/// `None` entry marks the chain as unsupported for the current configuration.
pub type SubgraphApiBaseUrls = BTreeMap<SupportedChainId, Option<String>>;

/// Static subgraph client configuration.
///
/// The default configuration targets mainnet production routes derived from the
/// API key supplied when constructing [`SubgraphApi`].
#[derive(Clone, PartialEq, Eq)]
pub struct SubgraphConfig {
    /// Active chain id used for helper methods and generic queries.
    pub chain_id: SupportedChainId,
    /// Optional per-chain base URL overrides.
    ///
    /// When this is `None`, [`SubgraphApi`] uses its API-key-derived production
    /// routing map internally and exposes only redacted route identity through
    /// its stable public metadata.
    pub base_urls: Option<SubgraphApiBaseUrls>,
}

impl fmt::Debug for SubgraphConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubgraphConfig")
            .field("chain_id", &self.chain_id)
            .field("base_urls", &sanitized_base_urls(self.base_urls.as_ref()))
            .finish()
    }
}

impl Default for SubgraphConfig {
    fn default() -> Self {
        Self {
            chain_id: SupportedChainId::Mainnet,
            base_urls: None,
        }
    }
}

/// Per-call overrides for [`SubgraphConfig`].
#[derive(Clone, Default, PartialEq, Eq)]
pub struct SubgraphConfigOverride {
    /// Optional chain override for a single request.
    pub chain_id: Option<SupportedChainId>,
    /// Optional base-URL map override for a single request.
    pub base_urls: Option<SubgraphApiBaseUrls>,
}

impl fmt::Debug for SubgraphConfigOverride {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubgraphConfigOverride")
            .field("chain_id", &self.chain_id)
            .field("base_urls", &sanitized_base_urls(self.base_urls.as_ref()))
            .finish()
    }
}

/// Shared HTTP client policy for subgraph requests.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubgraphTransportPolicy {
    http_policy: HttpClientPolicy,
}

impl Default for SubgraphTransportPolicy {
    fn default() -> Self {
        Self {
            http_policy: HttpClientPolicy::new(DEFAULT_SUBGRAPH_USER_AGENT)
                .expect("static subgraph user-agent must remain valid"),
        }
    }
}

impl SubgraphTransportPolicy {
    /// Creates a transport policy from an explicit HTTP client policy.
    #[must_use]
    pub const fn new(client: HttpClientPolicy) -> Self {
        Self {
            http_policy: client,
        }
    }

    /// Returns the shared HTTP client policy.
    #[must_use]
    pub const fn client_policy(&self) -> &HttpClientPolicy {
        &self.http_policy
    }

    /// Returns a copy of this transport policy with a new HTTP client policy.
    #[must_use]
    pub fn with_client_policy(mut self, client: HttpClientPolicy) -> Self {
        self.http_policy = client;
        self
    }
}

/// Typed client for `CoW` Protocol subgraph queries.
///
/// The client owns API-key-derived production routing, optional per-instance
/// configuration overrides, and a typed raw-query path through
/// [`SubgraphQueryRequest`]. Public metadata exposes only redacted production
/// route identity or sanitized override identity.
#[derive(Clone)]
pub struct SubgraphApi {
    config: SubgraphConfig,
    api_key: Redacted<String>,
    prod_config: SubgraphApiBaseUrls,
    transport_policy: SubgraphTransportPolicy,
    transport: Arc<dyn HttpTransport + Send + Sync>,
}

impl fmt::Debug for SubgraphApi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let supported_prod_chains = self
            .prod_config
            .iter()
            .filter_map(|(chain_id, base_url)| base_url.as_ref().map(|_| chain_id))
            .collect::<Vec<_>>();

        f.debug_struct("SubgraphApi")
            .field("config", &self.config)
            .field("supported_prod_chains", &supported_prod_chains)
            .field("transport_policy", &self.transport_policy)
            .finish_non_exhaustive()
    }
}

impl SubgraphApi {
    /// Returns a fresh [`SubgraphApiBuilder`] for typestate-checked
    /// construction.
    ///
    /// The builder enforces at compile time that the chain id, API key,
    /// and HTTP transport are all supplied before
    /// [`SubgraphApiBuilder::build`] becomes callable. On native targets
    /// the builder also exposes a `build` overload that defaults the
    /// transport to the
    /// [`ReqwestTransport`](cow_sdk_core::ReqwestTransport) when the
    /// caller does not supply one.
    #[must_use]
    pub fn builder() -> SubgraphApiBuilder<ChainIdUnset, ApiKeyUnset, TransportUnset> {
        SubgraphApiBuilder::new()
    }

    /// Crate-private constructor used by [`SubgraphApiBuilder::build`].
    #[must_use]
    pub(crate) fn from_parts(
        config: SubgraphConfig,
        api_key: Redacted<String>,
        prod_config: SubgraphApiBaseUrls,
        transport_policy: SubgraphTransportPolicy,
        transport: Arc<dyn HttpTransport + Send + Sync>,
    ) -> Self {
        Self {
            config,
            api_key,
            prod_config,
            transport_policy,
            transport,
        }
    }

    /// Returns the [`HttpTransport`] handle injected at construction time.
    ///
    /// Downstream consumers reach the runtime-neutral transport seam
    /// through this accessor when they need to share the same transport
    /// with other typed clients constructed from the workspace.
    #[must_use]
    pub fn transport(&self) -> &Arc<dyn HttpTransport + Send + Sync> {
        &self.transport
    }

    /// Returns the human-readable API name for this client.
    #[must_use]
    pub const fn api_name(&self) -> &'static str {
        API_NAME
    }

    /// Returns the static configuration stored in this client.
    #[must_use]
    pub const fn config(&self) -> &SubgraphConfig {
        &self.config
    }

    /// Returns the redacted production route-identity map.
    ///
    /// Unsupported chains remain present with `None` values so the support
    /// posture stays explicit, while the Graph API key remains private to the
    /// request-routing path.
    #[must_use]
    pub const fn prod_config(&self) -> &SubgraphApiBaseUrls {
        &self.prod_config
    }

    /// Returns the active transport policy.
    #[must_use]
    pub const fn transport_policy(&self) -> &SubgraphTransportPolicy {
        &self.transport_policy
    }

    /// Returns the shared HTTP client policy embedded in the transport policy.
    #[must_use]
    pub const fn client_policy(&self) -> &HttpClientPolicy {
        self.transport_policy.client_policy()
    }

    /// Returns a copy of this client with a different transport policy.
    ///
    /// The injected HTTP transport continues to carry every live request;
    /// replacing the policy updates the user-agent and timeout inputs that
    /// the request helper threads into the transport call.
    #[must_use]
    pub fn with_transport_policy(mut self, transport_policy: SubgraphTransportPolicy) -> Self {
        self.transport_policy = transport_policy;
        self
    }

    /// Fetches the first totals row from the canonical totals query.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns [`SubgraphError::NoTotalsFound`] when the response contains no
    /// totals rows, or any transport, HTTP, GraphQL, serialization, missing
    /// data, or unsupported-network error surfaced by the underlying query.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.config().chain_id,
                endpoint = "subgraph.totals",
                method = "POST",
            ),
        ),
    )]
    pub async fn get_totals(&self) -> Result<Total, SubgraphError> {
        self.get_totals_with_config(SubgraphConfigOverride::default())
            .await
    }

    /// Fetches the first totals row with per-call configuration overrides.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns [`SubgraphError::NoTotalsFound`] when the response contains no
    /// totals rows, or any transport, HTTP, GraphQL, serialization, missing
    /// data, or unsupported-network error surfaced by the underlying query.
    pub async fn get_totals_with_config(
        &self,
        config_override: SubgraphConfigOverride,
    ) -> Result<Total, SubgraphError> {
        let response: TotalsResponse = self
            .run_query_with_config(
                SubgraphQueryRequest::new(TOTALS_QUERY).with_operation_name("Totals"),
                config_override,
            )
            .await?;

        response
            .totals
            .into_iter()
            .next()
            .ok_or(SubgraphError::NoTotalsFound)
    }

    /// Fetches daily volume rows for the last `days` entries.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns any transport, HTTP, GraphQL, serialization, missing-data, or
    /// unsupported-network error surfaced by the underlying query.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.config().chain_id,
                endpoint = "subgraph.last_days_volume",
                method = "POST",
            ),
        ),
    )]
    pub async fn get_last_days_volume(
        &self,
        days: u32,
    ) -> Result<LastDaysVolumeResponse, SubgraphError> {
        self.get_last_days_volume_with_config(days, SubgraphConfigOverride::default())
            .await
    }

    /// Fetches daily volume rows for the last `days` entries with per-call overrides.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns any transport, HTTP, GraphQL, serialization, missing-data, or
    /// unsupported-network error surfaced by the underlying query.
    pub async fn get_last_days_volume_with_config(
        &self,
        days: u32,
        config_override: SubgraphConfigOverride,
    ) -> Result<LastDaysVolumeResponse, SubgraphError> {
        self.run_query_with_config(
            SubgraphQueryRequest::new(LAST_DAYS_VOLUME_QUERY)
                .with_variables(json!({ "days": days }))
                .with_operation_name("LastDaysVolume"),
            config_override,
        )
        .await
    }

    /// Fetches hourly volume rows for the last `hours` entries.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns any transport, HTTP, GraphQL, serialization, missing-data, or
    /// unsupported-network error surfaced by the underlying query.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.config().chain_id,
                endpoint = "subgraph.last_hours_volume",
                method = "POST",
            ),
        ),
    )]
    pub async fn get_last_hours_volume(
        &self,
        hours: u32,
    ) -> Result<LastHoursVolumeResponse, SubgraphError> {
        self.get_last_hours_volume_with_config(hours, SubgraphConfigOverride::default())
            .await
    }

    /// Fetches hourly volume rows for the last `hours` entries with per-call overrides.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns any transport, HTTP, GraphQL, serialization, missing-data, or
    /// unsupported-network error surfaced by the underlying query.
    pub async fn get_last_hours_volume_with_config(
        &self,
        hours: u32,
        config_override: SubgraphConfigOverride,
    ) -> Result<LastHoursVolumeResponse, SubgraphError> {
        self.run_query_with_config(
            SubgraphQueryRequest::new(LAST_HOURS_VOLUME_QUERY)
                .with_variables(json!({ "hours": hours }))
                .with_operation_name("LastHoursVolume"),
            config_override,
        )
        .await
    }

    /// Executes an explicit raw GraphQL request against the configured subgraph endpoint.
    ///
    /// Anonymous single-operation documents may omit `operation_name`.
    /// Multi-operation documents must provide an explicit operation name
    /// through [`SubgraphQueryRequest`]. Callers that need cooperative
    /// cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns [`SubgraphError`] for transport failures, non-success HTTP
    /// status codes, GraphQL error payloads, response-decoding failures,
    /// missing `data`, or unsupported networks.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.config().chain_id,
                endpoint = "subgraph.run_query",
                method = "POST",
            ),
        ),
    )]
    pub async fn run_query<T, R>(&self, request: R) -> Result<T, SubgraphError>
    where
        T: DeserializeOwned,
        R: Into<SubgraphQueryRequest>,
    {
        self.run_query_with_config(request, SubgraphConfigOverride::default())
            .await
    }

    /// Executes an explicit raw GraphQL request with per-call configuration overrides.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site. The
    /// in-flight request future is dropped on cancellation so the underlying
    /// socket is released promptly rather than waiting for the request
    /// deadline.
    ///
    /// # Errors
    ///
    /// Returns [`SubgraphError`] for transport failures, non-success HTTP
    /// status codes, GraphQL error payloads, response-decoding failures,
    /// missing `data`, or unsupported networks.
    pub async fn run_query_with_config<T, R>(
        &self,
        request: R,
        config_override: SubgraphConfigOverride,
    ) -> Result<T, SubgraphError>
    where
        T: DeserializeOwned,
        R: Into<SubgraphQueryRequest>,
    {
        let request = request.into();
        let resolved_config = self.config_with_override(&config_override);
        let api = self.base_url_for(&resolved_config)?;
        let public_api = self.public_base_url_for(&resolved_config)?;
        let graphql_request = GraphQlRequest {
            query: request.document(),
            variables: request.variables(),
            operation_name: request.operation_name(),
        };

        let body = serde_json::to_string(&graphql_request).map_err(|error| {
            serialization_error(
                &public_api,
                resolved_config.chain_id,
                &request,
                "",
                error.to_string(),
            )
        })?;
        let headers = [("content-type".to_owned(), "application/json".to_owned())];
        let timeout = self.transport_policy.client_policy().timeout();

        let body = match self.transport.post(&api, &body, &headers, timeout).await {
            Ok(body) => body,
            Err(TransportError::HttpStatus { status, body }) => {
                return Err(http_status_error(
                    &public_api,
                    resolved_config.chain_id,
                    &request,
                    status,
                    body,
                ));
            }
            Err(TransportError::Transport { class, detail }) => {
                return Err(transport_error(
                    &public_api,
                    resolved_config.chain_id,
                    &request,
                    format_transport_failure(class, &detail),
                ));
            }
            Err(TransportError::Configuration { message }) => {
                return Err(transport_error(
                    &public_api,
                    resolved_config.chain_id,
                    &request,
                    message,
                ));
            }
            Err(other) => {
                return Err(transport_error(
                    &public_api,
                    resolved_config.chain_id,
                    &request,
                    other.to_string(),
                ));
            }
        };

        let response: GraphQlResponse<T> = serde_json::from_str(&body).map_err(|error| {
            serialization_error(
                &public_api,
                resolved_config.chain_id,
                &request,
                &body,
                error.to_string(),
            )
        })?;

        if !response.errors.is_empty() {
            return Err(graphql_error(
                &public_api,
                resolved_config.chain_id,
                &request,
                response.errors,
            ));
        }

        response
            .data
            .ok_or_else(|| missing_data_error(&public_api, resolved_config.chain_id, &request))
    }

    fn config_with_override(&self, config_override: &SubgraphConfigOverride) -> SubgraphConfig {
        let mut config = self.config.clone();

        if let Some(chain_id) = config_override.chain_id {
            config.chain_id = chain_id;
        }

        if let Some(base_urls) = &config_override.base_urls {
            config.base_urls = Some(base_urls.clone());
        }

        config
    }

    fn base_url_for(&self, config: &SubgraphConfig) -> Result<String, SubgraphError> {
        if let Some(base_urls) = &config.base_urls {
            return base_urls.get(&config.chain_id).cloned().flatten().ok_or(
                SubgraphError::UnsupportedNetwork {
                    chain_id: config.chain_id as u64,
                },
            );
        }

        prod_subgraph_id(config.chain_id)
            .map(|subgraph_id| build_prod_gateway_url(self.api_key.as_inner(), subgraph_id))
            .ok_or(SubgraphError::UnsupportedNetwork {
                chain_id: config.chain_id as u64,
            })
    }

    fn public_base_url_for(&self, config: &SubgraphConfig) -> Result<String, SubgraphError> {
        if let Some(base_urls) = &config.base_urls {
            return base_urls
                .get(&config.chain_id)
                .cloned()
                .flatten()
                .map(|base_url| sanitize_public_base_url(&base_url))
                .ok_or(SubgraphError::UnsupportedNetwork {
                    chain_id: config.chain_id as u64,
                });
        }

        self.prod_config
            .get(&config.chain_id)
            .cloned()
            .flatten()
            .ok_or(SubgraphError::UnsupportedNetwork {
                chain_id: config.chain_id as u64,
            })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphQlRequest<'a> {
    query: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<&'a Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    operation_name: Option<&'a str>,
}

#[derive(Deserialize)]
struct GraphQlResponse<T> {
    data: Option<T>,
    #[serde(default)]
    errors: Vec<SubgraphGraphQlError>,
}

pub(crate) fn build_prod_config() -> SubgraphApiBaseUrls {
    BTreeMap::from([
        (
            SupportedChainId::Mainnet,
            Some(build_prod_gateway_url(
                REDACTED_API_KEY_SEGMENT,
                "8mdwJG7YCSwqfxUbhCypZvoubeZcFVpCHb4zmHhvuKTD",
            )),
        ),
        (
            SupportedChainId::GnosisChain,
            Some(build_prod_gateway_url(
                REDACTED_API_KEY_SEGMENT,
                "HTQcP2gLuAy235CMNE8ApN4cbzpLVjjNxtCAUfpzRubq",
            )),
        ),
        (
            SupportedChainId::ArbitrumOne,
            Some(build_prod_gateway_url(
                REDACTED_API_KEY_SEGMENT,
                "CQ8g2uJCjdAkUSNkVbd9oqqRP2GALKu1jJCD3fyY5tdc",
            )),
        ),
        (
            SupportedChainId::Base,
            Some(build_prod_gateway_url(
                REDACTED_API_KEY_SEGMENT,
                "EYfBtJDj2thuBCVhdpYDpzfsWzDg3qzpEsitqMouU4Rg",
            )),
        ),
        (
            SupportedChainId::Sepolia,
            Some(build_prod_gateway_url(
                REDACTED_API_KEY_SEGMENT,
                "31isonmztVX9ejBneP6SaVDQwEtyKCGBb3RTafB9Uf2y",
            )),
        ),
        (SupportedChainId::Polygon, None),
        (SupportedChainId::Avalanche, None),
        (SupportedChainId::Bnb, None),
        (SupportedChainId::Linea, None),
        (SupportedChainId::Plasma, None),
        (SupportedChainId::Ink, None),
    ])
}

const fn prod_subgraph_id(chain_id: SupportedChainId) -> Option<&'static str> {
    match chain_id {
        SupportedChainId::Mainnet => Some("8mdwJG7YCSwqfxUbhCypZvoubeZcFVpCHb4zmHhvuKTD"),
        SupportedChainId::GnosisChain => Some("HTQcP2gLuAy235CMNE8ApN4cbzpLVjjNxtCAUfpzRubq"),
        SupportedChainId::ArbitrumOne => Some("CQ8g2uJCjdAkUSNkVbd9oqqRP2GALKu1jJCD3fyY5tdc"),
        SupportedChainId::Base => Some("EYfBtJDj2thuBCVhdpYDpzfsWzDg3qzpEsitqMouU4Rg"),
        SupportedChainId::Sepolia => Some("31isonmztVX9ejBneP6SaVDQwEtyKCGBb3RTafB9Uf2y"),
        _ => None,
    }
}

fn build_prod_gateway_url(api_key: &str, subgraph_id: &str) -> String {
    format!("{SUBGRAPH_BASE_URL}{api_key}/subgraphs/id/{subgraph_id}")
}

fn format_transport_failure(class: TransportErrorClass, detail: &str) -> String {
    format!("{class}: {detail}")
}

fn transport_error(
    api: &str,
    chain_id: SupportedChainId,
    request: &SubgraphQueryRequest,
    details: String,
) -> SubgraphError {
    SubgraphError::Transport {
        context: Box::new(request_error_context(api, chain_id, request)),
        details,
    }
}

fn http_status_error(
    api: &str,
    chain_id: SupportedChainId,
    request: &SubgraphQueryRequest,
    status: u16,
    body: String,
) -> SubgraphError {
    SubgraphError::HttpStatus {
        context: Box::new(request_error_context(api, chain_id, request)),
        status,
        body,
    }
}

fn serialization_error(
    api: &str,
    chain_id: SupportedChainId,
    request: &SubgraphQueryRequest,
    body: &str,
    details: String,
) -> SubgraphError {
    SubgraphError::Serialization {
        context: Box::new(request_error_context(api, chain_id, request)),
        body: body.to_owned(),
        details,
    }
}

fn graphql_error(
    api: &str,
    chain_id: SupportedChainId,
    request: &SubgraphQueryRequest,
    errors: Vec<SubgraphGraphQlError>,
) -> SubgraphError {
    SubgraphError::GraphQl {
        context: Box::new(request_error_context(api, chain_id, request)),
        errors,
    }
}

fn missing_data_error(
    api: &str,
    chain_id: SupportedChainId,
    request: &SubgraphQueryRequest,
) -> SubgraphError {
    SubgraphError::MissingData {
        context: Box::new(request_error_context(api, chain_id, request)),
    }
}

fn request_error_context(
    api: &str,
    chain_id: SupportedChainId,
    request: &SubgraphQueryRequest,
) -> SubgraphRequestErrorContext {
    SubgraphRequestErrorContext {
        chain_id: u64::from(chain_id),
        api: api.to_owned(),
        document: request.document().to_owned(),
        operation_name: request.operation_name().map(str::to_owned),
        variables: request.variables().cloned(),
    }
}

#[allow(
    clippy::option_if_let_else,
    reason = "the Ok arm binds an intermediate origin and carries a nested conditional; the combinator form would collapse that multi-statement body into a closure and obscure the two-branch parallel structure"
)]
fn sanitize_public_base_url(base_url: &str) -> String {
    match reqwest::Url::parse(base_url) {
        Ok(url) => {
            let origin = url.origin().ascii_serialization();
            if origin == "null" {
                CUSTOM_OVERRIDE_ROUTE_IDENTITY.to_owned()
            } else {
                origin.trim_end_matches('/').to_owned()
            }
        }
        Err(_) => CUSTOM_OVERRIDE_ROUTE_IDENTITY.to_owned(),
    }
}

#[allow(
    clippy::single_option_map,
    reason = "the Option mapping preserves explicit Option<SubgraphApiBaseUrls> semantics for the public sanitization helper"
)]
fn sanitized_base_urls(base_urls: Option<&SubgraphApiBaseUrls>) -> Option<SubgraphApiBaseUrls> {
    base_urls.map(|base_urls| {
        base_urls
            .iter()
            .map(|(chain_id, base_url)| {
                (*chain_id, base_url.as_deref().map(sanitize_public_base_url))
            })
            .collect()
    })
}
