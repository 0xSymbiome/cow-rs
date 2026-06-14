//! Typed subgraph client configuration and query execution.

use std::{fmt, sync::Arc};

use cow_sdk_core::transport::policy::{
    AttemptOutcome as RetryOutcome, LimiterKey, RequestRateLimiter, RetrySignal, TransportPolicy,
    run_with_retry,
};
use cow_sdk_core::transport::sanitize_public_base_url;
use cow_sdk_core::{
    HttpClientPolicy, HttpTransport, Redacted, RedactedOptionalUrlMap, SupportedChainId,
    TransportError, TransportErrorClass, redact_response_body,
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

/// Human-readable name for the `CoW` Protocol subgraph service.
pub const API_NAME: &str = "CoW Protocol Subgraph";
/// Redacting base-URL overrides keyed by chain id.
///
/// A `Some(url)` entry enables that chain and routes requests to `url`. A
/// `None` entry marks the chain as unsupported for the current configuration.
pub type SubgraphApiBaseUrls = RedactedOptionalUrlMap<SupportedChainId>;

/// Static subgraph client configuration.
///
/// The default configuration targets mainnet production routes derived from the
/// API key supplied when constructing [`SubgraphApi`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubgraphConfig {
    /// Active chain id used for helper methods and generic queries.
    pub chain_id: SupportedChainId,
    /// Optional per-chain base URL overrides.
    ///
    /// When this is `None`, [`SubgraphApi`] uses its API-key-derived production
    /// routing map internally and exposes only redacted route identity through
    /// its stable public metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_urls: Option<SubgraphApiBaseUrls>,
}

impl SubgraphConfig {
    /// Creates a static subgraph client configuration.
    #[must_use]
    pub const fn new(chain_id: SupportedChainId, base_urls: Option<SubgraphApiBaseUrls>) -> Self {
        Self {
            chain_id,
            base_urls,
        }
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

/// Routing overrides applied through [`SubgraphApi::with_config_override`].
///
/// Use [`SubgraphConfigOverride::for_chain`] for the common case of querying a
/// different supported chain from the same client.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubgraphConfigOverride {
    /// Optional chain override for a single request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<SupportedChainId>,
    /// Optional base-URL map override for a single request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_urls: Option<SubgraphApiBaseUrls>,
}

impl SubgraphConfigOverride {
    /// Creates subgraph configuration overrides.
    #[must_use]
    pub const fn new(
        chain_id: Option<SupportedChainId>,
        base_urls: Option<SubgraphApiBaseUrls>,
    ) -> Self {
        Self {
            chain_id,
            base_urls,
        }
    }

    /// Creates an override that switches the queried chain.
    #[must_use]
    pub const fn for_chain(chain_id: SupportedChainId) -> Self {
        Self {
            chain_id: Some(chain_id),
            base_urls: None,
        }
    }

    /// Returns a copy with an explicit chain-id override.
    #[must_use]
    pub const fn with_chain_id(mut self, chain_id: SupportedChainId) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    /// Returns a copy with an explicit base-URL map override.
    #[must_use]
    pub fn with_base_urls(mut self, base_urls: SubgraphApiBaseUrls) -> Self {
        self.base_urls = Some(base_urls);
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
    transport_policy: TransportPolicy,
    rate_limiter: RequestRateLimiter,
    transport: Arc<dyn HttpTransport + Send + Sync>,
}

impl fmt::Debug for SubgraphApi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let supported_prod_chains = self
            .prod_config
            .as_inner()
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
    pub const fn builder() -> SubgraphApiBuilder<ChainIdUnset, ApiKeyUnset, TransportUnset> {
        SubgraphApiBuilder::new()
    }

    /// Crate-private constructor used by [`SubgraphApiBuilder::build`].
    #[must_use]
    pub(crate) fn from_parts(
        config: SubgraphConfig,
        api_key: Redacted<String>,
        prod_config: SubgraphApiBaseUrls,
        transport_policy: TransportPolicy,
        rate_limiter: RequestRateLimiter,
        transport: Arc<dyn HttpTransport + Send + Sync>,
    ) -> Self {
        Self {
            config,
            api_key,
            prod_config,
            transport_policy,
            rate_limiter,
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
    pub const fn transport_policy(&self) -> &TransportPolicy {
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
    pub fn with_transport_policy(mut self, transport_policy: TransportPolicy) -> Self {
        self.rate_limiter = transport_policy.rate_limit().clone();
        self.transport_policy = transport_policy;
        self
    }

    /// Returns a copy of this client with routing configuration overrides applied.
    ///
    /// The returned client targets the overridden chain and/or base URLs for
    /// every subsequent query; the injected transport and transport policy are
    /// unchanged. Compose it inline to query a different supported chain from a
    /// single client:
    ///
    /// ```rust,ignore
    /// let totals = api
    ///     .with_config_override(SubgraphConfigOverride::for_chain(SupportedChainId::GnosisChain))
    ///     .totals()
    ///     .await?;
    /// ```
    #[must_use]
    pub fn with_config_override(mut self, config_override: SubgraphConfigOverride) -> Self {
        if let Some(chain_id) = config_override.chain_id {
            self.config.chain_id = chain_id;
        }
        if let Some(base_urls) = config_override.base_urls {
            self.config.base_urls = Some(base_urls);
        }
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
    pub async fn totals(&self) -> Result<Total, SubgraphError> {
        let response: TotalsResponse = self
            .query(SubgraphQueryRequest::new(TOTALS_QUERY).with_operation_name("Totals"))
            .await?;

        response
            .totals
            .into_iter()
            .next()
            .ok_or(SubgraphError::NoTotalsFound)
    }

    /// Fetches daily volume rows for the last `days` entries.
    ///
    /// `days` is forwarded to the GraphQL `first` argument, which The Graph caps
    /// at 1000; for larger or keyset-paginated windows use
    /// [`SubgraphApi::query`].
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
    pub async fn last_days_volume(
        &self,
        days: u32,
    ) -> Result<LastDaysVolumeResponse, SubgraphError> {
        self.query(
            SubgraphQueryRequest::new(LAST_DAYS_VOLUME_QUERY)
                .with_variables(json!({ "days": days }))
                .with_operation_name("LastDaysVolume"),
        )
        .await
    }

    /// Fetches hourly volume rows for the last `hours` entries.
    ///
    /// `hours` is forwarded to the GraphQL `first` argument, which The Graph
    /// caps at 1000; for larger or keyset-paginated windows use
    /// [`SubgraphApi::query`].
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
    pub async fn last_hours_volume(
        &self,
        hours: u32,
    ) -> Result<LastHoursVolumeResponse, SubgraphError> {
        self.query(
            SubgraphQueryRequest::new(LAST_HOURS_VOLUME_QUERY)
                .with_variables(json!({ "hours": hours }))
                .with_operation_name("LastHoursVolume"),
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
                endpoint = "subgraph.query",
                method = "POST",
            ),
        ),
    )]
    pub async fn query<T, R>(&self, request: R) -> Result<T, SubgraphError>
    where
        T: DeserializeOwned,
        R: Into<SubgraphQueryRequest>,
    {
        let request = request.into();
        let chain_id = self.config.chain_id;
        let api = self.base_url_for(&self.config)?;
        let public_api = self.public_base_url_for(&self.config)?;
        let graphql_request = GraphQlRequest {
            query: request.document(),
            variables: request.variables(),
            operation_name: request.operation_name(),
        };

        let body = serde_json::to_string(&graphql_request).map_err(|error| {
            serialization_error(&public_api, chain_id, &request, "", error.to_string())
        })?;
        let body = self
            .post_graphql_with_policy(&api, &public_api, chain_id, &request, &body)
            .await?;

        let response: GraphQlResponse<T> = serde_json::from_str(&body).map_err(|error| {
            serialization_error(&public_api, chain_id, &request, &body, error.to_string())
        })?;

        if !response.errors.is_empty() {
            return Err(graphql_error(
                &public_api,
                chain_id,
                &request,
                response.errors,
            ));
        }

        response
            .data
            .ok_or_else(|| missing_data_error(&public_api, chain_id, &request))
    }

    async fn post_graphql_with_policy(
        &self,
        api: &str,
        public_api: &str,
        chain_id: SupportedChainId,
        request: &SubgraphQueryRequest,
        body: &str,
    ) -> Result<String, SubgraphError> {
        let headers = [("content-type".to_owned(), "application/json".to_owned())];
        let headers = &headers;
        let timeout = self.transport_policy.timeout();
        let limiter_url = url::Url::parse(api).map_err(|error| {
            transport_error(
                public_api,
                chain_id,
                request,
                TransportErrorClass::Builder,
                format!("could not parse subgraph URL for rate limiting: {error}"),
            )
        })?;

        // The shared driver in `cow_sdk_core::transport::policy` owns the retry loop,
        // rate-limit acquisition, backoff, `Retry-After` clock, and retry
        // telemetry; the closure performs one GraphQL POST and classifies the
        // result, building the typed `SubgraphError` for the terminal path.
        run_with_retry::<String, SubgraphError, _, _>(
            self.transport_policy.retry(),
            &self.rate_limiter,
            LimiterKey::PerUrl(&limiter_url),
            |_attempt_index| async move {
                match self.transport.post(api, body, headers, timeout).await {
                    Ok(response) => RetryOutcome::Success(response.into_body()),
                    Err(TransportError::HttpStatus {
                        status,
                        headers,
                        body,
                    }) => {
                        let header_pairs = headers
                            .into_iter()
                            .map(|(name, value)| (name, value.into_inner()))
                            .collect::<Vec<_>>();
                        RetryOutcome::Failure {
                            error: http_status_error(
                                public_api,
                                chain_id,
                                request,
                                status,
                                body.as_inner(),
                            ),
                            signal: RetrySignal::HttpStatus {
                                status,
                                headers: header_pairs,
                            },
                        }
                    }
                    Err(error) => {
                        let (class, details) = transport_failure_parts(error);
                        RetryOutcome::Failure {
                            error: transport_error(public_api, chain_id, request, class, details),
                            signal: RetrySignal::Transport { class },
                        }
                    }
                }
            },
        )
        .await
    }

    fn base_url_for(&self, config: &SubgraphConfig) -> Result<String, SubgraphError> {
        if let Some(base_urls) = &config.base_urls {
            return base_urls
                .as_inner()
                .get(&config.chain_id)
                .cloned()
                .flatten()
                .ok_or(SubgraphError::UnsupportedNetwork {
                    chain_id: config.chain_id as u64,
                });
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
                .as_inner()
                .get(&config.chain_id)
                .cloned()
                .flatten()
                .map(|base_url| sanitize_public_base_url(&base_url))
                .ok_or(SubgraphError::UnsupportedNetwork {
                    chain_id: config.chain_id as u64,
                });
        }

        self.prod_config
            .as_inner()
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

/// Single source of truth for production subgraph deployments: each supported
/// chain paired with its The Graph subgraph id. Both the routing path
/// ([`prod_subgraph_id`]) and the redacted display map ([`build_prod_config`])
/// read this slice, so a deployment-id rotation is a one-line edit and the two
/// surfaces cannot drift apart.
const PROD_SUBGRAPH_IDS: &[(SupportedChainId, &str)] = &[
    (
        SupportedChainId::Mainnet,
        "8mdwJG7YCSwqfxUbhCypZvoubeZcFVpCHb4zmHhvuKTD",
    ),
    (
        SupportedChainId::GnosisChain,
        "HTQcP2gLuAy235CMNE8ApN4cbzpLVjjNxtCAUfpzRubq",
    ),
    (
        SupportedChainId::ArbitrumOne,
        "CQ8g2uJCjdAkUSNkVbd9oqqRP2GALKu1jJCD3fyY5tdc",
    ),
    (
        SupportedChainId::Base,
        "EYfBtJDj2thuBCVhdpYDpzfsWzDg3qzpEsitqMouU4Rg",
    ),
    (
        SupportedChainId::Sepolia,
        "31isonmztVX9ejBneP6SaVDQwEtyKCGBb3RTafB9Uf2y",
    ),
];

/// Chains the production configuration explicitly marks unsupported. They stay
/// in the public route map with `None` values so the support posture remains
/// visible rather than silently absent.
const UNSUPPORTED_PROD_CHAINS: &[SupportedChainId] = &[
    SupportedChainId::Polygon,
    SupportedChainId::Avalanche,
    SupportedChainId::Bnb,
    SupportedChainId::Linea,
    SupportedChainId::Plasma,
    SupportedChainId::Ink,
];

pub(crate) fn build_prod_config() -> SubgraphApiBaseUrls {
    PROD_SUBGRAPH_IDS
        .iter()
        .map(|(chain_id, subgraph_id)| {
            (
                *chain_id,
                Some(build_prod_gateway_url(
                    REDACTED_API_KEY_SEGMENT,
                    subgraph_id,
                )),
            )
        })
        .chain(
            UNSUPPORTED_PROD_CHAINS
                .iter()
                .map(|chain_id| (*chain_id, None)),
        )
        .collect()
}

fn prod_subgraph_id(chain_id: SupportedChainId) -> Option<&'static str> {
    PROD_SUBGRAPH_IDS
        .iter()
        .find(|(id, _)| *id == chain_id)
        .map(|(_, subgraph_id)| *subgraph_id)
}

fn build_prod_gateway_url(api_key: &str, subgraph_id: &str) -> String {
    format!("{SUBGRAPH_BASE_URL}{api_key}/subgraphs/id/{subgraph_id}")
}

fn transport_error(
    api: &str,
    chain_id: SupportedChainId,
    request: &SubgraphQueryRequest,
    class: TransportErrorClass,
    details: String,
) -> SubgraphError {
    SubgraphError::Transport {
        context: Box::new(request_error_context(api, chain_id, request)),
        class,
        details: details.into(),
    }
}

fn transport_failure_parts(error: TransportError) -> (TransportErrorClass, String) {
    match error {
        TransportError::Transport { class, detail } => (class, detail.into_inner()),
        TransportError::Configuration { message } => {
            (TransportErrorClass::Builder, message.into_inner())
        }
        TransportError::HttpStatus { .. } => (
            TransportErrorClass::Status,
            "unexpected HTTP status error branch".to_owned(),
        ),
        other => (TransportErrorClass::Other, other.to_string()),
    }
}

fn http_status_error(
    api: &str,
    chain_id: SupportedChainId,
    request: &SubgraphQueryRequest,
    status: u16,
    body: &str,
) -> SubgraphError {
    SubgraphError::HttpStatus {
        context: Box::new(request_error_context(api, chain_id, request)),
        status,
        body: Redacted::new(redact_response_body(body)),
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
        body: Redacted::new(redact_response_body(body)),
        details: details.into(),
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
        api: api.to_owned().into(),
        document: request.document().to_owned().into(),
        operation_name: request
            .operation_name()
            .map(|value| value.to_owned().into()),
        variables: request.variables().cloned().map(Redacted::new),
    }
}
