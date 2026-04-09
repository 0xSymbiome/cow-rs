use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};
use serde_json::json;

use crate::{
    error::OrderbookError,
    request::{
        FetchParams, HttpMethod, RequestPolicy, RequestRateLimiter, request_empty, request_json,
        request_text,
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

#[derive(Clone)]
pub struct OrderBookApi {
    client: Client,
    context: ApiContext,
    request_policy: RequestPolicy,
    rate_limiter: RequestRateLimiter,
    env_base_url_overrides: EnvBaseUrlOverrides,
}

impl OrderBookApi {
    pub fn new(context: ApiContext) -> Self {
        let request_policy = RequestPolicy::default();

        Self {
            client: Client::new(),
            rate_limiter: RequestRateLimiter::new(request_policy.rate_limit),
            request_policy,
            context,
            env_base_url_overrides: EnvBaseUrlOverrides::default(),
        }
    }

    pub fn new_with_base_url(context: ApiContext, base_url: impl Into<String>) -> Self {
        let env = context.env;
        Self::new(context).with_env_base_url(env, base_url.into())
    }

    pub fn with_env_base_url(mut self, env: CowEnv, base_url: impl Into<String>) -> Self {
        self.env_base_url_overrides
            .set(env, normalize_base_url(base_url.into()));
        self
    }

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

    pub fn context(&self) -> &ApiContext {
        &self.context
    }

    pub fn request_policy(&self) -> &RequestPolicy {
        &self.request_policy
    }

    pub fn get_order_link(&self, order_uid: &OrderUid) -> Result<String, OrderbookError> {
        Ok(format!(
            "{}/api/v1/orders/{}",
            self.resolved_base_url(&self.context)?,
            order_uid.as_str()
        ))
    }

    pub async fn get_version(&self) -> Result<String, OrderbookError> {
        self.fetch_text(FetchParams::new("/api/v1/version", HttpMethod::Get))
            .await
    }

    pub async fn get_quote(
        &self,
        request: &OrderQuoteRequest,
    ) -> Result<OrderQuoteResponse, OrderbookError> {
        if !request.is_valid() {
            return Err(OrderbookError::InvalidQuoteRequest(
                "quote side must set exactly one of sellAmountBeforeFee or buyAmountAfterFee"
                    .to_owned(),
            ));
        }

        self.fetch_json(
            FetchParams::new("/api/v1/quote", HttpMethod::Post).with_body(
                serde_json::to_value(request)
                    .map_err(|error| OrderbookError::Serialization(error.to_string()))?,
            ),
        )
        .await
    }

    pub async fn send_order(&self, request: &OrderCreation) -> Result<OrderUid, OrderbookError> {
        self.fetch_json(
            FetchParams::new("/api/v1/orders", HttpMethod::Post).with_body(
                serde_json::to_value(request)
                    .map_err(|error| OrderbookError::Serialization(error.to_string()))?,
            ),
        )
        .await
    }

    pub async fn send_signed_order_cancellations(
        &self,
        request: &OrderCancellations,
    ) -> Result<(), OrderbookError> {
        self.fetch_empty(
            FetchParams::new("/api/v1/orders", HttpMethod::Delete).with_body(
                serde_json::to_value(request)
                    .map_err(|error| OrderbookError::Serialization(error.to_string()))?,
            ),
        )
        .await
    }

    pub async fn get_order(&self, order_uid: &OrderUid) -> Result<Order, OrderbookError> {
        let order: Order = self
            .fetch_json(FetchParams::new(
                format!("/api/v1/orders/{}", order_uid.as_str()),
                HttpMethod::Get,
            ))
            .await?;

        transform_order(order)
    }

    pub async fn get_order_multi_env(&self, order_uid: &OrderUid) -> Result<Order, OrderbookError> {
        match self.get_order(order_uid).await {
            Ok(order) => Ok(order),
            Err(OrderbookError::Api(error)) if error.status == 404 => {
                let current_env = self.context.env;
                let fallback_env = ENVS_LIST
                    .into_iter()
                    .find(|env| *env != current_env)
                    .expect("ENVS_LIST must contain at least one alternative environment");

                self.clone()
                    .with_context_override(ApiContextOverride {
                        env: Some(fallback_env),
                        ..ApiContextOverride::default()
                    })
                    .get_order(order_uid)
                    .await
            }
            Err(error) => Err(error),
        }
    }

    pub async fn get_orders(
        &self,
        request: &GetOrdersRequest,
    ) -> Result<Vec<Order>, OrderbookError> {
        let orders: Vec<Order> = self
            .fetch_json(
                FetchParams::new(
                    format!("/api/v1/account/{}/orders", request.owner.as_str()),
                    HttpMethod::Get,
                )
                .with_query("offset", request.offset.to_string())
                .with_query("limit", request.limit.to_string()),
            )
            .await?;

        transform_orders(orders)
    }

    pub async fn get_tx_orders(&self, tx_hash: &str) -> Result<Vec<Order>, OrderbookError> {
        let orders: Vec<Order> = self
            .fetch_json(FetchParams::new(
                format!("/api/v1/transactions/{tx_hash}/orders"),
                HttpMethod::Get,
            ))
            .await?;

        transform_orders(orders)
    }

    pub async fn get_trades(
        &self,
        request: &GetTradesRequest,
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

        self.fetch_json(
            params
                .with_query("offset", request.offset.to_string())
                .with_query("limit", request.limit.to_string()),
        )
        .await
    }

    pub async fn get_order_competition_status(
        &self,
        order_uid: &OrderUid,
    ) -> Result<CompetitionOrderStatus, OrderbookError> {
        self.fetch_json(FetchParams::new(
            format!("/api/v1/orders/{}/status", order_uid.as_str()),
            HttpMethod::Get,
        ))
        .await
    }

    pub async fn get_native_price(
        &self,
        token: &crate::types::Address,
    ) -> Result<NativePriceResponse, OrderbookError> {
        self.fetch_json(FetchParams::new(
            format!("/api/v1/token/{}/native_price", token.as_str()),
            HttpMethod::Get,
        ))
        .await
    }

    pub async fn get_total_surplus(
        &self,
        owner: &crate::types::Address,
    ) -> Result<TotalSurplus, OrderbookError> {
        self.fetch_json(FetchParams::new(
            format!("/api/v1/users/{}/total_surplus", owner.as_str()),
            HttpMethod::Get,
        ))
        .await
    }

    pub async fn get_app_data(
        &self,
        app_data_hash: &AppDataHash,
    ) -> Result<AppDataObject, OrderbookError> {
        self.fetch_json(FetchParams::new(
            format!("/api/v1/app_data/{}", app_data_hash.as_str()),
            HttpMethod::Get,
        ))
        .await
    }

    pub async fn upload_app_data(
        &self,
        app_data_hash: &AppDataHash,
        full_app_data: &str,
    ) -> Result<AppDataObject, OrderbookError> {
        self.fetch_json(
            FetchParams::new(
                format!("/api/v1/app_data/{}", app_data_hash.as_str()),
                HttpMethod::Put,
            )
            .with_body(json!({ "fullAppData": full_app_data })),
        )
        .await
    }

    pub async fn get_solver_competition_by_auction_id(
        &self,
        auction_id: i64,
    ) -> Result<SolverCompetitionResponse, OrderbookError> {
        self.fetch_json(FetchParams::new(
            format!("/api/v1/solver_competition/{auction_id}"),
            HttpMethod::Get,
        ))
        .await
    }

    pub async fn get_solver_competition_by_tx_hash(
        &self,
        tx_hash: &str,
    ) -> Result<SolverCompetitionResponse, OrderbookError> {
        self.fetch_json(FetchParams::new(
            format!("/api/v1/solver_competition/by_tx_hash/{tx_hash}"),
            HttpMethod::Get,
        ))
        .await
    }

    pub async fn get_auction(&self) -> Result<Auction, OrderbookError> {
        self.fetch_json(FetchParams::new("/api/v1/auction", HttpMethod::Get))
            .await
    }

    async fn fetch_json<T>(&self, params: FetchParams) -> Result<T, OrderbookError>
    where
        T: serde::de::DeserializeOwned,
    {
        request_json(
            &self.client,
            &self.resolved_base_url(&self.context)?,
            &params,
            &self.request_policy,
            &self.rate_limiter,
            self.additional_headers(),
        )
        .await
    }

    async fn fetch_empty(&self, params: FetchParams) -> Result<(), OrderbookError> {
        request_empty(
            &self.client,
            &self.resolved_base_url(&self.context)?,
            &params,
            &self.request_policy,
            &self.rate_limiter,
            self.additional_headers(),
        )
        .await
    }

    async fn fetch_text(&self, params: FetchParams) -> Result<String, OrderbookError> {
        request_text(
            &self.client,
            &self.resolved_base_url(&self.context)?,
            &params,
            &self.request_policy,
            &self.rate_limiter,
            self.additional_headers(),
        )
        .await
    }

    fn resolved_base_url(&self, context: &ApiContext) -> Result<String, OrderbookError> {
        if let Some(override_url) = self.env_base_url_overrides.get(context.env) {
            return Ok(override_url.to_owned());
        }

        Ok(normalize_base_url(context.resolved_base_url()?))
    }

    fn additional_headers(&self) -> Option<HeaderMap> {
        self.context.api_key.as_ref().and_then(|api_key| {
            let mut headers = HeaderMap::new();
            let header_value = HeaderValue::from_str(api_key).ok()?;
            headers.insert(API_KEY_HEADER, header_value);
            Some(headers)
        })
    }
}

fn normalize_base_url(base_url: String) -> String {
    base_url.trim_end_matches('/').to_owned()
}
