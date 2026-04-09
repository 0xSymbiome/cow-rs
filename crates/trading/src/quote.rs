use serde_json::{Map, Value, json};

use cow_sdk_app_data::{AppDataParams, generate_app_data_doc, get_app_data_info};
use cow_sdk_core::{AsyncSigner, ProtocolOptions, Signer};
use cow_sdk_orderbook::{OrderQuoteRequest, PriceQuality, QuoteSide, SigningScheme};
use cow_sdk_signing::order_typed_data;

use crate::{
    DEFAULT_QUOTE_VALIDITY, OrderbookClient, QuoteRequestOverride, QuoteResults, QuoterParameters,
    SwapAdvancedSettings, TradeParameters, TraderParameters, TradingAppDataInfo, TradingError,
    adjust_ethflow_trade_parameters, calculate_quote_amounts_and_costs, default_slippage_bps,
    get_order_to_sign, is_ethflow_order, partner_fee_bps, resolve_slippage_suggestion,
    sanitize_protocol_fee_bps,
};

pub async fn get_quote_only<O>(
    trade_parameters: &TradeParameters,
    trader: &QuoterParameters,
    advanced_settings: Option<&SwapAdvancedSettings>,
    orderbook: &O,
) -> Result<QuoteResults, TradingError>
where
    O: OrderbookClient + ?Sized,
{
    get_quote_internal(trade_parameters, trader, advanced_settings, orderbook).await
}

pub async fn get_quote_results<O, S>(
    trade_parameters: &TradeParameters,
    trader: &TraderParameters,
    signer: &S,
    advanced_settings: Option<&SwapAdvancedSettings>,
    orderbook: &O,
) -> Result<QuoteResults, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display,
{
    get_quote_results_async(
        trade_parameters,
        trader,
        signer,
        advanced_settings,
        orderbook,
    )
    .await
}

pub async fn get_quote_results_async<O, S>(
    trade_parameters: &TradeParameters,
    trader: &TraderParameters,
    signer: &S,
    advanced_settings: Option<&SwapAdvancedSettings>,
    orderbook: &O,
) -> Result<QuoteResults, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: AsyncSigner,
    S::Error: std::fmt::Display,
{
    let account = match trade_parameters.owner.clone() {
        Some(owner) => owner,
        None => signer
            .get_address()
            .await
            .map_err(|error| TradingError::Signer {
                operation: "get_address",
                message: error.to_string(),
            })?,
    };
    let quoter = QuoterParameters {
        chain_id: trader.chain_id,
        app_code: trader.app_code.clone(),
        account,
        env: trader.env,
        settlement_contract_override: trader.settlement_contract_override.clone(),
        eth_flow_contract_override: trader.eth_flow_contract_override.clone(),
    };

    get_quote_internal(trade_parameters, &quoter, advanced_settings, orderbook).await
}

pub async fn build_app_data(
    app_code: &str,
    slippage_bps: u32,
    order_class: &str,
    partner_fee: Option<&Value>,
    advanced_params: Option<&AppDataParams>,
) -> Result<TradingAppDataInfo, TradingError> {
    let mut metadata = Map::new();
    metadata.insert("quote".to_owned(), json!({ "slippageBips": slippage_bps }));
    metadata.insert(
        "orderClass".to_owned(),
        json!({ "orderClass": order_class }),
    );
    if let Some(partner_fee) = partner_fee {
        metadata.insert("partnerFee".to_owned(), partner_fee.clone());
    }

    let mut params = AppDataParams {
        app_code: Some(app_code.to_owned()),
        environment: None,
        metadata,
    };
    if let Some(advanced_params) = advanced_params {
        params = merge_app_data_params(&params, advanced_params);
    }

    let doc = generate_app_data_doc(params);
    let info = get_app_data_info(doc.clone())?;

    Ok(TradingAppDataInfo {
        doc,
        full_app_data: info.app_data_content,
        app_data_keccak256: cow_sdk_core::AppDataHash::new(info.app_data_hex)?,
    })
}

pub fn merge_app_data_doc(
    base_doc: &Value,
    app_data_override: &AppDataParams,
) -> Result<TradingAppDataInfo, TradingError> {
    let mut merged = base_doc.clone();
    if let Some(app_code) = &app_data_override.app_code {
        merged["appCode"] = Value::String(app_code.clone());
    }
    if let Some(environment) = &app_data_override.environment {
        merged["environment"] = Value::String(environment.clone());
    }
    merged["metadata"] = deep_merge_values(
        merged
            .get("metadata")
            .cloned()
            .unwrap_or_else(|| Value::Object(Map::new())),
        Value::Object(app_data_override.metadata.clone()),
    );

    let info = get_app_data_info(merged.clone())?;

    Ok(TradingAppDataInfo {
        doc: merged,
        full_app_data: info.app_data_content,
        app_data_keccak256: cow_sdk_core::AppDataHash::new(info.app_data_hex)?,
    })
}

async fn get_quote_internal<O>(
    trade_parameters: &TradeParameters,
    trader: &QuoterParameters,
    advanced_settings: Option<&SwapAdvancedSettings>,
    orderbook: &O,
) -> Result<QuoteResults, TradingError>
where
    O: OrderbookClient + ?Sized,
{
    if trade_parameters.valid_for.is_some() && trade_parameters.valid_to.is_some() {
        return Err(TradingError::QuoteValidityConflict);
    }

    let is_ethflow = is_ethflow_order(&trade_parameters.sell_token);
    let trade_parameters_for_quote = if is_ethflow {
        adjust_ethflow_trade_parameters(trader.chain_id, trade_parameters)
    } else {
        trade_parameters.clone()
    };
    let default_slippage = default_slippage_bps(trader.chain_id, is_ethflow);
    let initial_slippage = trade_parameters.slippage_bps.unwrap_or(default_slippage);
    let initial_app_data = build_app_data(
        &trader.app_code,
        initial_slippage,
        "market",
        trade_parameters.partner_fee.as_ref(),
        advanced_settings.and_then(|settings| settings.app_data.as_ref()),
    )
    .await?;

    let request = build_quote_request(
        &trade_parameters_for_quote,
        trader,
        is_ethflow,
        &initial_app_data,
        advanced_settings.and_then(|settings| settings.quote_request.as_ref()),
    )?;
    let quote_response = orderbook.get_quote(&request).await?;
    let suggested_slippage = resolve_slippage_suggestion(
        trader.chain_id,
        &trade_parameters_for_quote,
        trader,
        &quote_response,
        is_ethflow,
        advanced_settings,
    )
    .await?
    .slippage_bps
    .unwrap_or(default_slippage);

    let (trade_parameters, app_data_info) =
        if trade_parameters.slippage_bps.is_none() && suggested_slippage != initial_slippage {
            let mut updated = trade_parameters.clone();
            updated.slippage_bps = Some(suggested_slippage);
            let app_data = build_app_data(
                &trader.app_code,
                suggested_slippage,
                "market",
                trade_parameters.partner_fee.as_ref(),
                advanced_settings.and_then(|settings| settings.app_data.as_ref()),
            )
            .await?;
            (updated, app_data)
        } else {
            let mut updated = trade_parameters.clone();
            updated.slippage_bps = Some(initial_slippage);
            (updated, initial_app_data)
        };

    let amounts_and_costs = calculate_quote_amounts_and_costs(
        &quote_response.quote,
        trade_parameters
            .slippage_bps
            .unwrap_or_else(|| default_slippage_bps(trader.chain_id, is_ethflow)),
        partner_fee_bps(trade_parameters.partner_fee.as_ref()),
        sanitize_protocol_fee_bps(quote_response.protocol_fee_bps.as_deref()),
    )?;
    let options = ProtocolOptions {
        env: trade_parameters.env.or(trader.env),
        settlement_contract_override: trade_parameters
            .settlement_contract_override
            .clone()
            .or_else(|| trader.settlement_contract_override.clone()),
        eth_flow_contract_override: trade_parameters
            .eth_flow_contract_override
            .clone()
            .or_else(|| trader.eth_flow_contract_override.clone()),
    };
    let order_to_sign = get_order_to_sign(
        crate::order::OrderToSignParams {
            chain_id: trader.chain_id,
            from: trader.account.clone(),
            is_ethflow,
            network_costs_amount: Some(quote_response.quote.fee_amount.clone()),
            apply_costs_slippage_and_fees: true,
            protocol_fee_bps: sanitize_protocol_fee_bps(quote_response.protocol_fee_bps.as_deref()),
        },
        &crate::swap_params_to_limit_order_params(&trade_parameters, &quote_response),
        &app_data_info.app_data_keccak256,
    )?;
    let order_typed_data = order_typed_data(trader.chain_id, &order_to_sign, Some(&options))?;

    Ok(QuoteResults {
        trade_parameters,
        suggested_slippage_bps: suggested_slippage,
        amounts_and_costs,
        order_to_sign,
        quote_response,
        app_data_info,
        order_typed_data,
    })
}

fn build_quote_request(
    trade_parameters: &TradeParameters,
    trader: &QuoterParameters,
    is_ethflow: bool,
    app_data_info: &TradingAppDataInfo,
    request_override: Option<&QuoteRequestOverride>,
) -> Result<OrderQuoteRequest, TradingError> {
    let receiver = trade_parameters
        .receiver
        .clone()
        .unwrap_or_else(|| trader.account.clone());
    let side = match trade_parameters.kind {
        cow_sdk_core::OrderKind::Sell => QuoteSide::sell(trade_parameters.amount.clone()),
        cow_sdk_core::OrderKind::Buy => QuoteSide::buy(trade_parameters.amount.clone()),
    };
    let mut request = OrderQuoteRequest::new(
        trade_parameters.sell_token.clone(),
        trade_parameters.buy_token.clone(),
        trader.account.clone(),
        side,
    )
    .with_receiver(receiver)
    .with_app_data(app_data_info.full_app_data.clone())
    .with_app_data_hash(app_data_info.app_data_keccak256.clone())
    .with_price_quality(PriceQuality::Optimal);

    if let Some(valid_to) = trade_parameters.valid_to {
        request = request.with_valid_to(valid_to);
    } else {
        request =
            request.with_valid_for(trade_parameters.valid_for.unwrap_or(DEFAULT_QUOTE_VALIDITY));
    }

    if is_ethflow {
        request = request
            .with_signing_scheme(SigningScheme::Eip1271)
            .with_onchain_order()
            .with_verification_gas_limit(0);
    }

    apply_quote_request_override(&mut request, request_override);

    if request.valid_for.is_some() && request.valid_to.is_some() {
        return Err(TradingError::QuoteValidityConflict);
    }

    Ok(request)
}

fn apply_quote_request_override(
    request: &mut OrderQuoteRequest,
    request_override: Option<&QuoteRequestOverride>,
) {
    let Some(request_override) = request_override else {
        return;
    };

    if let Some(sell_token) = &request_override.sell_token {
        request.sell_token = sell_token.clone();
    }
    if let Some(buy_token) = &request_override.buy_token {
        request.buy_token = buy_token.clone();
    }
    if let Some(receiver) = &request_override.receiver {
        request.receiver = Some(receiver.clone());
    }
    if let Some(valid_for) = request_override.valid_for {
        request.valid_for = Some(valid_for);
        request.valid_to = None;
    }
    if let Some(valid_to) = request_override.valid_to {
        request.valid_to = Some(valid_to);
        request.valid_for = None;
    }
    if let Some(from) = &request_override.from {
        request.from = from.clone();
    }
    if let Some(price_quality) = request_override.price_quality {
        request.price_quality = price_quality;
    }
    if let Some(signing_scheme) = request_override.signing_scheme {
        request.signing_scheme = signing_scheme;
    }
    if let Some(onchain_order) = request_override.onchain_order {
        request.onchain_order = onchain_order;
    }
    if let Some(verification_gas_limit) = request_override.verification_gas_limit {
        request.verification_gas_limit = Some(verification_gas_limit);
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
}

fn merge_app_data_params(base: &AppDataParams, override_params: &AppDataParams) -> AppDataParams {
    let metadata = match deep_merge_values(
        Value::Object(base.metadata.clone()),
        Value::Object(override_params.metadata.clone()),
    ) {
        Value::Object(map) => map,
        _ => Map::new(),
    };

    AppDataParams {
        app_code: override_params
            .app_code
            .clone()
            .or_else(|| base.app_code.clone()),
        environment: override_params
            .environment
            .clone()
            .or_else(|| base.environment.clone()),
        metadata,
    }
}

fn deep_merge_values(base: Value, override_value: Value) -> Value {
    match (base, override_value) {
        (Value::Object(mut base), Value::Object(override_map)) => {
            for (key, value) in override_map {
                let merged = deep_merge_values(base.remove(&key).unwrap_or(Value::Null), value);
                base.insert(key, merged);
            }
            Value::Object(base)
        }
        (_, value) => value,
    }
}
