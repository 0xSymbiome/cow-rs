use std::sync::Arc;

use cow_sdk_core::{CoreError, HttpClientPolicy, HttpTransport, ValidationError};
use cow_sdk_transport_policy::{RequestRateLimiter, RetryPolicy, TransportPolicy};
use http::header::{HeaderMap, HeaderValue};
use serde_json::json;

use crate::{
    builder::{ChainIdSet, ChainIdUnset, EnvSet, EnvUnset, OrderBookApiBuilder, TransportUnset},
    error::OrderbookError,
    request::{
        FetchParams, HttpMethod, request_empty_with_timeout, request_json_with_timeout,
        request_text_with_timeout,
    },
    transform::{transform_order, transform_orders},
    types::{
        ApiContext, ApiContextOverride, AppDataHash, AppDataObject, Auction,
        CompetitionOrderStatus, CowEnv, ENVS_LIST, EnvBaseUrlOverrides, GetOrdersRequest,
        GetTradesRequest, NativePriceResponse, Order, OrderCancellations, OrderCreation,
        OrderQuoteRequest, OrderQuoteResponse, OrderUid, SolverCompetitionResponse, TotalSurplus,
        Trade,
    },
};

const API_KEY_HEADER: &str = "X-API-Key";

/// Typed `CoW` Protocol orderbook client.
///
/// The client keeps transport policy, rate-limiter state, and endpoint
/// resolution instance-scoped. Clones of the same client share one limiter.
///
/// Every long-running public method is a single canonical entry point. Callers
/// that need cooperative cancellation wrap the returned future through
/// [`cow_sdk_core::Cancellable::cancel_with`] at the call site; the combinator
/// drops the in-flight request future when the token fires, so the underlying
/// socket is released promptly rather than waiting for the request deadline.
#[derive(Debug, Clone)]
pub struct OrderBookApi {
    context: ApiContext,
    transport_policy: TransportPolicy,
    rate_limiter: RequestRateLimiter,
    env_base_url_overrides: EnvBaseUrlOverrides,
    transport: Arc<dyn HttpTransport + Send + Sync>,
}

impl OrderBookApi {
    /// Returns a fresh [`OrderBookApiBuilder`] for typestate-checked
    /// construction.
    ///
    /// The builder enforces at compile time that the chain id, environment,
    /// and HTTP transport are all supplied before
    /// [`OrderBookApiBuilder::build`] becomes callable. On native targets the
    /// builder also exposes a `build` overload that defaults the transport to
    /// the [`ReqwestTransport`](cow_sdk_core::ReqwestTransport) when the
    /// caller does not supply one.
    #[must_use]
    pub fn builder() -> OrderBookApiBuilder<ChainIdUnset, EnvUnset, TransportUnset> {
        OrderBookApiBuilder::new()
    }

    /// Returns a builder seeded from the supplied [`ApiContext`].
    ///
    /// Convenience entry point that fans the context's chain id, environment,
    /// API key, and base-URL map onto the typestate builder. The transport
    /// is left unset so the caller can either inject an explicit
    /// [`HttpTransport`] or fall through to the native-default
    /// [`OrderBookApiBuilder::build`] path.
    #[must_use]
    pub fn builder_from_context(
        context: ApiContext,
    ) -> OrderBookApiBuilder<ChainIdSet, EnvSet, TransportUnset> {
        OrderBookApiBuilder::from_context(context)
    }

    /// Crate-private constructor used by [`OrderBookApiBuilder::build`].
    #[must_use]
    pub(crate) fn from_parts(
        context: ApiContext,
        transport_policy: TransportPolicy,
        rate_limiter: RequestRateLimiter,
        env_base_url_overrides: EnvBaseUrlOverrides,
        transport: Arc<dyn HttpTransport + Send + Sync>,
    ) -> Self {
        Self {
            context,
            transport_policy,
            rate_limiter,
            env_base_url_overrides,
            transport,
        }
    }

    /// Returns the [`HttpTransport`] handle injected at construction time.
    ///
    /// Downstream consumers reach the runtime-neutral transport seam through
    /// this accessor when they need to share the same transport with other
    /// typed clients constructed from the workspace.
    #[must_use]
    pub fn transport(&self) -> &Arc<dyn HttpTransport + Send + Sync> {
        &self.transport
    }

    /// Returns a copy of this client with a new transport policy.
    ///
    /// Replacing the transport policy rebuilds the instance-scoped rate
    /// limiter; the injected HTTP transport continues to carry every live
    /// request.
    #[must_use]
    pub fn with_transport_policy(mut self, transport_policy: TransportPolicy) -> Self {
        self.rate_limiter = transport_policy.rate_limit().clone();
        self.transport_policy = transport_policy;
        self
    }

    /// Returns a copy of this client with an explicit base URL for `env`.
    ///
    /// These per-environment overrides take precedence over URLs resolved from
    /// [`ApiContext::resolved_base_url`].
    #[must_use]
    pub fn with_env_base_url(mut self, env: CowEnv, base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        self.env_base_url_overrides
            .set(env, normalize_base_url(&base_url));
        self
    }

    /// Returns a copy of this client with context fields overridden.
    ///
    /// This updates chain id, environment, base URLs, and API key in one step.
    /// Per-environment overrides configured with [`Self::with_env_base_url`]
    /// still take precedence for base URL selection.
    #[must_use]
    pub fn with_context_override(mut self, context_override: ApiContextOverride) -> Self {
        if let Some(chain_id) = context_override.chain_id {
            self.context.chain_id = chain_id;
        }
        if let Some(env) = context_override.env {
            self.context.env = env;
        }
        if let Some(base_urls) = context_override.base_urls {
            self.context.base_urls = Some(base_urls);
        }
        if let Some(api_key) = context_override.api_key {
            self.context.api_key = Some(api_key);
        }
        self
    }

    /// Returns the effective API context stored in this client.
    #[must_use]
    pub const fn context(&self) -> &ApiContext {
        &self.context
    }

    /// Returns the active transport policy for this client instance.
    #[must_use]
    pub const fn transport_policy(&self) -> &TransportPolicy {
        &self.transport_policy
    }

    /// Returns the effective base URL used by this client instance.
    ///
    /// # Errors
    ///
    /// Returns any base-URL resolution error from [`ApiContext::resolved_base_url`].
    pub fn effective_base_url(&self) -> Result<String, OrderbookError> {
        self.resolved_base_url(&self.context)
    }

    /// Returns the shared HTTP client policy embedded in the transport policy.
    #[must_use]
    pub const fn client_policy(&self) -> &HttpClientPolicy {
        self.transport_policy.client_policy()
    }

    /// Returns the orderbook request policy embedded in the transport policy.
    #[must_use]
    pub const fn request_policy(&self) -> &RetryPolicy {
        self.transport_policy.retry()
    }

    /// Returns the canonical order details link for `order_uid`.
    ///
    /// # Errors
    ///
    /// Returns any base-URL resolution error from [`ApiContext::resolved_base_url`].
    pub fn get_order_link(&self, order_uid: &OrderUid) -> Result<String, OrderbookError> {
        Ok(format!(
            "{}/api/v1/orders/{}",
            self.effective_base_url()?,
            order_uid.to_hex_string()
        ))
    }

    /// Fetches the orderbook service version string.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] when request execution fails or the response
    /// body cannot be decoded as plain text.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.context().chain_id,
                env = ?self.context().env,
                endpoint = "/api/v1/version",
                method = "GET",
            ),
        ),
    )]
    pub async fn get_version(&self) -> Result<String, OrderbookError> {
        self.fetch_text(FetchParams::new("/api/v1/version", HttpMethod::Get))
            .await
    }

    /// Fetches a quote for the provided request payload.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::InvalidQuoteRequest`] when the quote side is
    /// not well-formed, or any transport/API/serialization error returned by
    /// the orderbook request helpers.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.context().chain_id,
                env = ?self.context().env,
                endpoint = "/api/v1/quote",
                method = "POST",
                quote_id = tracing::field::Empty,
                attempts = tracing::field::Empty,
                status = tracing::field::Empty,
            ),
        ),
    )]
    pub async fn get_quote(
        &self,
        request: &OrderQuoteRequest,
    ) -> Result<OrderQuoteResponse, OrderbookError> {
        if !request.is_valid() {
            return Err(OrderbookError::InvalidQuoteRequest {
                field: "side",
                reason: cow_sdk_core::ValidationReason::Precondition {
                    details: "exactly one of sellAmountBeforeFee or buyAmountAfterFee must be set",
                },
            });
        }

        let body = serde_json::to_value(request)?;
        let params = FetchParams::new("/api/v1/quote", HttpMethod::Post).with_body(body);

        let response: OrderQuoteResponse = self.fetch_json(params).await?;
        #[cfg(feature = "tracing")]
        if let Some(quote_id) = response.id {
            tracing::Span::current().record("quote_id", quote_id);
        }
        Ok(response)
    }

    /// Submits a signed order to the orderbook.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site; the
    /// orderbook treats a submission dropped before it reaches the wire as a
    /// no-op.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] if the request cannot be serialized, the API
    /// rejects the order, or request execution fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.context().chain_id,
                env = ?self.context().env,
                endpoint = "/api/v1/orders",
                method = "POST",
                quote_id = request.quote_id.unwrap_or_default(),
                attempts = tracing::field::Empty,
                status = tracing::field::Empty,
            ),
        ),
    )]
    pub async fn send_order(&self, request: &OrderCreation) -> Result<OrderUid, OrderbookError> {
        let body = serde_json::to_value(request)?;
        let params = FetchParams::new("/api/v1/orders", HttpMethod::Post).with_body(body);

        self.fetch_json(params).await
    }

    /// Submits a signed order-cancellation payload.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site; an
    /// unacknowledged cancellation payload is a no-op on the orderbook
    /// service.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] if the request cannot be serialized, the API
    /// rejects the cancellation, or request execution fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.context().chain_id,
                env = ?self.context().env,
                endpoint = "/api/v1/orders",
                method = "DELETE",
            ),
        ),
    )]
    pub async fn send_signed_order_cancellations(
        &self,
        request: &OrderCancellations,
    ) -> Result<(), OrderbookError> {
        let body = serde_json::to_value(request)?;
        let params = FetchParams::new("/api/v1/orders", HttpMethod::Delete).with_body(body);

        self.fetch_empty(params).await
    }

    /// Fetches and normalizes a single order by UID.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site. Response
    /// normalization is synchronous and runs only after the fetch resolves.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] if request execution fails or the response
    /// cannot be transformed into the crate's stable order DTO.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.context().chain_id,
                env = ?self.context().env,
                endpoint = "/api/v1/orders/:uid",
                method = "GET",
                order_uid = order_uid.to_hex_string(),
            ),
        ),
    )]
    pub async fn get_order(&self, order_uid: &OrderUid) -> Result<Order, OrderbookError> {
        let params = FetchParams::new(
            format!("/api/v1/orders/{}", order_uid.to_hex_string()),
            HttpMethod::Get,
        );
        let order: Order = self.fetch_json(params).await?;
        transform_order(order)
    }

    /// Fetches an order by UID, retrying once against the other environment on a `404`.
    ///
    /// The active environment in [`ApiContext`] is queried first. Only a typed
    /// API `404` triggers fallback to the other known environment. Callers
    /// that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site; the
    /// combinator aborts the primary request promptly, so the fallback only
    /// runs when the primary observed a genuine `404`.
    ///
    /// # Errors
    ///
    /// Returns any error from the primary or fallback order lookup.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.context().chain_id,
                env = ?self.context().env,
                endpoint = "/api/v1/orders/:uid",
                method = "GET",
                order_uid = order_uid.to_hex_string(),
            ),
        ),
    )]
    pub async fn get_order_multi_env(&self, order_uid: &OrderUid) -> Result<Order, OrderbookError> {
        match self.get_order(order_uid).await {
            Ok(order) => Ok(order),
            Err(error) if is_not_found(&error) => {
                let current_env = self.context.env;
                if let Some(fallback_env) = ENVS_LIST.into_iter().find(|env| *env != current_env) {
                    self.clone()
                        .with_context_override(ApiContextOverride {
                            env: Some(fallback_env),
                            ..ApiContextOverride::default()
                        })
                        .get_order(order_uid)
                        .await
                } else {
                    Err(error)
                }
            }
            Err(error) => Err(error),
        }
    }

    /// Fetches and normalizes orders for a specific owner.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site. Response
    /// normalization is synchronous and runs only after the fetch resolves.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] if request execution fails or any order in
    /// the response cannot be normalized.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.context().chain_id,
                env = ?self.context().env,
                endpoint = "/api/v1/account/:owner/orders",
                method = "GET",
                owner = request.owner.to_hex_string(),
            ),
        ),
    )]
    pub async fn get_orders(
        &self,
        request: &GetOrdersRequest,
    ) -> Result<Vec<Order>, OrderbookError> {
        let params = FetchParams::new(
            format!("/api/v1/account/{}/orders", request.owner.to_hex_string()),
            HttpMethod::Get,
        )
        .with_query("offset", request.offset.to_string())
        .with_query("limit", request.limit.to_string());

        let orders: Vec<Order> = self.fetch_json(params).await?;
        transform_orders(orders)
    }

    /// Fetches and normalizes orders associated with a settlement transaction.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site. Response
    /// normalization is synchronous and runs only after the fetch resolves.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] if request execution fails or any order in
    /// the response cannot be normalized.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.context().chain_id,
                env = ?self.context().env,
                endpoint = "/api/v1/transactions/:tx/orders",
                method = "GET",
            ),
        ),
    )]
    pub async fn get_tx_orders(&self, tx_hash: &str) -> Result<Vec<Order>, OrderbookError> {
        let params = FetchParams::new(
            format!("/api/v1/transactions/{tx_hash}/orders"),
            HttpMethod::Get,
        );

        let orders: Vec<Order> = self.fetch_json(params).await?;
        transform_orders(orders)
    }

    /// Fetches trades filtered by owner or order UID.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::InvalidTradesQuery`] when both or neither
    /// filters are set, or any transport/API/serialization error from the
    /// request helpers.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.context().chain_id,
                env = ?self.context().env,
                endpoint = "/api/v2/trades",
                method = "GET",
            ),
        ),
    )]
    pub async fn get_trades(
        &self,
        request: &GetTradesRequest,
    ) -> Result<Vec<Trade>, OrderbookError> {
        if !request.is_valid() {
            return Err(OrderbookError::InvalidTradesQuery {
                field: "filter",
                reason: cow_sdk_core::ValidationReason::Precondition {
                    details: "exactly one of owner or orderUid must be set",
                },
            });
        }

        let mut params = FetchParams::new("/api/v2/trades", HttpMethod::Get);

        if let Some(owner) = &request.owner {
            params = params.with_query("owner", owner.to_hex_string());
        }

        if let Some(order_uid) = &request.order_uid {
            params = params.with_query("orderUid", order_uid.to_hex_string());
        }

        let params = params
            .with_query("offset", request.offset.to_string())
            .with_query("limit", request.limit.to_string());

        self.fetch_json(params).await
    }

    /// Fetches the current competition status for an order.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] when request execution or response decoding fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.context().chain_id,
                env = ?self.context().env,
                endpoint = "/api/v1/orders/:uid/status",
                method = "GET",
                order_uid = order_uid.to_hex_string(),
            ),
        ),
    )]
    pub async fn get_order_competition_status(
        &self,
        order_uid: &OrderUid,
    ) -> Result<CompetitionOrderStatus, OrderbookError> {
        let params = FetchParams::new(
            format!("/api/v1/orders/{}/status", order_uid.to_hex_string()),
            HttpMethod::Get,
        );

        self.fetch_json(params).await
    }

    /// Fetches the token price quoted in the chain's native asset.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] when request execution or response decoding fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.context().chain_id,
                env = ?self.context().env,
                endpoint = "/api/v1/token/:address/native_price",
                method = "GET",
            ),
        ),
    )]
    pub async fn get_native_price(
        &self,
        token: &crate::types::Address,
    ) -> Result<NativePriceResponse, OrderbookError> {
        let params = FetchParams::new(
            format!("/api/v1/token/{}/native_price", token.to_hex_string()),
            HttpMethod::Get,
        );

        self.fetch_json(params).await
    }

    /// Fetches the recorded total surplus for a user.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] when request execution or response decoding fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.context().chain_id,
                env = ?self.context().env,
                endpoint = "/api/v1/users/:address/total_surplus",
                method = "GET",
                owner = owner.to_hex_string(),
            ),
        ),
    )]
    pub async fn get_total_surplus(
        &self,
        owner: &crate::types::Address,
    ) -> Result<TotalSurplus, OrderbookError> {
        let params = FetchParams::new(
            format!("/api/v1/users/{}/total_surplus", owner.to_hex_string()),
            HttpMethod::Get,
        );

        self.fetch_json(params).await
    }

    /// Fetches full app-data JSON for the provided app-data hash.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] when request execution or response decoding fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.context().chain_id,
                env = ?self.context().env,
                endpoint = "/api/v1/app_data/:hash",
                method = "GET",
            ),
        ),
    )]
    pub async fn get_app_data(
        &self,
        app_data_hash: &AppDataHash,
    ) -> Result<AppDataObject, OrderbookError> {
        let params = FetchParams::new(
            format!("/api/v1/app_data/{}", app_data_hash.as_str()),
            HttpMethod::Get,
        );

        self.fetch_json(params).await
    }

    /// Uploads full app-data JSON for the provided app-data hash.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site; an
    /// unacknowledged upload is a no-op on the orderbook service.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] if the request body cannot be encoded, the
    /// API rejects the upload, or request execution fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.context().chain_id,
                env = ?self.context().env,
                endpoint = "/api/v1/app_data/:hash",
                method = "PUT",
            ),
        ),
    )]
    pub async fn upload_app_data(
        &self,
        app_data_hash: &AppDataHash,
        full_app_data: &str,
    ) -> Result<AppDataObject, OrderbookError> {
        let params = FetchParams::new(
            format!("/api/v1/app_data/{}", app_data_hash.as_str()),
            HttpMethod::Put,
        )
        .with_body(json!({ "fullAppData": full_app_data }));

        self.fetch_json(params).await
    }

    /// Fetches solver-competition data by auction id.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] when request execution or response decoding fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.context().chain_id,
                env = ?self.context().env,
                endpoint = "/api/v1/solver_competition/:auction",
                method = "GET",
            ),
        ),
    )]
    pub async fn get_solver_competition_by_auction_id(
        &self,
        auction_id: i64,
    ) -> Result<SolverCompetitionResponse, OrderbookError> {
        let params = FetchParams::new(
            format!("/api/v1/solver_competition/{auction_id}"),
            HttpMethod::Get,
        );

        self.fetch_json(params).await
    }

    /// Fetches solver-competition data by settlement transaction hash.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] when request execution or response decoding fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.context().chain_id,
                env = ?self.context().env,
                endpoint = "/api/v1/solver_competition/by_tx_hash/:tx",
                method = "GET",
            ),
        ),
    )]
    pub async fn get_solver_competition_by_tx_hash(
        &self,
        tx_hash: &str,
    ) -> Result<SolverCompetitionResponse, OrderbookError> {
        let params = FetchParams::new(
            format!("/api/v1/solver_competition/by_tx_hash/{tx_hash}"),
            HttpMethod::Get,
        );

        self.fetch_json(params).await
    }

    /// Fetches the latest solver-competition snapshot from the orderbook.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] when request execution or response decoding fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.context().chain_id,
                env = ?self.context().env,
                endpoint = "/api/v1/solver_competition/latest",
                method = "GET",
            ),
        ),
    )]
    pub async fn get_latest_solver_competition(
        &self,
    ) -> Result<SolverCompetitionResponse, OrderbookError> {
        let params = FetchParams::new("/api/v1/solver_competition/latest", HttpMethod::Get);

        self.fetch_json(params).await
    }

    /// Fetches the current auction snapshot from the orderbook.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] when request execution or response decoding fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.context().chain_id,
                env = ?self.context().env,
                endpoint = "/api/v1/auction",
                method = "GET",
            ),
        ),
    )]
    pub async fn get_auction(&self) -> Result<Auction, OrderbookError> {
        let params = FetchParams::new("/api/v1/auction", HttpMethod::Get);

        self.fetch_json(params).await
    }

    async fn fetch_json<T>(&self, params: FetchParams) -> Result<T, OrderbookError>
    where
        T: serde::de::DeserializeOwned,
    {
        request_json_with_timeout(
            &self.transport,
            &self.resolved_base_url(&self.context)?,
            &params,
            self.transport_policy.retry(),
            &self.rate_limiter,
            self.transport_policy.timeout(),
            self.additional_headers()?,
        )
        .await
    }

    async fn fetch_empty(&self, params: FetchParams) -> Result<(), OrderbookError> {
        request_empty_with_timeout(
            &self.transport,
            &self.resolved_base_url(&self.context)?,
            &params,
            self.transport_policy.retry(),
            &self.rate_limiter,
            self.transport_policy.timeout(),
            self.additional_headers()?,
        )
        .await
    }

    async fn fetch_text(&self, params: FetchParams) -> Result<String, OrderbookError> {
        request_text_with_timeout(
            &self.transport,
            &self.resolved_base_url(&self.context)?,
            &params,
            self.transport_policy.retry(),
            &self.rate_limiter,
            self.transport_policy.timeout(),
            self.additional_headers()?,
        )
        .await
    }

    fn resolved_base_url(&self, context: &ApiContext) -> Result<String, OrderbookError> {
        if let Some(override_url) = self.env_base_url_overrides.get(context.env) {
            return Ok(override_url.to_owned());
        }

        let resolved = context.resolved_base_url()?;
        Ok(normalize_base_url(&resolved))
    }

    fn additional_headers(&self) -> Result<Option<HeaderMap>, OrderbookError> {
        let Some(api_key) = self.context.validated_api_key().map_err(CoreError::from)? else {
            return Ok(None);
        };

        let header_value = HeaderValue::from_str(api_key).map_err(|_| {
            OrderbookError::Core(CoreError::Validation(
                ValidationError::InvalidHttpHeaderValue { field: "api_key" },
            ))
        })?;
        let mut headers = HeaderMap::new();
        headers.insert(API_KEY_HEADER, header_value);
        Ok(Some(headers))
    }
}

fn normalize_base_url(base_url: &str) -> String {
    base_url.trim_end_matches('/').to_owned()
}

fn is_not_found(error: &OrderbookError) -> bool {
    match error {
        OrderbookError::Api(envelope) => envelope.status == 404,
        OrderbookError::Rejected { status, .. } => status.as_u16() == 404,
        _ => false,
    }
}
