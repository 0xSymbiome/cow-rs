use std::{fmt, sync::Arc};

use crate::helpers as pure;
use cow_sdk_core::transport::policy::TransportPolicy;
use cow_sdk_core::{
    Address, Amount, BlockInfo, ContractCall, HexData, NATIVE_CURRENCY_ADDRESS, ProtocolOptions,
    Provider, Signer, TransactionBroadcast, TransactionHash, TransactionReceipt,
    TransactionRequest, UserRejection,
};
use cow_sdk_orderbook::{OrderbookApi, SigningScheme};
use cow_sdk_trading::{
    AllowanceParams as NativeAllowanceParams, ApprovalParams as NativeApprovalParams,
    Authorization, DEFAULT_GAS_LIMIT, LimitTradeParams, PostTradeAdditionalParams,
    QuoteRequestOverride, QuoteResults, TradeAdvancedSettings, TradeParams, Trading,
    unwrap_transaction, wrap_transaction,
};
use js_sys::{Function, Reflect};
use wasm_bindgen::prelude::*;

use crate::dto::{
    AllowanceParams, ApprovalParams, BuiltSellNativeCurrencyTx, CowEip1271SignRequest,
    OrderPlacement, SafeActivation, envelope_callback_value, from_json_value, parse_chain,
    payload_to_envelope, to_js_value, transport_policy_from_config,
};
use crate::exports::{
    cancel::{
        ClientCallScope, SdkClientOptions, SigningOptions, run_with_client_options,
        signing_wallet_timeout_ms,
    },
    eip1271::ResolvedEip1271Provider,
    envelope::WasmEnvelope,
    errors::{JsResultExt, WasmError},
    orderbook::{build_orderbook, orderbook_for_scope},
    signing::{await_callback_string, js_error_to_string, js_message, normalize_signature},
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

    /// Input object for `buildPresignActivationTransaction`.
    ///
    /// Carries the `orderUid`, `sellToken`, and `amount` of an already-posted
    /// pre-sign order whose on-chain activation bundle is being built.
    #[wasm_bindgen(typescript_type = "PresignActivationParams")]
    pub type PresignActivationParams;
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
    /// defaults for all trading methods. When constructed through the TypeScript
    /// facade, an omitted `transport` defaults to the runtime global `fetch`;
    /// that default is a facade affordance, so the raw constructor documented
    /// here requires the transport explicitly.
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
    /// asking a wallet to sign or before constructing a post request. Set
    /// `owner` on the swap parameters: quote-only flows resolve no signer, so a
    /// missing owner surfaces as an error rather than defaulting to an account.
    ///
    /// This returns the rich `QuoteResults` carrying `orderToSign` and
    /// `amountsAndCosts` for posting, distinct from `OrderBookClient.getQuote`,
    /// which returns the raw orderbook `OrderQuoteResponse`.
    ///
    /// @param params Swap parameters DTO; set `owner` for quote-only flows.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing the rich quote results.
    /// @throws CowError for a missing owner, invalid parameters, transport failure, timeout, or cancellation.
    #[wasm_bindgen(
        js_name = "getQuote",
        unchecked_return_type = "WasmEnvelope<QuoteResults>"
    )]
    pub async fn quote(
        &self,
        params: TradeParams,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        super::traced("wasm.trading.quote", async move {
            let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
            let inner = self.trading_for_scope(&scope)?;
            run_with_client_options(
                scope,
                async move { trading_get_quote(&inner, params).await },
            )
            .await
        })
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
    #[wasm_bindgen(
        js_name = "postSwapOrder",
        unchecked_return_type = "WasmEnvelope<OrderPostingResult>"
    )]
    pub async fn post_swap_order(
        &self,
        params: TradeParams,
        owner: String,
        #[wasm_bindgen(js_name = signerCallback, unchecked_param_type = "TypedDataSignerCallback")]
        signer_callback: Function,
        #[wasm_bindgen(js_name = options)] options: Option<SigningOptions>,
    ) -> Result<JsValue, JsValue> {
        super::traced("wasm.trading.post_swap_order", async move {
            let options_ref = options.as_ref().map(AsRef::as_ref);
            let scope = ClientCallScope::new(options_ref)?;
            let wallet_timeout_ms = signing_wallet_timeout_ms(options_ref)?;
            let inner = self.trading_for_scope(&scope)?;
            run_with_client_options(scope, async move {
                trading_post_swap_order(&inner, params, owner, signer_callback, wallet_timeout_ms)
                    .await
            })
            .await
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
    #[wasm_bindgen(
        js_name = "postSwapOrderFromQuote",
        unchecked_return_type = "WasmEnvelope<OrderPostingResult>"
    )]
    pub async fn post_swap_order_from_quote(
        &self,
        #[wasm_bindgen(js_name = quoteResults, unchecked_param_type = "QuoteResults")]
        quote_results: JsValue,
        owner: String,
        #[wasm_bindgen(js_name = signerCallback, unchecked_param_type = "TypedDataSignerCallback")]
        signer_callback: Function,
        #[wasm_bindgen(js_name = options)] options: Option<SigningOptions>,
    ) -> Result<JsValue, JsValue> {
        super::traced("wasm.trading.post_swap_order_from_quote", async move {
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
    #[wasm_bindgen(
        js_name = "postLimitOrder",
        unchecked_return_type = "WasmEnvelope<OrderPostingResult>"
    )]
    pub async fn post_limit_order(
        &self,
        params: LimitTradeParams,
        owner: String,
        #[wasm_bindgen(js_name = signerCallback, unchecked_param_type = "TypedDataSignerCallback")]
        signer_callback: Function,
        #[wasm_bindgen(js_name = options)] options: Option<SigningOptions>,
    ) -> Result<JsValue, JsValue> {
        super::traced("wasm.trading.post_limit_order", async move {
            let options_ref = options.as_ref().map(AsRef::as_ref);
            let scope = ClientCallScope::new(options_ref)?;
            let wallet_timeout_ms = signing_wallet_timeout_ms(options_ref)?;
            let inner = self.trading_for_scope(&scope)?;
            run_with_client_options(scope, async move {
                trading_post_limit_order(&inner, params, owner, signer_callback, wallet_timeout_ms)
                    .await
            })
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
    #[wasm_bindgen(
        js_name = "buildSellNativeCurrencyTx",
        unchecked_return_type = "WasmEnvelope<BuiltSellNativeCurrencyTx>"
    )]
    pub async fn build_sell_native_currency_tx(
        &self,
        order: cow_sdk_core::OrderData,
        #[wasm_bindgen(js_name = quoteId)] quote_id: f64,
        from: String,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        super::traced("wasm.trading.build_sell_native_currency_tx", async move {
            let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
            let quote_id = quote_id_from_js(quote_id)?;
            let chain_id = self.chain_id;
            let env = self.env.clone();
            run_with_client_options(scope, async move {
                trading_build_sell_native_currency_tx(chain_id, env, order, quote_id, from).await
            })
            .await
        })
        .await
    }

    /// Builds the native-currency sell transaction directly from a quote result.
    ///
    /// This is the native-sell sibling of `postSwapOrderFromQuote`: it consumes
    /// the `QuoteResults` that `getQuote` returns for a native-currency sell
    /// and derives the EthFlow transaction without the host reconstructing the
    /// order or extracting the quote id. The quote must have been requested with
    /// the native-token sentinel as the sell token and must carry the quote id
    /// the orderbook returns for EthFlow submission.
    ///
    /// @param quoteResults Quote result DTO returned by `getQuote` for a native sell.
    /// @param from Transaction sender address.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing order UID and transaction request.
    /// @throws CowError when the quote is not a native-currency sell, lacks a quote id, or the chain, deployment, or sender is invalid.
    #[wasm_bindgen(
        js_name = "buildSellNativeCurrencyTxFromQuote",
        unchecked_return_type = "WasmEnvelope<BuiltSellNativeCurrencyTx>"
    )]
    pub async fn build_sell_native_currency_tx_from_quote(
        &self,
        #[wasm_bindgen(js_name = quoteResults, unchecked_param_type = "QuoteResults")]
        quote_results: JsValue,
        from: String,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        super::traced(
            "wasm.trading.build_sell_native_currency_tx_from_quote",
            async move {
                let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
                let chain_id = self.chain_id;
                let env = self.env.clone();
                run_with_client_options(scope, async move {
                    trading_build_sell_native_currency_tx_from_quote(
                        chain_id,
                        env,
                        quote_results,
                        from,
                    )
                    .await
                })
                .await
            },
        )
        .await
    }

    /// Reads CoW Protocol allowance through a read-only contract callback.
    ///
    /// The SDK builds the contract call while the JavaScript host performs the
    /// actual chain read. Use this when a TypeScript runtime owns the RPC
    /// provider. The vault-relayer spender is resolved per chain and environment
    /// unless overridden in the parameters. The callback must return the
    /// ABI-decoded `uint256` allowance as a decimal string or JSON number — for
    /// example viem's `readContract` result passed through `String(value)` — not
    /// a raw `0x`-hex `eth_call` payload.
    ///
    /// @param params Allowance parameters DTO.
    /// @param readContractCallback Callback that executes the read-only call and returns the ABI-decoded allowance.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing the allowance amount as a decimal string.
    /// @throws CowError for invalid parameters, callback failure, timeout, or cancellation.
    #[wasm_bindgen(
        js_name = "getCowProtocolAllowance",
        unchecked_return_type = "WasmEnvelope<string>"
    )]
    pub async fn cow_protocol_allowance(
        &self,
        params: AllowanceParams,
        #[wasm_bindgen(js_name = readContractCallback, unchecked_param_type = "ContractReadCallback")]
        read_contract_callback: Function,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        super::traced("wasm.trading.cow_protocol_allowance", async move {
            let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
            let inner = self.trading_for_scope(&scope)?;
            run_with_client_options(scope, async move {
                trading_get_cow_protocol_allowance(&inner, params, read_contract_callback).await
            })
            .await
        })
        .await
    }

    /// Builds the ERC-20 approval transaction for the CoW Protocol vault relayer.
    ///
    /// The SDK encodes the unsigned `approve` transaction; the JavaScript host
    /// owns submission through its own wallet. This completes the
    /// read-allowance-then-approve path alongside `getCowProtocolAllowance`.
    ///
    /// @param params Approval parameters DTO (token, amount, optional vault-relayer override).
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing the unsigned approval transaction request.
    /// @throws CowError when the token, amount, or vault-relayer override is invalid.
    #[wasm_bindgen(
        js_name = "buildApprovalTx",
        unchecked_return_type = "WasmEnvelope<TransactionRequest>"
    )]
    pub async fn build_approval_tx(
        &self,
        params: ApprovalParams,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        super::traced("wasm.trading.build_approval_tx", async move {
            let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
            let chain_id = self.chain_id;
            let env = self.env.clone();
            run_with_client_options(scope, async move {
                trading_build_approval_tx(chain_id, env, params).await
            })
            .await
        })
        .await
    }

    /// Builds the transaction that wraps native currency into its wrapped-native
    /// token (for example ETH into WETH) on this client's chain.
    ///
    /// The target wrapped-native address is resolved from the chain; submit the
    /// returned request with the host wallet. Selling native currency through CoW
    /// Protocol does not require a manual wrap — the eth-flow path wraps on-chain
    /// during order creation — so use this for standalone wrap and treasury flows.
    ///
    /// @param amount Amount of native currency to wrap, in wei as a decimal string.
    /// @returns A versioned envelope containing the unsigned wrap transaction request.
    /// @throws CowError when the chain is unsupported or the amount is invalid.
    #[wasm_bindgen(
        js_name = "buildWrapTx",
        unchecked_return_type = "WasmEnvelope<TransactionRequest>"
    )]
    pub fn build_wrap_tx(&self, amount: String) -> Result<JsValue, JsValue> {
        let chain = parse_chain(self.chain_id)?;
        let amount = pure::dto::parse_amount("amount", &amount).map_js()?;
        let tx = TransactionRequest::from(wrap_transaction(chain, amount));
        to_js_value(&WasmEnvelope::v1(tx))
    }

    /// Builds the transaction that unwraps the wrapped-native token back into
    /// native currency (for example WETH into ETH) on this client's chain.
    ///
    /// `withdraw` burns the caller's own wrapped-native balance, so no token
    /// approval is required. Submit the returned request with the host wallet.
    ///
    /// @param amount Amount of the wrapped-native token to unwrap, in wei as a decimal string.
    /// @returns A versioned envelope containing the unsigned unwrap transaction request.
    /// @throws CowError when the chain is unsupported or the amount is invalid.
    #[wasm_bindgen(
        js_name = "buildUnwrapTx",
        unchecked_return_type = "WasmEnvelope<TransactionRequest>"
    )]
    pub fn build_unwrap_tx(&self, amount: String) -> Result<JsValue, JsValue> {
        let chain = parse_chain(self.chain_id)?;
        let amount = pure::dto::parse_amount("amount", &amount).map_js()?;
        let tx = TransactionRequest::from(unwrap_transaction(chain, amount));
        to_js_value(&WasmEnvelope::v1(tx))
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
    #[wasm_bindgen(
        js_name = "postSwapOrderWithEip1271",
        unchecked_return_type = "WasmEnvelope<OrderPostingResult>"
    )]
    pub async fn post_swap_order_with_eip1271(
        &self,
        params: TradeParams,
        owner: String,
        #[wasm_bindgen(js_name = customCallback, unchecked_param_type = "CustomEip1271Callback")]
        custom_callback: Function,
        #[wasm_bindgen(js_name = options)] options: Option<SigningOptions>,
    ) -> Result<JsValue, JsValue> {
        super::traced("wasm.trading.post_swap_order_with_eip1271", async move {
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
        })
        .await
    }

    /// Posts a swap order, selecting the authorization mode from `auth` and
    /// returning the typed placement result (ADR 0073).
    ///
    /// `auth` is a tagged union: `{ kind: "ecdsa", sign }` signs the EIP-712
    /// envelope through the typed-data callback (gasless EOA / EIP-712);
    /// `{ kind: "eip1271", sign }` resolves the smart-account contract signature
    /// through the custom callback at the boundary (gasless Safe); `{ kind:
    /// "presign" }` posts with no signer. The `ecdsa` and `eip1271` arms resolve
    /// to a `live` placement, while `presign` resolves to `pendingActivation`
    /// carrying the on-chain approve-then-set-pre-signature bundle the owner must
    /// send or propose from the smart account.
    ///
    /// @param quoteResults Quote result DTO returned by `getQuote`.
    /// @param owner Owner address to bind to the order.
    /// @param auth Authorization mode: ECDSA, EIP-1271, or pre-sign.
    /// @param options Optional cancellation, timeout, and wallet timeout settings.
    /// @returns A versioned envelope containing the typed order placement.
    /// @throws CowError for invalid input, quote mismatch, wallet failure, timeout, or rejection.
    #[wasm_bindgen(
        js_name = "placeSwap",
        unchecked_return_type = "WasmEnvelope<OrderPlacement>"
    )]
    pub async fn place_swap(
        &self,
        #[wasm_bindgen(js_name = quoteResults, unchecked_param_type = "QuoteResults")]
        quote_results: JsValue,
        owner: String,
        #[wasm_bindgen(unchecked_param_type = "Authorization")] auth: JsValue,
        #[wasm_bindgen(js_name = options)] options: Option<SigningOptions>,
    ) -> Result<JsValue, JsValue> {
        super::traced("wasm.trading.place_swap", async move {
            let options_ref = options.as_ref().map(AsRef::as_ref);
            let scope = ClientCallScope::new(options_ref)?;
            let wallet_timeout_ms = signing_wallet_timeout_ms(options_ref)?;
            let inner = self.trading_for_scope(&scope)?;
            let chain_id = self.chain_id;
            run_with_client_options(scope, async move {
                trading_place_swap(
                    &inner,
                    chain_id,
                    quote_results,
                    owner,
                    auth,
                    wallet_timeout_ms,
                )
                .await
            })
            .await
        })
        .await
    }

    /// Posts a limit order, selecting the authorization mode from `auth` and
    /// returning the typed placement result (ADR 0073).
    ///
    /// `auth` is a tagged union: `{ kind: "ecdsa", sign }` signs the EIP-712
    /// envelope through the typed-data callback (gasless EOA / EIP-712);
    /// `{ kind: "eip1271", sign }` resolves a smart-account contract signature
    /// against the SDK-built limit order through the callback (gasless smart
    /// account); `{ kind: "presign" }` posts with no signer and resolves to
    /// `pendingActivation` carrying the on-chain
    /// approve-then-set-pre-signature bundle the owner must send or propose from
    /// the smart account. The `ecdsa` and `eip1271` arms resolve to a `live`
    /// placement.
    ///
    /// For a Safe, `presign` is the recommended default: it matches the CoW Swap
    /// reference UI, which never asks a Safe to produce an off-chain limit
    /// signature. The `eip1271` arm is for smart accounts that do sign messages
    /// (a 1-of-1 Safe owner, a smart-account wallet). This mirrors the `placeSwap`
    /// guidance, so the limit and swap paths read consistently.
    ///
    /// @param params Limit-order parameters DTO.
    /// @param owner Owner address to bind to the order.
    /// @param auth Authorization mode: ECDSA, EIP-1271, or pre-sign.
    /// @param options Optional cancellation, timeout, and wallet timeout settings.
    /// @returns A versioned envelope containing the typed order placement.
    /// @throws CowError for invalid input, wallet failure, timeout, or rejection.
    #[wasm_bindgen(
        js_name = "placeLimit",
        unchecked_return_type = "WasmEnvelope<OrderPlacement>"
    )]
    pub async fn place_limit(
        &self,
        params: LimitTradeParams,
        owner: String,
        #[wasm_bindgen(unchecked_param_type = "Authorization")] auth: JsValue,
        #[wasm_bindgen(js_name = options)] options: Option<SigningOptions>,
    ) -> Result<JsValue, JsValue> {
        super::traced("wasm.trading.place_limit", async move {
            let options_ref = options.as_ref().map(AsRef::as_ref);
            let scope = ClientCallScope::new(options_ref)?;
            let wallet_timeout_ms = signing_wallet_timeout_ms(options_ref)?;
            let inner = self.trading_for_scope(&scope)?;
            let chain_id = self.chain_id;
            run_with_client_options(scope, async move {
                trading_place_limit(&inner, chain_id, params, owner, auth, wallet_timeout_ms).await
            })
            .await
        })
        .await
    }

    /// Builds the on-chain activation bundle for an already-posted pre-sign order
    /// (ADR 0073).
    ///
    /// Composes the ordered approve-then-set-pre-signature pair a smart-contract
    /// wallet runs to authorize the order: the ERC-20 `approve` of the sell-token
    /// allowance for the vault relayer and the settlement
    /// `setPreSignature(uid, true)`. The builder is pure — it reads no on-chain
    /// allowance and always emits both calls, so a caller whose vault-relayer
    /// allowance already covers the sell amount may drop the approve leg.
    ///
    /// @param params Order UID, sell token, and sell amount of the pre-sign order.
    /// @returns A versioned envelope containing the activation calls.
    /// @throws CowError when the order UID, sell token, amount, or settlement deployment is invalid.
    #[wasm_bindgen(
        js_name = "buildPresignActivationTransaction",
        unchecked_return_type = "WasmEnvelope<SafeActivation>"
    )]
    pub fn build_presign_activation_transaction(
        &self,
        params: PresignActivationParams,
    ) -> Result<JsValue, JsValue> {
        let params = params.as_ref();
        let order_uid_str = required_string(params, "orderUid")?;
        let sell_token_str = required_string(params, "sellToken")?;
        let amount_str = required_string(params, "amount")?;
        let order_uid = cow_sdk_core::OrderUid::new(order_uid_str)
            .map_err(|error| WasmError::invalid("orderUid", error.to_string()).into_js())?;
        let sell_token = parse_address("sellToken", sell_token_str)?;
        let amount = Amount::new(amount_str)
            .map_err(|error| WasmError::invalid("amount", error.to_string()).into_js())?;
        let inner = self.trading_for_scope(&ClientCallScope::new(None)?)?;
        let activation = inner
            .build_presign_activation(&order_uid, sell_token, amount)
            .map_js()?;
        to_js_value(&WasmEnvelope::v1(SafeActivation::from_native(activation)))
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
    let env_value = pure::chains::env_from_str(env.as_deref()).map_js()?;
    Trading::builder()
        .chain_id(chain)
        .app_code(app_code)
        .env(env_value)
        .orderbook_shared(orderbook)
        .build()
        .map_js()
}

async fn trading_get_quote(inner: &Trading, params: TradeParams) -> Result<JsValue, JsValue> {
    let quote = inner.quote_only(params, None).await.map_js()?;
    to_js_value(&WasmEnvelope::v1(quote))
}

async fn trading_post_swap_order(
    inner: &Trading,
    mut params: TradeParams,
    owner: String,
    signer_callback: Function,
    wallet_timeout_ms: Option<u32>,
) -> Result<JsValue, JsValue> {
    let owner = parse_address("owner", owner)?;
    params.owner = Some(owner.clone());
    let signer = JsTradingSigner::new(owner, signer_callback, wallet_timeout_ms);
    let result = inner
        .post_swap_order(params, &signer, None)
        .await
        .map_js()?;
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
        .map_js()?;
    to_js_value(&WasmEnvelope::v1(result))
}

async fn trading_post_limit_order(
    inner: &Trading,
    mut params: LimitTradeParams,
    owner: String,
    signer_callback: Function,
    wallet_timeout_ms: Option<u32>,
) -> Result<JsValue, JsValue> {
    let owner = parse_address("owner", owner)?;
    // The positional owner is canonical for the post* exports — it is the only
    // address the signing callback can report — so it overrides any owner echoed
    // in the DTO, matching postSwapOrder rather than letting the DTO win.
    params.owner = Some(owner.clone());
    let signer = JsTradingSigner::new(owner, signer_callback, wallet_timeout_ms);
    let result = inner
        .post_limit_order(params, &signer, None)
        .await
        .map_js()?;
    to_js_value(&WasmEnvelope::v1(result))
}

async fn trading_build_sell_native_currency_tx(
    chain_id: u32,
    env: Option<String>,
    order: cow_sdk_core::OrderData,
    quote_id: i64,
    from: String,
) -> Result<JsValue, JsValue> {
    let from = parse_address("from", from)?;
    if order.sell_token != NATIVE_CURRENCY_ADDRESS {
        return Err(WasmError::invalid(
            "order.sellToken",
            "native-currency sell transactions require the native token sentinel address",
        )
        .into_js());
    }
    let chain = parse_chain(chain_id)?;
    let env = pure::chains::env_from_str(env.as_deref()).map_js()?;
    let options = ProtocolOptions::new().with_env(env);
    let unsigned = cow_sdk_contracts::ethflow_create_order_transaction(
        &order,
        quote_id,
        chain,
        Some(&options),
    )
    .map_js()?;
    let generated = cow_sdk_trading::calculate_unique_order_id(chain, &order, None, Some(&options))
        .await
        .map_js()?;
    let mut tx = TransactionRequest::from(unsigned);
    tx.gas_limit = Some(default_gas_limit()?);
    let result = BuiltSellNativeCurrencyTx {
        order_uid: generated.order_id.to_hex_string(),
        transaction: tx,
        order_to_sign: order,
        from: from.to_hex_string(),
    };
    to_js_value(&WasmEnvelope::v1(result))
}

async fn trading_build_sell_native_currency_tx_from_quote(
    chain_id: u32,
    env: Option<String>,
    quote_results: JsValue,
    from: String,
) -> Result<JsValue, JsValue> {
    let results: QuoteResults = serde_wasm_bindgen::from_value(quote_results)
        .map_err(|error| WasmError::invalid("quoteResults", error.to_string()).into_js())?;
    if results.trade_parameters.sell_token != NATIVE_CURRENCY_ADDRESS {
        return Err(WasmError::invalid(
            "quoteResults",
            "the quote was not requested for a native-currency sell",
        )
        .into_js());
    }
    let quote_id = results.quote_response.id.ok_or_else(|| {
        WasmError::invalid(
            "quoteResults.quoteResponse.id",
            "the quote did not return an id required for a native-currency sell",
        )
        .into_js()
    })?;
    // The orderbook quotes a native sell against the wrapped-native token, so the
    // signed order carries the wrapped sell token. The EthFlow builder expects the
    // native sentinel, matching the lower-level `buildSellNativeCurrencyTx` entry
    // point, so this restores it before delegating to the shared builder.
    let mut order = results.order_to_sign;
    order.sell_token = NATIVE_CURRENCY_ADDRESS;
    trading_build_sell_native_currency_tx(chain_id, env, order, quote_id, from).await
}

/// Converts a JavaScript `number` quote id into the native `i64`, rejecting
/// non-integral or out-of-range values so a lossy float cannot reach the
/// on-chain order data. Quote ids are non-negative database integers well
/// within the JavaScript safe-integer range, so `number` is the precise and
/// consistent representation across the ABI.
fn quote_id_from_js(value: f64) -> Result<i64, WasmError> {
    super::js_safe_integer_to_i64(value, "quoteId")
}

async fn trading_get_cow_protocol_allowance(
    inner: &Trading,
    params: AllowanceParams,
    read_contract_callback: Function,
) -> Result<JsValue, JsValue> {
    let params: NativeAllowanceParams = from_json_value("params", params.into_value()?)?;
    let provider = JsContractReadProvider::new(read_contract_callback);
    let allowance = inner
        .cow_protocol_allowance(&provider, &params)
        .await
        .map_js()?;
    to_js_value(&WasmEnvelope::v1(allowance))
}

async fn trading_build_approval_tx(
    chain_id: u32,
    env: Option<String>,
    params: ApprovalParams,
) -> Result<JsValue, JsValue> {
    let chain = parse_chain(chain_id)?;
    let env = pure::chains::env_from_str(env.as_deref()).map_js()?;
    let params: NativeApprovalParams = from_json_value("params", params.into_value()?)?;
    let tx = cow_sdk_trading::approval_transaction(&params, chain, env);
    to_js_value(&WasmEnvelope::v1(tx))
}

async fn trading_post_swap_order_with_eip1271(
    inner: &Trading,
    chain_id: u32,
    mut params: TradeParams,
    owner: String,
    custom_callback: Function,
    wallet_timeout_ms: Option<u32>,
) -> Result<JsValue, JsValue> {
    let owner_address = parse_address("owner", owner)?;
    params.owner = Some(owner_address.clone());
    let quote_settings = TradeAdvancedSettings::new().with_quote_request(
        QuoteRequestOverride::new()
            .with_from(owner_address.clone())
            .with_signing_scheme(SigningScheme::Eip1271),
    );
    let quote = inner
        .quote_only(params, Some(&quote_settings))
        .await
        .map_js()?;
    // Resolve the contract signature at the wallet boundary, then hand the managed
    // submission path a pure provider that carries it.
    let chain = parse_chain(chain_id)?;
    let payload = pure::signing::order_typed_data_payload(chain, &quote.order_to_sign).map_js()?;
    let typed_data = payload_to_envelope(&payload)?;
    let request = CowEip1271SignRequest {
        order: quote.order_to_sign.clone(),
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
        .map_js()?;
    to_js_value(&WasmEnvelope::v1(result))
}

/// The `kind` discriminant of the tagged-union `Authorization` value.
enum AuthKind {
    Ecdsa,
    Eip1271,
    PreSign,
}

/// Reads the `kind` discriminant of an `Authorization` value, rejecting an
/// absent or unrecognised tag with a typed error.
fn auth_kind(auth: &JsValue) -> Result<AuthKind, JsValue> {
    let kind = Reflect::get(auth, &JsValue::from_str("kind"))
        .ok()
        .and_then(|kind| kind.as_string())
        .ok_or_else(|| {
            WasmError::invalid("auth.kind", "authorization must carry a kind discriminant")
                .into_js()
        })?;
    match kind.as_str() {
        "ecdsa" => Ok(AuthKind::Ecdsa),
        "eip1271" => Ok(AuthKind::Eip1271),
        "presign" => Ok(AuthKind::PreSign),
        other => Err(WasmError::invalid(
            "auth.kind",
            format!("unknown authorization kind `{other}`"),
        )
        .into_js()),
    }
}

/// Reads the `sign` callback carried by the `ecdsa` and `eip1271` authorization
/// arms, rejecting a missing or non-function value with a typed error.
fn auth_sign_callback(auth: &JsValue) -> Result<Function, JsValue> {
    let sign = Reflect::get(auth, &JsValue::from_str("sign"))
        .map_err(|error| WasmError::invalid("auth.sign", js_message(&error)).into_js())?;
    sign.dyn_into::<Function>().map_err(|_| {
        WasmError::invalid("auth.sign", "authorization sign must be a function").into_js()
    })
}

async fn trading_place_swap(
    inner: &Trading,
    chain_id: u32,
    quote_results: JsValue,
    owner: String,
    auth: JsValue,
    wallet_timeout_ms: Option<u32>,
) -> Result<JsValue, JsValue> {
    let quote: QuoteResults = serde_wasm_bindgen::from_value(quote_results)
        .map_err(|error| WasmError::invalid("quoteResults", error.to_string()).into_js())?;
    let owner = parse_address("owner", owner)?;
    let placement = match auth_kind(&auth)? {
        AuthKind::Ecdsa => {
            let callback = auth_sign_callback(&auth)?;
            let signer = JsTradingSigner::new(owner, callback, wallet_timeout_ms);
            inner
                .place_swap(&quote, owner, Authorization::ecdsa(&signer), None)
                .await
                .map_js()?
        }
        AuthKind::Eip1271 => {
            // Resolve the contract signature at the wallet boundary, then hand the
            // native placement path a pure provider carrying the resolved blob —
            // the same eager-resolution `postSwapOrderWithEip1271` performs.
            let callback = auth_sign_callback(&auth)?;
            let provider = resolve_eip1271_provider(
                chain_id,
                &quote.order_to_sign,
                &owner,
                &callback,
                wallet_timeout_ms,
            )
            .await?;
            inner
                .place_swap(
                    &quote,
                    owner,
                    Authorization::eip1271(Arc::new(provider)),
                    None,
                )
                .await
                .map_js()?
        }
        AuthKind::PreSign => inner
            .place_swap(&quote, owner, Authorization::pre_sign(), None)
            .await
            .map_js()?,
    };
    to_js_value(&WasmEnvelope::v1(OrderPlacement::from_native(placement)))
}

async fn trading_place_limit(
    inner: &Trading,
    chain_id: u32,
    mut params: LimitTradeParams,
    owner: String,
    auth: JsValue,
    wallet_timeout_ms: Option<u32>,
) -> Result<JsValue, JsValue> {
    let owner = parse_address("owner", owner)?;
    // The positional owner is canonical for the placement exports — it is the
    // only address the signing callback can report — so it overrides any owner
    // echoed in the DTO, matching the post* exports.
    params.owner = Some(owner);
    let placement = match auth_kind(&auth)? {
        AuthKind::Ecdsa => {
            let callback = auth_sign_callback(&auth)?;
            let signer = JsTradingSigner::new(owner, callback, wallet_timeout_ms);
            inner
                .place_limit(params, owner, Authorization::ecdsa(&signer), None)
                .await
                .map_js()?
        }
        AuthKind::PreSign => inner
            .place_limit(params, owner, Authorization::pre_sign(), None)
            .await
            .map_js()?,
        AuthKind::Eip1271 => {
            // Build the limit order the native placement path will sign, resolve
            // the contract signature against it at the wallet boundary, then hand
            // the native placement path a pure provider carrying the resolved blob
            // — the limit counterpart of the swap arm above. `place_limit` rebuilds
            // the same `order_to_sign` and the resolved provider echoes the blob,
            // so the digest cannot drift.
            let callback = auth_sign_callback(&auth)?;
            let (order_to_sign, _app_data) = inner
                .build_limit_order_to_sign(&params, owner, None)
                .await
                .map_js()?;
            let provider = resolve_eip1271_provider(
                chain_id,
                &order_to_sign,
                &owner,
                &callback,
                wallet_timeout_ms,
            )
            .await?;
            inner
                .place_limit(
                    params,
                    owner,
                    Authorization::eip1271(Arc::new(provider)),
                    None,
                )
                .await
                .map_js()?
        }
    };
    to_js_value(&WasmEnvelope::v1(OrderPlacement::from_native(placement)))
}

/// Resolves a smart-account EIP-1271 contract signature at the wallet boundary
/// and wraps it in the pure [`ResolvedEip1271Provider`] the native placement
/// path consults. Mirrors the resolution `postSwapOrderWithEip1271` performs.
async fn resolve_eip1271_provider(
    chain_id: u32,
    order_to_sign: &cow_sdk_core::OrderData,
    owner: &Address,
    callback: &Function,
    wallet_timeout_ms: Option<u32>,
) -> Result<ResolvedEip1271Provider, JsValue> {
    let chain = parse_chain(chain_id)?;
    let payload = pure::signing::order_typed_data_payload(chain, order_to_sign).map_js()?;
    let typed_data = payload_to_envelope(&payload)?;
    let request = CowEip1271SignRequest {
        order: order_to_sign.clone(),
        typed_data,
        owner: owner.to_hex_string(),
        chain_id,
    };
    let signature = await_callback_string(
        callback,
        to_js_value(&request)?,
        "eip1271",
        wallet_timeout_ms,
    )
    .await?;
    Ok(ResolvedEip1271Provider::new(signature))
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

/// Typed error for the JS typed-data callback signer.
///
/// `Signer::Error` must be `Display + UserRejection` for the trading terminals:
/// `Display` feeds the redacted message path, and `UserRejection` lets a
/// deliberate wallet rejection (EIP-1193 `4001`) surface as a `walletRequest`
/// error the JS `isUserRejection` predicate recognises, instead of collapsing to
/// an opaque `signing` failure. Only the structured provider `code` is kept; the
/// provider-authored message is replaced by SDK guidance downstream (ADR 0053).
struct WalletCallbackError {
    code: Option<i32>,
    message: String,
}

impl WalletCallbackError {
    /// Reads the structured EIP-1193 `code` and the message from a callback error
    /// `JsValue`, which `await_callback_string` already shaped as an SDK error.
    fn from_js(value: &JsValue) -> Self {
        let code = Reflect::get(value, &JsValue::from_str("code"))
            .ok()
            .and_then(|code| code.as_f64())
            .map(|code| code as i32);
        Self {
            code,
            message: js_message(value),
        }
    }

    /// A signer operation this typed-data callback does not support.
    fn unsupported(message: &'static str) -> Self {
        Self {
            code: None,
            message: message.to_owned(),
        }
    }
}

impl fmt::Display for WalletCallbackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl UserRejection for WalletCallbackError {
    fn user_rejection_code(&self) -> Option<i32> {
        // EIP-1193 `4001` is the standard "user rejected the request" code; every
        // other provider code is a non-rejection fault that takes the redacted
        // message path.
        (self.code == Some(4001)).then_some(4001)
    }
}

impl Signer for JsTradingSigner {
    type Error = WalletCallbackError;

    async fn address(&self) -> Result<Address, Self::Error> {
        Ok(self.owner.clone())
    }

    async fn sign_message(&self, _message: &[u8]) -> Result<String, Self::Error> {
        Err(WalletCallbackError::unsupported(
            "message signing is not available through this typed-data callback",
        ))
    }

    async fn sign_typed_data_payload(
        &self,
        payload: &cow_sdk_core::TypedDataPayload,
    ) -> Result<String, Self::Error> {
        let envelope = payload_to_envelope(payload)
            .map_err(|error| WalletCallbackError::from_js(&error.into_js()))?;
        let value = envelope_callback_value(&envelope)
            .map_err(|error| WalletCallbackError::from_js(&error))?;
        let signature = await_callback_string(
            &self.callback,
            value,
            "signTypedData",
            self.wallet_timeout_ms,
        )
        .await
        .map_err(|error| WalletCallbackError::from_js(&error))?;
        normalize_signature(&signature).map_err(|error| WalletCallbackError::from_js(&error))
    }

    async fn send_transaction(
        &self,
        _tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        Err(WalletCallbackError::unsupported(
            "transaction submission is not available through this typed-data callback",
        ))
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Err(WalletCallbackError::unsupported(
            "gas estimation is not available through this typed-data callback",
        ))
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

    async fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        Err("raw calls are not available through this contract-read callback".to_owned())
    }

    async fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
        let value = to_js_value(request).map_err(js_error_to_string)?;
        await_callback_string(&self.callback, value, "readContract", None)
            .await
            .map_err(js_error_to_string)
    }

    async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Err("block reads are not available through this contract-read callback".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn wallet_callback_error_flags_only_the_eip1193_user_rejection_code() {
        // 4001 is the EIP-1193 "user rejected" code -> routes to SignerRejection.
        let rejected = WalletCallbackError {
            code: Some(4001),
            message: String::new(),
        };
        assert_eq!(rejected.user_rejection_code(), Some(4001));

        // Any other provider code is a non-rejection fault -> redacted message path.
        let other = WalletCallbackError {
            code: Some(4100),
            message: String::new(),
        };
        assert_eq!(other.user_rejection_code(), None);

        // A DTO/serialization fault carries no code and is never a user rejection.
        let codeless = WalletCallbackError::unsupported("nope");
        assert_eq!(codeless.user_rejection_code(), None);
    }
}
