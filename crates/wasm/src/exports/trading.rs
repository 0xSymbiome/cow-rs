use std::sync::Arc;

use cow_sdk_core::{Address, AsyncTypedDataSigner};
use cow_sdk_orderbook::{OrderBookApi, SigningScheme};
use cow_sdk_pure_helpers as pure;
use cow_sdk_trading::{
    OrderPostingResult, QuoteRequestOverride, SwapAdvancedSettings, TradeParameters, TradingSdk,
};
use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::exports::{
    cancel::{
        ClientCallScope, SdkClientOptions, SigningOptions, run_with_client_options,
        signing_wallet_timeout_ms,
    },
    dto::{
        CowEip1271SignRequest, OrderInput, SwapParametersInput, TypedDataEnvelopeDto,
        from_json_value, parse_chain, to_js_value,
    },
    envelope::WasmEnvelope,
    errors::WasmError,
    orderbook::{build_orderbook, order_creation_from_signed, orderbook_for_scope},
    signing::{JsTypedDataSigner, await_callback_string, signed_order_from_parts},
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
    orderbook: OrderBookApi,
    chain_id: u32,
    env: Option<String>,
    app_code: String,
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
        let orderbook = build_orderbook(chain_id, env.clone(), Arc::clone(&transport))?;
        build_trading_with_orderbook(
            chain_id,
            env.clone(),
            app_code.clone(),
            Arc::new(orderbook.clone()),
        )?;
        Ok(Self {
            orderbook,
            chain_id,
            env,
            app_code,
            _callback_guard: callback_guard,
        })
    }

    /// Fetches a quote without submitting an order.
    #[wasm_bindgen(js_name = "getQuote")]
    pub async fn get_quote(
        &self,
        params: SwapParametersInput,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let inner = self.trading_for_scope(&scope)?;
        run_with_client_options(
            scope,
            async move { trading_get_quote(&inner, params).await },
        )
        .await
    }

    /// Quotes, signs, and posts a swap order through a typed-data callback.
    #[wasm_bindgen(js_name = "postSwapOrder")]
    pub async fn post_swap_order(
        &self,
        params: SwapParametersInput,
        owner: String,
        #[wasm_bindgen(js_name = signerCallback, unchecked_param_type = "TypedDataSignerCallback")]
        signer_callback: Function,
        #[wasm_bindgen(js_name = options)] options: Option<SigningOptions>,
    ) -> Result<JsValue, JsValue> {
        let options_ref = options.as_ref().map(AsRef::as_ref);
        let scope = ClientCallScope::new(options_ref)?;
        let wallet_timeout_ms = signing_wallet_timeout_ms(options_ref)?;
        let inner = self.trading_for_scope(&scope)?;
        let orderbook = orderbook_for_scope(&self.orderbook, &scope);
        let chain_id = self.chain_id;
        run_with_client_options(scope, async move {
            trading_post_swap_order(
                &inner,
                &orderbook,
                chain_id,
                params,
                owner,
                signer_callback,
                wallet_timeout_ms,
            )
            .await
        })
        .await
    }

    /// Quotes and posts a swap order with a custom EIP-1271 signature callback.
    #[wasm_bindgen(js_name = "postSwapOrderWithEip1271")]
    pub async fn post_swap_order_with_eip1271(
        &self,
        params: SwapParametersInput,
        owner: String,
        #[wasm_bindgen(js_name = customCallback, unchecked_param_type = "CustomEip1271Callback")]
        custom_callback: Function,
        #[wasm_bindgen(js_name = options)] options: Option<SigningOptions>,
    ) -> Result<JsValue, JsValue> {
        let options_ref = options.as_ref().map(AsRef::as_ref);
        let scope = ClientCallScope::new(options_ref)?;
        let wallet_timeout_ms = signing_wallet_timeout_ms(options_ref)?;
        let inner = self.trading_for_scope(&scope)?;
        let orderbook = orderbook_for_scope(&self.orderbook, &scope);
        let chain_id = self.chain_id;
        run_with_client_options(scope, async move {
            trading_post_swap_order_with_eip1271(
                &inner,
                &orderbook,
                chain_id,
                params,
                owner,
                custom_callback,
                wallet_timeout_ms,
            )
            .await
        })
        .await
    }
}

impl TradingClient {
    fn trading_for_scope(&self, scope: &ClientCallScope) -> Result<TradingSdk, JsValue> {
        let orderbook = orderbook_for_scope(&self.orderbook, scope);
        build_trading_with_orderbook(
            self.chain_id,
            self.env.clone(),
            self.app_code.clone(),
            Arc::new(orderbook),
        )
    }
}

fn build_trading_with_orderbook(
    chain_id: u32,
    env: Option<String>,
    app_code: String,
    orderbook: Arc<OrderBookApi>,
) -> Result<TradingSdk, JsValue> {
    let chain = parse_chain(chain_id)?;
    let env_value = pure::chains::env_from_str(env.as_deref())
        .map_err(|error| WasmError::from(error).into_js())?;
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
    let params: TradeParameters = from_json_value("params", params.into_value()?)?;
    let quote = inner
        .get_quote_only(params, None)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(quote))
}

async fn trading_post_swap_order(
    inner: &TradingSdk,
    orderbook: &OrderBookApi,
    chain_id: u32,
    params: SwapParametersInput,
    owner: String,
    signer_callback: Function,
    wallet_timeout_ms: Option<u32>,
) -> Result<JsValue, JsValue> {
    let owner = parse_address("owner", owner)?;
    let mut params: TradeParameters = from_json_value("params", params.into_value()?)?;
    params.owner = Some(owner.clone());
    let quote = inner
        .get_quote_only(params, None)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    let chain = parse_chain(chain_id)?;
    let payload = pure::signing::order_typed_data_payload(chain, &quote.order_to_sign)
        .map_err(|error| WasmError::from(error).into_js())?;
    let typed_data = TypedDataEnvelopeDto::from_payload(&payload)?;
    let signer = JsTypedDataSigner::new(owner.clone(), signer_callback, wallet_timeout_ms);
    let signature = signer
        .sign_typed_data_payload(&payload)
        .await
        .map_err(|error| WasmError::wallet("signTypedData", error).into_js())?;
    let generated = pure::signing::generate_order_id(chain, &quote.order_to_sign, &owner)
        .map_err(|error| WasmError::from(error).into_js())?;
    let signed = signed_order_from_parts(
        generated,
        owner,
        typed_data,
        signature.clone(),
        "eip712",
        quote.quote_response.id,
    );
    let request = order_creation_from_signed(signed)?;
    let uid = orderbook
        .send_order(&request)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    let result =
        OrderPostingResult::new(uid, SigningScheme::Eip712, signature, quote.order_to_sign);
    to_js_value(&WasmEnvelope::v1(result))
}

async fn trading_post_swap_order_with_eip1271(
    inner: &TradingSdk,
    orderbook: &OrderBookApi,
    chain_id: u32,
    params: SwapParametersInput,
    owner: String,
    custom_callback: Function,
    wallet_timeout_ms: Option<u32>,
) -> Result<JsValue, JsValue> {
    let owner_address = parse_address("owner", owner.clone())?;
    let mut params: TradeParameters = from_json_value("params", params.into_value()?)?;
    params.owner = Some(owner_address.clone());
    let quote_settings = SwapAdvancedSettings::new().with_quote_request(
        QuoteRequestOverride::new()
            .with_from(owner_address.clone())
            .with_signing_scheme(SigningScheme::Eip1271),
    );
    let quote = inner
        .get_quote_only(params, Some(&quote_settings))
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    let chain = parse_chain(chain_id)?;
    let payload = pure::signing::order_typed_data_payload(chain, &quote.order_to_sign)
        .map_err(|error| WasmError::from(error).into_js())?;
    let typed_data = TypedDataEnvelopeDto::from_payload(&payload)?;
    let request = CowEip1271SignRequest {
        order: OrderInput::from(&quote.order_to_sign),
        typed_data: typed_data.clone(),
        owner: owner_address.as_str().to_owned(),
        chain_id,
    };
    let signature = await_callback_string(
        &custom_callback,
        to_js_value(&request)?,
        "eip1271",
        wallet_timeout_ms,
    )
    .await?;
    let generated = pure::signing::generate_order_id(chain, &quote.order_to_sign, &owner_address)
        .map_err(|error| WasmError::from(error).into_js())?;
    let signed = signed_order_from_parts(
        generated,
        owner_address,
        typed_data,
        signature.clone(),
        "eip1271",
        quote.quote_response.id,
    );
    let request = order_creation_from_signed(signed)?;
    let uid = orderbook
        .send_order(&request)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    let result =
        OrderPostingResult::new(uid, SigningScheme::Eip1271, signature, quote.order_to_sign);
    to_js_value(&WasmEnvelope::v1(result))
}

fn parse_address(field: &'static str, value: String) -> Result<Address, JsValue> {
    Address::new(value).map_err(|error| WasmError::invalid(field, error.to_string()).into_js())
}
