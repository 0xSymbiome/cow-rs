use cow_sdk_core::{Address, AppDataHash, OrderUid};
use cow_sdk_orderbook::{
    GetOrdersRequest, GetTradesRequest, OrderCancellations, OrderCreation, OrderbookApi,
};
use cow_sdk_pure_helpers as pure;
use cow_sdk_transport_policy::TransportPolicy;
use serde_json::json;
use wasm_bindgen::prelude::*;

use crate::exports::{
    cancel::{
        ClientCallScope, SdkClientOptions, run_with_client_options, transport_policy_with_timeout,
    },
    dto::{
        AppDataObjectDto, OrderCreationInput, OrderInput, OrderQuoteRequestInput,
        PaginationOptions, SignedCancellationsInput, SignedOrderDto, TradesQueryInput,
        ecdsa_signing_scheme, from_json_value, orderbook_signing_scheme, parse_chain, parse_order,
        to_js_value, transport_policy_from_config,
    },
    envelope::WasmEnvelope,
    errors::WasmError,
    transport::{configured_fetch_transport, optional_string, optional_timeout, required_u32},
};

#[wasm_bindgen]
extern "C" {
    /// Configuration object used to construct an `OrderBookClient`.
    ///
    /// The public TypeScript facade accepts `chainId`, optional `env`, optional
    /// `apiKey`, an explicit `transport`, optional `transportPolicy`, and
    /// default cancellation settings.
    #[wasm_bindgen(typescript_type = "OrderBookClientConfig")]
    pub type OrderBookClientConfig;
}

/// Orderbook client backed by an explicitly configured HTTP transport.
///
/// Construct this client when JavaScript needs direct access to quote,
/// submission, lookup, trade, native-price, app-data, and cancellation orderbook
/// endpoints. The client owns one callback registration and releases raw wasm
/// resources through the facade `dispose()` method.
#[wasm_bindgen]
pub struct OrderBookClient {
    inner: OrderbookApi,
    _callback_guard: crate::exports::registry::FetchCallbackGuard,
}

#[wasm_bindgen]
impl OrderBookClient {
    /// Creates an orderbook client from a single config object.
    ///
    /// The config must include `chainId` and `transport`. The optional
    /// `timeoutMs`, `signal`, and `transportPolicy` fields become defaults for
    /// calls made through this client unless a method call overrides them.
    ///
    /// @param config Orderbook client configuration.
    /// @throws CowError when the chain, environment, transport, or policy is invalid.
    #[wasm_bindgen(constructor)]
    pub fn new(config: OrderBookClientConfig) -> Result<OrderBookClient, JsValue> {
        let config = config.as_ref();
        let chain_id = required_u32(config, "chainId")?;
        let env = optional_string(config, "env")?;
        let api_key = optional_string(config, "apiKey")?;
        let timeout = optional_timeout(config)?;
        let transport_policy =
            transport_policy_from_config(config, TransportPolicy::default_orderbook(), timeout)?;
        let (transport, callback_guard) = configured_fetch_transport(
            config,
            timeout,
            transport_policy.client_policy().max_response_bytes(),
        )?;
        Ok(Self {
            inner: build_orderbook(chain_id, env, transport, transport_policy, api_key)?,
            _callback_guard: callback_guard,
        })
    }

    /// Fetches a price quote from the orderbook API.
    ///
    /// The request is converted to the typed orderbook quote request and sent
    /// through the configured transport. Per-call options can override the
    /// constructor timeout or attach an `AbortSignal`.
    ///
    /// @param request Quote request DTO.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing the quote response.
    /// @throws CowError for invalid input, transport failure, timeout, or cancellation.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.orderbook.quote"))
    )]
    #[wasm_bindgen(
        js_name = "getQuote",
        unchecked_return_type = "WasmEnvelope<OrderQuoteResponseDto>"
    )]
    pub async fn quote(
        &self,
        request: OrderQuoteRequestInput,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let inner = orderbook_for_scope(&self.inner, &scope);
        run_with_client_options(
            scope,
            async move { orderbook_get_quote(&inner, request).await },
        )
        .await
    }

    /// Submits a signed order to the orderbook.
    ///
    /// The signed DTO normally comes from a signing helper in the same package.
    /// The SDK reconstructs the typed order creation payload and returns the
    /// order UID assigned by the orderbook service.
    ///
    /// @param signed Signed order DTO including typed data, signature, owner, and scheme.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing the submitted order UID.
    /// @throws CowError for invalid signatures, transport failure, timeout, or rejection.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.orderbook.send_order"))
    )]
    #[wasm_bindgen(js_name = "sendOrder", unchecked_return_type = "WasmEnvelope<string>")]
    pub async fn send_order(
        &self,
        signed: SignedOrderDto,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let inner = orderbook_for_scope(&self.inner, &scope);
        run_with_client_options(
            scope,
            async move { orderbook_send_order(&inner, signed).await },
        )
        .await
    }

    /// Submits a raw order-creation payload to the orderbook.
    ///
    /// Use this method when the host already has a complete orderbook
    /// `OrderCreation` shape and does not need the facade to reconstruct it
    /// from a signed-order DTO.
    ///
    /// @param input Raw order-creation DTO.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing the submitted order UID.
    /// @throws CowError for malformed input, transport failure, timeout, or rejection.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.orderbook.send_order_creation"))
    )]
    #[wasm_bindgen(
        js_name = "sendOrderCreation",
        unchecked_return_type = "WasmEnvelope<string>"
    )]
    pub async fn send_order_creation(
        &self,
        input: OrderCreationInput,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let inner = orderbook_for_scope(&self.inner, &scope);
        run_with_client_options(scope, async move {
            orderbook_send_order_creation(&inner, input).await
        })
        .await
    }

    /// Fetches one order by its canonical order UID.
    ///
    /// The UID must be the full 56-byte CoW order UID encoded as a `0x`-prefixed
    /// string. The response is returned in the orderbook wire DTO shape.
    ///
    /// @param orderUid Full order UID to look up.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing the order response.
    /// @throws CowError for invalid UID, not-found responses, transport failure, or timeout.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.orderbook.order"))
    )]
    #[wasm_bindgen(js_name = "getOrder", unchecked_return_type = "WasmEnvelope<OrderDto>")]
    pub async fn order(
        &self,
        #[wasm_bindgen(js_name = orderUid)] order_uid: String,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let inner = orderbook_for_scope(&self.inner, &scope);
        run_with_client_options(scope, async move {
            orderbook_get_order(&inner, order_uid).await
        })
        .await
    }

    /// Fetches trades for exactly one owner address or order UID.
    ///
    /// The query must set one of `owner` or `orderUid`, not both. Optional
    /// pagination fields are forwarded to the orderbook request.
    ///
    /// @param query Trade query DTO.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing matching trades.
    /// @throws CowError when the query is ambiguous or transport fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.orderbook.trades"))
    )]
    #[wasm_bindgen(
        js_name = "getTrades",
        unchecked_return_type = "WasmEnvelope<TradeDto[]>"
    )]
    pub async fn trades(
        &self,
        query: TradesQueryInput,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let inner = orderbook_for_scope(&self.inner, &scope);
        run_with_client_options(
            scope,
            async move { orderbook_get_trades(&inner, query).await },
        )
        .await
    }

    /// Fetches orders owned by an address.
    ///
    /// This compatibility method is equivalent to `getOrders` and accepts the
    /// same pagination options. New TypeScript code can use `getOrders`.
    ///
    /// @param owner Owner address to query.
    /// @param pagination Optional offset and limit.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing matching orders.
    /// @throws CowError for invalid owner, transport failure, timeout, or cancellation.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.orderbook.orders_by_owner"))
    )]
    #[wasm_bindgen(
        js_name = "getOrdersByOwner",
        unchecked_return_type = "WasmEnvelope<OrderDto[]>"
    )]
    pub async fn orders_by_owner(
        &self,
        owner: String,
        #[wasm_bindgen(js_name = pagination)] pagination: Option<PaginationOptions>,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let inner = orderbook_for_scope(&self.inner, &scope);
        run_with_client_options(scope, async move {
            orderbook_get_orders_by_owner(&inner, owner, pagination).await
        })
        .await
    }

    /// Fetches orders owned by an address with optional pagination.
    ///
    /// The owner address is validated before the request is dispatched. The
    /// response preserves the typed orderbook order shape.
    ///
    /// @param owner Owner address to query.
    /// @param pagination Optional offset and limit.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing matching orders.
    /// @throws CowError for invalid owner, transport failure, timeout, or cancellation.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.orderbook.orders"))
    )]
    #[wasm_bindgen(
        js_name = "getOrders",
        unchecked_return_type = "WasmEnvelope<OrderDto[]>"
    )]
    pub async fn orders(
        &self,
        owner: String,
        #[wasm_bindgen(js_name = pagination)] pagination: Option<PaginationOptions>,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let inner = orderbook_for_scope(&self.inner, &scope);
        run_with_client_options(scope, async move {
            orderbook_get_orders_by_owner(&inner, owner, pagination).await
        })
        .await
    }

    /// Fetches a token's native price from the orderbook API.
    ///
    /// The token must be an EVM address. The returned value follows the
    /// orderbook native-price response shape.
    ///
    /// @param token Token address to price.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing native price data.
    /// @throws CowError for invalid token address, transport failure, or timeout.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.orderbook.native_price"))
    )]
    #[wasm_bindgen(
        js_name = "getNativePrice",
        unchecked_return_type = "WasmEnvelope<NativePriceResponseDto>"
    )]
    pub async fn native_price(
        &self,
        token: String,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let inner = orderbook_for_scope(&self.inner, &scope);
        run_with_client_options(scope, async move {
            orderbook_get_native_price(&inner, token).await
        })
        .await
    }

    /// Submits signed off-chain order cancellations.
    ///
    /// Build the signed cancellation payload with one of the cancellation
    /// signing helpers, then submit it through the same orderbook runtime
    /// configuration used for order operations.
    ///
    /// @param signed Signed cancellation payload.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing `{ cancelled: true }` on success.
    /// @throws CowError for invalid UID, signature, transport failure, or timeout.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.orderbook.cancel_orders"))
    )]
    #[wasm_bindgen(
        js_name = "cancelOrders",
        unchecked_return_type = "WasmEnvelope<{ cancelled: true }>"
    )]
    pub async fn cancel_orders(
        &self,
        signed: SignedCancellationsInput,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let inner = orderbook_for_scope(&self.inner, &scope);
        run_with_client_options(scope, async move {
            orderbook_cancel_orders(&inner, signed).await
        })
        .await
    }

    /// Fetches the full app-data document registered for an app-data hash.
    ///
    /// Use this to retrieve the canonical app-data payload the orderbook holds
    /// for a given hash, for example to display or re-verify a document
    /// referenced by an order.
    ///
    /// @param appDataHash App-data hash as a `0x`-prefixed 32-byte hex string.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing the app-data document.
    /// @throws CowError for an invalid hash, transport failure, or timeout.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.orderbook.app_data"))
    )]
    #[wasm_bindgen(
        js_name = "getAppData",
        unchecked_return_type = "WasmEnvelope<AppDataObjectDto>"
    )]
    pub async fn app_data(
        &self,
        #[wasm_bindgen(js_name = appDataHash)] app_data_hash: String,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let inner = orderbook_for_scope(&self.inner, &scope);
        run_with_client_options(scope, async move {
            orderbook_get_app_data(&inner, app_data_hash).await
        })
        .await
    }

    /// Uploads the full app-data JSON for a content-addressed app-data hash.
    ///
    /// The SDK enforces the content-addressed-write invariant: the keccak-256
    /// digest of `fullAppData` must equal `appDataHash`, or the call rejects
    /// before any network request. Serialize `fullAppData` with the canonical
    /// app-data writer so the digest matches.
    ///
    /// @param appDataHash App-data hash as a `0x`-prefixed 32-byte hex string.
    /// @param fullAppData Canonically serialized app-data JSON payload.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing `{ uploaded: true }` on success.
    /// @throws CowError for a hash mismatch, invalid hash, transport failure, or timeout.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.orderbook.upload_app_data"))
    )]
    #[wasm_bindgen(
        js_name = "uploadAppData",
        unchecked_return_type = "WasmEnvelope<{ uploaded: true }>"
    )]
    pub async fn upload_app_data(
        &self,
        #[wasm_bindgen(js_name = appDataHash)] app_data_hash: String,
        #[wasm_bindgen(js_name = fullAppData)] full_app_data: String,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let inner = orderbook_for_scope(&self.inner, &scope);
        run_with_client_options(scope, async move {
            orderbook_upload_app_data(&inner, app_data_hash, full_app_data).await
        })
        .await
    }
}

pub(crate) fn build_orderbook(
    chain_id: u32,
    env: Option<String>,
    transport: std::sync::Arc<dyn cow_sdk_core::HttpTransport + Send + Sync>,
    transport_policy: TransportPolicy,
    api_key: Option<String>,
) -> Result<OrderbookApi, JsValue> {
    let chain = parse_chain(chain_id)?;
    let env = pure::chains::env_from_str(env.as_deref())
        .map_err(|error| WasmError::from(error).into_js())?;
    let mut builder = OrderbookApi::builder()
        .chain(chain)
        .environment(env)
        .transport(transport)
        .transport_policy(transport_policy);
    if let Some(api_key) = api_key {
        builder = builder.api_key(api_key);
    }
    builder
        .build()
        .map_err(|error| WasmError::from(error).into_js())
}

pub(crate) fn orderbook_for_scope(inner: &OrderbookApi, scope: &ClientCallScope) -> OrderbookApi {
    inner
        .clone()
        .with_transport_policy(transport_policy_with_timeout(
            inner.transport_policy(),
            scope.timeout(),
        ))
}

async fn orderbook_get_app_data(
    inner: &OrderbookApi,
    app_data_hash: String,
) -> Result<JsValue, JsValue> {
    let hash = parse_app_data_hash(&app_data_hash)?;
    let object = inner
        .app_data(&hash)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(AppDataObjectDto::from(object)))
}

async fn orderbook_upload_app_data(
    inner: &OrderbookApi,
    app_data_hash: String,
    full_app_data: String,
) -> Result<JsValue, JsValue> {
    let hash = parse_app_data_hash(&app_data_hash)?;
    inner
        .upload_app_data(&hash, &full_app_data)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(json!({ "uploaded": true })))
}

fn parse_app_data_hash(value: &str) -> Result<AppDataHash, JsValue> {
    AppDataHash::new(value.to_owned())
        .map_err(|error| WasmError::invalid("appDataHash", error.to_string()).into_js())
}

async fn orderbook_get_quote(
    inner: &OrderbookApi,
    request: OrderQuoteRequestInput,
) -> Result<JsValue, JsValue> {
    let request = from_json_value("quote", request.into_value()?)?;
    let response = inner
        .quote(&request)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(response))
}

async fn orderbook_send_order(
    inner: &OrderbookApi,
    signed: SignedOrderDto,
) -> Result<JsValue, JsValue> {
    let request = order_creation_from_signed(signed)?;
    let uid = inner
        .send_order(&request)
        .await
        .map(|uid| uid.to_hex_string())
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(uid))
}

async fn orderbook_send_order_creation(
    inner: &OrderbookApi,
    input: OrderCreationInput,
) -> Result<JsValue, JsValue> {
    let request = from_json_value("order", input.into_value()?)?;
    let uid = inner
        .send_order(&request)
        .await
        .map(|uid| uid.to_hex_string())
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(uid))
}

async fn orderbook_get_order(inner: &OrderbookApi, order_uid: String) -> Result<JsValue, JsValue> {
    let order_uid = parse_order_uid(order_uid)?;
    let order = inner
        .order(&order_uid)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(order))
}

async fn orderbook_get_trades(
    inner: &OrderbookApi,
    query: TradesQueryInput,
) -> Result<JsValue, JsValue> {
    let mut request = match (query.owner, query.order_uid) {
        (Some(owner), None) => GetTradesRequest::by_owner(parse_address("owner", owner)?),
        (None, Some(order_uid)) => GetTradesRequest::by_order_uid(parse_order_uid(order_uid)?),
        _ => {
            return Err(WasmError::invalid(
                "trades",
                "exactly one of owner or orderUid must be set",
            )
            .into_js());
        }
    };
    if let Some(offset) = query.offset {
        request = request.with_offset(offset);
    }
    if let Some(limit) = query.limit {
        request = request.with_limit(limit);
    }
    let trades = inner
        .trades(&request)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(trades))
}

async fn orderbook_get_orders_by_owner(
    inner: &OrderbookApi,
    owner: String,
    pagination: Option<PaginationOptions>,
) -> Result<JsValue, JsValue> {
    let owner = parse_address("owner", owner)?;
    let mut request = GetOrdersRequest::new(owner);
    if let Some(pagination) = pagination {
        if let Some(offset) = pagination.offset {
            request = request.with_offset(offset);
        }
        if let Some(limit) = pagination.limit {
            request = request.with_limit(limit);
        }
    }
    let orders = inner
        .orders(&request)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(orders))
}

async fn orderbook_get_native_price(
    inner: &OrderbookApi,
    token: String,
) -> Result<JsValue, JsValue> {
    let token = parse_address("token", token)?;
    let price = inner
        .native_price(&token)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(price))
}

async fn orderbook_cancel_orders(
    inner: &OrderbookApi,
    signed: SignedCancellationsInput,
) -> Result<JsValue, JsValue> {
    let order_uids = signed
        .order_uids
        .into_iter()
        .map(parse_order_uid)
        .collect::<Result<Vec<_>, _>>()?;
    let scheme = ecdsa_signing_scheme(&signed.signing_scheme)?;
    let request = OrderCancellations::new(order_uids, signed.signature).with_signing_scheme(scheme);
    inner
        .send_signed_order_cancellations(&request)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(json!({ "cancelled": true })))
}

pub(crate) fn order_creation_from_signed(signed: SignedOrderDto) -> Result<OrderCreation, JsValue> {
    let order_input: OrderInput = serde_json::from_value(signed.typed_data.message.clone())
        .map_err(|error| WasmError::invalid("typedData.message", error.to_string()).into_js())?;
    let order = parse_order(order_input)?;
    let from = parse_address("from", signed.from)?;
    let signing_scheme = orderbook_signing_scheme(&signed.signing_scheme)?;
    let mut creation = OrderCreation::new(
        order.sell_token.clone(),
        order.buy_token.clone(),
        order.sell_amount.clone(),
        order.buy_amount.clone(),
        order.valid_to,
        order.kind,
        signing_scheme,
        signed.signature,
        from,
    )
    .with_app_data_hash(order.app_data.clone())
    .with_partially_fillable(order.partially_fillable)
    .with_sell_token_balance(order.sell_token_balance)
    .with_buy_token_balance(order.buy_token_balance);

    if !is_zero_address(&order.receiver) {
        creation = creation.with_receiver(order.receiver.clone());
    }
    if let Some(quote_id) = signed.quote_id {
        creation = creation.with_quote_id(quote_id);
    }

    Ok(creation)
}

fn parse_order_uid(order_uid: String) -> Result<OrderUid, JsValue> {
    OrderUid::new(order_uid)
        .map_err(|error| WasmError::invalid("orderUid", error.to_string()).into_js())
}

fn parse_address(field: &'static str, value: String) -> Result<Address, JsValue> {
    Address::new(value).map_err(|error| WasmError::invalid(field, error.to_string()).into_js())
}

fn is_zero_address(address: &Address) -> bool {
    address
        .to_hex_string()
        .eq_ignore_ascii_case("0x0000000000000000000000000000000000000000")
}
