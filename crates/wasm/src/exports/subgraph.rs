use cow_sdk_subgraph::{SubgraphApi, SubgraphQueryRequest};
use js_sys::Function;
use serde_json::Value;
use wasm_bindgen::prelude::*;

use crate::exports::{
    dto::{SubgraphQueryInput, WasmEnvelope, from_json_value, parse_chain, to_js_value},
    errors::WasmError,
    transport::{
        callback_fetch_transport, callback_fetch_transport_from_handle, default_fetch_transport,
    },
};

/// Subgraph client backed by the browser fetch transport.
#[wasm_bindgen]
pub struct SubgraphClient {
    inner: SubgraphApi,
}

#[wasm_bindgen]
impl SubgraphClient {
    /// Creates a subgraph client for a chain and Graph API key.
    #[wasm_bindgen(constructor)]
    pub fn new(chain_id: u32, api_key: String) -> Result<SubgraphClient, JsValue> {
        Ok(Self {
            inner: build_subgraph(chain_id, api_key, default_fetch_transport(None))?,
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

/// Subgraph client backed by a JavaScript fetch callback.
#[wasm_bindgen]
pub struct SubgraphClientWithFetch {
    inner: SubgraphApi,
    _handle: Option<crate::exports::registry::FetchCallbackHandle>,
}

#[wasm_bindgen]
impl SubgraphClientWithFetch {
    /// Creates a subgraph client that owns a registered fetch callback.
    #[wasm_bindgen(constructor)]
    pub fn new(
        chain_id: u32,
        api_key: String,
        fetch_callback: Function,
    ) -> Result<SubgraphClientWithFetch, JsValue> {
        let (transport, handle) = callback_fetch_transport(fetch_callback, None)?;
        Ok(Self {
            inner: build_subgraph(chain_id, api_key, transport)?,
            _handle: Some(handle),
        })
    }

    /// Creates a subgraph client from an existing fetch-callback handle id.
    #[wasm_bindgen(js_name = "fromHandle")]
    pub fn from_handle(
        chain_id: u32,
        api_key: String,
        fetch_callback_id: u32,
    ) -> Result<SubgraphClientWithFetch, JsValue> {
        let transport = callback_fetch_transport_from_handle(fetch_callback_id, None)?;
        Ok(Self {
            inner: build_subgraph(chain_id, api_key, transport)?,
            _handle: None,
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
    let request = parse_subgraph_request(request.value)?;
    let value: Value = inner
        .run_query(request)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(value))
}

fn parse_subgraph_request(value: Value) -> Result<SubgraphQueryRequest, JsValue> {
    match value {
        Value::String(document) => Ok(SubgraphQueryRequest::new(document)),
        value => from_json_value("query", value),
    }
}
