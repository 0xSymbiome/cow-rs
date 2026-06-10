use cow_sdk_core::{ProtocolOptions, Signer};
use cow_sdk_orderbook::{OrderQuoteRequest, OrderQuoteSide, PriceQuality, SigningScheme};
use cow_sdk_signing::order_typed_data;

pub use crate::app_data::{build_app_data, merge_and_seal_app_data, params_from_doc};
use crate::types::{
    QuoteRequestParameterTargets, apply_app_data_parameter_overrides,
    apply_quote_request_parameter_overrides, validate_orderbook_context,
    validate_orderbook_env_context,
};
use crate::{
    DEFAULT_QUOTE_VALIDITY, OrderbookClient, OrderbookRuntimeBinding, QuoteRequestOverride,
    QuoteResults, QuoterParams, TradeAdvancedSettings, TradeParams, TraderParams,
    TradingAppDataInfo, TradingError, adjust_eth_flow_trade_params,
    calculate_quote_amounts_and_costs, default_slippage_bps, is_eth_flow_order, order_to_sign,
    partner_fee_bps, resolve_slippage_suggestion, sanitize_protocol_fee_bps,
};

/// Builds a quote and signing payload without requiring a signer.
///
/// This path is intended for quote-only workflows. The effective owner is resolved from
/// `trade_parameters.owner` first and otherwise falls back to `trader.account`. Advanced settings
/// override overlapping trade fields before the request is assembled.
///
/// # Errors
///
/// Returns an error when quote validity inputs conflict, when app-data generation fails, when the
/// orderbook quote request fails, or when the derived signing payload cannot be constructed.
pub async fn quote_only<O>(
    trade_parameters: &TradeParams,
    trader: &QuoterParams,
    advanced_settings: Option<&TradeAdvancedSettings>,
    orderbook: &O,
) -> Result<QuoteResults, TradingError>
where
    O: OrderbookClient + ?Sized,
{
    let mut effective_trade_parameters =
        apply_advanced_settings_to_trade_parameters(trade_parameters, advanced_settings)?;
    let account = effective_trade_parameters.owner.unwrap_or(trader.account);
    effective_trade_parameters.owner = Some(account);
    let mut effective_trader = trader.clone();
    effective_trader.account = account;

    get_quote_internal(
        &effective_trade_parameters,
        &effective_trader,
        advanced_settings,
        orderbook,
    )
    .await
}

/// Builds quote results, the entry point for the unmanaged "quote then sign"
/// path.
///
/// The returned [`QuoteResults`] carries the projected
/// `order_to_sign` (the signable order derived from the orderbook `/quote`
/// response and the configured slippage, fees, and validity), the
/// `amounts_and_costs` breakdown, and the resolved app-data. A caller that wants
/// to sign and submit the order itself reads `order_to_sign`, computes its UID,
/// signs it, and posts it — no managed submission is performed here. The managed
/// one-call path is `post_swap_order`.
///
/// `trade_parameters.owner` takes precedence. When it is absent, the signer
/// address becomes the effective owner. Advanced settings override overlapping
/// trade fields before quote construction.
///
/// # Errors
///
/// Returns an error when signer address resolution fails, when quote validity inputs conflict,
/// when app-data generation fails, when the orderbook quote request fails, or when the derived
/// signing payload cannot be constructed.
pub async fn quote_results<O, S>(
    trade_parameters: &TradeParams,
    trader: &TraderParams,
    signer: &S,
    advanced_settings: Option<&TradeAdvancedSettings>,
    orderbook: &O,
) -> Result<QuoteResults, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    let mut effective_trade_parameters =
        apply_advanced_settings_to_trade_parameters(trade_parameters, advanced_settings)?;
    let account = match effective_trade_parameters.owner {
        Some(owner) => owner,
        None => signer
            .address()
            .await
            .map_err(|error| TradingError::Signer {
                operation: "address",
                message: error.to_string().into(),
            })?,
    };
    effective_trade_parameters.owner = Some(account);
    let quoter = QuoterParams {
        chain_id: trader.chain_id,
        app_code: trader.app_code.clone(),
        account,
        env: trader.env,
        settlement_contract_override: trader.settlement_contract_override.clone(),
        eth_flow_contract_override: trader.eth_flow_contract_override.clone(),
    };

    get_quote_internal(
        &effective_trade_parameters,
        &quoter,
        advanced_settings,
        orderbook,
    )
    .await
}

async fn get_quote_internal<O>(
    trade_parameters: &TradeParams,
    trader: &QuoterParams,
    advanced_settings: Option<&TradeAdvancedSettings>,
    orderbook: &O,
) -> Result<QuoteResults, TradingError>
where
    O: OrderbookClient + ?Sized,
{
    if trade_parameters.valid_for.is_some() && trade_parameters.valid_to.is_some() {
        return Err(TradingError::QuoteValidityConflict);
    }

    validate_orderbook_context(orderbook, Some(trader.chain_id), trader.env)?;
    validate_orderbook_env_context(orderbook, trade_parameters.env)?;

    let orderbook_context = orderbook.context();
    let canonical_chain_id = orderbook_context.chain_id;
    let canonical_env = orderbook_context.env;
    let mut effective_trade_parameters = trade_parameters.clone();
    effective_trade_parameters.env = Some(canonical_env);
    let mut effective_trader = trader.clone();
    effective_trader.chain_id = canonical_chain_id;
    effective_trader.env = Some(canonical_env);

    let is_eth_flow = is_eth_flow_order(&effective_trade_parameters.sell_token);
    let trade_parameters_for_quote = if is_eth_flow {
        adjust_eth_flow_trade_params(canonical_chain_id, &effective_trade_parameters)
    } else {
        effective_trade_parameters.clone()
    };
    let default_slippage = default_slippage_bps(canonical_chain_id, is_eth_flow);
    let initial_slippage = trade_parameters.slippage_bps.unwrap_or(default_slippage);
    let initial_app_data = build_app_data(
        &effective_trader.app_code,
        initial_slippage,
        "market",
        effective_trade_parameters.partner_fee.as_ref(),
        advanced_settings.and_then(|settings| settings.app_data.as_ref()),
    )
    .await?;

    let request = build_quote_request(
        &trade_parameters_for_quote,
        &effective_trader,
        is_eth_flow,
        &initial_app_data,
        advanced_settings.and_then(|settings| settings.quote_request.as_ref()),
    )?;
    let quote_response = orderbook.quote(&request).await?;
    let suggested_slippage = resolve_slippage_suggestion(
        canonical_chain_id,
        &trade_parameters_for_quote,
        &effective_trader,
        &quote_response,
        is_eth_flow,
        advanced_settings,
    )
    .await?
    .slippage_bps
    .unwrap_or(default_slippage);

    let mut updated_parameters = effective_trade_parameters.clone();
    let (trade_parameters, app_data_info) = if effective_trade_parameters.slippage_bps.is_none()
        && suggested_slippage != initial_slippage
    {
        updated_parameters.slippage_bps = Some(suggested_slippage);
        let app_data = build_app_data(
            &effective_trader.app_code,
            suggested_slippage,
            "market",
            effective_trade_parameters.partner_fee.as_ref(),
            advanced_settings.and_then(|settings| settings.app_data.as_ref()),
        )
        .await?;
        (updated_parameters, app_data)
    } else {
        updated_parameters.slippage_bps = Some(initial_slippage);
        (updated_parameters, initial_app_data)
    };

    let amounts_and_costs = calculate_quote_amounts_and_costs(
        &quote_response.quote,
        trade_parameters
            .slippage_bps
            .unwrap_or_else(|| default_slippage_bps(canonical_chain_id, is_eth_flow)),
        partner_fee_bps(trade_parameters.partner_fee.as_ref()),
        sanitize_protocol_fee_bps(quote_response.protocol_fee_bps.as_deref()),
    )?;
    build_quote_results(QuoteResultInputs {
        trader: &effective_trader,
        trade_parameters,
        quote_response,
        app_data_info,
        orderbook_binding: orderbook.runtime_binding(),
        suggested_slippage,
        amounts_and_costs,
        is_eth_flow,
        resolved_env: canonical_env,
    })
}

struct QuoteResultInputs<'a> {
    trader: &'a QuoterParams,
    trade_parameters: TradeParams,
    quote_response: cow_sdk_orderbook::OrderQuoteResponse,
    app_data_info: TradingAppDataInfo,
    orderbook_binding: OrderbookRuntimeBinding,
    suggested_slippage: u32,
    amounts_and_costs: cow_sdk_core::QuoteAmountsAndCosts,
    is_eth_flow: bool,
    resolved_env: cow_sdk_orderbook::CowEnv,
}

fn build_quote_results(inputs: QuoteResultInputs<'_>) -> Result<QuoteResults, TradingError> {
    let mut options = ProtocolOptions::new().with_env(inputs.resolved_env);
    if let Some(overrides) = inputs
        .trade_parameters
        .settlement_contract_override
        .clone()
        .or_else(|| inputs.trader.settlement_contract_override.clone())
    {
        options = options.with_settlement_contract_override(overrides);
    }
    if let Some(overrides) = inputs
        .trade_parameters
        .eth_flow_contract_override
        .clone()
        .or_else(|| inputs.trader.eth_flow_contract_override.clone())
    {
        options = options.with_eth_flow_contract_override(overrides);
    }
    let order_to_sign = order_to_sign(
        crate::order::OrderToSignParams {
            chain_id: inputs.trader.chain_id,
            from: inputs.trader.account,
            is_eth_flow: inputs.is_eth_flow,
            network_costs_amount: Some(*inputs.quote_response.quote.network_cost_amount()),
            apply_costs_slippage_and_fees: true,
            protocol_fee_bps: sanitize_protocol_fee_bps(
                inputs.quote_response.protocol_fee_bps.as_deref(),
            ),
        },
        crate::swap_params_to_limit_order_params(&inputs.trade_parameters, &inputs.quote_response)?
            .as_limit(),
        &inputs.app_data_info.app_data_keccak256,
    )?;
    let order_typed_data =
        order_typed_data(inputs.trader.chain_id, &order_to_sign, Some(&options))?;

    Ok(QuoteResults {
        trade_parameters: inputs.trade_parameters,
        suggested_slippage_bps: inputs.suggested_slippage,
        amounts_and_costs: inputs.amounts_and_costs,
        order_to_sign,
        quote_response: inputs.quote_response,
        app_data_info: inputs.app_data_info,
        orderbook_binding: Some(inputs.orderbook_binding),
        order_typed_data,
    })
}

pub(crate) fn apply_advanced_settings_to_trade_parameters(
    trade_parameters: &TradeParams,
    advanced_settings: Option<&TradeAdvancedSettings>,
) -> Result<TradeParams, TradingError> {
    let mut trade_parameters = trade_parameters.clone();

    apply_app_data_parameter_overrides(
        &mut trade_parameters.slippage_bps,
        &mut trade_parameters.partner_fee,
        advanced_settings.and_then(|settings| settings.app_data.as_ref()),
    )?;
    apply_quote_request_parameter_overrides(
        &mut QuoteRequestParameterTargets {
            owner: &mut trade_parameters.owner,
            sell_token: &mut trade_parameters.sell_token,
            buy_token: &mut trade_parameters.buy_token,
            receiver: &mut trade_parameters.receiver,
            valid_for: &mut trade_parameters.valid_for,
            valid_to: &mut trade_parameters.valid_to,
            partially_fillable: &mut trade_parameters.partially_fillable,
            sell_token_balance: &mut trade_parameters.sell_token_balance,
            buy_token_balance: &mut trade_parameters.buy_token_balance,
        },
        advanced_settings.and_then(|settings| settings.quote_request.as_ref()),
    );

    Ok(trade_parameters)
}

fn build_quote_request(
    trade_parameters: &TradeParams,
    trader: &QuoterParams,
    is_eth_flow: bool,
    app_data_info: &TradingAppDataInfo,
    request_override: Option<&QuoteRequestOverride>,
) -> Result<OrderQuoteRequest, TradingError> {
    let receiver = trade_parameters.receiver.unwrap_or(trader.account);
    let side = match trade_parameters.kind {
        cow_sdk_core::OrderKind::Sell => OrderQuoteSide::sell(trade_parameters.amount),
        cow_sdk_core::OrderKind::Buy => OrderQuoteSide::buy(trade_parameters.amount),
    };
    let mut request = OrderQuoteRequest::new(
        trade_parameters.sell_token,
        trade_parameters.buy_token,
        trader.account,
        side,
    )
    .with_receiver(receiver)
    .with_app_data(app_data_info.full_app_data.clone())
    .with_app_data_hash(app_data_info.app_data_keccak256)
    .with_price_quality(PriceQuality::Optimal);

    if trade_parameters.partially_fillable {
        request = request.with_partially_fillable();
    }
    request.sell_token_balance = trade_parameters.sell_token_balance;
    request.buy_token_balance = trade_parameters.buy_token_balance;

    if let Some(valid_to) = trade_parameters.valid_to {
        request = request.with_valid_to(valid_to);
    } else {
        request =
            request.with_valid_for(trade_parameters.valid_for.unwrap_or(DEFAULT_QUOTE_VALIDITY));
    }

    if is_eth_flow {
        request = request
            .with_signing_scheme(SigningScheme::Eip1271)
            .with_onchain_order()
            .with_verification_gas_limit(0);
    }

    apply_quote_request_override(&mut request, request_override)?;
    request.validate()?;

    Ok(request)
}

fn apply_quote_request_override(
    request: &mut OrderQuoteRequest,
    request_override: Option<&QuoteRequestOverride>,
) -> Result<(), TradingError> {
    let Some(request_override) = request_override else {
        return Ok(());
    };

    if let Some(sell_token) = &request_override.sell_token {
        request.sell_token = *sell_token;
    }
    if let Some(buy_token) = &request_override.buy_token {
        request.buy_token = *buy_token;
    }
    if let Some(receiver) = &request_override.receiver {
        request.receiver = Some(*receiver);
    }
    if let Some(valid_for) = request_override.valid_for {
        request.validity = cow_sdk_orderbook::QuoteValidity::ValidFor(valid_for);
    }
    if let Some(valid_to) = request_override.valid_to {
        request.validity = cow_sdk_orderbook::QuoteValidity::ValidTo(valid_to);
    }
    if let Some(from) = &request_override.from {
        request.from = *from;
    }
    if let Some(price_quality) = request_override.price_quality {
        request.price_quality = price_quality;
    }
    if request_override.signing_scheme.is_some()
        || request_override.onchain_order.is_some()
        || request_override.verification_gas_limit.is_some()
    {
        let base = request_override
            .signing_scheme
            .unwrap_or_else(|| request.signing_scheme.scheme());
        let onchain = request_override
            .onchain_order
            .unwrap_or_else(|| request.signing_scheme.is_onchain_order());
        let gas = request_override
            .verification_gas_limit
            .unwrap_or(match request.signing_scheme {
                cow_sdk_orderbook::QuoteSigningScheme::Eip1271 {
                    verification_gas_limit,
                    ..
                } => verification_gas_limit,
                _ => cow_sdk_orderbook::default_verification_gas_limit(),
            });
        // An ECDSA scheme can never be an on-chain order; reject an override
        // that pairs them rather than silently dropping the on-chain intent.
        if onchain
            && matches!(
                base,
                cow_sdk_orderbook::SigningScheme::Eip712
                    | cow_sdk_orderbook::SigningScheme::EthSign
            )
        {
            return Err(TradingError::Orderbook(
                cow_sdk_orderbook::OrderbookError::IncompatibleSigningScheme {
                    signing_scheme: base,
                    onchain_order: true,
                },
            ));
        }
        request.signing_scheme = match base {
            cow_sdk_orderbook::SigningScheme::Eip712 => {
                cow_sdk_orderbook::QuoteSigningScheme::Eip712
            }
            cow_sdk_orderbook::SigningScheme::EthSign => {
                cow_sdk_orderbook::QuoteSigningScheme::EthSign
            }
            cow_sdk_orderbook::SigningScheme::Eip1271 => {
                cow_sdk_orderbook::QuoteSigningScheme::Eip1271 {
                    onchain_order: onchain,
                    verification_gas_limit: gas,
                }
            }
            cow_sdk_orderbook::SigningScheme::PreSign => {
                cow_sdk_orderbook::QuoteSigningScheme::PreSign {
                    onchain_order: onchain,
                }
            }
            // `SigningScheme` is upstream-growing; an unrecognized scheme falls
            // back to the EIP-712 default rather than guessing on-chain intent.
            _ => cow_sdk_orderbook::QuoteSigningScheme::Eip712,
        };
    }
    if let Some(timeout) = request_override.timeout {
        request.timeout = Some(timeout);
    }
    if let Some(partially_fillable) = request_override.partially_fillable {
        request.partially_fillable = partially_fillable;
    }
    if let Some(balance) = request_override.sell_token_balance {
        request.sell_token_balance = balance;
    }
    if let Some(balance) = request_override.buy_token_balance {
        request.buy_token_balance = balance;
    }

    Ok(())
}
