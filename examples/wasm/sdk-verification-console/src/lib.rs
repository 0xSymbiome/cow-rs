use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

use cow_sdk::app_data::{
    app_data_hex_to_cid, cid_to_app_data_hex, get_app_data_info, get_app_data_schema,
    validate_app_data_doc,
};
use cow_sdk::contracts::deployment_for_chain;
use cow_sdk::core::{
    AppDataHex, BuyTokenDestination, OrderKind, SellTokenSource, UnsignedOrder,
    wrapped_native_token,
};
use cow_sdk::orderbook::{
    ApiContext, AppDataHash, GetOrdersRequest, GetTradesRequest, OrderQuoteRequest,
};
use cow_sdk::prelude::{
    Address, Amount, CowEnv, OrderBookApi, OrderUid, OrderbookError, SupportedChainId, TradingSdk,
};
use cow_sdk::signing::{
    ORDER_PRIMARY_TYPE, domain_separator, eip1271_signature_payload, generate_order_id,
    order_typed_data,
};
use cow_sdk::trading::{
    ApprovalParameters, DEFAULT_QUOTE_VALIDITY, DEFAULT_SLIPPAGE_BPS, GAS_LIMIT_DEFAULT,
    GAS_MARGIN_PERCENT, MAX_SLIPPAGE_BPS, PartialTraderParameters, PartnerFee, PartnerFeePolicy,
    TradingSdkOptions, approval_transaction, default_slippage_bps, is_ethflow_order,
    partner_fee_bps, sanitize_protocol_fee_bps, suggest_slippage_from_fee,
    suggest_slippage_from_volume, swap_params_to_limit_order_params,
};
use cow_sdk_subgraph::SubgraphApi;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn supported_chains_json() -> Result<String, JsValue> {
    let report = SupportedChainId::ALL
        .into_iter()
        .map(|chain_id| {
            let wrapped = wrapped_native_token(chain_id);
            json!({
                "chainId": u64::from(chain_id),
                "name": chain_label(chain_id),
                "apiPath": chain_id.api_path(),
                "wrappedNative": {
                    "address": wrapped.address.as_str(),
                    "symbol": wrapped.symbol,
                    "decimals": wrapped.decimals
                }
            })
        })
        .collect::<Vec<_>>();

    pretty_json(&report)
}

#[wasm_bindgen]
pub fn capability_report_json(chain_id: u32, env: &str) -> Result<String, JsValue> {
    use std::sync::Arc;

    let chain_id = parse_chain_id(chain_id)?;
    let env = parse_env(env)?;
    let orderbook_client = orderbook_api(chain_id, env);
    let sdk = TradingSdk::new(
        PartialTraderParameters::new()
            .with_chain_id(chain_id)
            .with_app_code("cow-rs/wasm-console".to_owned())
            .with_owner(sample_owner())
            .with_env(env),
        TradingSdkOptions::default().with_orderbook_client(Arc::new(orderbook_client)),
    )
    .map_err(js_string_error)?;
    let api_context = api_context(chain_id, env);
    let deployment = deployment_for_chain(u64::from(chain_id))
        .map_err(|error| to_js_error(error.to_string()))?;
    let wrapped_native = wrapped_native_token(chain_id);
    let order = sample_unsigned_order(chain_id);
    let generated = generate_order_id(chain_id, &order, &sample_owner(), None)
        .map_err(|error| to_js_error(error.to_string()))?;

    pretty_json(&json!({
        "surface": "cow-sdk",
        "mode": "wasm-console",
        "chainId": u64::from(chain_id),
        "chain": chain_label(chain_id),
        "env": env.as_str(),
        "apiBaseUrl": api_context
            .resolved_base_url()
            .map_err(|error| to_js_error(error.to_string()))?,
        "sdkConstructed": sdk.trader_defaults().chain_id == Some(chain_id),
        "wrappedNative": {
            "address": wrapped_native.address.as_str(),
            "symbol": wrapped_native.symbol,
            "decimals": wrapped_native.decimals
        },
        "deployment": {
            "settlement": deployment.settlement.as_str(),
            "vaultRelayer": deployment.vault_relayer.as_str(),
            "ethFlow": deployment.eth_flow.as_str()
        },
        "sampleOrder": {
            "sellToken": order.sell_token.as_str(),
            "buyToken": order.buy_token.as_str(),
            "receiver": order.receiver.as_str()
        },
        "sampleOrderNotes": {
            "sellToken": "Selected-chain wrapped native token.",
            "buyToken": "Static USDC example address used only for deterministic previews."
        },
        "orderId": generated.order_id.as_str()
    }))
}

/// Drive the deterministic verification cycle end-to-end so a first-time
/// visitor has a single primary walkthrough entry. Composes
/// `supported_chains_json`, `capability_report_json(1, "prod")`,
/// `app_data_report_json`, and the CID roundtrip helpers in order and
/// returns a tagged envelope.
#[wasm_bindgen]
pub fn walkthrough_determinism_cycle_json() -> Result<String, JsValue> {
    let sample_doc = r#"{
        "version": "1.14.0",
        "appCode": "cow-rs/wasm-console",
        "environment": "browser",
        "metadata": {
          "quote": { "slippageBips": 50 }
        }
      }"#;

    let mut steps: Vec<Value> = Vec::new();

    let chains = supported_chains_json()?;
    steps.push(json!({
        "name": "supported-chains",
        "result": parse_json::<Value>(&chains, "supportedChains")?
    }));

    let capability = capability_report_json(1, "prod")?;
    steps.push(json!({
        "name": "capability-report",
        "result": parse_json::<Value>(&capability, "capabilityReport")?
    }));

    let app_data = app_data_report_json(sample_doc)?;
    let app_data_value: Value = parse_json(&app_data, "appDataReport")?;
    steps.push(json!({
        "name": "app-data-report",
        "result": app_data_value.clone(),
    }));

    let cid = app_data_value
        .get("cid")
        .and_then(Value::as_str)
        .ok_or_else(|| to_js_error("app-data report did not expose a cid"))?;
    let app_data_hex = app_data_value
        .get("appDataHex")
        .and_then(Value::as_str)
        .ok_or_else(|| to_js_error("app-data report did not expose an appDataHex"))?;

    let hex_from_cid = hex_from_cid_json(cid)?;
    steps.push(json!({
        "name": "hex-from-cid",
        "result": parse_json::<Value>(&hex_from_cid, "hexFromCid")?
    }));

    let cid_from_hex = cid_from_hex_json(app_data_hex)?;
    steps.push(json!({
        "name": "cid-from-hex",
        "result": parse_json::<Value>(&cid_from_hex, "cidFromHex")?
    }));

    pretty_json(&json!({
        "name": "sdk-verification-console.determinism-cycle",
        "completed": true,
        "failedAt": Value::Null,
        "steps": steps,
    }))
}

#[wasm_bindgen]
pub fn app_data_report_json(doc_json: &str) -> Result<String, JsValue> {
    let document: Value = parse_json(doc_json, "appDataDoc")?;
    let validation = validate_app_data_doc(&document);
    let info = get_app_data_info(&document).map_err(|error| to_js_error(error.to_string()))?;
    let deterministic = cow_sdk::app_data::stringify_deterministic(&document)
        .map_err(|error| to_js_error(error.to_string()))?;

    pretty_json(&json!({
        "valid": validation.success,
        "errors": validation.errors,
        "cid": info.cid,
        "appDataHex": info.app_data_hex,
        "deterministicJson": deterministic,
        "content": info.app_data_content
    }))
}

#[wasm_bindgen]
pub fn app_data_schema_json(doc_json: &str) -> Result<String, JsValue> {
    let document: Value = parse_json(doc_json, "appDataDoc")?;
    let version = document
        .get("version")
        .and_then(Value::as_str)
        .ok_or_else(|| to_js_error("app-data document must include a string `version` field"))?;
    let schema = get_app_data_schema(version).map_err(|error| to_js_error(error.to_string()))?;

    pretty_json(&json!({
        "version": version,
        "schema": schema
    }))
}

#[wasm_bindgen]
pub fn cid_from_hex_json(app_data_hex: &str) -> Result<String, JsValue> {
    pretty_json(&json!({
        "appDataHex": app_data_hex,
        "cid": app_data_hex_to_cid(app_data_hex)
            .map_err(|error| to_js_error(error.to_string()))?
    }))
}

#[wasm_bindgen]
pub fn hex_from_cid_json(cid: &str) -> Result<String, JsValue> {
    pretty_json(&json!({
        "cid": cid,
        "appDataHex": cid_to_app_data_hex(cid).map_err(|error| to_js_error(error.to_string()))?
    }))
}

#[wasm_bindgen]
pub fn order_envelope_preview_json(
    chain_id: u32,
    order_json: &str,
    owner: &str,
) -> Result<String, JsValue> {
    let chain_id = parse_chain_id(chain_id)?;
    let order = parse_order(order_json)?;
    let owner = parse_address(owner, "owner")?;
    let typed = order_typed_data(chain_id, &order, None).map_err(js_string_error)?;
    let generated = generate_order_id(chain_id, &order, &owner, None).map_err(js_string_error)?;
    let domain_separator = domain_separator(chain_id, None)
        .map_err(|error| to_js_error(error.to_string()))?;

    pretty_json(&json!({
        "primaryType": typed.primary_type,
        "expectedPrimaryType": ORDER_PRIMARY_TYPE,
        "domainSeparator": domain_separator,
        "domain": {
            "name": typed.domain.name,
            "version": typed.domain.version,
            "chainId": typed.domain.chain_id,
            "verifyingContract": typed.domain.verifying_contract.as_str()
        },
        "types": typed.types,
        "message": typed.message,
        "digest": generated.order_digest,
        "orderId": generated.order_id.as_str()
    }))
}

#[wasm_bindgen]
pub fn eip1271_payload_preview_json(
    order_json: &str,
    ecdsa_signature: &str,
) -> Result<String, JsValue> {
    let order = parse_order(order_json)?;
    let payload = eip1271_signature_payload(&order, ecdsa_signature).map_err(js_string_error)?;

    pretty_json(&json!({
        "payload": payload
    }))
}

#[wasm_bindgen]
pub fn approval_transaction_preview_json(
    chain_id: u32,
    env: &str,
    approval_json: &str,
) -> Result<String, JsValue> {
    let chain_id = parse_chain_id(chain_id)?;
    let env = parse_env(env)?;
    let mut params: ApprovalParameters = parse_json(approval_json, "approvalParameters")?;
    params.chain_id = params.chain_id.or(Some(chain_id));
    params.env = params.env.or(Some(env));
    let transaction = approval_transaction(&params, chain_id, env)
        .map_err(|error| to_js_error(error.to_string()))?;

    pretty_json(&json!({
        "chainId": u64::from(chain_id),
        "env": env.as_str(),
        "transaction": transaction,
        "gas": {
            "defaultLimit": GAS_LIMIT_DEFAULT,
            "marginPercent": GAS_MARGIN_PERCENT
        }
    }))
}

#[wasm_bindgen]
pub fn trading_defaults_json() -> Result<String, JsValue> {
    let partner_fee = PartnerFee::from(
        PartnerFeePolicy::volume(42, sample_owner())
            .expect("sample partner-fee value is valid"),
    );

    pretty_json(&json!({
        "quoteValiditySeconds": DEFAULT_QUOTE_VALIDITY,
        "defaultSlippageBps": DEFAULT_SLIPPAGE_BPS,
        "mainnetDefaultSlippageBps": default_slippage_bps(SupportedChainId::Mainnet, false),
        "ethflowFloorSlippageBps": default_slippage_bps(SupportedChainId::Mainnet, true),
        "maxSlippageBps": MAX_SLIPPAGE_BPS,
        "sanitizedProtocolFeeBps": sanitize_protocol_fee_bps(Some("12.5")),
        "partnerFeeBps": partner_fee_bps(Some(&partner_fee)),
        "slippageFromFee": suggest_slippage_from_fee("1000000000000000", 50.0)
            .map_err(js_string_error)?,
        "slippageFromVolume": suggest_slippage_from_volume(
            true,
            "1000000000000000000",
            "999000000000000000",
            0.5
        )
        .map_err(js_string_error)?
    }))
}

#[wasm_bindgen]
pub async fn trading_quote_preview_json(
    chain_id: u32,
    env: &str,
    app_code: &str,
    owner: &str,
    trade_json: &str,
) -> Result<String, JsValue> {
    use std::sync::Arc;

    let chain_id = parse_chain_id(chain_id)?;
    let env = parse_env(env)?;
    let owner = parse_address(owner, "owner")?;
    let mut params = parse_trade_parameters(trade_json)?;
    params.owner = params.owner.or(Some(owner.clone()));
    params.env = params.env.or(Some(env));

    let orderbook_client = orderbook_api(chain_id, env);
    let sdk = TradingSdk::new(
        PartialTraderParameters::new()
            .with_chain_id(chain_id)
            .with_app_code(app_code.trim().to_owned())
            .with_owner(owner)
            .with_env(env),
        TradingSdkOptions::default().with_orderbook_client(Arc::new(orderbook_client)),
    )
    .map_err(js_string_error)?;
    let results = sdk
        .get_quote_only(params, None)
        .await
        .map_err(|error| to_js_error(error.to_string()))?;
    let limit_parameters =
        swap_params_to_limit_order_params(&results.trade_parameters, &results.quote_response)
            .map_err(|error| to_js_error(error.to_string()))?;

    pretty_json(&json!({
        "quoteResults": results,
        "derived": {
            "isEthflowOrder": is_ethflow_order(&limit_parameters.sell_token),
            "limitTradeParameters": limit_parameters
        }
    }))
}

#[wasm_bindgen]
pub async fn orderbook_version_json(chain_id: u32, env: &str) -> Result<String, JsValue> {
    let chain_id = parse_chain_id(chain_id)?;
    let env = parse_env(env)?;
    let api = orderbook_api(chain_id, env);
    let version = api.get_version().await.map_err(orderbook_js_error)?;

    pretty_json(&json!({
        "chainId": u64::from(chain_id),
        "env": env.as_str(),
        "apiBaseUrl": api
            .context()
            .resolved_base_url()
            .map_err(|error| to_js_error(error.to_string()))?,
        "version": version
    }))
}

#[wasm_bindgen]
pub async fn orderbook_quote_json(
    chain_id: u32,
    env: &str,
    request_json: &str,
) -> Result<String, JsValue> {
    let chain_id = parse_chain_id(chain_id)?;
    let env = parse_env(env)?;
    let request: OrderQuoteRequest = parse_json(request_json, "orderQuoteRequest")?;
    let response = orderbook_api(chain_id, env)
        .get_quote(&request)
        .await
        .map_err(orderbook_js_error)?;

    pretty_json(&response)
}

#[wasm_bindgen]
pub async fn orderbook_order_json(
    chain_id: u32,
    env: &str,
    order_uid: &str,
) -> Result<String, JsValue> {
    let chain_id = parse_chain_id(chain_id)?;
    let env = parse_env(env)?;
    let order_uid = parse_order_uid(order_uid)?;
    let order = orderbook_api(chain_id, env)
        .get_order(&order_uid)
        .await
        .map_err(orderbook_js_error)?;

    pretty_json(&order)
}

#[wasm_bindgen]
pub async fn orderbook_orders_by_owner_json(
    chain_id: u32,
    env: &str,
    owner: &str,
) -> Result<String, JsValue> {
    let chain_id = parse_chain_id(chain_id)?;
    let env = parse_env(env)?;
    let owner = parse_address(owner, "owner")?;
    let request = GetOrdersRequest::new(owner);
    let orders = orderbook_api(chain_id, env)
        .get_orders(&request)
        .await
        .map_err(orderbook_js_error)?;

    pretty_json(&orders)
}

#[wasm_bindgen]
pub async fn orderbook_trades_by_owner_json(
    chain_id: u32,
    env: &str,
    owner: &str,
) -> Result<String, JsValue> {
    let chain_id = parse_chain_id(chain_id)?;
    let env = parse_env(env)?;
    let owner = parse_address(owner, "owner")?;
    let request = GetTradesRequest::by_owner(owner);
    let trades = orderbook_api(chain_id, env)
        .get_trades(&request)
        .await
        .map_err(orderbook_js_error)?;

    pretty_json(&trades)
}

#[wasm_bindgen]
pub async fn orderbook_trades_by_order_json(
    chain_id: u32,
    env: &str,
    order_uid: &str,
) -> Result<String, JsValue> {
    let chain_id = parse_chain_id(chain_id)?;
    let env = parse_env(env)?;
    let order_uid = parse_order_uid(order_uid)?;
    let request = GetTradesRequest::by_order_uid(order_uid);
    let trades = orderbook_api(chain_id, env)
        .get_trades(&request)
        .await
        .map_err(orderbook_js_error)?;

    pretty_json(&trades)
}

#[wasm_bindgen]
pub async fn orderbook_native_price_json(
    chain_id: u32,
    env: &str,
    token: &str,
) -> Result<String, JsValue> {
    let chain_id = parse_chain_id(chain_id)?;
    let env = parse_env(env)?;
    let token = parse_address(token, "token")?;
    let price = orderbook_api(chain_id, env)
        .get_native_price(&token)
        .await
        .map_err(orderbook_js_error)?;

    pretty_json(&price)
}

#[wasm_bindgen]
pub async fn orderbook_total_surplus_json(
    chain_id: u32,
    env: &str,
    owner: &str,
) -> Result<String, JsValue> {
    let chain_id = parse_chain_id(chain_id)?;
    let env = parse_env(env)?;
    let owner = parse_address(owner, "owner")?;
    let surplus = orderbook_api(chain_id, env)
        .get_total_surplus(&owner)
        .await
        .map_err(orderbook_js_error)?;

    pretty_json(&surplus)
}

#[wasm_bindgen]
pub async fn orderbook_app_data_json(
    chain_id: u32,
    env: &str,
    app_data_hex: &str,
) -> Result<String, JsValue> {
    let chain_id = parse_chain_id(chain_id)?;
    let env = parse_env(env)?;
    let app_data_hash =
        AppDataHash::new(app_data_hex).map_err(|error| to_js_error(error.to_string()))?;
    let app_data = orderbook_api(chain_id, env)
        .get_app_data(&app_data_hash)
        .await
        .map_err(orderbook_js_error)?;

    pretty_json(&app_data)
}

#[wasm_bindgen]
pub async fn orderbook_latest_competition_json(
    chain_id: u32,
    env: &str,
) -> Result<String, JsValue> {
    let chain_id = parse_chain_id(chain_id)?;
    let env = parse_env(env)?;
    let competition = orderbook_api(chain_id, env)
        .get_latest_solver_competition()
        .await
        .map_err(orderbook_js_error)?;

    pretty_json(&competition)
}

#[wasm_bindgen]
pub async fn orderbook_auction_json(chain_id: u32, env: &str) -> Result<String, JsValue> {
    orderbook_latest_competition_json(chain_id, env).await
}

#[wasm_bindgen]
pub async fn subgraph_totals_json(chain_id: u32, api_key: &str) -> Result<String, JsValue> {
    let chain_id = parse_chain_id(chain_id)?;
    let totals = subgraph_api(chain_id, api_key)?
        .get_totals()
        .await
        .map_err(|error| to_js_error(error.to_string()))?;

    pretty_json(&totals)
}

#[wasm_bindgen]
pub async fn subgraph_last_days_volume_json(
    chain_id: u32,
    api_key: &str,
    days: u32,
) -> Result<String, JsValue> {
    let chain_id = parse_chain_id(chain_id)?;
    let volume = subgraph_api(chain_id, api_key)?
        .get_last_days_volume(days)
        .await
        .map_err(|error| to_js_error(error.to_string()))?;

    pretty_json(&volume)
}

#[wasm_bindgen]
pub async fn subgraph_last_hours_volume_json(
    chain_id: u32,
    api_key: &str,
    hours: u32,
) -> Result<String, JsValue> {
    let chain_id = parse_chain_id(chain_id)?;
    let volume = subgraph_api(chain_id, api_key)?
        .get_last_hours_volume(hours)
        .await
        .map_err(|error| to_js_error(error.to_string()))?;

    pretty_json(&volume)
}

fn orderbook_api(chain_id: SupportedChainId, env: CowEnv) -> OrderBookApi {
    let context = api_context(chain_id, env);

    #[cfg(target_arch = "wasm32")]
    {
        use std::sync::Arc;

        use cow_sdk::core::HttpTransport;
        use cow_sdk_transport_wasm::{FetchTransport, FetchTransportConfig};

        let base_url = context.resolved_base_url().unwrap_or_default();
        let transport: Arc<dyn HttpTransport + Send + Sync> =
            Arc::new(FetchTransport::new(&FetchTransportConfig::new(base_url)));
        OrderBookApi::builder_from_context(context)
            .transport(transport)
            .build()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        OrderBookApi::builder_from_context(context).build()
    }
}

fn api_context(chain_id: SupportedChainId, env: CowEnv) -> ApiContext {
    ApiContext::new(chain_id, env)
}

fn subgraph_api(chain_id: SupportedChainId, api_key: &str) -> Result<SubgraphApi, JsValue> {
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err(to_js_error("subgraph API key is required"));
    }

    #[cfg(target_arch = "wasm32")]
    {
        use std::sync::Arc;

        use cow_sdk::core::HttpTransport;
        use cow_sdk_transport_wasm::{FetchTransport, FetchTransportConfig};

        let transport: Arc<dyn HttpTransport + Send + Sync> = Arc::new(FetchTransport::new(
            &FetchTransportConfig::new("https://gateway.thegraph.com/api"),
        ));
        Ok(SubgraphApi::builder()
            .chain(chain_id)
            .api_key(api_key)
            .transport(transport)
            .build())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Ok(SubgraphApi::builder()
            .chain(chain_id)
            .api_key(api_key)
            .build())
    }
}

fn parse_chain_id(chain_id: u32) -> Result<SupportedChainId, JsValue> {
    SupportedChainId::try_from(u64::from(chain_id)).map_err(|error| to_js_error(error.to_string()))
}

fn parse_env(env: &str) -> Result<CowEnv, JsValue> {
    match env.trim().to_ascii_lowercase().as_str() {
        "prod" => Ok(CowEnv::Prod),
        "staging" => Ok(CowEnv::Staging),
        other => Err(to_js_error(format!(
            "unsupported env `{other}`; expected `prod` or `staging`"
        ))),
    }
}

fn parse_address(value: &str, field: &str) -> Result<Address, JsValue> {
    Address::new(value).map_err(|error| to_js_error(format!("{field}: {error}")))
}

fn parse_order_uid(value: &str) -> Result<OrderUid, JsValue> {
    OrderUid::new(value).map_err(|error| to_js_error(error.to_string()))
}

fn parse_order(order_json: &str) -> Result<UnsignedOrder, JsValue> {
    parse_json(order_json, "unsignedOrder")
}

fn parse_trade_parameters(trade_json: &str) -> Result<cow_sdk::TradeParameters, JsValue> {
    parse_json(trade_json, "tradeParameters")
}

fn parse_json<T>(json_text: &str, label: &str) -> Result<T, JsValue>
where
    T: DeserializeOwned,
{
    serde_json::from_str(json_text)
        .map_err(|error| to_js_error(format!("invalid {label} JSON: {error}")))
}

fn pretty_json<T>(value: &T) -> Result<String, JsValue>
where
    T: Serialize,
{
    serde_json::to_string_pretty(value).map_err(|error| to_js_error(error.to_string()))
}

fn chain_label(chain_id: SupportedChainId) -> &'static str {
    match chain_id {
        SupportedChainId::Mainnet => "Mainnet",
        SupportedChainId::Bnb => "BNB Chain",
        SupportedChainId::GnosisChain => "Gnosis Chain",
        SupportedChainId::Polygon => "Polygon",
        SupportedChainId::Base => "Base",
        SupportedChainId::Plasma => "Plasma",
        SupportedChainId::ArbitrumOne => "Arbitrum One",
        SupportedChainId::Avalanche => "Avalanche",
        SupportedChainId::Ink => "Ink",
        SupportedChainId::Linea => "Linea",
        SupportedChainId::Sepolia => "Sepolia",
        _ => "Supported CoW Chain",
    }
}

fn sample_owner() -> Address {
    Address::new("0x4444444444444444444444444444444444444444")
        .expect("static example owner must remain valid")
}

fn sample_unsigned_order(chain_id: SupportedChainId) -> UnsignedOrder {
    UnsignedOrder::new(
        wrapped_native_token(chain_id).address,
        Address::new("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
            .expect("static example address must remain valid"),
        sample_owner(),
        Amount::new("100000000000000000").expect("static example sell amount must remain valid"),
        Amount::new("250000000").expect("static example buy amount must remain valid"),
        1_900_000_000,
        AppDataHex::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .expect("static example app-data hex must remain valid"),
        Amount::zero(),
        OrderKind::Sell,
        false,
        SellTokenSource::Erc20,
        BuyTokenDestination::Erc20,
    )
}

fn orderbook_js_error(error: OrderbookError) -> JsValue {
    to_js_error(error.to_string())
}

fn js_string_error(error: impl ToString) -> JsValue {
    to_js_error(error.to_string())
}

fn to_js_error(message: impl Into<String>) -> JsValue {
    JsValue::from_str(&message.into())
}
