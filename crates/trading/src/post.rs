use cow_sdk_core::{AsyncSigner, ProtocolOptions, Signer};
use cow_sdk_orderbook::{OrderCreation, SigningScheme};
use cow_sdk_signing::{
    SigningScheme as SigningSchemeContract, eip1271_signature_payload, sign_order_async,
    sign_order_with_scheme_async,
};

use crate::{
    LimitOrderAdvancedSettings, LimitTradeParameters, OrderPostingResult, OrderbookClient,
    QuoteResults, SwapAdvancedSettings, TradeParameters, TraderParameters, TradingAppDataInfo,
    TradingError, adjust_ethflow_limit_parameters, build_app_data, get_order_to_sign,
    is_ethflow_order, merge_app_data_doc, swap_params_to_limit_order_params,
};

pub async fn post_swap_order<O, S>(
    trade_parameters: &TradeParameters,
    trader: &TraderParameters,
    signer: &S,
    advanced_settings: Option<&SwapAdvancedSettings>,
    orderbook: &O,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display,
{
    post_swap_order_async(
        trade_parameters,
        trader,
        signer,
        advanced_settings,
        orderbook,
    )
    .await
}

pub async fn post_swap_order_async<O, S>(
    trade_parameters: &TradeParameters,
    trader: &TraderParameters,
    signer: &S,
    advanced_settings: Option<&SwapAdvancedSettings>,
    orderbook: &O,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: AsyncSigner,
    S::Error: std::fmt::Display,
{
    let quote_results = crate::get_quote_results_async(
        trade_parameters,
        trader,
        signer,
        advanced_settings,
        orderbook,
    )
    .await?;

    post_swap_order_from_quote_async(&quote_results, trader, signer, advanced_settings, orderbook)
        .await
}

pub async fn post_swap_order_from_quote<O, S>(
    quote_results: &QuoteResults,
    trader: &TraderParameters,
    signer: &S,
    advanced_settings: Option<&SwapAdvancedSettings>,
    orderbook: &O,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display,
{
    post_swap_order_from_quote_async(quote_results, trader, signer, advanced_settings, orderbook)
        .await
}

pub async fn post_swap_order_from_quote_async<O, S>(
    quote_results: &QuoteResults,
    trader: &TraderParameters,
    signer: &S,
    advanced_settings: Option<&SwapAdvancedSettings>,
    orderbook: &O,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: AsyncSigner,
    S::Error: std::fmt::Display,
{
    let app_data_info = match advanced_settings.and_then(|settings| settings.app_data.as_ref()) {
        Some(app_data_override) => {
            merge_app_data_doc(&quote_results.app_data_info.doc, app_data_override)?
        }
        None => quote_results.app_data_info.clone(),
    };
    let params = apply_settings_to_limit_trade_parameters(
        &swap_params_to_limit_order_params(
            &quote_results.trade_parameters,
            &quote_results.quote_response,
        ),
        advanced_settings.and_then(|settings| settings.quote_request.as_ref()),
        advanced_settings.and_then(|settings| settings.app_data.as_ref()),
    );
    let additional = advanced_settings
        .and_then(|settings| settings.additional_params.clone())
        .unwrap_or_default();

    post_cow_protocol_trade_async(
        orderbook,
        &app_data_info,
        &params,
        &crate::types::PostTradeAdditionalParams {
            signing_scheme: advanced_settings
                .and_then(|settings| settings.quote_request.as_ref())
                .and_then(|request| request.signing_scheme),
            network_costs_amount: Some(quote_results.quote_response.quote.fee_amount.clone()),
            ..additional
        },
        trader,
        signer,
    )
    .await
}

pub async fn post_limit_order<O, S>(
    params: &LimitTradeParameters,
    trader: &TraderParameters,
    signer: &S,
    advanced_settings: Option<&LimitOrderAdvancedSettings>,
    orderbook: &O,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display,
{
    post_limit_order_async(params, trader, signer, advanced_settings, orderbook).await
}

pub async fn post_limit_order_async<O, S>(
    params: &LimitTradeParameters,
    trader: &TraderParameters,
    signer: &S,
    advanced_settings: Option<&LimitOrderAdvancedSettings>,
    orderbook: &O,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: AsyncSigner,
    S::Error: std::fmt::Display,
{
    let mut params = apply_settings_to_limit_trade_parameters(
        params,
        advanced_settings.and_then(|settings| settings.quote_request.as_ref()),
        advanced_settings.and_then(|settings| settings.app_data.as_ref()),
    );
    if params.slippage_bps.is_none() {
        params.slippage_bps = Some(0);
    }

    let app_data_info = build_app_data(
        &trader.app_code,
        params.slippage_bps.unwrap_or(0),
        "limit",
        params.partner_fee.as_ref(),
        advanced_settings.and_then(|settings| settings.app_data.as_ref()),
    )
    .await?;

    let mut additional = advanced_settings
        .and_then(|settings| settings.additional_params.clone())
        .unwrap_or_default();
    if additional.apply_costs_slippage_and_fees.is_none() {
        additional.apply_costs_slippage_and_fees = Some(false);
    }

    post_cow_protocol_trade_async(
        orderbook,
        &app_data_info,
        &params,
        &additional,
        trader,
        signer,
    )
    .await
}

pub async fn post_sell_native_currency_order<O, S>(
    orderbook: &O,
    app_data: &TradingAppDataInfo,
    params: &LimitTradeParameters,
    additional_params: &crate::types::PostTradeAdditionalParams,
    trader: &TraderParameters,
    signer: &S,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display,
{
    post_sell_native_currency_order_async(
        orderbook,
        app_data,
        params,
        additional_params,
        trader,
        signer,
    )
    .await
}

pub async fn post_sell_native_currency_order_async<O, S>(
    orderbook: &O,
    app_data: &TradingAppDataInfo,
    params: &LimitTradeParameters,
    additional_params: &crate::types::PostTradeAdditionalParams,
    trader: &TraderParameters,
    signer: &S,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: AsyncSigner,
    S::Error: std::fmt::Display,
{
    let tx = crate::get_eth_flow_transaction_async(
        &app_data.app_data_keccak256,
        params,
        orderbook.context().chain_id,
        additional_params,
        trader,
        signer,
    )
    .await?;

    orderbook
        .upload_app_data(&app_data.app_data_keccak256, &app_data.full_app_data)
        .await?;
    let receipt = signer
        .send_transaction(&tx.transaction)
        .await
        .map_err(|error| TradingError::Signer {
            operation: "send_transaction",
            message: error.to_string(),
        })?;

    Ok(OrderPostingResult {
        order_id: tx.order_id,
        tx_hash: Some(receipt.transaction_hash),
        order_to_sign: tx.order_to_sign,
        signature: String::new(),
        signing_scheme: SigningScheme::Eip1271,
    })
}

pub async fn post_cow_protocol_trade<O, S>(
    orderbook: &O,
    app_data: &TradingAppDataInfo,
    params: &LimitTradeParameters,
    additional_params: &crate::types::PostTradeAdditionalParams,
    trader: &TraderParameters,
    signer: &S,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display,
{
    post_cow_protocol_trade_async(
        orderbook,
        app_data,
        params,
        additional_params,
        trader,
        signer,
    )
    .await
}

pub async fn post_cow_protocol_trade_async<O, S>(
    orderbook: &O,
    app_data: &TradingAppDataInfo,
    params: &LimitTradeParameters,
    additional_params: &crate::types::PostTradeAdditionalParams,
    trader: &TraderParameters,
    signer: &S,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: AsyncSigner,
    S::Error: std::fmt::Display,
{
    let is_ethflow = is_ethflow_order(&params.sell_token);
    if is_ethflow {
        if params.quote_id.is_none() {
            return Err(TradingError::MissingQuoteId("EthFlow order posting"));
        }
        let adjusted = adjust_ethflow_limit_parameters(orderbook.context().chain_id, params);
        return post_sell_native_currency_order_async(
            orderbook,
            app_data,
            &adjusted,
            additional_params,
            trader,
            signer,
        )
        .await;
    }

    let chain_id = orderbook.context().chain_id;
    let from = match params.owner.clone() {
        Some(owner) => owner,
        None => signer
            .get_address()
            .await
            .map_err(|error| TradingError::Signer {
                operation: "get_address",
                message: error.to_string(),
            })?,
    };
    let options = ProtocolOptions {
        env: params.env.or(trader.env),
        settlement_contract_override: params
            .settlement_contract_override
            .clone()
            .or_else(|| trader.settlement_contract_override.clone()),
        eth_flow_contract_override: params
            .eth_flow_contract_override
            .clone()
            .or_else(|| trader.eth_flow_contract_override.clone()),
    };
    let order_to_sign = get_order_to_sign(
        crate::order::OrderToSignParams {
            chain_id,
            from: from.clone(),
            is_ethflow: false,
            network_costs_amount: additional_params.network_costs_amount.clone(),
            apply_costs_slippage_and_fees: additional_params
                .apply_costs_slippage_and_fees
                .unwrap_or(true),
            protocol_fee_bps: None,
        },
        params,
        &app_data.app_data_keccak256,
    )?;

    orderbook
        .upload_app_data(&app_data.app_data_keccak256, &app_data.full_app_data)
        .await?;

    let requested_scheme = additional_params
        .signing_scheme
        .unwrap_or(SigningScheme::Eip712);
    let (signature, signing_scheme) = sign_order_for_submission(
        &order_to_sign,
        chain_id,
        signer,
        requested_scheme,
        additional_params,
        &options,
        &from,
    )
    .await?;

    let order_body = OrderCreation {
        sell_token: order_to_sign.sell_token.clone(),
        buy_token: order_to_sign.buy_token.clone(),
        receiver: Some(order_to_sign.receiver.clone()),
        sell_amount: order_to_sign.sell_amount.clone(),
        buy_amount: order_to_sign.buy_amount.clone(),
        valid_to: order_to_sign.valid_to,
        app_data: Some(app_data.full_app_data.clone()),
        app_data_hash: Some(app_data.app_data_keccak256.clone()),
        fee_amount: order_to_sign.fee_amount.clone(),
        kind: order_to_sign.kind,
        partially_fillable: order_to_sign.partially_fillable,
        sell_token_balance: order_to_sign.sell_token_balance,
        buy_token_balance: order_to_sign.buy_token_balance,
        signing_scheme,
        signature: signature.clone(),
        from,
        quote_id: params.quote_id,
    };
    let order_id = orderbook.send_order(&order_body).await?;

    Ok(OrderPostingResult {
        order_id,
        tx_hash: None,
        signing_scheme,
        signature,
        order_to_sign,
    })
}

fn apply_settings_to_limit_trade_parameters(
    params: &LimitTradeParameters,
    quote_request: Option<&crate::QuoteRequestOverride>,
    app_data_override: Option<&cow_sdk_app_data::AppDataParams>,
) -> LimitTradeParameters {
    let mut params = params.clone();

    if let Some(app_data_override) = app_data_override {
        if let Some(slippage) = app_data_override
            .metadata
            .get("quote")
            .and_then(|quote| quote.get("slippageBips"))
            .and_then(|value| value.as_u64())
            .and_then(|value| u32::try_from(value).ok())
        {
            params.slippage_bps = Some(slippage);
        }
        if let Some(partner_fee) = app_data_override.metadata.get("partnerFee").cloned() {
            params.partner_fee = Some(partner_fee);
        }
    }

    if let Some(quote_request) = quote_request {
        if let Some(receiver) = &quote_request.receiver {
            params.receiver = Some(receiver.clone());
        }
        if let Some(valid_to) = quote_request.valid_to {
            params.valid_to = Some(valid_to);
        }
        if let Some(sell_token) = &quote_request.sell_token {
            params.sell_token = sell_token.clone();
        }
        if let Some(buy_token) = &quote_request.buy_token {
            params.buy_token = buy_token.clone();
        }
        if let Some(from) = &quote_request.from {
            params.owner = Some(from.clone());
        }
    }

    if params.env.is_none() {
        params.env = Some(cow_sdk_core::CowEnv::Prod);
    }

    params
}

async fn sign_order_for_submission<S>(
    order_to_sign: &cow_sdk_core::UnsignedOrder,
    chain_id: cow_sdk_core::SupportedChainId,
    signer: &S,
    scheme: SigningScheme,
    additional_params: &crate::types::PostTradeAdditionalParams,
    options: &ProtocolOptions,
    from: &cow_sdk_core::Address,
) -> Result<(String, SigningScheme), TradingError>
where
    S: AsyncSigner,
    S::Error: std::fmt::Display,
{
    match scheme {
        SigningScheme::PreSign => Ok((from.as_str().to_owned(), SigningScheme::PreSign)),
        SigningScheme::Eip1271 => {
            if let Some(provider) = &additional_params.custom_eip1271_signature {
                let signature = provider.sign(order_to_sign).await?;
                Ok((signature, SigningScheme::Eip1271))
            } else {
                let signing_result =
                    sign_order_async(order_to_sign, chain_id, signer, Some(options)).await?;
                let payload = eip1271_signature_payload(order_to_sign, &signing_result.signature)?;
                Ok((payload, SigningScheme::Eip1271))
            }
        }
        SigningScheme::Eip712 | SigningScheme::EthSign => {
            let scheme = match scheme {
                SigningScheme::Eip712 => SigningSchemeContract::Eip712,
                SigningScheme::EthSign => SigningSchemeContract::EthSign,
                SigningScheme::Eip1271 | SigningScheme::PreSign => unreachable!(),
            };
            let signing_result = sign_order_with_scheme_async(
                order_to_sign,
                chain_id,
                signer,
                scheme,
                Some(options),
            )
            .await?;
            Ok((
                signing_result.signature,
                map_contract_scheme(signing_result.signing_scheme),
            ))
        }
    }
}

fn map_contract_scheme(scheme: SigningSchemeContract) -> SigningScheme {
    match scheme {
        SigningSchemeContract::Eip712 => SigningScheme::Eip712,
        SigningSchemeContract::EthSign => SigningScheme::EthSign,
        SigningSchemeContract::Eip1271 => SigningScheme::Eip1271,
        SigningSchemeContract::PreSign => SigningScheme::PreSign,
    }
}
