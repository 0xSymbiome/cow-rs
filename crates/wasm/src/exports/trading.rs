use std::sync::Arc;

use cow_sdk_core::Address;
use cow_sdk_orderbook::SigningScheme;
use cow_sdk_pure_helpers as pure;
use cow_sdk_trading::{
    PostTradeAdditionalParams, QuoteRequestOverride, SwapAdvancedSettings, TradeParameters,
    TradingSdk,
};
use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::exports::{
    dto::{SwapParametersInput, WasmEnvelope, from_json_value, parse_chain, to_js_value},
    eip1271::{Eip1271CallbackGuard, RegisteredEip1271Provider},
    errors::WasmError,
    orderbook::build_orderbook,
    signing::{JsTypedDataSigner, OwnerOnlySigner},
    transport::{
        configured_fetch_transport, optional_string, optional_timeout, required_string,
        required_u32,
    },
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "TradingClientConfig")]
    pub type TradingClientConfig;
}

/// Trading facade backed by an explicitly configured HTTP transport.
#[wasm_bindgen]
pub struct TradingClient {
    inner: TradingSdk,
    chain_id: u32,
    _callback_guard: crate::exports::registry::FetchCallbackGuard,
}

#[wasm_bindgen]
impl TradingClient {
    /// Creates a trading client from a single config object.
    #[wasm_bindgen(constructor)]
    pub fn new(config: TradingClientConfig) -> Result<TradingClient, JsValue> {
        let config = config.as_ref();
        let chain_id = required_u32(config, "chainId")?;
        let env = optional_string(config, "env")?;
        let app_code = required_string(config, "appCode")?;
        let timeout = optional_timeout(config)?;
        let (transport, callback_guard) = configured_fetch_transport(config, timeout)?;
        Ok(Self {
            inner: build_trading(chain_id, env, app_code, transport)?,
            chain_id,
            _callback_guard: callback_guard,
        })
    }

    /// Fetches a quote without submitting an order.
    #[wasm_bindgen(js_name = "getQuote")]
    pub async fn get_quote(&self, params: SwapParametersInput) -> Result<JsValue, JsValue> {
        trading_get_quote(&self.inner, params).await
    }

    /// Quotes, signs, and posts a swap order through a typed-data callback.
    #[wasm_bindgen(js_name = "postSwapOrder")]
    pub async fn post_swap_order(
        &self,
        params: SwapParametersInput,
        owner: String,
        signer_callback: Function,
    ) -> Result<JsValue, JsValue> {
        trading_post_swap_order(&self.inner, params, owner, signer_callback).await
    }

    /// Quotes and posts a swap order with a custom EIP-1271 signature callback.
    #[wasm_bindgen(js_name = "postSwapOrderWithEip1271")]
    pub async fn post_swap_order_with_eip1271(
        &self,
        params: SwapParametersInput,
        owner: String,
        custom_callback: Function,
    ) -> Result<JsValue, JsValue> {
        trading_post_swap_order_with_eip1271(
            &self.inner,
            self.chain_id,
            params,
            owner,
            custom_callback,
        )
        .await
    }
}

fn build_trading(
    chain_id: u32,
    env: Option<String>,
    app_code: String,
    transport: Arc<dyn cow_sdk_core::HttpTransport + Send + Sync>,
) -> Result<TradingSdk, JsValue> {
    let chain = parse_chain(chain_id)?;
    let env_value = pure::chains::env_from_str(env.as_deref())
        .map_err(|error| WasmError::from(error).into_js())?;
    let orderbook = Arc::new(build_orderbook(chain_id, env, transport)?);
    TradingSdk::builder()
        .with_chain_id(chain)
        .with_app_code(app_code)
        .with_env(env_value)
        .with_orderbook_client(orderbook)
        .build_ready()
        .map_err(|error| WasmError::from(error).into_js())
}

async fn trading_get_quote(
    inner: &TradingSdk,
    params: SwapParametersInput,
) -> Result<JsValue, JsValue> {
    let params: TradeParameters = from_json_value("params", params.value)?;
    let quote = inner
        .get_quote_only(params, None)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(quote))
}

async fn trading_post_swap_order(
    inner: &TradingSdk,
    params: SwapParametersInput,
    owner: String,
    signer_callback: Function,
) -> Result<JsValue, JsValue> {
    let owner = parse_address("owner", owner)?;
    let mut params: TradeParameters = from_json_value("params", params.value)?;
    params.owner = Some(owner.clone());
    let signer = JsTypedDataSigner::new(owner, signer_callback);
    let result = inner
        .post_swap_order_async(params, &signer, None)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(result))
}

async fn trading_post_swap_order_with_eip1271(
    inner: &TradingSdk,
    chain_id: u32,
    params: SwapParametersInput,
    owner: String,
    custom_callback: Function,
) -> Result<JsValue, JsValue> {
    let owner_address = parse_address("owner", owner.clone())?;
    let mut params: TradeParameters = from_json_value("params", params.value)?;
    params.owner = Some(owner_address.clone());
    let guard = Eip1271CallbackGuard::register(custom_callback)?;
    let provider = Arc::new(RegisteredEip1271Provider::new(
        guard.id(),
        owner_address.as_str().to_owned(),
        chain_id,
    ));
    let quote_request = QuoteRequestOverride::new()
        .with_from(owner_address.clone())
        .with_signing_scheme(SigningScheme::Eip1271);
    let additional = PostTradeAdditionalParams::new()
        .with_signing_scheme(SigningScheme::Eip1271)
        .with_custom_eip1271_signature(provider);
    let advanced = SwapAdvancedSettings::new()
        .with_quote_request(quote_request)
        .with_additional_params(additional);
    let signer = OwnerOnlySigner::new(owner_address);
    let result = inner
        .post_swap_order_async(params, &signer, Some(&advanced))
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    drop(guard);
    to_js_value(&WasmEnvelope::v1(result))
}

fn parse_address(field: &'static str, value: String) -> Result<Address, JsValue> {
    Address::new(value).map_err(|error| WasmError::invalid(field, error.to_string()).into_js())
}
