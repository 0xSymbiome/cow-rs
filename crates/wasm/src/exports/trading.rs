use std::sync::Arc;

use crate::helpers as pure;
use cow_sdk_contracts::eth_flow::{EthFlowOrderData, encode_create_order_calldata};
use cow_sdk_core::transport::policy::TransportPolicy;
use cow_sdk_core::{
    Address, Amount, BlockInfo, ContractCall, ContractHandle, HexData, NATIVE_CURRENCY_ADDRESS,
    ProtocolOptions, Provider, Signer, TransactionBroadcast, TransactionHash, TransactionReceipt,
    TransactionRequest,
};
use cow_sdk_orderbook::{OrderbookApi, SigningScheme};
use cow_sdk_trading::{
    AllowanceParams, DEFAULT_GAS_LIMIT, LimitTradeParams, PostTradeAdditionalParams,
    QuoteRequestOverride, QuoteResults, TradeAdvancedSettings, TradeParams, Trading,
};
use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::exports::{
    cancel::{
        ClientCallScope, SdkClientOptions, SigningOptions, run_with_client_options,
        signing_wallet_timeout_ms,
    },
    dto::{
        AllowanceParametersInput, BuiltSellNativeCurrencyTxDto, ContractCallDto,
        CowEip1271SignRequest, LimitTradeParametersInput, OrderInput, SwapParametersInput,
        TransactionRequestDto, TypedDataEnvelopeDto, from_json_value, parse_chain, parse_order,
        to_js_value, transport_policy_from_config,
    },
    eip1271::ResolvedEip1271Provider,
    envelope::WasmEnvelope,
    errors::WasmError,
    orderbook::{build_orderbook, orderbook_for_scope},
    signing::{await_callback_string, js_error_to_string, normalize_signature},
    transport::{
        configured_fetch_transport, optional_string, optional_timeout, required_string,
        required_u32,
    },
};

#[wasm_bindgen]
extern "C" {
    /// Configuration object used to construct a `TradingClient`.
    ///
    /// The public TypeScript facade accepts `chainId`, `appCode`, optional
    /// environment and API key, explicit HTTP transport, optional transport
    /// policy, and default cancellation settings.
    #[wasm_bindgen(typescript_type = "TradingClientConfig")]
    pub type TradingClientConfig;
}

/// High-level trading client backed by an explicitly configured orderbook.
///
/// Construct this client when JavaScript needs quote, sign, post, allowance,
/// and native-sell helper workflows rather than direct orderbook calls. The
/// client keeps app-code, chain, environment, transport, and policy defaults.
#[wasm_bindgen]
pub struct TradingClient {
    orderbook: OrderbookApi,
    chain_id: u32,
    env: Option<String>,
    app_code: String,
    _callback_guard: crate::exports::registry::FetchCallbackGuard,
}

#[wasm_bindgen]
impl TradingClient {
    /// Creates a trading client from a single config object.
    ///
    /// The config must include `chainId`, `appCode`, and `transport`. Optional
    /// environment, API key, timeout, signal, and transport policy fields become
    /// defaults for all trading methods.
    ///
    /// @param config Trading client configuration.
    /// @throws CowError when chain, app-code, environment, transport, or policy validation fails.
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
        let (transport, callback_guard) = configured_fetch_transport(
            config,
            timeout,
            transport_policy.client_policy().max_response_bytes(),
        )?;
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

    /// Fetches a quote without signing or submitting an order.
    ///
    /// Use this method when a host wants to preview the quote response before
    /// asking a wallet to sign or before constructing a post request.
    ///
    /// @param params Swap parameters DTO.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing quote results.
    /// @throws CowError for invalid parameters, transport failure, timeout, or cancellation.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.trading.quote"))
    )]
    #[wasm_bindgen(
        js_name = "getQuote",
        unchecked_return_type = "WasmEnvelope<QuoteResultsDto>"
    )]
    pub async fn quote(
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
    ///
    /// The SDK fetches a quote, builds the order to sign, invokes the callback
    /// with the EIP-712 envelope, posts the signed order, and returns posting
    /// output from the trading workflow.
    ///
    /// @param params Swap parameters DTO.
    /// @param owner Owner address to bind to the order.
    /// @param signerCallback Callback that signs the typed-data envelope.
    /// @param options Optional cancellation, timeout, and wallet timeout settings.
    /// @returns A versioned envelope containing order posting output.
    /// @throws CowError for invalid input, quote failure, wallet failure, timeout, or rejection.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.trading.post_swap_order"))
    )]
    #[wasm_bindgen(
        js_name = "postSwapOrder",
        unchecked_return_type = "WasmEnvelope<OrderPostingResultDto>"
    )]
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
        run_with_client_options(scope, async move {
            trading_post_swap_order(&inner, params, owner, signer_callback, wallet_timeout_ms).await
        })
        .await
    }

    /// Signs and posts a previously quoted swap order.
    ///
    /// Use this method when a host has already called `getQuote` and wants to
    /// reuse that quote result for posting without requesting a new quote.
    ///
    /// @param quoteResults Quote result DTO returned by `getQuote`.
    /// @param owner Owner address to bind to the order.
    /// @param signerCallback Callback that signs the typed-data envelope.
    /// @param options Optional cancellation, timeout, and wallet timeout settings.
    /// @returns A versioned envelope containing order posting output.
    /// @throws CowError for invalid quote data, wallet failure, timeout, or rejection.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(endpoint = "wasm.trading.post_swap_order_from_quote")
        )
    )]
    #[wasm_bindgen(
        js_name = "postSwapOrderFromQuote",
        unchecked_return_type = "WasmEnvelope<OrderPostingResultDto>"
    )]
    pub async fn post_swap_order_from_quote(
        &self,
        #[wasm_bindgen(js_name = quoteResults, unchecked_param_type = "QuoteResultsDto")]
        quote_results: JsValue,
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
            trading_post_swap_order_from_quote(
                &inner,
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
    ///
    /// This helper follows the native limit-order trading path and lets the SDK
    /// build, sign, and submit the order using the configured orderbook.
    ///
    /// @param params Limit-order parameters DTO.
    /// @param owner Owner address to bind to the order when absent from params.
    /// @param signerCallback Callback that signs the typed-data envelope.
    /// @param options Optional cancellation, timeout, and wallet timeout settings.
    /// @returns A versioned envelope containing order posting output.
    /// @throws CowError for invalid input, wallet failure, timeout, or rejection.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.trading.post_limit_order"))
    )]
    #[wasm_bindgen(
        js_name = "postLimitOrder",
        unchecked_return_type = "WasmEnvelope<OrderPostingResultDto>"
    )]
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
    ///
    /// The helper validates that the order sells the native-token sentinel,
    /// resolves the EthFlow deployment, and returns a transaction request for
    /// the host wallet to submit.
    ///
    /// @param order Unsigned native-sell order DTO.
    /// @param quoteId Quote identifier returned by the orderbook.
    /// @param from Transaction sender address.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing order UID and transaction request.
    /// @throws CowError when the order, chain, deployment, or sender is invalid.
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
        #[wasm_bindgen(js_name = quoteId)] quote_id: f64,
        from: String,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let quote_id = quote_id_from_js(quote_id)?;
        let chain_id = self.chain_id;
        let env = self.env.clone();
        run_with_client_options(scope, async move {
            trading_build_sell_native_currency_tx(chain_id, env, order, quote_id, from).await
        })
        .await
    }

    /// Reads CoW Protocol allowance through a read-only contract callback.
    ///
    /// The SDK builds the contract call while the JavaScript host performs the
    /// actual chain read. Use this when a TypeScript runtime owns the RPC
    /// provider.
    ///
    /// @param params Allowance parameters DTO.
    /// @param readContractCallback Callback that executes the read-only call.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing the allowance amount string.
    /// @throws CowError for invalid parameters, callback failure, timeout, or cancellation.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.trading.cow_protocol_allowance"))
    )]
    #[wasm_bindgen(
        js_name = "getCowProtocolAllowance",
        unchecked_return_type = "WasmEnvelope<string>"
    )]
    pub async fn cow_protocol_allowance(
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
    ///
    /// Use this method when a smart-account runtime owns final contract
    /// signature production. The SDK still quotes the swap, builds typed data,
    /// posts the signed order, and returns posting output.
    ///
    /// @param params Swap parameters DTO.
    /// @param owner Smart-account owner address.
    /// @param customCallback Callback that returns the final EIP-1271 signature.
    /// @param options Optional cancellation, timeout, and wallet timeout settings.
    /// @returns A versioned envelope containing order posting output.
    /// @throws CowError for invalid input, quote failure, callback failure, timeout, or rejection.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(endpoint = "wasm.trading.post_swap_order_with_eip1271")
        )
    )]
    #[wasm_bindgen(
        js_name = "postSwapOrderWithEip1271",
        unchecked_return_type = "WasmEnvelope<OrderPostingResultDto>"
    )]
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
        let chain_id = self.chain_id;
        run_with_client_options(scope, async move {
            trading_post_swap_order_with_eip1271(
                &inner,
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
    fn trading_for_scope(&self, scope: &ClientCallScope) -> Result<Trading, JsValue> {
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
    orderbook: Arc<OrderbookApi>,
) -> Result<Trading, JsValue> {
    let chain = parse_chain(chain_id)?;
    let env_value = pure::chains::env_from_str(env.as_deref())
        .map_err(|error| WasmError::from(error).into_js())?;
    Trading::builder()
        .chain_id(chain)
        .app_code(app_code)
        .env(env_value)
        .orderbook_client(orderbook)
        .build()
        .map_err(|error| WasmError::from(error).into_js())
}

async fn trading_get_quote(
    inner: &Trading,
    params: SwapParametersInput,
) -> Result<JsValue, JsValue> {
    let params: TradeParams = from_json_value("params", params.into_value()?)?;
    let quote = inner
        .quote_only(params, None)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(quote))
}

async fn trading_post_swap_order(
    inner: &Trading,
    params: SwapParametersInput,
    owner: String,
    signer_callback: Function,
    wallet_timeout_ms: Option<u32>,
) -> Result<JsValue, JsValue> {
    let owner = parse_address("owner", owner)?;
    let mut params: TradeParams = from_json_value("params", params.into_value()?)?;
    params.owner = Some(owner.clone());
    let signer = JsTradingSigner::new(owner, signer_callback, wallet_timeout_ms);
    let result = inner
        .post_swap_order(params, &signer, None)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(result))
}

async fn trading_post_swap_order_from_quote(
    inner: &Trading,
    quote_results: JsValue,
    owner: String,
    signer_callback: Function,
    wallet_timeout_ms: Option<u32>,
) -> Result<JsValue, JsValue> {
    let results: QuoteResults = serde_wasm_bindgen::from_value(quote_results)
        .map_err(|error| WasmError::invalid("quoteResults", error.to_string()).into_js())?;
    let owner = parse_address("owner", owner)?;
    let signer = JsTradingSigner::new(owner, signer_callback, wallet_timeout_ms);
    let result = inner
        .post_swap_order_from_quote(&results, &signer, None)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(result))
}

async fn trading_post_limit_order(
    inner: &Trading,
    params: LimitTradeParametersInput,
    owner: String,
    signer_callback: Function,
    wallet_timeout_ms: Option<u32>,
) -> Result<JsValue, JsValue> {
    let owner = parse_address("owner", owner)?;
    let mut params: LimitTradeParams = from_json_value("params", params.into_value()?)?;
    params.owner = params.owner.or_else(|| Some(owner.clone()));
    let signer = JsTradingSigner::new(owner, signer_callback, wallet_timeout_ms);
    let result = inner
        .post_limit_order(params, &signer, None)
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
    if order.sell_token != NATIVE_CURRENCY_ADDRESS {
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
    let payload = EthFlowOrderData::from_unsigned_order(&order, quote_id)
        .map_err(|error| WasmError::from(error).into_js())?;
    let data = HexData::new(format!(
        "0x{}",
        alloy_primitives::hex::encode(encode_create_order_calldata(&payload))
    ))
    .map_err(|error| WasmError::from(error).into_js())?;
    let generated = cow_sdk_trading::calculate_unique_order_id(chain, &order, None, Some(&options))
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    let tx = TransactionRequest::new(
        Some(eth_flow),
        Some(data),
        Some(order.sell_amount),
        Some(default_gas_limit()?),
    );
    let result = BuiltSellNativeCurrencyTxDto {
        order_uid: generated.order_id.to_hex_string(),
        transaction: TransactionRequestDto::from(&tx),
        order_to_sign: OrderInput::from(&order),
        from: from.to_hex_string(),
    };
    to_js_value(&WasmEnvelope::v1(result))
}

/// Converts a JavaScript `number` quote id into the native `i64`, rejecting
/// non-integral or out-of-range values so a lossy float cannot reach the
/// on-chain order data. Quote ids are non-negative database integers well
/// within the JavaScript safe-integer range, so `number` is the precise and
/// consistent representation across the ABI.
#[allow(
    clippy::cast_possible_truncation,
    reason = "value is validated as a non-negative integer at most 2^53-1 before the cast, so the i64 conversion is exact"
)]
fn quote_id_from_js(value: f64) -> Result<i64, WasmError> {
    /// Largest integer a JavaScript `number` represents exactly (2^53 - 1).
    const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;
    if value.is_finite() && value.fract() == 0.0 && (0.0..=MAX_SAFE_INTEGER).contains(&value) {
        Ok(value as i64)
    } else {
        Err(WasmError::invalid(
            "quoteId",
            "quote id must be a non-negative integer within the JavaScript safe-integer range",
        ))
    }
}

async fn trading_get_cow_protocol_allowance(
    inner: &Trading,
    params: AllowanceParametersInput,
    read_contract_callback: Function,
) -> Result<JsValue, JsValue> {
    let params: AllowanceParams = from_json_value("params", params.into_value()?)?;
    let provider = JsContractReadProvider::new(read_contract_callback);
    let allowance = inner
        .cow_protocol_allowance(&provider, &params)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(allowance))
}

async fn trading_post_swap_order_with_eip1271(
    inner: &Trading,
    chain_id: u32,
    params: SwapParametersInput,
    owner: String,
    custom_callback: Function,
    wallet_timeout_ms: Option<u32>,
) -> Result<JsValue, JsValue> {
    let owner_address = parse_address("owner", owner)?;
    let mut params: TradeParams = from_json_value("params", params.into_value()?)?;
    params.owner = Some(owner_address.clone());
    let quote_settings = TradeAdvancedSettings::new().with_quote_request(
        QuoteRequestOverride::new()
            .with_from(owner_address.clone())
            .with_signing_scheme(SigningScheme::Eip1271),
    );
    let quote = inner
        .quote_only(params, Some(&quote_settings))
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    // Resolve the contract signature at the wallet boundary, then hand the managed
    // submission path a pure provider that carries it.
    let chain = parse_chain(chain_id)?;
    let payload = pure::signing::order_typed_data_payload(chain, &quote.order_to_sign)
        .map_err(|error| WasmError::from(error).into_js())?;
    let typed_data = TypedDataEnvelopeDto::from_payload(&payload)?;
    let request = CowEip1271SignRequest {
        order: OrderInput::from(&quote.order_to_sign),
        typed_data,
        owner: owner_address.to_hex_string(),
        chain_id,
    };
    let signature = await_callback_string(
        &custom_callback,
        to_js_value(&request)?,
        "eip1271",
        wallet_timeout_ms,
    )
    .await?;
    let settings = quote_settings.with_additional_params(
        PostTradeAdditionalParams::new()
            .with_custom_eip1271_signature(ResolvedEip1271Provider::new(signature)),
    );
    let signer = JsTradingSigner::new(owner_address, custom_callback, wallet_timeout_ms);
    let result = inner
        .post_swap_order_from_quote(&quote, &signer, Some(&settings))
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(result))
}

fn parse_address(field: &'static str, value: String) -> Result<Address, JsValue> {
    Address::new(value).map_err(|error| WasmError::invalid(field, error.to_string()).into_js())
}

fn default_gas_limit() -> Result<Amount, JsValue> {
    Amount::new(DEFAULT_GAS_LIMIT.to_string())
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

impl Signer for JsTradingSigner {
    type Error = String;

    async fn address(&self) -> Result<Address, Self::Error> {
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

impl Provider for JsContractReadProvider {
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
