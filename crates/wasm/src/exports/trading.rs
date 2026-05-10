use std::sync::Arc;

use cow_sdk_contracts::eth_flow::{EthFlowOrderData, encode_create_order_calldata};
use cow_sdk_core::{
    Address, Amount, AsyncProvider, AsyncSigner, AsyncTypedDataSigner, BlockInfo, ContractCall,
    ContractHandle, EVM_NATIVE_CURRENCY_ADDRESS, HexData, ProtocolOptions, TransactionBroadcast,
    TransactionHash, TransactionReceipt, TransactionRequest, TypedDataDomain, TypedDataField,
};
use cow_sdk_orderbook::{OrderBookApi, SigningScheme};
use cow_sdk_pure_helpers as pure;
use cow_sdk_trading::{
    AllowanceParameters, GAS_LIMIT_DEFAULT, LimitTradeParameters, OrderPostingResult,
    QuoteRequestOverride, SwapAdvancedSettings, TradeParameters, TradingSdk,
};
use cow_sdk_transport_policy::TransportPolicy;
use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::exports::{
    cancel::{
        ClientCallScope, SdkClientOptions, SigningOptions, run_with_client_options,
        signing_wallet_timeout_ms,
    },
    dto::{
        AllowanceParametersInput, BuiltSellNativeCurrencyTxDto, ContractCallDto,
        CowEip1271SignRequest, LimitTradeParametersInput, OrderInput, QuoteResultsInput,
        SwapParametersInput, TransactionRequestDto, TypedDataEnvelopeDto, from_json_value,
        parse_chain, parse_order, to_js_value, transport_policy_from_config,
    },
    envelope::WasmEnvelope,
    errors::WasmError,
    orderbook::{build_orderbook, order_creation_from_signed, orderbook_for_scope},
    signing::{
        JsTypedDataSigner, await_callback_string, js_error_to_string, normalize_signature,
        signed_order_from_parts,
    },
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
        let api_key = optional_string(config, "apiKey")?;
        let timeout = optional_timeout(config)?;
        let transport_policy =
            transport_policy_from_config(config, TransportPolicy::default_trading(), timeout)?;
        let (transport, callback_guard) = configured_fetch_transport(config, timeout)?;
        let orderbook = build_orderbook(
            chain_id,
            env.clone(),
            Arc::clone(&transport),
            transport_policy,
            api_key,
        )?;
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
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.trading.get_quote"))
    )]
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
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.trading.post_swap_order"))
    )]
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

    /// Signs and posts a previously quoted swap order.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(endpoint = "wasm.trading.post_swap_order_from_quote")
        )
    )]
    #[wasm_bindgen(js_name = "postSwapOrderFromQuote")]
    pub async fn post_swap_order_from_quote(
        &self,
        #[wasm_bindgen(js_name = quoteResults)] quote_results: QuoteResultsInput,
        owner: String,
        #[wasm_bindgen(js_name = signerCallback, unchecked_param_type = "TypedDataSignerCallback")]
        signer_callback: Function,
        #[wasm_bindgen(js_name = options)] options: Option<SigningOptions>,
    ) -> Result<JsValue, JsValue> {
        let options_ref = options.as_ref().map(AsRef::as_ref);
        let scope = ClientCallScope::new(options_ref)?;
        let wallet_timeout_ms = signing_wallet_timeout_ms(options_ref)?;
        let orderbook = orderbook_for_scope(&self.orderbook, &scope);
        let chain_id = self.chain_id;
        run_with_client_options(scope, async move {
            trading_post_swap_order_from_quote(
                &orderbook,
                chain_id,
                quote_results,
                owner,
                signer_callback,
                wallet_timeout_ms,
            )
            .await
        })
        .await
    }

    /// Signs and posts a limit order through a typed-data callback.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.trading.post_limit_order"))
    )]
    #[wasm_bindgen(js_name = "postLimitOrder")]
    pub async fn post_limit_order(
        &self,
        params: LimitTradeParametersInput,
        owner: String,
        #[wasm_bindgen(js_name = signerCallback, unchecked_param_type = "TypedDataSignerCallback")]
        signer_callback: Function,
        #[wasm_bindgen(js_name = options)] options: Option<SigningOptions>,
    ) -> Result<JsValue, JsValue> {
        let options_ref = options.as_ref().map(AsRef::as_ref);
        let scope = ClientCallScope::new(options_ref)?;
        let wallet_timeout_ms = signing_wallet_timeout_ms(options_ref)?;
        let inner = self.trading_for_scope(&scope)?;
        run_with_client_options(scope, async move {
            trading_post_limit_order(&inner, params, owner, signer_callback, wallet_timeout_ms)
                .await
        })
        .await
    }

    /// Builds the transaction for a native-currency sell order.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(endpoint = "wasm.trading.build_sell_native_currency_tx")
        )
    )]
    #[wasm_bindgen(
        js_name = "buildSellNativeCurrencyTx",
        unchecked_return_type = "WasmEnvelope<BuiltSellNativeCurrencyTxDto>"
    )]
    pub async fn build_sell_native_currency_tx(
        &self,
        order: OrderInput,
        #[wasm_bindgen(js_name = quoteId)] quote_id: i64,
        from: String,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let chain_id = self.chain_id;
        let env = self.env.clone();
        run_with_client_options(scope, async move {
            trading_build_sell_native_currency_tx(chain_id, env, order, quote_id, from).await
        })
        .await
    }

    /// Reads CoW Protocol allowance through a read-only contract callback.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(endpoint = "wasm.trading.get_cow_protocol_allowance")
        )
    )]
    #[wasm_bindgen(
        js_name = "getCowProtocolAllowance",
        unchecked_return_type = "WasmEnvelope<string>"
    )]
    pub async fn get_cow_protocol_allowance(
        &self,
        params: AllowanceParametersInput,
        #[wasm_bindgen(js_name = readContractCallback, unchecked_param_type = "ContractReadCallback")]
        read_contract_callback: Function,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let inner = self.trading_for_scope(&scope)?;
        run_with_client_options(scope, async move {
            trading_get_cow_protocol_allowance(&inner, params, read_contract_callback).await
        })
        .await
    }

    /// Quotes and posts a swap order with a custom EIP-1271 signature callback.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(endpoint = "wasm.trading.post_swap_order_with_eip1271")
        )
    )]
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

async fn trading_post_swap_order_from_quote(
    orderbook: &OrderBookApi,
    chain_id: u32,
    quote_results: QuoteResultsInput,
    owner: String,
    signer_callback: Function,
    wallet_timeout_ms: Option<u32>,
) -> Result<JsValue, JsValue> {
    let owner = parse_address("owner", owner)?;
    let order = parse_order(quote_results.order_to_sign.clone())?;
    let chain = parse_chain(chain_id)?;
    let payload = pure::signing::order_typed_data_payload(chain, &order)
        .map_err(|error| WasmError::from(error).into_js())?;
    let typed_data = TypedDataEnvelopeDto::from_payload(&payload)?;
    let signer = JsTypedDataSigner::new(owner.clone(), signer_callback, wallet_timeout_ms);
    let signature = signer
        .sign_typed_data_payload(&payload)
        .await
        .map_err(|error| WasmError::wallet("signTypedData", error).into_js())?;
    let generated = pure::signing::generate_order_id(chain, &order, &owner)
        .map_err(|error| WasmError::from(error).into_js())?;
    let signed = signed_order_from_parts(
        generated,
        owner,
        typed_data,
        signature.clone(),
        "eip712",
        quote_results.quote_id(),
    );
    let request = order_creation_from_signed(signed)?;
    let uid = orderbook
        .send_order(&request)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    let result = OrderPostingResult::new(uid, SigningScheme::Eip712, signature, order);
    to_js_value(&WasmEnvelope::v1(result))
}

async fn trading_post_limit_order(
    inner: &TradingSdk,
    params: LimitTradeParametersInput,
    owner: String,
    signer_callback: Function,
    wallet_timeout_ms: Option<u32>,
) -> Result<JsValue, JsValue> {
    let owner = parse_address("owner", owner)?;
    let mut params: LimitTradeParameters = from_json_value("params", params.into_value()?)?;
    params.owner = params.owner.or_else(|| Some(owner.clone()));
    let signer = JsTradingSigner::new(owner, signer_callback, wallet_timeout_ms);
    let result = inner
        .post_limit_order_async(params, &signer, None)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(result))
}

async fn trading_build_sell_native_currency_tx(
    chain_id: u32,
    env: Option<String>,
    input: OrderInput,
    quote_id: i64,
    from: String,
) -> Result<JsValue, JsValue> {
    let from = parse_address("from", from)?;
    let order = parse_order(input)?;
    if !order
        .sell_token
        .as_str()
        .eq_ignore_ascii_case(EVM_NATIVE_CURRENCY_ADDRESS)
    {
        return Err(WasmError::invalid(
            "order.sellToken",
            "native-currency sell transactions require the native token sentinel address",
        )
        .into_js());
    }
    let chain = parse_chain(chain_id)?;
    let env = pure::chains::env_from_str(env.as_deref())
        .map_err(|error| WasmError::from(error).into_js())?;
    let options = ProtocolOptions::new().with_env(env);
    let eth_flow = cow_sdk_contracts::Registry::default()
        .address(cow_sdk_contracts::ContractId::EthFlow, chain, env)
        .ok_or_else(|| {
            WasmError::invalid(
                "chainId",
                "EthFlow deployment is not available for this chain and environment",
            )
            .into_js()
        })?;
    let payload = EthFlowOrderData::from_unsigned_order(&order, quote_id);
    let data = HexData::new(format!(
        "0x{}",
        hex::encode(
            encode_create_order_calldata(&payload)
                .map_err(|error| WasmError::from(error).into_js())?
        )
    ))
    .map_err(|error| WasmError::from(error).into_js())?;
    let generated = cow_sdk_trading::calculate_unique_order_id(chain, &order, None, Some(&options))
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    let tx = TransactionRequest::new(
        Some(eth_flow),
        Some(data),
        Some(order.sell_amount.clone()),
        Some(default_gas_limit()?),
    );
    let result = BuiltSellNativeCurrencyTxDto {
        order_uid: generated.order_id.as_str().to_owned(),
        transaction: TransactionRequestDto::from(&tx),
        order_to_sign: OrderInput::from(&order),
        from: from.as_str().to_owned(),
    };
    to_js_value(&WasmEnvelope::v1(result))
}

async fn trading_get_cow_protocol_allowance(
    inner: &TradingSdk,
    params: AllowanceParametersInput,
    read_contract_callback: Function,
) -> Result<JsValue, JsValue> {
    let params: AllowanceParameters = from_json_value("params", params.into_value()?)?;
    let provider = JsContractReadProvider::new(read_contract_callback);
    let allowance = inner
        .get_cow_protocol_allowance_async(&provider, &params)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(allowance))
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

fn default_gas_limit() -> Result<Amount, JsValue> {
    Amount::new(GAS_LIMIT_DEFAULT.to_string())
        .map_err(|error| WasmError::invalid("gasLimit", error.to_string()).into_js())
}

struct JsTradingSigner {
    owner: Address,
    callback: Function,
    wallet_timeout_ms: Option<u32>,
}

impl JsTradingSigner {
    const fn new(owner: Address, callback: Function, wallet_timeout_ms: Option<u32>) -> Self {
        Self {
            owner,
            callback,
            wallet_timeout_ms,
        }
    }
}

impl AsyncSigner for JsTradingSigner {
    type Error = String;

    async fn get_address(&self) -> Result<Address, Self::Error> {
        Ok(self.owner.clone())
    }

    async fn sign_message(&self, _message: &[u8]) -> Result<String, Self::Error> {
        Err("message signing is not available through this typed-data callback".to_owned())
    }

    async fn sign_transaction(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
        Err("transaction signing is not available through this typed-data callback".to_owned())
    }

    async fn sign_typed_data_payload(
        &self,
        payload: &cow_sdk_core::TypedDataPayload,
    ) -> Result<String, Self::Error> {
        let envelope = TypedDataEnvelopeDto::from_payload(payload)
            .map_err(|error| js_error_to_string(error.into_js()))?;
        let value = envelope.callback_value().map_err(js_error_to_string)?;
        let signature = await_callback_string(
            &self.callback,
            value,
            "signTypedData",
            self.wallet_timeout_ms,
        )
        .await
        .map_err(js_error_to_string)?;
        normalize_signature(&signature).map_err(js_error_to_string)
    }

    async fn sign_typed_data(
        &self,
        _domain: &TypedDataDomain,
        _fields: &[TypedDataField],
        _value_json: &str,
    ) -> Result<String, Self::Error> {
        Err("field-based typed-data signing is not available through this callback".to_owned())
    }

    async fn send_transaction(
        &self,
        _tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        Err("transaction submission is not available through this typed-data callback".to_owned())
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Err("gas estimation is not available through this typed-data callback".to_owned())
    }
}

struct JsContractReadProvider {
    callback: Function,
}

impl JsContractReadProvider {
    const fn new(callback: Function) -> Self {
        Self { callback }
    }
}

impl AsyncProvider for JsContractReadProvider {
    type Error = String;

    async fn get_chain_id(&self) -> Result<u64, Self::Error> {
        Err("chain id reads are not available through this contract-read callback".to_owned())
    }

    async fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
        Err("code reads are not available through this contract-read callback".to_owned())
    }

    async fn get_transaction_receipt(
        &self,
        _transaction_hash: &TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        Err("receipt reads are not available through this contract-read callback".to_owned())
    }

    async fn get_storage_at(
        &self,
        _address: &Address,
        _slot: &str,
    ) -> Result<HexData, Self::Error> {
        Err("storage reads are not available through this contract-read callback".to_owned())
    }

    async fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        Err("raw calls are not available through this contract-read callback".to_owned())
    }

    async fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
        let value = to_js_value(&ContractCallDto::from(request)).map_err(js_error_to_string)?;
        await_callback_string(&self.callback, value, "readContract", None)
            .await
            .map_err(js_error_to_string)
    }

    async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Err("block reads are not available through this contract-read callback".to_owned())
    }

    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle::new(address.clone(), abi_json.to_owned()))
    }
}
