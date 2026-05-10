use cow_sdk_core::{Address, OrderUid};
use cow_sdk_orderbook::{
    GetOrdersRequest, GetTradesRequest, OrderBookApi, OrderCancellations, OrderCreation,
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
        OrderCreationInput, OrderInput, OrderQuoteRequestInput, SignedCancellationsInput,
        SignedOrderDto, ecdsa_signing_scheme, from_json_value, orderbook_signing_scheme,
        parse_chain, parse_order, to_js_value, transport_policy_from_config,
    },
    envelope::WasmEnvelope,
    errors::WasmError,
    transport::{configured_fetch_transport, optional_string, optional_timeout, required_u32},
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "OrderBookClientConfig")]
    pub type OrderBookClientConfig;
}

/// Orderbook client backed by an explicitly configured HTTP transport.
#[wasm_bindgen]
pub struct OrderBookClient {
    inner: OrderBookApi,
    _callback_guard: crate::exports::registry::FetchCallbackGuard,
}

#[wasm_bindgen]
impl OrderBookClient {
    /// Creates an orderbook client from a single config object.
    #[wasm_bindgen(constructor)]
    pub fn new(config: OrderBookClientConfig) -> Result<OrderBookClient, JsValue> {
        let config = config.as_ref();
        let chain_id = required_u32(config, "chainId")?;
        let env = optional_string(config, "env")?;
        let timeout = optional_timeout(config)?;
        let transport_policy =
            transport_policy_from_config(config, TransportPolicy::default_orderbook(), timeout)?;
        let (transport, callback_guard) = configured_fetch_transport(config, timeout)?;
        Ok(Self {
            inner: build_orderbook(chain_id, env, transport, transport_policy)?,
            _callback_guard: callback_guard,
        })
    }

    /// Fetches a quote.
    #[wasm_bindgen(js_name = "getQuote")]
    pub async fn get_quote(
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

    /// Submits a signed order.
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

    /// Submits a raw order-creation payload.
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

    /// Fetches an order by UID.
    #[wasm_bindgen(js_name = "getOrder")]
    pub async fn get_order(
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

    /// Fetches trades for an order UID.
    #[wasm_bindgen(js_name = "getTrades")]
    pub async fn get_trades(
        &self,
        #[wasm_bindgen(js_name = orderUid)] order_uid: String,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let inner = orderbook_for_scope(&self.inner, &scope);
        run_with_client_options(scope, async move {
            orderbook_get_trades(&inner, order_uid).await
        })
        .await
    }

    /// Fetches orders owned by an address.
    #[wasm_bindgen(js_name = "getOrdersByOwner")]
    pub async fn get_orders_by_owner(
        &self,
        owner: String,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let inner = orderbook_for_scope(&self.inner, &scope);
        run_with_client_options(scope, async move {
            orderbook_get_orders_by_owner(&inner, owner).await
        })
        .await
    }

    /// Fetches a token's native price.
    #[wasm_bindgen(js_name = "getNativePrice")]
    pub async fn get_native_price(
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

    /// Cancels orders through a signed cancellation payload.
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
}

pub(crate) fn build_orderbook(
    chain_id: u32,
    env: Option<String>,
    transport: std::sync::Arc<dyn cow_sdk_core::HttpTransport + Send + Sync>,
    transport_policy: TransportPolicy,
) -> Result<OrderBookApi, JsValue> {
    let chain = parse_chain(chain_id)?;
    let env = pure::chains::env_from_str(env.as_deref())
        .map_err(|error| WasmError::from(error).into_js())?;
    OrderBookApi::builder()
        .chain(chain)
        .environment(env)
        .transport(transport)
        .transport_policy(transport_policy)
        .build()
        .map_err(|error| WasmError::from(error).into_js())
}

pub(crate) fn orderbook_for_scope(inner: &OrderBookApi, scope: &ClientCallScope) -> OrderBookApi {
    inner
        .clone()
        .with_transport_policy(transport_policy_with_timeout(
            inner.transport_policy(),
            scope.timeout(),
        ))
}

async fn orderbook_get_quote(
    inner: &OrderBookApi,
    request: OrderQuoteRequestInput,
) -> Result<JsValue, JsValue> {
    let request = from_json_value("quote", request.into_value()?)?;
    let response = inner
        .get_quote(&request)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(response))
}

async fn orderbook_send_order(
    inner: &OrderBookApi,
    signed: SignedOrderDto,
) -> Result<JsValue, JsValue> {
    let request = order_creation_from_signed(signed)?;
    let uid = inner
        .send_order(&request)
        .await
        .map(|uid| uid.as_str().to_owned())
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(uid))
}

async fn orderbook_send_order_creation(
    inner: &OrderBookApi,
    input: OrderCreationInput,
) -> Result<JsValue, JsValue> {
    let request = from_json_value("order", input.into_value()?)?;
    let uid = inner
        .send_order(&request)
        .await
        .map(|uid| uid.as_str().to_owned())
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(uid))
}

async fn orderbook_get_order(inner: &OrderBookApi, order_uid: String) -> Result<JsValue, JsValue> {
    let order_uid = parse_order_uid(order_uid)?;
    let order = inner
        .get_order(&order_uid)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(order))
}

async fn orderbook_get_trades(inner: &OrderBookApi, order_uid: String) -> Result<JsValue, JsValue> {
    let order_uid = parse_order_uid(order_uid)?;
    let request = GetTradesRequest::by_order_uid(order_uid);
    let trades = inner
        .get_trades(&request)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(trades))
}

async fn orderbook_get_orders_by_owner(
    inner: &OrderBookApi,
    owner: String,
) -> Result<JsValue, JsValue> {
    let owner = parse_address("owner", owner)?;
    let request = GetOrdersRequest::new(owner);
    let orders = inner
        .get_orders(&request)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(orders))
}

async fn orderbook_get_native_price(
    inner: &OrderBookApi,
    token: String,
) -> Result<JsValue, JsValue> {
    let token = parse_address("token", token)?;
    let price = inner
        .get_native_price(&token)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(price))
}

async fn orderbook_cancel_orders(
    inner: &OrderBookApi,
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
        .as_str()
        .eq_ignore_ascii_case("0x0000000000000000000000000000000000000000")
}
