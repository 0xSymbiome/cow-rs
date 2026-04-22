use cow_sdk_core::{
    Address, Amount, AsyncProvider, AsyncSigner, ProtocolOptions, Provider, Signer,
};
use cow_sdk_orderbook::{OrderCreation, SigningScheme};
use cow_sdk_signing::{
    SigningScheme as SigningSchemeContract, eip1271_signature_payload, sign_order_async,
    sign_order_with_scheme_async,
};

use crate::types::{
    QuoteRequestParameterTargets, apply_app_data_parameter_overrides,
    apply_quote_request_parameter_overrides, validate_orderbook_context,
    validate_orderbook_env_context, validate_quote_orderbook_binding,
};
use crate::validation::OrderBoundsValidator;
use crate::{
    LimitOrderAdvancedSettings, LimitTradeParameters, OrderPostingResult, OrderbookClient,
    QuoteResults, SwapAdvancedSettings, TradeParameters, TraderParameters, TradingAppDataInfo,
    TradingError, adjust_ethflow_limit_parameters, build_app_data, get_order_to_sign,
    is_ethflow_order, merge_and_seal_app_data, params_from_doc, swap_params_to_limit_order_params,
};

// Non-suffixed posting functions are async entry points for synchronous Signer implementors.
// Keep workflow logic in the AsyncSigner implementations so both public paths stay aligned.
/// Quotes, signs, and submits a swap order using a synchronous signer.
///
/// Advanced settings override overlapping trade and app-data fields before submission.
///
/// # Errors
///
/// Returns an error when quoting fails, when app-data generation or merging fails, when signing
/// fails, or when the orderbook rejects the order submission.
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
    post_swap_order_with_bounds(
        trade_parameters,
        trader,
        signer,
        advanced_settings,
        orderbook,
        crate::validation::OrderValidityBounds::SERVICES_DEFAULT,
    )
    .await
}

/// Variant of [`post_swap_order`] that accepts a caller-supplied
/// [`crate::validation::OrderValidityBounds`] so the reviewed lifetime
/// ceiling can be tightened by policy.
///
/// # Errors
///
/// Returns an error when quoting fails, when app-data generation or merging
/// fails, when signing fails, or when the orderbook rejects the order
/// submission.
pub async fn post_swap_order_with_bounds<O, S>(
    trade_parameters: &TradeParameters,
    trader: &TraderParameters,
    signer: &S,
    advanced_settings: Option<&SwapAdvancedSettings>,
    orderbook: &O,
    order_bounds: crate::validation::OrderValidityBounds,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display,
{
    post_swap_order_async_with_bounds(
        trade_parameters,
        trader,
        signer,
        advanced_settings,
        orderbook,
        order_bounds,
    )
    .await
}

/// Quotes, signs, and submits a swap order using an asynchronous signer.
///
/// Advanced settings override overlapping trade and app-data fields before submission.
///
/// # Errors
///
/// Returns an error when quoting fails, when app-data generation or merging fails, when signing
/// fails, or when the orderbook rejects the order submission.
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
    post_swap_order_async_with_bounds(
        trade_parameters,
        trader,
        signer,
        advanced_settings,
        orderbook,
        crate::validation::OrderValidityBounds::SERVICES_DEFAULT,
    )
    .await
}

/// Variant of [`post_swap_order_async`] that accepts caller-supplied
/// [`crate::validation::OrderValidityBounds`].
///
/// # Errors
///
/// Returns an error when quoting fails, when app-data generation or merging
/// fails, when signing fails, or when the orderbook rejects the order
/// submission.
pub async fn post_swap_order_async_with_bounds<O, S>(
    trade_parameters: &TradeParameters,
    trader: &TraderParameters,
    signer: &S,
    advanced_settings: Option<&SwapAdvancedSettings>,
    orderbook: &O,
    order_bounds: crate::validation::OrderValidityBounds,
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

    post_swap_order_from_quote_async_with_bounds(
        &quote_results,
        trader,
        signer,
        advanced_settings,
        orderbook,
        order_bounds,
    )
    .await
}

/// Signs and submits a swap order from previously computed quote results using a synchronous
/// signer.
///
/// When advanced app-data settings are provided, they are merged on top of the quote-derived
/// document before submission. The submission orderbook must match the runtime
/// binding captured by the quote flow.
///
/// # Errors
///
/// Returns an error when the quoted trade cannot be converted into a postable order, when app-data
/// merging fails, when signing fails, or when the orderbook rejects the order submission.
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
    post_swap_order_from_quote_async_with_bounds(
        quote_results,
        trader,
        signer,
        advanced_settings,
        orderbook,
        crate::validation::OrderValidityBounds::SERVICES_DEFAULT,
    )
    .await
}

/// Signs and submits a swap order from previously computed quote results using an asynchronous
/// signer.
///
/// When advanced app-data settings are provided, they are merged on top of the quote-derived
/// document before submission. The submission orderbook must match the runtime
/// binding captured by the quote flow, and any explicit chain or environment
/// must agree with the injected orderbook client, which remains the canonical
/// runtime authority for signing and submission. Callers that need cooperative
/// cancellation wrap this future through
/// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
///
/// # Errors
///
/// Returns an error when the quoted trade cannot be converted into a postable order, when app-data
/// merging fails, when signing fails, or when the orderbook rejects the order submission.
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        skip_all,
        fields(
            chain = ?trader.chain_id,
            env = ?trader.env,
            endpoint = "trading.post_swap_order_from_quote_async",
        ),
    ),
)]
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
    post_swap_order_from_quote_async_with_bounds(
        quote_results,
        trader,
        signer,
        advanced_settings,
        orderbook,
        crate::validation::OrderValidityBounds::SERVICES_DEFAULT,
    )
    .await
}

/// Variant of [`post_swap_order_from_quote_async`] that accepts a
/// caller-supplied [`crate::validation::OrderValidityBounds`].
///
/// # Errors
///
/// Returns an error when the quoted trade cannot be converted into a
/// postable order, when app-data merging fails, when signing fails, or when
/// the orderbook rejects the order submission.
pub async fn post_swap_order_from_quote_async_with_bounds<O, S>(
    quote_results: &QuoteResults,
    trader: &TraderParameters,
    signer: &S,
    advanced_settings: Option<&SwapAdvancedSettings>,
    orderbook: &O,
    order_bounds: crate::validation::OrderValidityBounds,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: AsyncSigner,
    S::Error: std::fmt::Display,
{
    validate_quote_orderbook_binding(orderbook, quote_results.orderbook_binding.as_ref())?;

    let (app_data_info, merged_params) = if let Some(app_data_override) =
        advanced_settings.and_then(|settings| settings.app_data.as_ref())
    {
        merge_and_seal_app_data(&quote_results.app_data_info.doc, app_data_override)?
    } else {
        // Even without an override the submission seam reads the typed
        // `metadata.signer` field for the `AppdataFromMismatch`
        // invariant, so parse the sealed base doc back into typed
        // params and surface its signer alongside the existing
        // `TradingAppDataInfo`.
        let base_params = params_from_doc(&quote_results.app_data_info.doc)?;
        (quote_results.app_data_info.clone(), base_params)
    };
    let app_data_signer = merged_params.signer.clone();
    let params = apply_settings_to_limit_trade_parameters(
        &swap_params_to_limit_order_params(
            &quote_results.trade_parameters,
            &quote_results.quote_response,
        )?,
        advanced_settings.and_then(|settings| settings.quote_request.as_ref()),
        advanced_settings.and_then(|settings| settings.app_data.as_ref()),
    )?;
    let additional = swap_additional_params(advanced_settings);
    let additional_params = crate::types::PostTradeAdditionalParams {
        signing_scheme: advanced_settings
            .and_then(|settings| settings.quote_request.as_ref())
            .and_then(|request| request.signing_scheme),
        network_costs_amount: Some(Amount::new(
            quote_results
                .quote_response
                .quote
                .network_cost_amount()
                .to_owned(),
        )?),
        ..additional
    };

    post_cow_protocol_trade_async(
        orderbook,
        &app_data_info,
        &params,
        &additional_params,
        trader,
        signer,
        order_bounds,
        app_data_signer,
    )
    .await
}

/// Signs and submits a limit order using a synchronous signer.
///
/// Advanced settings override overlapping quote-request and app-data fields before submission.
///
/// # Errors
///
/// Returns an error when app-data generation fails, when signing fails, or when the orderbook
/// rejects the order submission.
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
    post_limit_order_async_with_bounds(
        params,
        trader,
        signer,
        advanced_settings,
        orderbook,
        crate::validation::OrderValidityBounds::SERVICES_DEFAULT,
    )
    .await
}

/// Signs and submits a limit order using an asynchronous signer.
///
/// Advanced settings override overlapping quote-request and app-data fields before submission.
/// When no slippage is supplied, limit-order posting uses `0` basis points in app-data and order
/// construction.
///
/// # Errors
///
/// Returns an error when app-data generation fails, when signing fails, or when the orderbook
/// rejects the order submission.
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
    post_limit_order_async_with_bounds(
        params,
        trader,
        signer,
        advanced_settings,
        orderbook,
        crate::validation::OrderValidityBounds::SERVICES_DEFAULT,
    )
    .await
}

/// Variant of [`post_limit_order_async`] that accepts a caller-supplied
/// [`crate::validation::OrderValidityBounds`].
///
/// # Errors
///
/// Returns an error when app-data generation fails, when signing fails, or
/// when the orderbook rejects the order submission.
pub async fn post_limit_order_async_with_bounds<O, S>(
    params: &LimitTradeParameters,
    trader: &TraderParameters,
    signer: &S,
    advanced_settings: Option<&LimitOrderAdvancedSettings>,
    orderbook: &O,
    order_bounds: crate::validation::OrderValidityBounds,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: AsyncSigner,
    S::Error: std::fmt::Display,
{
    let app_data_signer = advanced_settings
        .and_then(|settings| settings.app_data.as_ref())
        .and_then(|params| params.signer.clone());

    let mut params = apply_settings_to_limit_trade_parameters(
        params,
        advanced_settings.and_then(|settings| settings.quote_request.as_ref()),
        advanced_settings.and_then(|settings| settings.app_data.as_ref()),
    )?;
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

    let mut additional = limit_additional_params(advanced_settings);
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
        order_bounds,
        app_data_signer,
    )
    .await
}

/// Submits an `EthFlow`-style native-currency sell order using a synchronous signer.
///
/// This path uploads the supplied app-data, sends the prepared transaction through the signer, and
/// returns the resulting transaction hash. Callers that need cooperative
/// cancellation wrap this future through
/// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
///
/// # Errors
///
/// Returns an error when transaction preparation fails, when app-data upload fails, or when the
/// signer cannot send the transaction.
#[allow(
    clippy::too_many_arguments,
    reason = "the post-trade submission seam threads orchestration, validator, and runtime context through one entry point for parity with the reviewed services authority"
)]
pub async fn post_sell_native_currency_order<O, S>(
    orderbook: &O,
    app_data: &TradingAppDataInfo,
    params: &LimitTradeParameters,
    additional_params: &crate::types::PostTradeAdditionalParams,
    trader: &TraderParameters,
    signer: &S,
    order_bounds: crate::validation::OrderValidityBounds,
    app_data_signer: Option<Address>,
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
        order_bounds,
        app_data_signer,
    )
    .await
}

/// Submits an `EthFlow`-style native-currency sell order using an asynchronous signer.
///
/// This path uploads the supplied app-data, sends the prepared transaction through the signer, and
/// returns the resulting transaction hash. Callers that need cooperative
/// cancellation wrap this future through
/// [`cow_sdk_core::Cancellable::cancel_with`] at the call site; cancellation
/// only affects pre-broadcast work, because once the signer has broadcast the
/// prepared transaction, it cannot be withdrawn and the returned receipt will
/// reflect the chain result even if cancellation fires after submission. A
/// cancellation fired between transaction preparation and app-data upload is
/// a no-op on the orderbook service.
///
/// # Errors
///
/// Returns an error when transaction preparation fails, when app-data upload fails, or when the
/// signer cannot send the transaction.
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        skip_all,
        fields(
            chain = ?trader.chain_id,
            env = ?trader.env,
            endpoint = "trading.post_sell_native_currency_order_async",
        ),
    ),
)]
#[allow(
    clippy::too_many_arguments,
    reason = "the eth-flow submission seam threads orchestration, validator, and runtime context through one entry point for parity with the reviewed services authority"
)]
pub async fn post_sell_native_currency_order_async<O, S>(
    orderbook: &O,
    app_data: &TradingAppDataInfo,
    params: &LimitTradeParameters,
    additional_params: &crate::types::PostTradeAdditionalParams,
    trader: &TraderParameters,
    signer: &S,
    order_bounds: crate::validation::OrderValidityBounds,
    app_data_signer: Option<Address>,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: AsyncSigner,
    S::Error: std::fmt::Display,
{
    validate_orderbook_context(orderbook, Some(trader.chain_id), trader.env)?;
    validate_orderbook_env_context(orderbook, params.env)?;

    let orderbook_context = orderbook.context();
    let canonical_chain_id = orderbook_context.chain_id;
    let canonical_env = orderbook_context.env;
    let mut params = params.clone();
    params.env = Some(canonical_env);

    let tx = crate::get_eth_flow_transaction_async(
        &app_data.app_data_keccak256,
        &params,
        canonical_chain_id,
        additional_params,
        trader,
        signer,
    )
    .await?;

    let preview_from = tx.from.clone();
    let preview = OrderCreation::new(
        tx.order_to_sign.sell_token.clone(),
        tx.order_to_sign.buy_token.clone(),
        tx.order_to_sign.sell_amount.to_string(),
        tx.order_to_sign.buy_amount.to_string(),
        tx.order_to_sign.valid_to,
        tx.order_to_sign.kind,
        SigningScheme::Eip1271,
        String::new(),
        preview_from,
    );
    let validator =
        OrderBoundsValidator::new(order_bounds, crate::validation::SubmissionClass::Limit)
            .with_weth_address(wrapped_native_address(canonical_chain_id));
    validator
        .validate(
            &preview,
            SigningScheme::Eip1271,
            app_data_signer,
            current_unix_seconds(),
            true,
        )
        .map_err(TradingError::ClientRejected)?;

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

/// Signs and submits a `CoW` Protocol order using a synchronous signer.
///
/// `EthFlow` sell orders are routed to the native-currency transaction path. Other orders are signed
/// and submitted through the orderbook.
///
/// # Errors
///
/// Returns an error when `EthFlow` routing prerequisites are missing, when signing fails, when
/// app-data upload fails, or when the orderbook rejects the order submission.
#[allow(
    clippy::too_many_arguments,
    reason = "the trade-posting submission seam threads orchestration, validator, and runtime context through one entry point for parity with the reviewed services authority"
)]
pub async fn post_cow_protocol_trade<O, S>(
    orderbook: &O,
    app_data: &TradingAppDataInfo,
    params: &LimitTradeParameters,
    additional_params: &crate::types::PostTradeAdditionalParams,
    trader: &TraderParameters,
    signer: &S,
    order_bounds: crate::validation::OrderValidityBounds,
    app_data_signer: Option<Address>,
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
        order_bounds,
        app_data_signer,
    )
    .await
}

/// Signs and submits a `CoW` Protocol order using an asynchronous signer.
///
/// Any explicit chain or environment must agree with the injected orderbook client, which is then
/// used as the canonical runtime authority for order construction, signing, and submission.
/// `EthFlow` sell orders require a quote identifier and are routed to the native-currency
/// transaction path. Other orders are uploaded to the orderbook after signing with the requested
/// or default signing scheme.
///
/// # Errors
///
/// Returns an error when owner resolution fails, when `EthFlow` routing prerequisites are missing,
/// when order construction or signing fails, when app-data upload fails, or when the orderbook
/// rejects the order submission.
#[allow(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    reason = "the function linearly sequences one trade-posting orchestration path whose steps must stay co-located to preserve reviewed precedence; the parameter list threads orchestration, validator, and runtime context through one entry point"
)]
pub async fn post_cow_protocol_trade_async<O, S>(
    orderbook: &O,
    app_data: &TradingAppDataInfo,
    params: &LimitTradeParameters,
    additional_params: &crate::types::PostTradeAdditionalParams,
    trader: &TraderParameters,
    signer: &S,
    order_bounds: crate::validation::OrderValidityBounds,
    app_data_signer: Option<Address>,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: AsyncSigner,
    S::Error: std::fmt::Display,
{
    validate_orderbook_context(orderbook, Some(trader.chain_id), trader.env)?;
    validate_orderbook_env_context(orderbook, params.env)?;

    let orderbook_context = orderbook.context();
    let canonical_chain_id = orderbook_context.chain_id;
    let canonical_env = orderbook_context.env;
    let mut params = params.clone();
    params.env = Some(canonical_env);
    let is_ethflow = is_ethflow_order(&params.sell_token);
    if is_ethflow {
        if params.quote_id.is_none() {
            return Err(TradingError::MissingQuoteId("EthFlow order posting"));
        }
        let adjusted = adjust_ethflow_limit_parameters(canonical_chain_id, &params);
        return post_sell_native_currency_order_async(
            orderbook,
            app_data,
            &adjusted,
            additional_params,
            trader,
            signer,
            order_bounds,
            app_data_signer,
        )
        .await;
    }

    let chain_id = canonical_chain_id;
    let requested_scheme = additional_params
        .signing_scheme
        .unwrap_or(SigningScheme::Eip712);
    let signer_address = if params.owner.is_none()
        || matches!(
            requested_scheme,
            SigningScheme::Eip712 | SigningScheme::EthSign
        ) {
        Some(
            signer
                .get_address()
                .await
                .map_err(|error| TradingError::Signer {
                    operation: "get_address",
                    message: error.to_string(),
                })?,
        )
    } else {
        None
    };
    let from = params
        .owner
        .clone()
        .or_else(|| signer_address.clone())
        .ok_or(TradingError::MissingSubmissionOwner)?;
    if matches!(
        requested_scheme,
        SigningScheme::Eip712 | SigningScheme::EthSign
    ) && let Some(signer_address) = signer_address.as_ref()
    {
        crate::validation::assert_owner_matches_signer(&from, signer_address)
            .map_err(TradingError::ClientRejected)?;
    }
    let mut options = ProtocolOptions::new();
    if let Some(env) = params.env {
        options = options.with_env(env);
    }
    if let Some(overrides) = params
        .settlement_contract_override
        .clone()
        .or_else(|| trader.settlement_contract_override.clone())
    {
        options = options.with_settlement_contract_override(overrides);
    }
    if let Some(overrides) = params
        .eth_flow_contract_override
        .clone()
        .or_else(|| trader.eth_flow_contract_override.clone())
    {
        options = options.with_eth_flow_contract_override(overrides);
    }
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
        &params,
        &app_data.app_data_keccak256,
    )?;

    orderbook
        .upload_app_data(&app_data.app_data_keccak256, &app_data.full_app_data)
        .await?;

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

    let mut order_body = OrderCreation::new(
        order_to_sign.sell_token.clone(),
        order_to_sign.buy_token.clone(),
        order_to_sign.sell_amount.to_string(),
        order_to_sign.buy_amount.to_string(),
        order_to_sign.valid_to,
        order_to_sign.kind,
        signing_scheme,
        signature.clone(),
        from,
    )
    .with_receiver(order_to_sign.receiver.clone())
    .with_app_data(app_data.full_app_data.clone())
    .with_app_data_hash(app_data.app_data_keccak256.clone())
    .with_partially_fillable(order_to_sign.partially_fillable)
    .with_sell_token_balance(order_to_sign.sell_token_balance)
    .with_buy_token_balance(order_to_sign.buy_token_balance);
    if let Some(quote_id) = params.quote_id {
        order_body = order_body.with_quote_id(quote_id);
    }

    let validator =
        OrderBoundsValidator::new(order_bounds, crate::validation::SubmissionClass::Limit)
            .with_weth_address(wrapped_native_address(chain_id));
    validator
        .validate(
            &order_body,
            signing_scheme,
            app_data_signer,
            current_unix_seconds(),
            false,
        )
        .map_err(TradingError::ClientRejected)?;
    let order_id = orderbook.send_order(&order_body).await?;

    Ok(OrderPostingResult {
        order_id,
        tx_hash: None,
        signing_scheme,
        signature,
        order_to_sign,
    })
}

fn current_unix_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn wrapped_native_address(chain_id: cow_sdk_core::SupportedChainId) -> Address {
    cow_sdk_core::wrapped_native_token(chain_id).address
}

fn apply_settings_to_limit_trade_parameters(
    params: &LimitTradeParameters,
    quote_request: Option<&crate::QuoteRequestOverride>,
    app_data_override: Option<&cow_sdk_app_data::AppDataParams>,
) -> Result<LimitTradeParameters, TradingError> {
    let mut params = params.clone();

    apply_app_data_parameter_overrides(
        &mut params.slippage_bps,
        &mut params.partner_fee,
        app_data_override,
    )?;
    apply_quote_request_parameter_overrides(
        &mut QuoteRequestParameterTargets {
            owner: &mut params.owner,
            sell_token: &mut params.sell_token,
            buy_token: &mut params.buy_token,
            receiver: &mut params.receiver,
            valid_for: &mut params.valid_for,
            valid_to: &mut params.valid_to,
            partially_fillable: &mut params.partially_fillable,
            sell_token_balance: &mut params.sell_token_balance,
            buy_token_balance: &mut params.buy_token_balance,
        },
        quote_request,
    );

    Ok(params)
}

fn swap_additional_params(
    advanced_settings: Option<&SwapAdvancedSettings>,
) -> crate::types::PostTradeAdditionalParams {
    advanced_settings
        .and_then(|settings| settings.additional_params.clone())
        .unwrap_or_default()
}

fn limit_additional_params(
    advanced_settings: Option<&LimitOrderAdvancedSettings>,
) -> crate::types::PostTradeAdditionalParams {
    advanced_settings
        .and_then(|settings| settings.additional_params.clone())
        .unwrap_or_default()
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
                _ => {
                    return Err(TradingError::InvalidInput {
                        field: "signingScheme",
                        reason: cow_sdk_core::ValidationReason::Precondition {
                            details: "order signing scheme is not supported",
                        },
                    });
                }
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
                map_contract_scheme(signing_result.signing_scheme)?,
            ))
        }
        _ => Err(TradingError::InvalidInput {
            field: "signingScheme",
            reason: cow_sdk_core::ValidationReason::Precondition {
                details: "order signing scheme is not supported",
            },
        }),
    }
}

/// Builds an EIP-1271 verification request for a `CoW` order digest.
///
/// # Errors
///
/// Returns an error when the signing domain cannot be resolved or when the order digest cannot be
/// derived for the verification request.
pub fn eip1271_order_verification_request(
    order_to_sign: &cow_sdk_core::UnsignedOrder,
    chain_id: cow_sdk_core::SupportedChainId,
    verification: &crate::types::Eip1271VerificationParameters,
    options: Option<&ProtocolOptions>,
) -> Result<cow_sdk_contracts::Eip1271VerificationRequest, TradingError> {
    let domain = cow_sdk_signing::get_domain(chain_id, options)?;
    let digest =
        cow_sdk_contracts::hash_order(&domain, &cow_sdk_contracts::Order::from(order_to_sign))?;

    Ok(cow_sdk_contracts::Eip1271VerificationRequest {
        verifier: verification.verifier.clone(),
        digest,
        signature: verification.signature.clone(),
    })
}

/// Verifies an EIP-1271 order signature with a synchronous provider.
///
/// # Errors
///
/// Returns an error when the verification request cannot be derived or when the provider reports
/// missing code, malformed responses, or an invalid EIP-1271 magic value.
pub fn verify_eip1271_order_signature<P>(
    provider: &P,
    order_to_sign: &cow_sdk_core::UnsignedOrder,
    chain_id: cow_sdk_core::SupportedChainId,
    verification: &crate::types::Eip1271VerificationParameters,
    options: Option<&ProtocolOptions>,
) -> Result<(), TradingError>
where
    P: Provider,
    P::Error: std::fmt::Display,
{
    let request =
        eip1271_order_verification_request(order_to_sign, chain_id, verification, options)?;
    cow_sdk_contracts::verify_eip1271_signature(provider, &request)?;
    Ok(())
}

/// Verifies an EIP-1271 order signature with an asynchronous provider.
///
/// # Errors
///
/// Returns an error when the verification request cannot be derived or when the provider reports
/// missing code, malformed responses, or an invalid EIP-1271 magic value.
pub async fn verify_eip1271_order_signature_async<P>(
    provider: &P,
    order_to_sign: &cow_sdk_core::UnsignedOrder,
    chain_id: cow_sdk_core::SupportedChainId,
    verification: &crate::types::Eip1271VerificationParameters,
    options: Option<&ProtocolOptions>,
) -> Result<(), TradingError>
where
    P: AsyncProvider,
    P::Error: std::fmt::Display,
{
    let request =
        eip1271_order_verification_request(order_to_sign, chain_id, verification, options)?;
    cow_sdk_contracts::verify_eip1271_signature_async(
        provider,
        &request,
        &cow_sdk_signing::NoopEip1271VerificationCache,
    )
    .await?;
    Ok(())
}

const fn map_contract_scheme(scheme: SigningSchemeContract) -> Result<SigningScheme, TradingError> {
    match scheme {
        SigningSchemeContract::Eip712 => Ok(SigningScheme::Eip712),
        SigningSchemeContract::EthSign => Ok(SigningScheme::EthSign),
        SigningSchemeContract::Eip1271 => Ok(SigningScheme::Eip1271),
        SigningSchemeContract::PreSign => Ok(SigningScheme::PreSign),
        _ => Err(TradingError::UnsupportedLocalSigningScheme { scheme }),
    }
}
