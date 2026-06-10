use cow_sdk_core::transport::policy::TransportPolicy;
use cow_sdk_subgraph::{SubgraphApi, SubgraphQueryRequest};
use serde_json::Value;
use wasm_bindgen::prelude::*;

use crate::exports::{
    cancel::{
        ClientCallScope, SdkClientOptions, run_with_client_options, transport_policy_with_timeout,
    },
    dto::{SubgraphQueryInput, parse_chain, to_js_value, transport_policy_from_config},
    envelope::WasmEnvelope,
    errors::WasmError,
    transport::{configured_fetch_transport, optional_timeout, required_string, required_u32},
};

#[wasm_bindgen]
extern "C" {
    /// Configuration object used to construct a `SubgraphClient`.
    ///
    /// The public TypeScript facade accepts `chainId`, required `apiKey`, an
    /// explicit `transport`, optional `transportPolicy`, and default
    /// cancellation settings.
    #[wasm_bindgen(typescript_type = "SubgraphClientConfig")]
    pub type SubgraphClientConfig;
}

/// Read-only subgraph client backed by an explicitly configured transport.
///
/// Construct this client when JavaScript needs protocol totals, recent volume,
/// or custom GraphQL query execution through the same transport and policy
/// model as the orderbook clients.
#[wasm_bindgen]
pub struct SubgraphClient {
    inner: SubgraphApi,
    _callback_guard: crate::exports::registry::FetchCallbackGuard,
}

#[wasm_bindgen]
impl SubgraphClient {
    /// Creates a subgraph client from a single config object.
    ///
    /// The config must include `chainId`, `apiKey`, and `transport`. Optional
    /// timeout, signal, and policy fields become client defaults for later
    /// method calls.
    ///
    /// @param config Subgraph client configuration.
    /// @throws CowError when the chain, API key, transport, or policy is invalid.
    #[wasm_bindgen(constructor)]
    pub fn new(config: SubgraphClientConfig) -> Result<SubgraphClient, JsValue> {
        let config = config.as_ref();
        let chain_id = required_u32(config, "chainId")?;
        let api_key = required_string(config, "apiKey")?;
        let timeout = optional_timeout(config)?;
        let transport_policy =
            transport_policy_from_config(config, TransportPolicy::default_subgraph(), timeout)?;
        let (transport, callback_guard) = configured_fetch_transport(
            config,
            timeout,
            transport_policy.client_policy().max_response_bytes(),
        )?;
        Ok(Self {
            inner: build_subgraph(chain_id, api_key, transport, transport_policy)?,
            _callback_guard: callback_guard,
        })
    }

    /// Fetches aggregate protocol totals from the subgraph.
    ///
    /// The request uses the client's configured chain, API key, transport, and
    /// transport policy.
    ///
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing aggregate totals.
    /// @throws CowError for transport, cancellation, timeout, or subgraph errors.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.subgraph.totals"))
    )]
    #[wasm_bindgen(js_name = "getTotals")]
    pub async fn totals(
        &self,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let inner = subgraph_for_scope(&self.inner, &scope);
        run_with_client_options(scope, async move { subgraph_get_totals(&inner).await }).await
    }

    /// Fetches recent daily volume rows.
    ///
    /// The `days` value controls how many recent daily buckets the subgraph
    /// query requests.
    ///
    /// @param days Number of daily buckets to fetch.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing daily volume rows.
    /// @throws CowError for invalid query shape, transport failure, or timeout.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.subgraph.last_days_volume"))
    )]
    #[wasm_bindgen(js_name = "getLastDaysVolume")]
    pub async fn last_days_volume(
        &self,
        days: u32,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let inner = subgraph_for_scope(&self.inner, &scope);
        run_with_client_options(scope, async move {
            subgraph_get_last_days_volume(&inner, days).await
        })
        .await
    }

    /// Fetches recent hourly volume rows.
    ///
    /// The `hours` value controls how many recent hourly buckets the subgraph
    /// query requests.
    ///
    /// @param hours Number of hourly buckets to fetch.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing hourly volume rows.
    /// @throws CowError for invalid query shape, transport failure, or timeout.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.subgraph.last_hours_volume"))
    )]
    #[wasm_bindgen(js_name = "getLastHoursVolume")]
    pub async fn last_hours_volume(
        &self,
        hours: u32,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let inner = subgraph_for_scope(&self.inner, &scope);
        run_with_client_options(scope, async move {
            subgraph_get_last_hours_volume(&inner, hours).await
        })
        .await
    }

    /// Runs a caller-provided GraphQL query against the configured subgraph.
    ///
    /// Use this method when the built-in totals or volume helpers are too
    /// narrow. Variables and operation name are forwarded when present.
    ///
    /// @param request GraphQL query, variables, and optional operation name.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing the JSON GraphQL response.
    /// @throws CowError for transport, timeout, cancellation, or GraphQL errors.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, fields(endpoint = "wasm.subgraph.run_query"))
    )]
    #[wasm_bindgen(js_name = "runQuery")]
    pub async fn run_query(
        &self,
        request: SubgraphQueryInput,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let inner = subgraph_for_scope(&self.inner, &scope);
        run_with_client_options(
            scope,
            async move { subgraph_run_query(&inner, request).await },
        )
        .await
    }
}

fn build_subgraph(
    chain_id: u32,
    api_key: String,
    transport: std::sync::Arc<dyn cow_sdk_core::HttpTransport + Send + Sync>,
    transport_policy: TransportPolicy,
) -> Result<SubgraphApi, JsValue> {
    let chain = parse_chain(chain_id)?;
    SubgraphApi::builder()
        .chain(chain)
        .api_key(api_key)
        .transport(transport)
        .transport_policy(transport_policy)
        .build()
        .map_err(|error| WasmError::from(error).into_js())
}

fn subgraph_for_scope(inner: &SubgraphApi, scope: &ClientCallScope) -> SubgraphApi {
    inner
        .clone()
        .with_transport_policy(transport_policy_with_timeout(
            inner.transport_policy(),
            scope.timeout(),
        ))
}

async fn subgraph_get_totals(inner: &SubgraphApi) -> Result<JsValue, JsValue> {
    let totals = inner
        .totals()
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(totals))
}

async fn subgraph_get_last_days_volume(inner: &SubgraphApi, days: u32) -> Result<JsValue, JsValue> {
    let volume = inner
        .last_days_volume(days)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(volume))
}

async fn subgraph_get_last_hours_volume(
    inner: &SubgraphApi,
    hours: u32,
) -> Result<JsValue, JsValue> {
    let volume = inner
        .last_hours_volume(hours)
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
