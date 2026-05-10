use cow_sdk_core::{Address, OrderUid};
use cow_sdk_orderbook::{
    GetOrdersRequest, GetTradesRequest, OrderBookApi, OrderCancellations, OrderCreation,
};
use cow_sdk_pure_helpers as pure;
use js_sys::Function;
use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

use crate::exports::{
    dto::{
        OrderCreationInput, OrderInput, OrderQuoteRequestInput, SignedCancellationsInput,
        SignedOrderDto, WasmEnvelope, ecdsa_signing_scheme, from_json_value,
        orderbook_signing_scheme, parse_chain, parse_order, to_js_value,
    },
    errors::WasmError,
    transport::{
        callback_fetch_transport, callback_fetch_transport_from_handle, default_fetch_transport,
    },
};

/// Orderbook client backed by the browser fetch transport.
#[wasm_bindgen]
pub struct OrderBookClient {
    inner: OrderBookApi,
}

#[wasm_bindgen]
impl OrderBookClient {
    /// Creates an orderbook client for a chain and environment.
    #[wasm_bindgen(constructor)]
    pub fn new(chain_id: u32, env: Option<String>) -> Result<OrderBookClient, JsValue> {
        Ok(Self {
            inner: build_orderbook(chain_id, env, default_fetch_transport(None))?,
        })
    }

    /// Fetches a quote.
    #[wasm_bindgen(js_name = "getQuote")]
    pub async fn get_quote(&self, request: OrderQuoteRequestInput) -> Result<JsValue, JsValue> {
        orderbook_get_quote(&self.inner, request).await
    }

    /// Submits a signed order.
    #[wasm_bindgen(js_name = "sendOrder")]
    pub async fn send_order(&self, signed: SignedOrderDto) -> Result<String, JsValue> {
        orderbook_send_order(&self.inner, signed).await
    }

    /// Submits a raw order-creation payload.
    #[wasm_bindgen(js_name = "sendOrderCreation")]
    pub async fn send_order_creation(&self, input: OrderCreationInput) -> Result<String, JsValue> {
        orderbook_send_order_creation(&self.inner, input).await
    }

    /// Fetches an order by UID.
    #[wasm_bindgen(js_name = "getOrder")]
    pub async fn get_order(&self, order_uid: String) -> Result<JsValue, JsValue> {
        orderbook_get_order(&self.inner, order_uid).await
    }

    /// Fetches trades for an order UID.
    #[wasm_bindgen(js_name = "getTrades")]
    pub async fn get_trades(&self, order_uid: String) -> Result<JsValue, JsValue> {
        orderbook_get_trades(&self.inner, order_uid).await
    }

    /// Fetches orders owned by an address.
    #[wasm_bindgen(js_name = "getOrdersByOwner")]
    pub async fn get_orders_by_owner(&self, owner: String) -> Result<JsValue, JsValue> {
        orderbook_get_orders_by_owner(&self.inner, owner).await
    }

    /// Fetches a token's native price.
    #[wasm_bindgen(js_name = "getNativePrice")]
    pub async fn get_native_price(&self, token: String) -> Result<JsValue, JsValue> {
        orderbook_get_native_price(&self.inner, token).await
    }

    /// Cancels orders through a signed cancellation payload.
    #[wasm_bindgen(js_name = "cancelOrders")]
    pub async fn cancel_orders(
        &self,
        signed: SignedCancellationsInput,
    ) -> Result<JsValue, JsValue> {
        orderbook_cancel_orders(&self.inner, signed).await
    }
}

/// Orderbook client backed by a JavaScript fetch callback.
#[wasm_bindgen]
pub struct OrderBookClientWithFetch {
    inner: OrderBookApi,
    _handle: Option<crate::exports::registry::FetchCallbackHandle>,
}

#[wasm_bindgen]
impl OrderBookClientWithFetch {
    /// Creates an orderbook client that owns a registered fetch callback.
    #[wasm_bindgen(constructor)]
    pub fn new(
        chain_id: u32,
        env: Option<String>,
        fetch_callback: Function,
    ) -> Result<OrderBookClientWithFetch, JsValue> {
        let (transport, handle) = callback_fetch_transport(fetch_callback, None)?;
        Ok(Self {
            inner: build_orderbook(chain_id, env, transport)?,
            _handle: Some(handle),
        })
    }

    /// Creates an orderbook client from an existing fetch-callback handle id.
    #[wasm_bindgen(js_name = "fromHandle")]
    pub fn from_handle(
        chain_id: u32,
        env: Option<String>,
        fetch_callback_id: u32,
    ) -> Result<OrderBookClientWithFetch, JsValue> {
        let transport = callback_fetch_transport_from_handle(fetch_callback_id, None)?;
        Ok(Self {
            inner: build_orderbook(chain_id, env, transport)?,
            _handle: None,
        })
    }

    /// Fetches a quote.
    #[wasm_bindgen(js_name = "getQuote")]
    pub async fn get_quote(&self, request: OrderQuoteRequestInput) -> Result<JsValue, JsValue> {
        orderbook_get_quote(&self.inner, request).await
    }

    /// Submits a signed order.
    #[wasm_bindgen(js_name = "sendOrder")]
    pub async fn send_order(&self, signed: SignedOrderDto) -> Result<String, JsValue> {
        orderbook_send_order(&self.inner, signed).await
    }

    /// Submits a raw order-creation payload.
    #[wasm_bindgen(js_name = "sendOrderCreation")]
    pub async fn send_order_creation(&self, input: OrderCreationInput) -> Result<String, JsValue> {
        orderbook_send_order_creation(&self.inner, input).await
    }

    /// Fetches an order by UID.
    #[wasm_bindgen(js_name = "getOrder")]
    pub async fn get_order(&self, order_uid: String) -> Result<JsValue, JsValue> {
        orderbook_get_order(&self.inner, order_uid).await
    }

    /// Fetches trades for an order UID.
    #[wasm_bindgen(js_name = "getTrades")]
    pub async fn get_trades(&self, order_uid: String) -> Result<JsValue, JsValue> {
        orderbook_get_trades(&self.inner, order_uid).await
    }

    /// Fetches orders owned by an address.
    #[wasm_bindgen(js_name = "getOrdersByOwner")]
    pub async fn get_orders_by_owner(&self, owner: String) -> Result<JsValue, JsValue> {
        orderbook_get_orders_by_owner(&self.inner, owner).await
    }

    /// Fetches a token's native price.
    #[wasm_bindgen(js_name = "getNativePrice")]
    pub async fn get_native_price(&self, token: String) -> Result<JsValue, JsValue> {
        orderbook_get_native_price(&self.inner, token).await
    }

    /// Cancels orders through a signed cancellation payload.
    #[wasm_bindgen(js_name = "cancelOrders")]
    pub async fn cancel_orders(
        &self,
        signed: SignedCancellationsInput,
    ) -> Result<JsValue, JsValue> {
        orderbook_cancel_orders(&self.inner, signed).await
    }
}

pub(crate) fn build_orderbook(
    chain_id: u32,
    env: Option<String>,
    transport: std::sync::Arc<dyn cow_sdk_core::HttpTransport + Send + Sync>,
) -> Result<OrderBookApi, JsValue> {
    let chain = parse_chain(chain_id)?;
    let env = pure::chains::env_from_str(env.as_deref())
        .map_err(|error| WasmError::from(error).into_js())?;
    OrderBookApi::builder()
        .chain(chain)
        .environment(env)
        .transport(transport)
        .build()
        .map_err(|error| WasmError::from(error).into_js())
}

async fn orderbook_get_quote(
    inner: &OrderBookApi,
    request: OrderQuoteRequestInput,
) -> Result<JsValue, JsValue> {
    let request = from_json_value("quote", request.value)?;
    let response = inner
        .get_quote(&request)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(response))
}

async fn orderbook_send_order(
    inner: &OrderBookApi,
    signed: SignedOrderDto,
) -> Result<String, JsValue> {
    let request = order_creation_from_signed(signed)?;
    inner
        .send_order(&request)
        .await
        .map(|uid| uid.as_str().to_owned())
        .map_err(|error| WasmError::from(error).into_js())
}

async fn orderbook_send_order_creation(
    inner: &OrderBookApi,
    input: OrderCreationInput,
) -> Result<String, JsValue> {
    let request = from_json_value("order", input.value)?;
    inner
        .send_order(&request)
        .await
        .map(|uid| uid.as_str().to_owned())
        .map_err(|error| WasmError::from(error).into_js())
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

fn order_creation_from_signed(signed: SignedOrderDto) -> Result<OrderCreation, JsValue> {
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

#[allow(dead_code)]
fn value_envelope(value: Value) -> WasmEnvelope<Value> {
    WasmEnvelope::v1(value)
}
