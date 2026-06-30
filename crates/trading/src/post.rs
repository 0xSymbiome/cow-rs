//! Quote-to-post orchestration helpers grouped by order family.

use cow_sdk_contracts::RecoverableSignature;
use cow_sdk_core::{
    Address, Amount, ProtocolOptions, Signer, TransactionBroadcast, TransactionRequest,
    TypedDataPayload,
};
use cow_sdk_orderbook::{OrderClass, OrderCreation, SigningScheme};
use cow_sdk_signing::{
    SigningScheme as SigningSchemeContract, eip1271_signature_payload, generate_order_id,
    sign_order, sign_order_with_scheme,
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
    order_to_sign, params_from_doc, sanitize_protocol_fee_bps, swap_params_to_limit_order_params,
};

/// Recovers the ECDSA signer from a produced order signature and requires it to
/// equal the declared owner, the client-side mirror of the services
/// submission-side `WrongOwner` check (ADR 0015).
///
/// Only `Eip712`/`EthSign` carry a recoverable ECDSA signature; EIP-1271 and
/// pre-sign authorizations are verified by their own mechanisms and skip the
/// gate. The recovery is scheme-aware (`EthSign` uses the EIP-191 prehash),
/// and the digest is the order's EIP-712 hash — exactly what the signer signed
/// — so an honest signer always recovers to `from`.
fn assert_recovered_owner(
    signature: &str,
    order_to_sign: &cow_sdk_core::OrderData,
    chain_id: cow_sdk_core::SupportedChainId,
    options: &ProtocolOptions,
    scheme: SigningScheme,
    from: &Address,
) -> Result<(), TradingError> {
    let contract_scheme = match scheme {
        SigningScheme::Eip712 => SigningSchemeContract::Eip712,
        SigningScheme::EthSign => SigningSchemeContract::EthSign,
        // EIP-1271 and pre-sign carry no recoverable ECDSA signature.
        _ => return Ok(()),
    };
    let order_digest =
        generate_order_id(chain_id, order_to_sign, from, Some(options))?.order_digest;
    let recovered =
        RecoverableSignature::parse_hex(signature)?.recover(&order_digest, contract_scheme)?;
    crate::validation::assert_owner_matches_signer(from, &recovered)
        .map_err(TradingError::ClientRejected)
}

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
    S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
{
    validate_orderbook_context(orderbook, Some(trader.chain_id), trader.env)?;
    validate_orderbook_env_context(orderbook, params.env)?;

    let orderbook_context = orderbook.context();
    let canonical_chain_id = orderbook_context.chain_id;
    let canonical_env = orderbook_context.env;
    // Chain-coherence gate (ADR 0015): when the signer statically knows its
    // chain, it must match the trading client's canonical chain before any
    // signing happens. A signer bound to the wrong chain would produce an
    // EIP-712 signature with the wrong domain separator that the orderbook
    // would reject after a wasted round-trip; fail closed locally instead.
    // Signers that learn their chain at runtime return `None` and opt out
    // (this also covers the pre-sign placement stand-in).
    if let Some(signer_chain) = signer.chain_id()
        && signer_chain != canonical_chain_id
    {
        return Err(TradingError::ChainMismatch {
            signer: signer_chain,
            trading: canonical_chain_id,
        });
    }
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
    // Pre-sign fast-fail: reject a declared owner that disagrees with the
    // signer's self-reported address before wasting an app-data upload and a
    // signature. The post-sign recovery gate below is the authoritative check
    // (it inspects the signature itself); this is the cheap early-out.
    if matches!(
        requested_scheme,
        SigningScheme::Eip712 | SigningScheme::EthSign
    ) && let Some(signer_address) = signer_address.as_ref()
    {
        crate::validation::assert_owner_matches_signer(&from, signer_address)
            .map_err(TradingError::ClientRejected)?;
    }
    let options = crate::onchain::protocol_options(
        params.env,
        params.settlement_contract_override.as_ref(),
        trader.settlement_contract_override.as_ref(),
        params.eth_flow_contract_override.as_ref(),
        trader.eth_flow_contract_override.as_ref(),
    );
    let order_to_sign = build_order_to_sign(chain_id, from, additional_params, &params, app_data)?;

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

    // Post-sign owner recovery: the produced ECDSA signature must recover to the
    // declared owner before the order reaches the wire. This is the client-side
    // mirror of the services submission-side `WrongOwner` check (ADR 0015) and
    // catches a signer that reports one address but signs with a different key —
    // a case the signer's self-reported address cannot.
    assert_recovered_owner(
        &signature,
        &order_to_sign,
        chain_id,
        &options,
        signing_scheme,
        &from,
    )?;

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

/// Builds the unsigned order payload (`order_to_sign`) for a postable trade.
///
/// The single site that assembles [`OrderToSignParams`](crate::order::OrderToSignParams)
/// from the resolved owner and additional-params bag and calls the public
/// [`order_to_sign`] helper. [`post_cow_protocol_trade`] consumes it on the
/// submission path, and [`build_limit_order_to_sign`] consumes it to expose the
/// same digest at a boundary, so the two cannot diverge.
fn build_order_to_sign(
    chain_id: cow_sdk_core::SupportedChainId,
    from: Address,
    additional_params: &crate::types::PostTradeAdditionalParams,
    params: &LimitTradeParams,
    app_data: &TradingAppDataInfo,
) -> Result<cow_sdk_core::OrderData, TradingError> {
    order_to_sign(
        crate::order::OrderToSignParams {
            chain_id,
            from,
            is_eth_flow: false,
            network_costs_amount: additional_params.network_costs_amount,
            apply_costs_slippage_and_fees: additional_params
                .apply_costs_slippage_and_fees
                .unwrap_or(true),
            protocol_fee_bps: additional_params.protocol_fee_bps,
        },
        params,
        &app_data.app_data_keccak256,
    )
}

/// Resolves the limit-order app-data and the limit-specific posting defaults
/// from the caller's parameters and advanced settings.
///
/// Runs the pre-submission steps [`post_limit_order`] applies before building the
/// order: it folds quote-request and app-data overrides onto the parameters,
/// defaults the limit slippage to `0` basis points, builds the app-data document
/// under [`OrderClass::Limit`], and defaults `apply_costs_slippage_and_fees` to
/// `false` so the signed amounts are the raw limit amounts. Both
/// [`post_limit_order`] and [`build_limit_order_to_sign`] call it, so the limit
/// defaults live in one place.
async fn resolve_limit_order_build_inputs(
    params: &LimitTradeParams,
    trader: &TraderParams,
    advanced_settings: Option<&TradeAdvancedSettings>,
) -> Result<
    (
        LimitTradeParams,
        TradingAppDataInfo,
        crate::types::PostTradeAdditionalParams,
    ),
    TradingError,
> {
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
        OrderClass::Limit,
        params.partner_fee.as_ref(),
        advanced_settings.and_then(|settings| settings.app_data.as_ref()),
    )
    .await?;

    let mut additional = advanced_additional_params(advanced_settings);
    if additional.apply_costs_slippage_and_fees.is_none() {
        additional.apply_costs_slippage_and_fees = Some(false);
    }

    Ok((params, app_data_info, additional))
}

/// Builds the unsigned limit order (`order_to_sign`) and its app-data exactly as
/// [`post_limit_order`] does before posting, without contacting a signer or the
/// orderbook.
///
/// `owner` is injected as the order owner (the same assignment the placement path
/// makes), so `from == owner` and the receiver default resolve identically to the
/// posting path. The returned [`OrderData`](cow_sdk_core::OrderData) is the digest
/// a smart account signs for an EIP-1271 limit order: a caller that resolves a
/// contract signature against it can hand the resolved blob to
/// [`place_limit`](crate::place_limit) with [`Authorization::Eip1271`](crate::Authorization::Eip1271),
/// which rebuilds the same order and echoes the resolved signature. Pair it with
/// [`cow_sdk_signing::order_typed_data`] for the EIP-712 payload to present to the
/// wallet.
///
/// This shares [`resolve_limit_order_build_inputs`] and [`build_order_to_sign`]
/// with [`post_limit_order`], so the produced order is byte-identical to the one
/// the posting path signs for the same inputs.
///
/// # Errors
///
/// Returns [`TradingError`] when app-data generation, amount calculation, or
/// order construction fails — the same failures [`post_limit_order`] surfaces
/// before posting.
pub async fn build_limit_order_to_sign(
    params: &LimitTradeParams,
    owner: Address,
    trader: &TraderParams,
    advanced_settings: Option<&TradeAdvancedSettings>,
) -> Result<(cow_sdk_core::OrderData, TradingAppDataInfo), TradingError> {
    let mut params = params.clone();
    params.owner = Some(owner);
    let (params, app_data_info, additional) =
        resolve_limit_order_build_inputs(&params, trader, advanced_settings).await?;
    let order = build_order_to_sign(trader.chain_id, owner, &additional, &params, &app_data_info)?;
    Ok((order, app_data_info))
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
    S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
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
    S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
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
    // Default the protocol fee from the quote response so the posted order signs
    // the same amounts the quote previewed (ADR 0058); an explicit caller value
    // wins, mirroring the reviewed upstream from-quote posting flow.
    let protocol_fee_bps = additional.protocol_fee_bps.or_else(|| {
        sanitize_protocol_fee_bps(quote_results.quote_response.protocol_fee_bps.as_deref())
    });
    let additional_params = crate::types::PostTradeAdditionalParams {
        signing_scheme: advanced_settings
            .and_then(|settings| settings.quote_request.as_ref())
            .and_then(|request| request.signing_scheme),
        network_costs_amount: Some(*quote_results.quote_response.quote.network_cost_amount()),
        protocol_fee_bps,
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

/// Posts a swap order from quote results under the pre-sign scheme without
/// consulting a signer.
///
/// The swap counterpart of [`post_limit_order_presign`]: the order is submitted
/// with [`SigningScheme::PreSign`] and only becomes fillable once the owner sets
/// the on-chain pre-signature flag via `setPreSignature` on the settlement
/// contract — for example by submitting the transaction built by
/// [`crate::pre_sign_transaction`], or the bundled
/// [`crate::build_presign_activation`]. This is the smart-contract-owner path:
/// Safes and other smart accounts place the order off-chain first and approve it
/// on-chain from the contract itself.
///
/// Because no signer participates, the owner must be explicit on the quote's
/// trade parameters (or an advanced-settings `quote_request.from` override). Any
/// signing-scheme override carried in `advanced_settings` is superseded by
/// [`SigningScheme::PreSign`].
///
/// # Errors
///
/// Returns [`TradingError::MissingSubmissionOwner`] when no explicit owner is
/// resolvable, and otherwise the same binding, app-data, signing-dispatch, and
/// submission errors as [`post_swap_order_from_quote`].
pub async fn post_swap_order_presign<O>(
    quote_results: &QuoteResults,
    trader: &TraderParams,
    advanced_settings: Option<&TradeAdvancedSettings>,
    orderbook: &O,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
{
    let mut settings = advanced_settings.cloned().unwrap_or_default();
    settings.quote_request = Some(
        settings
            .quote_request
            .take()
            .unwrap_or_default()
            .with_signing_scheme(SigningScheme::PreSign),
    );

    post_swap_order_from_quote(
        quote_results,
        trader,
        &PreSignPlacementSigner,
        Some(&settings),
        orderbook,
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
    S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
{
    let app_data_signer = advanced_settings
        .and_then(|settings| settings.app_data.as_ref())
        .and_then(|params| params.signer);

    let (params, app_data_info, additional) =
        resolve_limit_order_build_inputs(params, trader, advanced_settings).await?;

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

/// Posts a limit order under the pre-sign scheme without consulting a signer.
///
/// Pre-sign placements carry no cryptographic signature: the order is
/// submitted with [`SigningScheme::PreSign`] (the wire `signature` field
/// carries the owner address, mirroring the reviewed upstream SDK) and only
/// becomes fillable once the owner sets the on-chain pre-signature flag via
/// `setPreSignature` on the settlement contract — for example by submitting
/// the transaction built by [`crate::pre_sign_transaction`]. This is the
/// smart-contract-owner path: Safes and other smart accounts place the order
/// off-chain first and approve it on-chain from the contract itself.
///
/// Because no signer participates, the owner must be explicit:
/// [`LimitTradeParams::owner`] (or an advanced-settings `quote_request.from`
/// override) is required. Any signing-scheme override carried in
/// `advanced_settings` is superseded by [`SigningScheme::PreSign`].
///
/// # Errors
///
/// Returns [`TradingError::MissingSubmissionOwner`] when no explicit owner is
/// supplied, [`TradingError::InvalidInput`] for native-currency sell orders
/// (`EthFlow` orders are created on-chain and need a signer-backed entry),
/// and otherwise the same app-data, validation, and submission errors as
/// [`post_limit_order`].
pub async fn post_limit_order_presign<O>(
    params: &LimitTradeParams,
    trader: &TraderParams,
    advanced_settings: Option<&TradeAdvancedSettings>,
    orderbook: &O,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
{
    let effective = apply_settings_to_limit_trade_parameters(
        params,
        advanced_settings.and_then(|settings| settings.quote_request.as_ref()),
        advanced_settings.and_then(|settings| settings.app_data.as_ref()),
    )?;
    if effective.owner.is_none() {
        return Err(TradingError::MissingSubmissionOwner);
    }
    if is_eth_flow_order(&effective.sell_token) {
        return Err(TradingError::InvalidInput {
            field: "sellToken",
            reason: cow_sdk_core::ValidationReason::Precondition {
                details: "native-currency sell orders are created on-chain through the EthFlow \
                          contract and need a signer-backed posting entry, not a pre-sign placement",
            },
        });
    }

    let mut settings = advanced_settings.cloned().unwrap_or_default();
    settings.additional_params = Some(
        settings
            .additional_params
            .take()
            .unwrap_or_default()
            .with_signing_scheme(SigningScheme::PreSign),
    );

    post_limit_order(
        params,
        trader,
        &PreSignPlacementSigner,
        Some(&settings),
        orderbook,
    )
    .await
}

/// Signer stand-in threaded through the signer-less pre-sign posting entry.
///
/// Pre-sign placements never consult a signer: the owner is explicit by
/// construction and the pre-sign arm of the signing dispatch derives the
/// submission payload from that owner. Every operation therefore fails with a
/// description of the unexpected call, so a pipeline change that starts
/// consulting the signer on this path surfaces loudly instead of fabricating
/// a signature.
struct PreSignPlacementSigner;

impl PreSignPlacementSigner {
    fn unreachable_operation(operation: &str) -> String {
        format!("pre-sign posting must not consult a signer ({operation})")
    }
}

impl Signer for PreSignPlacementSigner {
    type Error = String;

    async fn address(&self) -> Result<Address, Self::Error> {
        Err(Self::unreachable_operation("address"))
    }

    async fn sign_message(&self, _message: &[u8]) -> Result<String, Self::Error> {
        Err(Self::unreachable_operation("sign_message"))
    }

    async fn sign_typed_data_payload(
        &self,
        _payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        Err(Self::unreachable_operation("sign_typed_data_payload"))
    }

    async fn send_transaction(
        &self,
        _tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        Err(Self::unreachable_operation("send_transaction"))
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Err(Self::unreachable_operation("estimate_gas"))
    }
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
    S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
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

    let transaction = TransactionRequest::from(tx.transaction);
    let broadcast = signer
        .send_transaction(&transaction)
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
    S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
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
