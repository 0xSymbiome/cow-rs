use cow_sdk_core::HttpClientPolicy;
use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};
use serde_json::json;

use crate::{
    error::OrderbookError,
    request::{
        FetchParams, HttpMethod, OrderBookTransportPolicy, RequestPolicy, RequestRateLimiter,
        request_empty_with_timeout, request_json_with_timeout, request_text_with_timeout,
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
#[derive(Debug, Clone)]
pub struct OrderBookApi {
    client: Client,
    context: ApiContext,
    transport_policy: OrderBookTransportPolicy,
    rate_limiter: RequestRateLimiter,
    env_base_url_overrides: EnvBaseUrlOverrides,
}

impl OrderBookApi {
    /// Creates a client with the default transport policy for `context`.
    #[must_use]
    pub fn new(context: ApiContext) -> Self {
        let transport_policy = OrderBookTransportPolicy::default();
        let (client, rate_limiter) = build_request_runtime(&transport_policy);

        Self {
            client,
            rate_limiter,
            transport_policy,
            context,
            env_base_url_overrides: EnvBaseUrlOverrides::default(),
        }
    }

    /// Creates a client with an explicit transport policy.
    ///
    /// The policy rebuilds both the underlying HTTP client and the
    /// instance-scoped request limiter.
    #[must_use]
    pub fn new_with_transport_policy(
        context: ApiContext,
        transport_policy: OrderBookTransportPolicy,
    ) -> Self {
        let (client, rate_limiter) = build_request_runtime(&transport_policy);

        Self {
            client,
            context,
            transport_policy,
            rate_limiter,
            env_base_url_overrides: EnvBaseUrlOverrides::default(),
        }
    }

    /// Creates a client that shares an externally built [`reqwest::Client`].
    ///
    /// Multi-chain consumers can pool one `reqwest::Client` (and its TCP,
    /// TLS, and HTTP/2 connection cache) across every `OrderBookApi` instance
    /// they construct, which is the recommended pattern for production bots
    /// that issue requests on behalf of several chains or trading accounts.
    /// The supplied client keeps any custom keep-alive, timeout, or TLS
    /// configuration the caller chose; see `docs/performance.md` for the
    /// production-bot HTTP/2 keep-alive recipe.
    #[must_use]
    pub fn from_shared_client(client: Client, context: ApiContext) -> Self {
        let transport_policy = OrderBookTransportPolicy::default();
        let rate_limiter = RequestRateLimiter::new(transport_policy.request_policy().rate_limit);

        Self {
            client,
            context,
            transport_policy,
            rate_limiter,
            env_base_url_overrides: EnvBaseUrlOverrides::default(),
        }
    }

    /// Creates a client that shares an externally built [`reqwest::Client`] and uses an
    /// explicit transport policy for request-timeout and retry behaviour.
    ///
    /// The shared client is reused verbatim so its keep-alive and connection
    /// pool settings stay under caller control. Only the request-policy side
    /// of the supplied [`OrderBookTransportPolicy`] drives retry, rate-limit,
    /// and timeout decisions on this instance.
    #[must_use]
    pub fn from_shared_client_with_transport_policy(
        client: Client,
        context: ApiContext,
        transport_policy: OrderBookTransportPolicy,
    ) -> Self {
        let rate_limiter = RequestRateLimiter::new(transport_policy.request_policy().rate_limit);

        Self {
            client,
            context,
            transport_policy,
            rate_limiter,
            env_base_url_overrides: EnvBaseUrlOverrides::default(),
        }
    }

    /// Creates a client with an explicit base URL override for the current environment.
    ///
    /// This override takes precedence over URLs resolved from [`ApiContext`].
    #[must_use]
    pub fn new_with_base_url(context: ApiContext, base_url: impl Into<String>) -> Self {
        let env = context.env;
        Self::new(context).with_env_base_url(env, base_url.into())
    }

    /// Returns a copy of this client with a new transport policy.
    ///
    /// Replacing the transport policy rebuilds the underlying HTTP client and
    /// creates a new instance-scoped rate limiter.
    #[must_use]
    pub fn with_transport_policy(mut self, transport_policy: OrderBookTransportPolicy) -> Self {
        let (client, rate_limiter) = build_request_runtime(&transport_policy);
        self.client = client;
        self.rate_limiter = rate_limiter;
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
    pub const fn transport_policy(&self) -> &OrderBookTransportPolicy {
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
    pub const fn request_policy(&self) -> &RequestPolicy {
        self.transport_policy.request_policy()
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
            order_uid.as_str()
        ))
    }

    /// Fetches the orderbook service version string.
    ///
    /// This is a thin wrapper around
    /// [`get_version_with_cancellation`](Self::get_version_with_cancellation)
    /// that passes a fresh [`cow_sdk_core::CancellationToken`]; existing
    /// callers observe no behavioural change.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] when request execution fails or the response
    /// body cannot be decoded as plain text.
    pub async fn get_version(&self) -> Result<String, OrderbookError> {
        self.get_version_with_cancellation(&cow_sdk_core::CancellationToken::new())
            .await
    }

    /// Fetches the orderbook API version string with cooperative cancellation support.
    ///
    /// The call returns [`OrderbookError::Cancelled`] if the supplied token
    /// fires before a response is received. In-flight request futures are
    /// dropped on cancellation so the underlying socket is released
    /// promptly rather than waiting for the request deadline.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::Cancelled`] when `token` fires during the
    /// call, or any transport/decoding error returned by the orderbook
    /// request helpers.
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
    pub async fn get_version_with_cancellation(
        &self,
        token: &cow_sdk_core::CancellationToken,
    ) -> Result<String, OrderbookError> {
        tokio::select! {
            biased;
            () = token.cancelled() => Err(OrderbookError::Cancelled),
            result = self.fetch_text(FetchParams::new("/api/v1/version", HttpMethod::Get)) => result,
        }
    }

    /// Fetches a quote for the provided request payload.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::InvalidQuoteRequest`] when the quote side is
    /// not well-formed, or any transport/API/serialization error returned by
    /// the orderbook request helpers.
    pub async fn get_quote(
        &self,
        request: &OrderQuoteRequest,
    ) -> Result<OrderQuoteResponse, OrderbookError> {
        self.get_quote_with_cancellation(request, &cow_sdk_core::CancellationToken::new())
            .await
    }

    /// Fetches a quote for the provided request payload with cooperative cancellation support.
    ///
    /// Returns [`OrderbookError::Cancelled`] if `token` fires before the
    /// orderbook returns a response. In-flight request futures are dropped on
    /// cancellation so the underlying socket is released promptly.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::InvalidQuoteRequest`] when the quote side is
    /// not well-formed, [`OrderbookError::Cancelled`] when `token` fires
    /// during the call, or any transport/API/serialization error returned by
    /// the orderbook request helpers.
    pub async fn get_quote_with_cancellation(
        &self,
        request: &OrderQuoteRequest,
        token: &cow_sdk_core::CancellationToken,
    ) -> Result<OrderQuoteResponse, OrderbookError> {
        if !request.is_valid() {
            return Err(OrderbookError::InvalidQuoteRequest(
                "quote side must set exactly one of sellAmountBeforeFee or buyAmountAfterFee"
                    .to_owned(),
            ));
        }

        let body = serde_json::to_value(request)
            .map_err(|error| OrderbookError::Serialization(error.to_string()))?;
        let params = FetchParams::new("/api/v1/quote", HttpMethod::Post).with_body(body);

        tokio::select! {
            biased;
            () = token.cancelled() => Err(OrderbookError::Cancelled),
            result = self.fetch_json(params) => result,
        }
    }

    /// Submits a signed order to the orderbook.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] if the request cannot be serialized, the API
    /// rejects the order, or request execution fails.
    pub async fn send_order(&self, request: &OrderCreation) -> Result<OrderUid, OrderbookError> {
        self.send_order_with_cancellation(request, &cow_sdk_core::CancellationToken::new())
            .await
    }

    /// Submits a signed order to the orderbook with cooperative cancellation support.
    ///
    /// Returns [`OrderbookError::Cancelled`] if `token` fires before the
    /// orderbook acknowledges the submission. The payload is consumed only if
    /// the submission actually reaches the wire; the orderbook treats an
    /// unsubmitted order as a no-op.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::Cancelled`] when `token` fires during the
    /// call, or any transport/API/serialization error returned by the
    /// orderbook request helpers.
    pub async fn send_order_with_cancellation(
        &self,
        request: &OrderCreation,
        token: &cow_sdk_core::CancellationToken,
    ) -> Result<OrderUid, OrderbookError> {
        let body = serde_json::to_value(request)
            .map_err(|error| OrderbookError::Serialization(error.to_string()))?;
        let params = FetchParams::new("/api/v1/orders", HttpMethod::Post).with_body(body);

        tokio::select! {
            biased;
            () = token.cancelled() => Err(OrderbookError::Cancelled),
            result = self.fetch_json(params) => result,
        }
    }

    /// Submits a signed order-cancellation payload.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] if the request cannot be serialized, the API
    /// rejects the cancellation, or request execution fails.
    pub async fn send_signed_order_cancellations(
        &self,
        request: &OrderCancellations,
    ) -> Result<(), OrderbookError> {
        self.send_signed_order_cancellations_with_cancellation(
            request,
            &cow_sdk_core::CancellationToken::new(),
        )
        .await
    }

    /// Submits a signed order-cancellation payload with cooperative cancellation support.
    ///
    /// Returns [`OrderbookError::Cancelled`] if `token` fires before the
    /// orderbook acknowledges the cancellation. An unacknowledged cancellation
    /// payload is a no-op on the orderbook service.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::Cancelled`] when `token` fires during the
    /// call, or any transport/API/serialization error returned by the
    /// orderbook request helpers.
    pub async fn send_signed_order_cancellations_with_cancellation(
        &self,
        request: &OrderCancellations,
        token: &cow_sdk_core::CancellationToken,
    ) -> Result<(), OrderbookError> {
        let body = serde_json::to_value(request)
            .map_err(|error| OrderbookError::Serialization(error.to_string()))?;
        let params = FetchParams::new("/api/v1/orders", HttpMethod::Delete).with_body(body);

        tokio::select! {
            biased;
            () = token.cancelled() => Err(OrderbookError::Cancelled),
            result = self.fetch_empty(params) => result,
        }
    }

    /// Fetches and normalizes a single order by UID.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] if request execution fails or the response
    /// cannot be transformed into the crate's stable order DTO.
    pub async fn get_order(&self, order_uid: &OrderUid) -> Result<Order, OrderbookError> {
        self.get_order_with_cancellation(order_uid, &cow_sdk_core::CancellationToken::new())
            .await
    }

    /// Fetches and normalizes a single order by UID with cooperative cancellation support.
    ///
    /// Returns [`OrderbookError::Cancelled`] if `token` fires before the
    /// orderbook responds. Response normalization is synchronous and happens
    /// only after the fetch resolves.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::Cancelled`] when `token` fires during the
    /// call, or any transport, API, or normalization error returned by the
    /// orderbook request helpers.
    pub async fn get_order_with_cancellation(
        &self,
        order_uid: &OrderUid,
        token: &cow_sdk_core::CancellationToken,
    ) -> Result<Order, OrderbookError> {
        let params = FetchParams::new(
            format!("/api/v1/orders/{}", order_uid.as_str()),
            HttpMethod::Get,
        );

        let order: Order = tokio::select! {
            biased;
            () = token.cancelled() => return Err(OrderbookError::Cancelled),
            result = self.fetch_json(params) => result?,
        };

        transform_order(order)
    }

    /// Fetches an order by UID, retrying once against the other environment on a `404`.
    ///
    /// The active environment in [`ApiContext`] is queried first. Only a typed
    /// API `404` triggers fallback to the other known environment.
    ///
    /// # Errors
    ///
    /// Returns any error from the primary or fallback order lookup.
    pub async fn get_order_multi_env(&self, order_uid: &OrderUid) -> Result<Order, OrderbookError> {
        self.get_order_multi_env_with_cancellation(
            order_uid,
            &cow_sdk_core::CancellationToken::new(),
        )
        .await
    }

    /// Fetches an order with environment fallback and cooperative cancellation support.
    ///
    /// Both the primary and the fallback lookup respect `token`; cancelling
    /// during the primary request aborts before the fallback is attempted.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::Cancelled`] when `token` fires during either
    /// lookup, or any error from the primary or fallback order lookup.
    pub async fn get_order_multi_env_with_cancellation(
        &self,
        order_uid: &OrderUid,
        token: &cow_sdk_core::CancellationToken,
    ) -> Result<Order, OrderbookError> {
        match self.get_order_with_cancellation(order_uid, token).await {
            Ok(order) => Ok(order),
            Err(OrderbookError::Api(error)) if error.status == 404 => {
                let current_env = self.context.env;
                if let Some(fallback_env) = ENVS_LIST.into_iter().find(|env| *env != current_env) {
                    self.clone()
                        .with_context_override(ApiContextOverride {
                            env: Some(fallback_env),
                            ..ApiContextOverride::default()
                        })
                        .get_order_with_cancellation(order_uid, token)
                        .await
                } else {
                    Err(OrderbookError::Api(error))
                }
            }
            Err(error) => Err(error),
        }
    }

    /// Fetches and normalizes orders for a specific owner.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] if request execution fails or any order in
    /// the response cannot be normalized.
    pub async fn get_orders(
        &self,
        request: &GetOrdersRequest,
    ) -> Result<Vec<Order>, OrderbookError> {
        self.get_orders_with_cancellation(request, &cow_sdk_core::CancellationToken::new())
            .await
    }

    /// Fetches and normalizes orders for a specific owner with cooperative cancellation support.
    ///
    /// Returns [`OrderbookError::Cancelled`] if `token` fires before the
    /// orderbook responds. Response normalization is synchronous and happens
    /// only after the fetch resolves.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::Cancelled`] when `token` fires during the
    /// call, or any transport, API, or normalization error returned by the
    /// orderbook request helpers.
    pub async fn get_orders_with_cancellation(
        &self,
        request: &GetOrdersRequest,
        token: &cow_sdk_core::CancellationToken,
    ) -> Result<Vec<Order>, OrderbookError> {
        let params = FetchParams::new(
            format!("/api/v1/account/{}/orders", request.owner.as_str()),
            HttpMethod::Get,
        )
        .with_query("offset", request.offset.to_string())
        .with_query("limit", request.limit.to_string());

        let orders: Vec<Order> = tokio::select! {
            biased;
            () = token.cancelled() => return Err(OrderbookError::Cancelled),
            result = self.fetch_json(params) => result?,
        };

        transform_orders(orders)
    }

    /// Fetches and normalizes orders associated with a settlement transaction.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] if request execution fails or any order in
    /// the response cannot be normalized.
    pub async fn get_tx_orders(&self, tx_hash: &str) -> Result<Vec<Order>, OrderbookError> {
        self.get_tx_orders_with_cancellation(tx_hash, &cow_sdk_core::CancellationToken::new())
            .await
    }

    /// Fetches orders by settlement transaction hash with cooperative cancellation support.
    ///
    /// Returns [`OrderbookError::Cancelled`] if `token` fires before the
    /// orderbook responds. Response normalization is synchronous and happens
    /// only after the fetch resolves.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::Cancelled`] when `token` fires during the
    /// call, or any transport, API, or normalization error returned by the
    /// orderbook request helpers.
    pub async fn get_tx_orders_with_cancellation(
        &self,
        tx_hash: &str,
        token: &cow_sdk_core::CancellationToken,
    ) -> Result<Vec<Order>, OrderbookError> {
        let params = FetchParams::new(
            format!("/api/v1/transactions/{tx_hash}/orders"),
            HttpMethod::Get,
        );

        let orders: Vec<Order> = tokio::select! {
            biased;
            () = token.cancelled() => return Err(OrderbookError::Cancelled),
            result = self.fetch_json(params) => result?,
        };

        transform_orders(orders)
    }

    /// Fetches trades filtered by owner or order UID.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::InvalidTradesQuery`] when both or neither
    /// filters are set, or any transport/API/serialization error from the
    /// request helpers.
    pub async fn get_trades(
        &self,
        request: &GetTradesRequest,
    ) -> Result<Vec<Trade>, OrderbookError> {
        self.get_trades_with_cancellation(request, &cow_sdk_core::CancellationToken::new())
            .await
    }

    /// Fetches trades with cooperative cancellation support.
    ///
    /// Returns [`OrderbookError::Cancelled`] if `token` fires before the
    /// orderbook responds.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::InvalidTradesQuery`] when both or neither
    /// filters are set, [`OrderbookError::Cancelled`] when `token` fires
    /// during the call, or any transport/API/serialization error from the
    /// request helpers.
    pub async fn get_trades_with_cancellation(
        &self,
        request: &GetTradesRequest,
        token: &cow_sdk_core::CancellationToken,
    ) -> Result<Vec<Trade>, OrderbookError> {
        if !request.is_valid() {
            return Err(OrderbookError::InvalidTradesQuery(
                "must specify exactly one of owner or orderUid".to_owned(),
            ));
        }

        let mut params = FetchParams::new("/api/v2/trades", HttpMethod::Get);

        if let Some(owner) = &request.owner {
            params = params.with_query("owner", owner.as_str().to_owned());
        }

        if let Some(order_uid) = &request.order_uid {
            params = params.with_query("orderUid", order_uid.as_str().to_owned());
        }

        let params = params
            .with_query("offset", request.offset.to_string())
            .with_query("limit", request.limit.to_string());

        tokio::select! {
            biased;
            () = token.cancelled() => Err(OrderbookError::Cancelled),
            result = self.fetch_json(params) => result,
        }
    }

    /// Fetches the current competition status for an order.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] when request execution or response decoding fails.
    pub async fn get_order_competition_status(
        &self,
        order_uid: &OrderUid,
    ) -> Result<CompetitionOrderStatus, OrderbookError> {
        self.get_order_competition_status_with_cancellation(
            order_uid,
            &cow_sdk_core::CancellationToken::new(),
        )
        .await
    }

    /// Fetches the current competition status for an order with cooperative cancellation support.
    ///
    /// Returns [`OrderbookError::Cancelled`] if `token` fires before the
    /// orderbook responds.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::Cancelled`] when `token` fires during the
    /// call, or any transport or decoding error.
    pub async fn get_order_competition_status_with_cancellation(
        &self,
        order_uid: &OrderUid,
        token: &cow_sdk_core::CancellationToken,
    ) -> Result<CompetitionOrderStatus, OrderbookError> {
        let params = FetchParams::new(
            format!("/api/v1/orders/{}/status", order_uid.as_str()),
            HttpMethod::Get,
        );

        tokio::select! {
            biased;
            () = token.cancelled() => Err(OrderbookError::Cancelled),
            result = self.fetch_json(params) => result,
        }
    }

    /// Fetches the token price quoted in the chain's native asset.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] when request execution or response decoding fails.
    pub async fn get_native_price(
        &self,
        token: &crate::types::Address,
    ) -> Result<NativePriceResponse, OrderbookError> {
        self.get_native_price_with_cancellation(token, &cow_sdk_core::CancellationToken::new())
            .await
    }

    /// Fetches the native-asset-denominated price of a token with cooperative cancellation support.
    ///
    /// Returns [`OrderbookError::Cancelled`] if `cancellation` fires before
    /// the orderbook responds.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::Cancelled`] when `cancellation` fires during
    /// the call, or any transport or decoding error.
    pub async fn get_native_price_with_cancellation(
        &self,
        token: &crate::types::Address,
        cancellation: &cow_sdk_core::CancellationToken,
    ) -> Result<NativePriceResponse, OrderbookError> {
        let params = FetchParams::new(
            format!("/api/v1/token/{}/native_price", token.as_str()),
            HttpMethod::Get,
        );

        tokio::select! {
            biased;
            () = cancellation.cancelled() => Err(OrderbookError::Cancelled),
            result = self.fetch_json(params) => result,
        }
    }

    /// Fetches the recorded total surplus for a user.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] when request execution or response decoding fails.
    pub async fn get_total_surplus(
        &self,
        owner: &crate::types::Address,
    ) -> Result<TotalSurplus, OrderbookError> {
        self.get_total_surplus_with_cancellation(owner, &cow_sdk_core::CancellationToken::new())
            .await
    }

    /// Fetches the recorded total surplus for a user with cooperative cancellation support.
    ///
    /// Returns [`OrderbookError::Cancelled`] if `token` fires before the
    /// orderbook responds.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::Cancelled`] when `token` fires during the
    /// call, or any transport or decoding error.
    pub async fn get_total_surplus_with_cancellation(
        &self,
        owner: &crate::types::Address,
        token: &cow_sdk_core::CancellationToken,
    ) -> Result<TotalSurplus, OrderbookError> {
        let params = FetchParams::new(
            format!("/api/v1/users/{}/total_surplus", owner.as_str()),
            HttpMethod::Get,
        );

        tokio::select! {
            biased;
            () = token.cancelled() => Err(OrderbookError::Cancelled),
            result = self.fetch_json(params) => result,
        }
    }

    /// Fetches full app-data JSON for the provided app-data hash.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] when request execution or response decoding fails.
    pub async fn get_app_data(
        &self,
        app_data_hash: &AppDataHash,
    ) -> Result<AppDataObject, OrderbookError> {
        self.get_app_data_with_cancellation(app_data_hash, &cow_sdk_core::CancellationToken::new())
            .await
    }

    /// Fetches full app-data JSON for the provided app-data hash with cooperative cancellation support.
    ///
    /// Returns [`OrderbookError::Cancelled`] if `token` fires before the
    /// orderbook responds.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::Cancelled`] when `token` fires during the
    /// call, or any transport or decoding error.
    pub async fn get_app_data_with_cancellation(
        &self,
        app_data_hash: &AppDataHash,
        token: &cow_sdk_core::CancellationToken,
    ) -> Result<AppDataObject, OrderbookError> {
        let params = FetchParams::new(
            format!("/api/v1/app_data/{}", app_data_hash.as_str()),
            HttpMethod::Get,
        );

        tokio::select! {
            biased;
            () = token.cancelled() => Err(OrderbookError::Cancelled),
            result = self.fetch_json(params) => result,
        }
    }

    /// Uploads full app-data JSON for the provided app-data hash.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] if the request body cannot be encoded, the
    /// API rejects the upload, or request execution fails.
    pub async fn upload_app_data(
        &self,
        app_data_hash: &AppDataHash,
        full_app_data: &str,
    ) -> Result<AppDataObject, OrderbookError> {
        self.upload_app_data_with_cancellation(
            app_data_hash,
            full_app_data,
            &cow_sdk_core::CancellationToken::new(),
        )
        .await
    }

    /// Uploads full app-data JSON with cooperative cancellation support.
    ///
    /// Returns [`OrderbookError::Cancelled`] if `token` fires before the
    /// orderbook acknowledges the upload. An unacknowledged upload is a no-op
    /// on the orderbook service.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::Cancelled`] when `token` fires during the
    /// call, or any transport, API, or serialization error from the request
    /// helpers.
    pub async fn upload_app_data_with_cancellation(
        &self,
        app_data_hash: &AppDataHash,
        full_app_data: &str,
        token: &cow_sdk_core::CancellationToken,
    ) -> Result<AppDataObject, OrderbookError> {
        let params = FetchParams::new(
            format!("/api/v1/app_data/{}", app_data_hash.as_str()),
            HttpMethod::Put,
        )
        .with_body(json!({ "fullAppData": full_app_data }));

        tokio::select! {
            biased;
            () = token.cancelled() => Err(OrderbookError::Cancelled),
            result = self.fetch_json(params) => result,
        }
    }

    /// Fetches solver-competition data by auction id.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] when request execution or response decoding fails.
    pub async fn get_solver_competition_by_auction_id(
        &self,
        auction_id: i64,
    ) -> Result<SolverCompetitionResponse, OrderbookError> {
        self.get_solver_competition_by_auction_id_with_cancellation(
            auction_id,
            &cow_sdk_core::CancellationToken::new(),
        )
        .await
    }

    /// Fetches solver-competition data by auction id with cooperative cancellation support.
    ///
    /// Returns [`OrderbookError::Cancelled`] if `token` fires before the
    /// orderbook responds.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::Cancelled`] when `token` fires during the
    /// call, or any transport or decoding error.
    pub async fn get_solver_competition_by_auction_id_with_cancellation(
        &self,
        auction_id: i64,
        token: &cow_sdk_core::CancellationToken,
    ) -> Result<SolverCompetitionResponse, OrderbookError> {
        let params = FetchParams::new(
            format!("/api/v1/solver_competition/{auction_id}"),
            HttpMethod::Get,
        );

        tokio::select! {
            biased;
            () = token.cancelled() => Err(OrderbookError::Cancelled),
            result = self.fetch_json(params) => result,
        }
    }

    /// Fetches solver-competition data by settlement transaction hash.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] when request execution or response decoding fails.
    pub async fn get_solver_competition_by_tx_hash(
        &self,
        tx_hash: &str,
    ) -> Result<SolverCompetitionResponse, OrderbookError> {
        self.get_solver_competition_by_tx_hash_with_cancellation(
            tx_hash,
            &cow_sdk_core::CancellationToken::new(),
        )
        .await
    }

    /// Fetches solver-competition data by settlement transaction hash with cooperative cancellation support.
    ///
    /// Returns [`OrderbookError::Cancelled`] if `token` fires before the
    /// orderbook responds.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::Cancelled`] when `token` fires during the
    /// call, or any transport or decoding error.
    pub async fn get_solver_competition_by_tx_hash_with_cancellation(
        &self,
        tx_hash: &str,
        token: &cow_sdk_core::CancellationToken,
    ) -> Result<SolverCompetitionResponse, OrderbookError> {
        let params = FetchParams::new(
            format!("/api/v1/solver_competition/by_tx_hash/{tx_hash}"),
            HttpMethod::Get,
        );

        tokio::select! {
            biased;
            () = token.cancelled() => Err(OrderbookError::Cancelled),
            result = self.fetch_json(params) => result,
        }
    }

    /// Fetches the latest solver-competition snapshot from the orderbook.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] when request execution or response decoding fails.
    pub async fn get_latest_solver_competition(
        &self,
    ) -> Result<SolverCompetitionResponse, OrderbookError> {
        self.get_latest_solver_competition_with_cancellation(
            &cow_sdk_core::CancellationToken::new(),
        )
        .await
    }

    /// Fetches the latest solver-competition snapshot with cooperative cancellation support.
    ///
    /// Returns [`OrderbookError::Cancelled`] if `token` fires before the
    /// orderbook responds.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::Cancelled`] when `token` fires during the
    /// call, or any transport or decoding error.
    pub async fn get_latest_solver_competition_with_cancellation(
        &self,
        token: &cow_sdk_core::CancellationToken,
    ) -> Result<SolverCompetitionResponse, OrderbookError> {
        let params = FetchParams::new("/api/v1/solver_competition/latest", HttpMethod::Get);

        tokio::select! {
            biased;
            () = token.cancelled() => Err(OrderbookError::Cancelled),
            result = self.fetch_json(params) => result,
        }
    }

    /// Fetches the current auction snapshot from the orderbook.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError`] when request execution or response decoding fails.
    pub async fn get_auction(&self) -> Result<Auction, OrderbookError> {
        self.get_auction_with_cancellation(&cow_sdk_core::CancellationToken::new())
            .await
    }

    /// Fetches the current auction snapshot with cooperative cancellation support.
    ///
    /// Returns [`OrderbookError::Cancelled`] if `token` fires before the
    /// orderbook responds.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::Cancelled`] when `token` fires during the
    /// call, or any transport or decoding error.
    pub async fn get_auction_with_cancellation(
        &self,
        token: &cow_sdk_core::CancellationToken,
    ) -> Result<Auction, OrderbookError> {
        let params = FetchParams::new("/api/v1/auction", HttpMethod::Get);

        tokio::select! {
            biased;
            () = token.cancelled() => Err(OrderbookError::Cancelled),
            result = self.fetch_json(params) => result,
        }
    }

    async fn fetch_json<T>(&self, params: FetchParams) -> Result<T, OrderbookError>
    where
        T: serde::de::DeserializeOwned,
    {
        request_json_with_timeout(
            &self.client,
            &self.resolved_base_url(&self.context)?,
            &params,
            self.transport_policy.request_policy(),
            &self.rate_limiter,
            self.client_policy().timeout(),
            self.additional_headers()?,
        )
        .await
    }

    async fn fetch_empty(&self, params: FetchParams) -> Result<(), OrderbookError> {
        request_empty_with_timeout(
            &self.client,
            &self.resolved_base_url(&self.context)?,
            &params,
            self.transport_policy.request_policy(),
            &self.rate_limiter,
            self.client_policy().timeout(),
            self.additional_headers()?,
        )
        .await
    }

    async fn fetch_text(&self, params: FetchParams) -> Result<String, OrderbookError> {
        request_text_with_timeout(
            &self.client,
            &self.resolved_base_url(&self.context)?,
            &params,
            self.transport_policy.request_policy(),
            &self.rate_limiter,
            self.client_policy().timeout(),
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
        self.context
            .validated_api_key()
            .map_err(cow_sdk_core::CoreError::from)?
            .map(|api_key| {
                let header_value = HeaderValue::from_str(api_key)
                    .expect("validated API keys must remain valid header values");
                let mut headers = HeaderMap::new();
                headers.insert(API_KEY_HEADER, header_value);
                headers
            })
            .map_or(Ok(None), |headers| Ok(Some(headers)))
    }
}

fn normalize_base_url(base_url: &str) -> String {
    base_url.trim_end_matches('/').to_owned()
}

fn build_client(policy: &HttpClientPolicy) -> Client {
    let builder = Client::builder().user_agent(policy.user_agent().to_owned());

    builder
        .build()
        .expect("validated orderbook client policy must remain buildable")
}

fn build_request_runtime(
    transport_policy: &OrderBookTransportPolicy,
) -> (Client, RequestRateLimiter) {
    (
        build_client(transport_policy.client_policy()),
        RequestRateLimiter::new(transport_policy.request_policy().rate_limit),
    )
}
