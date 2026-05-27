use cow_sdk_core::{Address, ProtocolOptions, Signer};
use cow_sdk_orderbook::{OrderCreation, SigningScheme};
use cow_sdk_signing::{
    SigningScheme as SigningSchemeContract, eip1271_signature_payload, sign_order,
    sign_order_with_scheme,
};

use super::native::post_sell_native_currency_order;
use crate::types::{
    QuoteRequestParameterTargets, apply_app_data_parameter_overrides,
    apply_quote_request_parameter_overrides, validate_orderbook_context,
    validate_orderbook_env_context,
};
use crate::validation::OrderBoundsValidator;
use crate::{
    LimitTradeParameters, LimitTradeParametersFromQuote, OrderPostingResult, OrderbookClient,
    TradeAdvancedSettings, TraderParameters, TradingAppDataInfo, TradingError,
    adjust_ethflow_limit_parameters, get_order_to_sign, is_ethflow_order,
};

fn build_order_body(
    order_to_sign: &cow_sdk_core::UnsignedOrder,
    app_data: &TradingAppDataInfo,
    scheme: SigningScheme,
    signature: String,
    from: Address,
    params: &LimitTradeParameters,
) -> OrderCreation {
    let mut order_body = OrderCreation::new(
        order_to_sign.sell_token,
        order_to_sign.buy_token,
        order_to_sign.sell_amount,
        order_to_sign.buy_amount,
        order_to_sign.valid_to,
        order_to_sign.kind,
        scheme,
        signature,
        from,
    )
    .with_receiver(order_to_sign.receiver)
    .with_app_data(app_data.full_app_data.clone())
    .with_app_data_hash(app_data.app_data_keccak256)
    .with_partially_fillable(order_to_sign.partially_fillable)
    .with_sell_token_balance(order_to_sign.sell_token_balance)
    .with_buy_token_balance(order_to_sign.buy_token_balance);
    if let Some(quote_id) = params.quote_id {
        order_body = order_body.with_quote_id(quote_id);
    }
    order_body
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
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
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
        let adjusted = adjust_ethflow_limit_parameters(canonical_chain_id, &params);
        let from_quote = LimitTradeParametersFromQuote::try_from_limit(adjusted)?;
        return post_sell_native_currency_order(
            orderbook,
            app_data,
            &from_quote,
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
    let order_to_sign = get_order_to_sign(
        crate::order::OrderToSignParams {
            chain_id,
            from,
            is_ethflow: false,
            network_costs_amount: additional_params.network_costs_amount,
            apply_costs_slippage_and_fees: additional_params
                .apply_costs_slippage_and_fees
                .unwrap_or(true),
            protocol_fee_bps: None,
        },
        &params,
        &app_data.app_data_keccak256,
    )?;

    let preview = build_order_body(
        &order_to_sign,
        app_data,
        requested_scheme,
        String::new(),
        from,
        &params,
    );
    let validator =
        OrderBoundsValidator::new(order_bounds, crate::validation::SubmissionClass::Limit)
            .with_weth_address(wrapped_native_address(chain_id));
    validator
        .validate(
            &preview,
            requested_scheme,
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

pub(super) fn wrapped_native_address(chain_id: cow_sdk_core::SupportedChainId) -> Address {
    cow_sdk_core::wrapped_native_token(chain_id).address
}

pub(super) fn apply_settings_to_limit_trade_parameters(
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

pub(super) fn advanced_additional_params(
    advanced_settings: Option<&TradeAdvancedSettings>,
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
