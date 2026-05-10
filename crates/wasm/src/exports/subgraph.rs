use cow_sdk_subgraph::{SubgraphApi, SubgraphQueryRequest};
use serde_json::Value;
use wasm_bindgen::prelude::*;

use crate::exports::{
    dto::{SubgraphQueryInput, parse_chain, to_js_value},
    envelope::WasmEnvelope,
    errors::WasmError,
    transport::{configured_fetch_transport, optional_timeout, required_string, required_u32},
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "SubgraphClientConfig")]
    pub type SubgraphClientConfig;
}

/// Subgraph client backed by an explicitly configured HTTP transport.
#[wasm_bindgen]
pub struct SubgraphClient {
    inner: SubgraphApi,
    _callback_guard: crate::exports::registry::FetchCallbackGuard,
}

#[wasm_bindgen]
impl SubgraphClient {
    /// Creates a subgraph client from a single config object.
    #[wasm_bindgen(constructor)]
    pub fn new(config: SubgraphClientConfig) -> Result<SubgraphClient, JsValue> {
        let config = config.as_ref();
        let chain_id = required_u32(config, "chainId")?;
        let api_key = required_string(config, "apiKey")?;
        let timeout = optional_timeout(config)?;
        let (transport, callback_guard) = configured_fetch_transport(config, timeout)?;
        Ok(Self {
            inner: build_subgraph(chain_id, api_key, transport)?,
            _callback_guard: callback_guard,
        })
    }

    /// Fetches aggregate totals.
    #[wasm_bindgen(js_name = "getTotals")]
    pub async fn get_totals(&self) -> Result<JsValue, JsValue> {
        subgraph_get_totals(&self.inner).await
    }

    /// Fetches daily volume rows.
    #[wasm_bindgen(js_name = "getLastDaysVolume")]
    pub async fn get_last_days_volume(&self, days: u32) -> Result<JsValue, JsValue> {
        subgraph_get_last_days_volume(&self.inner, days).await
    }

    /// Fetches hourly volume rows.
    #[wasm_bindgen(js_name = "getLastHoursVolume")]
    pub async fn get_last_hours_volume(&self, hours: u32) -> Result<JsValue, JsValue> {
        subgraph_get_last_hours_volume(&self.inner, hours).await
    }

    /// Runs a raw GraphQL query.
    #[wasm_bindgen(js_name = "runQuery")]
    pub async fn run_query(&self, request: SubgraphQueryInput) -> Result<JsValue, JsValue> {
        subgraph_run_query(&self.inner, request).await
    }
}

fn build_subgraph(
    chain_id: u32,
    api_key: String,
    transport: std::sync::Arc<dyn cow_sdk_core::HttpTransport + Send + Sync>,
) -> Result<SubgraphApi, JsValue> {
    let chain = parse_chain(chain_id)?;
    SubgraphApi::builder()
        .chain(chain)
        .api_key(api_key)
        .transport(transport)
        .build()
        .map_err(|error| WasmError::from(error).into_js())
}

async fn subgraph_get_totals(inner: &SubgraphApi) -> Result<JsValue, JsValue> {
    let totals = inner
        .get_totals()
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(totals))
}

async fn subgraph_get_last_days_volume(inner: &SubgraphApi, days: u32) -> Result<JsValue, JsValue> {
    let volume = inner
        .get_last_days_volume(days)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(volume))
}

async fn subgraph_get_last_hours_volume(
    inner: &SubgraphApi,
    hours: u32,
) -> Result<JsValue, JsValue> {
    let volume = inner
        .get_last_hours_volume(hours)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(volume))
}

async fn subgraph_run_query(
    inner: &SubgraphApi,
    request: SubgraphQueryInput,
) -> Result<JsValue, JsValue> {
    let request = parse_subgraph_request(request);
    let value: Value = inner
        .run_query(request)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(value))
}

fn parse_subgraph_request(input: SubgraphQueryInput) -> SubgraphQueryRequest {
    let mut request = SubgraphQueryRequest::new(input.query);
    if let Some(variables) = input.variables {
        request = request.with_variables(variables);
    }
    if let Some(operation_name) = input.operation_name {
        request = request.with_operation_name(operation_name);
    }
    request
}
