//! Quote-to-post orchestration helpers grouped by order family.

use cow_sdk_core::{Address, ProtocolOptions, Provider, Signer};
use cow_sdk_orderbook::{OrderCreation, SigningScheme};
use cow_sdk_signing::{
    SigningScheme as SigningSchemeContract, eip1271_signature_payload, sign_order,
    sign_order_with_scheme,
};

use crate::types::{
    QuoteRequestParameterTargets, apply_app_data_parameter_overrides,
    apply_quote_request_parameter_overrides, validate_orderbook_context,
    validate_orderbook_env_context, validate_quote_orderbook_binding,
};
use crate::validation::OrderBoundsValidator;
use crate::{
    LimitTradeParams, LimitTradeParamsFromQuote, OrderPostingResult, OrderbookClient, QuoteResults,
    TradeAdvancedSettings, TradeParams, TraderParams, TradingAppDataInfo, TradingError,
    adjust_eth_flow_limit_params, build_app_data, is_eth_flow_order, merge_and_seal_app_data,
    order_to_sign, params_from_doc, swap_params_to_limit_order_params,
};

fn build_order_body(
    order_to_sign: &cow_sdk_core::OrderData,
    app_data: &TradingAppDataInfo,
    scheme: SigningScheme,
    signature: String,
    from: Address,
    params: &LimitTradeParams,
) -> OrderCreation {
    OrderCreation::from_signed(
        order_to_sign,
        scheme,
        signature,
        from,
        Some(app_data.full_app_data.clone()),
        params.quote_id,
    )
}

/// Signs and submits a `CoW` Protocol order.
///
/// Any explicit chain or environment must agree with the injected orderbook client, which is then
/// used as the canonical runtime authority for order construction, signing, and submission.
/// `EthFlow` sell orders require a quote identifier and are routed to the native-currency
/// transaction path. Other orders are validated client-side before app-data upload and signing,
/// then submitted with the requested or default signing scheme.
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
pub async fn post_cow_protocol_trade<O, S>(
    orderbook: &O,
    app_data: &TradingAppDataInfo,
    params: &LimitTradeParams,
    additional_params: &crate::types::PostTradeAdditionalParams,
    trader: &TraderParams,
    signer: &S,
    app_data_signer: Option<Address>,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    validate_orderbook_context(orderbook, Some(trader.chain_id), trader.env)?;
    validate_orderbook_env_context(orderbook, params.env)?;

    let orderbook_context = orderbook.context();
    let canonical_chain_id = orderbook_context.chain_id;
    let canonical_env = orderbook_context.env;
    let mut params = params.clone();
    params.env = Some(canonical_env);
    let is_eth_flow = is_eth_flow_order(&params.sell_token);
    if is_eth_flow {
        let adjusted = adjust_eth_flow_limit_params(canonical_chain_id, &params);
        let from_quote = LimitTradeParamsFromQuote::try_from_limit(adjusted)?;
        return post_sell_native_currency_order(
            orderbook,
            app_data,
            &from_quote,
            additional_params,
            trader,
            signer,
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
                .address()
                .await
                .map_err(|error| TradingError::Signer {
                    operation: "address",
                    message: error.to_string().into(),
                })?,
        )
    } else {
        None
    };
    let from = params
        .owner
        .or(signer_address)
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
    let order_to_sign = order_to_sign(
        crate::order::OrderToSignParams {
            chain_id,
            from,
            is_eth_flow: false,
            network_costs_amount: additional_params.network_costs_amount,
            apply_costs_slippage_and_fees: additional_params
                .apply_costs_slippage_and_fees
                .unwrap_or(true),
            protocol_fee_bps: None,
        },
        &params,
        &app_data.app_data_keccak256,
    )?;

    let validator = OrderBoundsValidator::services_default_for_chain(chain_id);
    validator
        .validate(
            &order_to_sign,
            from,
            app_data_signer,
            current_unix_seconds(),
            false,
        )
        .map_err(TradingError::ClientRejected)?;

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

    let order_body = build_order_body(
        &order_to_sign,
        app_data,
        signing_scheme,
        signature.clone(),
        from,
        &params,
    );
    let order_id = orderbook.send_order(&order_body).await?;

    Ok(OrderPostingResult {
        order_id,
        tx_hash: None,
        signing_scheme,
        signature,
        order_to_sign,
    })
}

pub(super) fn current_unix_seconds() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    use std::time::{SystemTime, UNIX_EPOCH};
    #[cfg(target_arch = "wasm32")]
    use web_time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub(super) fn apply_settings_to_limit_trade_parameters(
    params: &LimitTradeParams,
    quote_request: Option<&crate::QuoteRequestOverride>,
    app_data_override: Option<&cow_sdk_app_data::AppDataParams>,
) -> Result<LimitTradeParams, TradingError> {
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

pub(super) fn advanced_additional_params(
    advanced_settings: Option<&TradeAdvancedSettings>,
) -> crate::types::PostTradeAdditionalParams {
    advanced_settings
        .and_then(|settings| settings.additional_params.clone())
        .unwrap_or_default()
}

async fn sign_order_for_submission<S>(
    order_to_sign: &cow_sdk_core::OrderData,
    chain_id: cow_sdk_core::SupportedChainId,
    signer: &S,
    scheme: SigningScheme,
    additional_params: &crate::types::PostTradeAdditionalParams,
    options: &ProtocolOptions,
    from: &cow_sdk_core::Address,
) -> Result<(String, SigningScheme), TradingError>
where
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    match scheme {
        SigningScheme::PreSign => Ok((from.to_hex_string(), SigningScheme::PreSign)),
        SigningScheme::Eip1271 => {
            if let Some(provider) = &additional_params.custom_eip1271_signature {
                let signature =
                    provider
                        .sign(order_to_sign)
                        .await
                        .map_err(|error| TradingError::Signer {
                            operation: "eip1271_signature",
                            message: error.to_string().into(),
                        })?;
                Ok((signature, SigningScheme::Eip1271))
            } else {
                let signing_result =
                    sign_order(order_to_sign, chain_id, signer, Some(options)).await?;
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
            let signing_result =
                sign_order_with_scheme(order_to_sign, chain_id, signer, scheme, Some(options))
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

const fn map_contract_scheme(scheme: SigningSchemeContract) -> Result<SigningScheme, TradingError> {
    match scheme {
        SigningSchemeContract::Eip712 => Ok(SigningScheme::Eip712),
        SigningSchemeContract::EthSign => Ok(SigningScheme::EthSign),
        SigningSchemeContract::Eip1271 => Ok(SigningScheme::Eip1271),
        SigningSchemeContract::PreSign => Ok(SigningScheme::PreSign),
        _ => Err(TradingError::UnsupportedLocalSigningScheme { scheme }),
    }
}

/// Signs and submits a swap order from previously computed quote results.
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
            endpoint = "trading.post_swap_order_from_quote",
        ),
    ),
)]
pub async fn post_swap_order_from_quote<O, S>(
    quote_results: &QuoteResults,
    trader: &TraderParams,
    signer: &S,
    advanced_settings: Option<&TradeAdvancedSettings>,
    orderbook: &O,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
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
    let app_data_signer = merged_params.signer;
    let limit_from_quote = swap_params_to_limit_order_params(
        &quote_results.trade_parameters,
        &quote_results.quote_response,
    )?;
    let params = apply_settings_to_limit_trade_parameters(
        limit_from_quote.as_limit(),
        advanced_settings.and_then(|settings| settings.quote_request.as_ref()),
        advanced_settings.and_then(|settings| settings.app_data.as_ref()),
    )?;
    let additional = advanced_additional_params(advanced_settings);
    let additional_params = crate::types::PostTradeAdditionalParams {
        signing_scheme: advanced_settings
            .and_then(|settings| settings.quote_request.as_ref())
            .and_then(|request| request.signing_scheme),
        network_costs_amount: Some(*quote_results.quote_response.quote.network_cost_amount()),
        ..additional
    };

    post_cow_protocol_trade(
        orderbook,
        &app_data_info,
        &params,
        &additional_params,
        trader,
        signer,
        app_data_signer,
    )
    .await
}

/// Signs and submits a limit order.
///
/// Advanced settings override overlapping quote-request and app-data fields before submission.
/// When no slippage is supplied, limit-order posting uses `0` basis points in app-data and order
/// construction.
///
/// # Errors
///
/// Returns an error when app-data generation fails, when signing fails, or when the orderbook
/// rejects the order submission.
pub async fn post_limit_order<O, S>(
    params: &LimitTradeParams,
    trader: &TraderParams,
    signer: &S,
    advanced_settings: Option<&TradeAdvancedSettings>,
    orderbook: &O,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    let app_data_signer = advanced_settings
        .and_then(|settings| settings.app_data.as_ref())
        .and_then(|params| params.signer);

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

    let mut additional = advanced_additional_params(advanced_settings);
    if additional.apply_costs_slippage_and_fees.is_none() {
        additional.apply_costs_slippage_and_fees = Some(false);
    }

    post_cow_protocol_trade(
        orderbook,
        &app_data_info,
        &params,
        &additional,
        trader,
        signer,
        app_data_signer,
    )
    .await
}

/// Submits an `EthFlow`-style native-currency sell order.
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
            endpoint = "trading.post_sell_native_currency_order",
        ),
    ),
)]
#[allow(
    clippy::too_many_arguments,
    reason = "the eth-flow submission seam threads orchestration, validator, and runtime context through one entry point for parity with the reviewed services authority"
)]
pub async fn post_sell_native_currency_order<O, S>(
    orderbook: &O,
    app_data: &TradingAppDataInfo,
    params: &LimitTradeParamsFromQuote,
    additional_params: &crate::types::PostTradeAdditionalParams,
    trader: &TraderParams,
    signer: &S,
    app_data_signer: Option<Address>,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    validate_orderbook_context(orderbook, Some(trader.chain_id), trader.env)?;
    validate_orderbook_env_context(orderbook, params.as_limit().env)?;

    let orderbook_context = orderbook.context();
    let canonical_chain_id = orderbook_context.chain_id;
    let canonical_env = orderbook_context.env;
    let mut inner = params.as_limit().clone();
    inner.env = Some(canonical_env);
    let params = LimitTradeParamsFromQuote::try_from_limit(inner)?;

    let tx = crate::eth_flow_transaction(
        &app_data.app_data_keccak256,
        &params,
        canonical_chain_id,
        additional_params,
        trader,
        signer,
    )
    .await?;

    let validator = OrderBoundsValidator::services_default_for_chain(canonical_chain_id);
    validator
        .validate(
            &tx.order_to_sign,
            tx.from,
            app_data_signer,
            current_unix_seconds(),
            true,
        )
        .map_err(TradingError::ClientRejected)?;

    orderbook
        .upload_app_data(&app_data.app_data_keccak256, &app_data.full_app_data)
        .await?;

    let broadcast = signer
        .send_transaction(&tx.transaction)
        .await
        .map_err(|error| TradingError::Signer {
            operation: "send_transaction",
            message: error.to_string().into(),
        })?;

    Ok(OrderPostingResult {
        order_id: tx.order_id,
        tx_hash: Some(broadcast.transaction_hash),
        order_to_sign: tx.order_to_sign,
        signature: String::new(),
        signing_scheme: SigningScheme::Eip1271,
    })
}

/// Quotes, signs, and submits a swap order.
///
/// Advanced settings override overlapping trade and app-data fields before submission.
///
/// # Errors
///
/// Returns an error when quoting fails, when app-data generation or merging fails, when signing
/// fails, or when the orderbook rejects the order submission.
pub async fn post_swap_order<O, S>(
    trade_parameters: &TradeParams,
    trader: &TraderParams,
    signer: &S,
    advanced_settings: Option<&TradeAdvancedSettings>,
    orderbook: &O,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    let quote_results = crate::quote_results(
        trade_parameters,
        trader,
        signer,
        advanced_settings,
        orderbook,
    )
    .await?;

    post_swap_order_from_quote(&quote_results, trader, signer, advanced_settings, orderbook).await
}

/// Builds an EIP-1271 verification request for a `CoW` order digest.
///
/// # Errors
///
/// Returns an error when the signing domain cannot be resolved or when the order digest cannot be
/// derived for the verification request.
pub fn eip1271_order_verification_request(
    order_to_sign: &cow_sdk_core::OrderData,
    chain_id: cow_sdk_core::SupportedChainId,
    verification: &crate::types::Eip1271VerificationParams,
    options: Option<&ProtocolOptions>,
) -> Result<cow_sdk_contracts::Eip1271VerificationRequest, TradingError> {
    let domain = cow_sdk_signing::domain(chain_id, options)?;
    let digest = cow_sdk_contracts::hash_order(&domain, order_to_sign)?;

    Ok(cow_sdk_contracts::Eip1271VerificationRequest::new(
        verification.verifier,
        digest,
        verification.signature.clone(),
    ))
}

/// Verifies an EIP-1271 order signature against a provider.
///
/// Use this to confirm that a smart-account (EIP-1271) wallet's signature over a
/// `CoW` order is valid — the verifier contract is called and must return the
/// EIP-1271 magic value.
///
/// # Errors
///
/// Returns an error when the verification request cannot be derived or when the provider reports
/// missing code, malformed responses, or an invalid EIP-1271 magic value.
///
/// ```no_run
/// # use cow_sdk_trading::{verify_eip1271_order_signature, Eip1271VerificationParams};
/// # use cow_sdk_core::{Address, HexData, OrderData, Provider, SupportedChainId};
/// # async fn demo<P>(provider: &P, order: &OrderData) -> Result<(), Box<dyn std::error::Error>>
/// # where P: Provider, P::Error: std::fmt::Display {
/// let verification = Eip1271VerificationParams::new(
///     Address::ZERO,              // the smart-account verifier contract
///     HexData::new("0x1234")?,    // the EIP-1271 signature payload (illustrative)
/// );
/// verify_eip1271_order_signature(provider, order, SupportedChainId::Mainnet, &verification, None)
///     .await?;
/// # Ok(())
/// # }
/// ```
pub async fn verify_eip1271_order_signature<P>(
    provider: &P,
    order_to_sign: &cow_sdk_core::OrderData,
    chain_id: cow_sdk_core::SupportedChainId,
    verification: &crate::types::Eip1271VerificationParams,
    options: Option<&ProtocolOptions>,
) -> Result<(), TradingError>
where
    P: Provider,
    P::Error: std::fmt::Display,
{
    let request =
        eip1271_order_verification_request(order_to_sign, chain_id, verification, options)?;
    let verification = cow_sdk_contracts::verify_eip1271_signature_cached(
        provider,
        &request,
        &cow_sdk_signing::NoopEip1271Cache,
    );
    #[cfg(feature = "tracing")]
    let verification = {
        use tracing::Instrument as _;

        verification.instrument(tracing::debug_span!(
            "trading.verify_eip1271_caller",
            chain_id = ?chain_id,
            verifier = %request.verifier,
        ))
    };
    verification.await?;
    Ok(())
}
