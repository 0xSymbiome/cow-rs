use serde_json::{Map, Value, json};

use cow_sdk_app_data::{AppDataParams, PartnerFee, generate_app_data_doc, get_app_data_info};
use cow_sdk_core::{Amount, AsyncSigner, ProtocolOptions, Signer};
use cow_sdk_orderbook::{OrderQuoteRequest, PriceQuality, QuoteSide, SigningScheme};
use cow_sdk_signing::order_typed_data;

use crate::types::{
    OrderbookRuntimeBinding, QuoteRequestParameterTargets, apply_app_data_parameter_overrides,
    apply_quote_request_parameter_overrides, validate_orderbook_context,
    validate_orderbook_env_context,
};
use crate::{
    DEFAULT_QUOTE_VALIDITY, OrderbookClient, QuoteRequestOverride, QuoteResults, QuoterParameters,
    SwapAdvancedSettings, TradeParameters, TraderParameters, TradingAppDataInfo, TradingError,
    adjust_ethflow_trade_parameters, calculate_quote_amounts_and_costs, default_slippage_bps,
    get_order_to_sign, is_ethflow_order, partner_fee_bps, resolve_slippage_suggestion,
    sanitize_protocol_fee_bps,
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
pub async fn get_quote_only<O>(
    trade_parameters: &TradeParameters,
    trader: &QuoterParameters,
    advanced_settings: Option<&SwapAdvancedSettings>,
    orderbook: &O,
) -> Result<QuoteResults, TradingError>
where
    O: OrderbookClient + ?Sized,
{
    let mut effective_trade_parameters =
        apply_advanced_settings_to_trade_parameters(trade_parameters, advanced_settings)?;
    let account = effective_trade_parameters
        .owner
        .clone()
        .unwrap_or_else(|| trader.account.clone());
    effective_trade_parameters.owner = Some(account.clone());
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

/// Builds quote results using a synchronous signer for owner resolution.
///
/// `trade_parameters.owner` takes precedence. When it is absent, the signer address becomes the
/// effective owner. Advanced settings override overlapping trade fields before quote construction.
///
/// # Errors
///
/// Returns an error when signer address resolution fails, when quote validity inputs conflict,
/// when app-data generation fails, when the orderbook quote request fails, or when the derived
/// signing payload cannot be constructed.
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

/// Builds quote results using an asynchronous signer for owner resolution.
///
/// `trade_parameters.owner` takes precedence. When it is absent, the signer address becomes the
/// effective owner. Advanced settings override overlapping trade fields before quote construction.
///
/// # Errors
///
/// Returns an error when signer address resolution fails, when quote validity inputs conflict,
/// when app-data generation fails, when the orderbook quote request fails, or when the derived
/// signing payload cannot be constructed.
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
    let mut effective_trade_parameters =
        apply_advanced_settings_to_trade_parameters(trade_parameters, advanced_settings)?;
    let account = match effective_trade_parameters.owner.clone() {
        Some(owner) => owner,
        None => signer
            .get_address()
            .await
            .map_err(|error| TradingError::Signer {
                operation: "get_address",
                message: error.to_string(),
            })?,
    };
    effective_trade_parameters.owner = Some(account.clone());
    let quoter = QuoterParameters {
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

/// `metadata.utm.utmSource` default stamped when the caller does not supply
/// an override `metadata.utm` block.
const UTM_SOURCE: &str = "cowmunity";

/// `metadata.utm.utmCampaign` default stamped when the caller does not supply
/// an override `metadata.utm` block.
const UTM_CAMPAIGN: &str = "developer-cohort";

/// `metadata.utm.utmTerm` default that identifies Rust-SDK traffic in
/// attribution analytics. Intentionally distinct from other SDK identifiers
/// so Rust-SDK adoption is not mislabelled.
const UTM_TERM: &str = "rs";

/// Builds the default `metadata.utm` block stamped on app-data documents
/// when the caller does not supply their own `metadata.utm`.
///
/// The block identifies the Rust SDK and its compile-time version so
/// protocol-side attribution analytics can distinguish Rust-SDK traffic
/// from other client SDKs. The `utmMedium` value embeds the trading
/// crate's published version through `env!("CARGO_PKG_VERSION")`.
fn default_utm() -> Value {
    json!({
        "utmSource": UTM_SOURCE,
        "utmMedium": format!("cow-rs@{}", env!("CARGO_PKG_VERSION")),
        "utmCampaign": UTM_CAMPAIGN,
        "utmContent": "",
        "utmTerm": UTM_TERM,
    })
}

/// Builds the trading app-data document and its derived hash.
///
/// The generated base document always includes quote slippage metadata and order class metadata.
/// When the caller does not supply `metadata.utm`, a Rust-identified default
/// UTM attribution block is stamped onto the base document so downstream
/// analytics can attribute the traffic to the Rust SDK. Any caller-supplied
/// `metadata.utm` — partial or full — disables the default stamp and is
/// carried through exactly as provided.
/// `advanced_params` then overrides `appCode`, `environment`, and metadata keys using a deep merge.
///
/// # Errors
///
/// Returns an error when the merged app-data document cannot be normalized into a valid app-data
/// payload or hash.
pub async fn build_app_data(
    app_code: &str,
    slippage_bps: u32,
    order_class: &str,
    partner_fee: Option<&PartnerFee>,
    advanced_params: Option<&AppDataParams>,
) -> Result<TradingAppDataInfo, TradingError> {
    let mut metadata = Map::new();
    metadata.insert("quote".to_owned(), json!({ "slippageBips": slippage_bps }));
    metadata.insert(
        "orderClass".to_owned(),
        json!({ "orderClass": order_class }),
    );
    if let Some(partner_fee) = partner_fee {
        metadata.insert("partnerFee".to_owned(), partner_fee.to_value());
    }

    let override_has_utm = advanced_params
        .and_then(|params| params.metadata.get("utm"))
        .is_some();
    if !override_has_utm {
        metadata.insert("utm".to_owned(), default_utm());
    }

    let mut params = AppDataParams::new(Some(app_code.to_owned()), None, None, None, metadata);
    if let Some(advanced_params) = advanced_params {
        params = merge_app_data_params(&params, advanced_params);
    }

    let doc = generate_app_data_doc(params);
    let info = get_app_data_info(doc.clone())?.info;

    Ok(TradingAppDataInfo {
        doc,
        full_app_data: info.app_data_content,
        app_data_keccak256: cow_sdk_core::AppDataHash::new(info.app_data_hex)?,
    })
}

/// Parses an already-sealed app-data wire document back into typed
/// [`AppDataParams`].
///
/// The existing [`AppDataParams`] deserializer lifts `metadata.signer` and
/// `metadata.flashloan` out of the wire shape into their typed fields so
/// the returned value is ready to drive the typed merge pipeline without
/// any additional coercion.
///
/// # Errors
///
/// Returns [`TradingError::AppData`] when the supplied document does not
/// conform to the [`AppDataParams`] wire shape — for example when
/// `metadata.signer` carries a value that is not a valid address, or when
/// `metadata.flashloan` carries an object that fails the typed flash-loan
/// hints validation.
pub fn params_from_doc(base_doc: &Value) -> Result<AppDataParams, TradingError> {
    serde_json::from_value::<AppDataParams>(base_doc.clone())
        .map_err(|error| TradingError::AppData(cow_sdk_app_data::AppDataError::from(error)))
}

/// Merges a typed [`AppDataParams`] override onto a previously-sealed
/// app-data wire document and re-emits the canonical wire form.
///
/// The base document is deserialized through the existing
/// [`AppDataParams`] deserializer so the typed `signer` and `flashloan`
/// fields on the base side participate in the merge on equal footing
/// with the override, and the resulting typed value drives
/// [`generate_app_data_doc`] and [`get_app_data_info`] to re-derive the
/// wire document and its digest from one authoritative typed shape.
///
/// The returned tuple carries both the [`TradingAppDataInfo`] (the
/// wire document, stringified content, and keccak256 hash) and the
/// typed merged [`AppDataParams`], so submission seams can read the
/// final `signer` field directly from the same merged value that
/// produced the wire document rather than re-reading the override.
///
/// The merge applies the reviewed hooks-replacement rule so
/// override-supplied `metadata.hooks` replace the base-side hooks
/// envelope in full instead of recursively merging pre/post
/// sibling arrays.
///
/// # Errors
///
/// Returns [`TradingError::AppData`] when the base document cannot be
/// parsed into typed [`AppDataParams`], or when the merged document
/// cannot be normalized into a valid app-data payload or hash.
pub fn merge_and_seal_app_data(
    base_doc: &Value,
    override_params: &AppDataParams,
) -> Result<(TradingAppDataInfo, AppDataParams), TradingError> {
    let base_params = params_from_doc(base_doc)?;
    let merged_params = merge_app_data_params(&base_params, override_params);
    let doc = generate_app_data_doc(merged_params.clone());
    let info = get_app_data_info(doc.clone())?.info;

    Ok((
        TradingAppDataInfo {
            doc,
            full_app_data: info.app_data_content,
            app_data_keccak256: cow_sdk_core::AppDataHash::new(info.app_data_hex)?,
        },
        merged_params,
    ))
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

    let is_ethflow = is_ethflow_order(&effective_trade_parameters.sell_token);
    let trade_parameters_for_quote = if is_ethflow {
        adjust_ethflow_trade_parameters(canonical_chain_id, &effective_trade_parameters)
    } else {
        effective_trade_parameters.clone()
    };
    let default_slippage = default_slippage_bps(canonical_chain_id, is_ethflow);
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
        is_ethflow,
        &initial_app_data,
        advanced_settings.and_then(|settings| settings.quote_request.as_ref()),
    )?;
    let quote_response = orderbook.get_quote(&request).await?;
    let suggested_slippage = resolve_slippage_suggestion(
        canonical_chain_id,
        &trade_parameters_for_quote,
        &effective_trader,
        &quote_response,
        is_ethflow,
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
            .unwrap_or_else(|| default_slippage_bps(canonical_chain_id, is_ethflow)),
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
        is_ethflow,
        resolved_env: canonical_env,
    })
}

struct QuoteResultInputs<'a> {
    trader: &'a QuoterParameters,
    trade_parameters: TradeParameters,
    quote_response: cow_sdk_orderbook::OrderQuoteResponse,
    app_data_info: TradingAppDataInfo,
    orderbook_binding: OrderbookRuntimeBinding,
    suggested_slippage: u32,
    amounts_and_costs: cow_sdk_core::QuoteAmountsAndCosts,
    is_ethflow: bool,
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
    let order_to_sign = get_order_to_sign(
        crate::order::OrderToSignParams {
            chain_id: inputs.trader.chain_id,
            from: inputs.trader.account.clone(),
            is_ethflow: inputs.is_ethflow,
            network_costs_amount: Some(Amount::new(
                inputs.quote_response.quote.network_cost_amount().to_owned(),
            )?),
            apply_costs_slippage_and_fees: true,
            protocol_fee_bps: sanitize_protocol_fee_bps(
                inputs.quote_response.protocol_fee_bps.as_deref(),
            ),
        },
        &crate::swap_params_to_limit_order_params(
            &inputs.trade_parameters,
            &inputs.quote_response,
        )?,
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
    trade_parameters: &TradeParameters,
    advanced_settings: Option<&SwapAdvancedSettings>,
) -> Result<TradeParameters, TradingError> {
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
        cow_sdk_core::OrderKind::Sell => QuoteSide::sell(trade_parameters.amount.to_string()),
        cow_sdk_core::OrderKind::Buy => QuoteSide::buy(trade_parameters.amount.to_string()),
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

/// Merges a typed [`AppDataParams`] override onto a typed base
/// [`AppDataParams`] and returns the typed merged value.
///
/// Scalar and optional top-level fields (`app_code`, `environment`,
/// `signer`, `flashloan`) follow override-wins semantics with a
/// base-value fallback. The nested `metadata` map is recursively deep
/// merged, with one carve-out: when the override contains a `hooks`
/// entry the base side's `hooks` envelope is dropped before the merge
/// so override-supplied hooks fully replace the base-side hooks envelope
/// instead of recursively merging into it. This keeps the metadata
/// merge shape aligned with the reviewed upstream SDK, where a caller
/// supplying a new `metadata.hooks` object means "use these hooks and
/// nothing else" rather than "merge these hooks on top of whatever
/// pre/post arrays the base doc happens to have".
///
/// Non-`hooks` metadata entries continue to follow standard recursive
/// deep-merge semantics. Arrays fall through to the override value in
/// full — including the `userConsents` array — so replacement rather
/// than concatenation is the default for any JSON array on the
/// metadata side.
#[must_use]
pub(crate) fn merge_app_data_params(
    base: &AppDataParams,
    override_params: &AppDataParams,
) -> AppDataParams {
    let mut base_metadata = base.metadata.clone();
    // The reviewed upstream SDK replaces rather than recursively merges
    // `metadata.hooks` — when the override supplies any hooks envelope,
    // pre/post sibling arrays from the base side are dropped before the
    // deep merge so the override's hooks envelope is the final shape.
    if override_params.metadata.contains_key("hooks") {
        base_metadata.remove("hooks");
    }

    let metadata = match deep_merge_values(
        Value::Object(base_metadata),
        Value::Object(override_params.metadata.clone()),
    ) {
        Value::Object(map) => map,
        _ => Map::new(),
    };

    AppDataParams::new(
        override_params
            .app_code
            .clone()
            .or_else(|| base.app_code.clone()),
        override_params
            .environment
            .clone()
            .or_else(|| base.environment.clone()),
        override_params
            .signer
            .clone()
            .or_else(|| base.signer.clone()),
        override_params
            .flashloan
            .clone()
            .or_else(|| base.flashloan.clone()),
        metadata,
    )
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
