use cow_sdk_core::{
    Amount, AsyncProvider, AsyncSigner, AtomAmount, ProtocolOptions, Provider, Signer,
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
use crate::{
    LimitOrderAdvancedSettings, LimitTradeParameters, OrderPostingResult, OrderbookClient,
    QuoteResults, SwapAdvancedSettings, TradeParameters, TraderParameters, TradingAppDataInfo,
    TradingError, adjust_ethflow_limit_parameters, build_app_data, get_order_to_sign,
    is_ethflow_order, merge_app_data_doc, swap_params_to_limit_order_params,
};

impl OrderPostingResult {
    /// Returns the posted order's sell amount as a typed [`AtomAmount`].
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::InvalidInput`] when the submitted order's
    /// wire-format sell amount cannot be parsed into the supported
    /// `uint256` range.
    pub fn sell_atom_amount(&self) -> Result<AtomAmount, TradingError> {
        AtomAmount::try_from(&self.order_to_sign.sell_amount)
            .map_err(|err| TradingError::InvalidInput(err.to_string()))
    }

    /// Returns the posted order's buy amount as a typed [`AtomAmount`].
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::InvalidInput`] when the submitted order's
    /// wire-format buy amount cannot be parsed into the supported
    /// `uint256` range.
    pub fn buy_atom_amount(&self) -> Result<AtomAmount, TradingError> {
        AtomAmount::try_from(&self.order_to_sign.buy_amount)
            .map_err(|err| TradingError::InvalidInput(err.to_string()))
    }

    /// Returns the posted order's fee amount as a typed [`AtomAmount`].
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::InvalidInput`] when the submitted order's
    /// wire-format fee amount cannot be parsed into the supported
    /// `uint256` range.
    pub fn fee_atom_amount(&self) -> Result<AtomAmount, TradingError> {
        AtomAmount::try_from(&self.order_to_sign.fee_amount)
            .map_err(|err| TradingError::InvalidInput(err.to_string()))
    }
}

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
    post_swap_order_async(
        trade_parameters,
        trader,
        signer,
        advanced_settings,
        orderbook,
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
    post_swap_order_from_quote_async(quote_results, trader, signer, advanced_settings, orderbook)
        .await
}

/// Signs and submits a swap order from previously computed quote results using an asynchronous
/// signer.
///
/// When advanced app-data settings are provided, they are merged on top of the quote-derived
/// document before submission. The submission orderbook must match the runtime
/// binding captured by the quote flow, and any explicit chain or environment
/// must agree with the injected orderbook client, which remains the canonical
/// runtime authority for signing and submission.
///
/// # Errors
///
/// Returns an error when the quoted trade cannot be converted into a postable order, when app-data
/// merging fails, when signing fails, or when the orderbook rejects the order submission.
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
    validate_quote_orderbook_binding(orderbook, quote_results.orderbook_binding.as_ref())?;

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
        )?,
        advanced_settings.and_then(|settings| settings.quote_request.as_ref()),
        advanced_settings.and_then(|settings| settings.app_data.as_ref()),
    )?;
    let additional = swap_additional_params(advanced_settings);

    post_cow_protocol_trade_async(
        orderbook,
        &app_data_info,
        &params,
        &crate::types::PostTradeAdditionalParams {
            signing_scheme: advanced_settings
                .and_then(|settings| settings.quote_request.as_ref())
                .and_then(|request| request.signing_scheme),
            network_costs_amount: Some(Amount::new(
                quote_results.quote_response.quote.fee_amount.clone(),
            )?),
            ..additional
        },
        trader,
        signer,
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
    post_limit_order_async(params, trader, signer, advanced_settings, orderbook).await
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
    )
    .await
}

/// Submits an `EthFlow`-style native-currency sell order using a synchronous signer.
///
/// This path uploads the supplied app-data, sends the prepared transaction through the signer, and
/// returns the resulting transaction hash.
///
/// # Errors
///
/// Returns an error when transaction preparation fails, when app-data upload fails, or when the
/// signer cannot send the transaction.
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

/// Submits an `EthFlow`-style native-currency sell order using an asynchronous signer.
///
/// This path uploads the supplied app-data, sends the prepared transaction through the signer, and
/// returns the resulting transaction hash.
///
/// # Errors
///
/// Returns an error when transaction preparation fails, when app-data upload fails, or when the
/// signer cannot send the transaction.
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
    reason = "the function linearly sequences one trade-posting orchestration path whose steps must stay co-located to preserve reviewed precedence"
)]
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
        && signer_address != &from
    {
        return Err(TradingError::RecoverableSignatureOwnerMismatch {
            scheme: requested_scheme,
            owner: from.as_str().to_owned(),
            signer: signer_address.as_str().to_owned(),
        });
    }
    let options = ProtocolOptions {
        env: params.env,
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

    let order_body = OrderCreation {
        sell_token: order_to_sign.sell_token.clone(),
        buy_token: order_to_sign.buy_token.clone(),
        receiver: Some(order_to_sign.receiver.clone()),
        sell_amount: order_to_sign.sell_amount.as_str().to_owned(),
        buy_amount: order_to_sign.buy_amount.as_str().to_owned(),
        valid_to: order_to_sign.valid_to,
        app_data: Some(app_data.full_app_data.clone()),
        app_data_hash: Some(app_data.app_data_keccak256.clone()),
        fee_amount: order_to_sign.fee_amount.as_str().to_owned(),
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
                    return Err(TradingError::InvalidInput(format!(
                        "unsupported order signing scheme `{scheme:?}`"
                    )));
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
        _ => Err(TradingError::InvalidInput(format!(
            "unsupported order signing scheme `{scheme:?}`"
        ))),
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
    cow_sdk_contracts::verify_eip1271_signature_async(provider, &request).await?;
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
