use std::sync::Arc;

use async_trait::async_trait;
use cow_sdk_app_data::{AppDataError, IpfsFetchTransport};
use cow_sdk_core::{HttpTransport, Redacted, TransportError, TransportErrorClass};
use cow_sdk_pure_helpers as pure;
use cow_sdk_transport_policy::{
    AttemptOutcome as RetryOutcome, LimiterKey, RetrySignal, TransportPolicy, run_with_retry,
};
use wasm_bindgen::prelude::*;

use crate::exports::{
    cancel::{
        ClientCallScope, SdkClientOptions, run_with_client_options, transport_policy_with_timeout,
    },
    dto::{AppDataDocDto, to_js_value, transport_policy_from_config},
    envelope::WasmEnvelope,
    errors::WasmError,
    transport::{configured_fetch_transport, optional_string, optional_timeout},
};

/// Adapter that lets app-data IPFS reads flow through an HTTP transport.
pub(crate) struct IpfsHttpAdapter {
    inner: Arc<dyn HttpTransport + Send + Sync>,
    transport_policy: TransportPolicy,
}

impl IpfsHttpAdapter {
    fn from_parts(
        inner: Arc<dyn HttpTransport + Send + Sync>,
        transport_policy: TransportPolicy,
    ) -> Self {
        Self {
            inner,
            transport_policy,
        }
    }

    fn with_call_timeout(&self, timeout: Option<std::time::Duration>) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            transport_policy: transport_policy_with_timeout(&self.transport_policy, timeout),
        }
    }
}

#[async_trait(?Send)]
impl IpfsFetchTransport for IpfsHttpAdapter {
    async fn get(&self, uri: &str) -> Result<String, AppDataError> {
        // The shared driver in `cow-sdk-transport-policy` owns the retry loop,
        // rate-limit acquisition, backoff, and `Retry-After` clock — including
        // the wasm-safe wall clock that keeps this browser path from aborting
        // on a retryable gateway status. The closure performs one fetch and
        // classifies the result into the unified outcome space.
        let timeout = self.transport_policy.timeout();
        run_with_retry::<String, AppDataError, _, _>(
            self.transport_policy.retry(),
            self.transport_policy.rate_limit(),
            LimiterKey::Global,
            |_attempt_index| async move {
                match self.inner.get(uri, &[], timeout).await {
                    Ok(body) => RetryOutcome::Success(body),
                    Err(TransportError::HttpStatus {
                        status,
                        headers,
                        body,
                    }) => {
                        let header_pairs = headers
                            .iter()
                            .map(|(name, value)| (name.clone(), value.as_inner().clone()))
                            .collect::<Vec<_>>();
                        RetryOutcome::Failure {
                            error: transport_to_app_data_error(TransportError::HttpStatus {
                                status,
                                headers,
                                body,
                            }),
                            signal: RetrySignal::HttpStatus {
                                status,
                                headers: header_pairs,
                            },
                        }
                    }
                    Err(error) => {
                        // Preserve the prior IPFS contract: only categorical
                        // transport failures are retried; configuration and any
                        // future variant stay terminal (non-retryable class).
                        let class = match &error {
                            TransportError::Transport { class, .. } => *class,
                            _ => TransportErrorClass::Builder,
                        };
                        RetryOutcome::Failure {
                            error: transport_to_app_data_error(error),
                            signal: RetrySignal::Transport { class },
                        }
                    }
                }
            },
        )
        .await
    }
}

#[wasm_bindgen]
extern "C" {
    /// Configuration object used to construct an `IpfsClient`.
    ///
    /// The public TypeScript facade accepts optional `ipfsUri`, an explicit
    /// `transport`, optional `transportPolicy`, and default cancellation
    /// settings.
    #[wasm_bindgen(typescript_type = "IpfsClientConfig")]
    pub type IpfsClientConfig;
}

/// IPFS app-data client backed by an explicitly configured HTTP transport.
///
/// Construct this client when JavaScript needs to fetch app-data documents by
/// CID or app-data hash while preserving SDK retry, timeout, and cancellation
/// behavior.
#[wasm_bindgen]
pub struct IpfsClient {
    adapter: IpfsHttpAdapter,
    ipfs_uri: Option<String>,
    _callback_guard: crate::exports::registry::FetchCallbackGuard,
}

#[wasm_bindgen]
impl IpfsClient {
    /// Creates an IPFS app-data client from a single config object.
    ///
    /// The config must include `transport`. Optional `ipfsUri` overrides the
    /// default gateway base, while timeout, signal, and policy fields become
    /// defaults for method calls.
    ///
    /// @param config IPFS client configuration.
    /// @throws SdkError when transport, policy, timeout, or gateway config is invalid.
    #[wasm_bindgen(constructor)]
    pub fn new(config: IpfsClientConfig) -> Result<IpfsClient, JsValue> {
        let config = config.as_ref();
        let timeout = optional_timeout(config)?;
        let ipfs_uri = optional_string(config, "ipfsUri")?;
        let transport_policy =
            transport_policy_from_config(config, TransportPolicy::default_ipfs(), timeout)?;
        let (transport, callback_guard) = configured_fetch_transport(
            config,
            timeout,
            transport_policy.client_policy().max_response_bytes(),
        )?;
        Ok(Self {
            adapter: IpfsHttpAdapter::from_parts(transport, transport_policy),
            ipfs_uri,
            _callback_guard: callback_guard,
        })
    }

    /// Fetches and parses an app-data document by CID.
    ///
    /// The CID is resolved through the configured gateway and transport. The
    /// returned document is normalized into the SDK app-data DTO shape.
    ///
    /// @param cid Canonical IPFS CID for the app-data document.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing the app-data document.
    /// @throws SdkError for invalid CID, transport failure, timeout, or parse failure.
    #[wasm_bindgen(
        js_name = "fetchAppDataFromCid",
        unchecked_return_type = "WasmEnvelope<AppDataDocDto>"
    )]
    pub async fn fetch_app_data_from_cid(
        &self,
        cid: String,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let adapter = self.adapter.with_call_timeout(scope.timeout());
        let ipfs_uri = self.ipfs_uri.clone();
        run_with_client_options(scope, async move {
            fetch_doc_from_cid_with_adapter(&cid, ipfs_uri.as_deref(), &adapter).await
        })
        .await
    }

    /// Fetches and parses an app-data document by app-data hash.
    ///
    /// The helper converts the app-data hash to the canonical CID before
    /// fetching through the configured gateway.
    ///
    /// @param appDataHex App-data hash as a `0x`-prefixed hex string.
    /// @param options Optional per-call cancellation and timeout settings.
    /// @returns A versioned envelope containing the app-data document.
    /// @throws SdkError for invalid hash, transport failure, timeout, or parse failure.
    #[wasm_bindgen(
        js_name = "fetchAppDataFromHex",
        unchecked_return_type = "WasmEnvelope<AppDataDocDto>"
    )]
    pub async fn fetch_app_data_from_hex(
        &self,
        #[wasm_bindgen(js_name = appDataHex)] app_data_hex: String,
        #[wasm_bindgen(js_name = options)] options: Option<SdkClientOptions>,
    ) -> Result<JsValue, JsValue> {
        let scope = ClientCallScope::new(options.as_ref().map(AsRef::as_ref))?;
        let adapter = self.adapter.with_call_timeout(scope.timeout());
        let ipfs_uri = self.ipfs_uri.clone();
        run_with_client_options(scope, async move {
            fetch_doc_from_hex_with_adapter(&app_data_hex, ipfs_uri.as_deref(), &adapter).await
        })
        .await
    }
}

async fn fetch_doc_from_cid_with_adapter(
    cid: &str,
    ipfs_uri: Option<&str>,
    adapter: &IpfsHttpAdapter,
) -> Result<JsValue, JsValue> {
    let document = pure::app_data::fetch_doc_from_cid(cid, adapter, ipfs_uri)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(AppDataDocDto::from(document)))
}

async fn fetch_doc_from_hex_with_adapter(
    app_data_hex: &str,
    ipfs_uri: Option<&str>,
    adapter: &IpfsHttpAdapter,
) -> Result<JsValue, JsValue> {
    let document = cow_sdk_app_data::fetch_doc_from_app_data_hex(app_data_hex, adapter, ipfs_uri)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(AppDataDocDto::from(document)))
}

fn transport_to_app_data_error(error: TransportError) -> AppDataError {
    match error {
        TransportError::Transport { class, detail } => AppDataError::Transport { class, detail },
        TransportError::Configuration { message } => AppDataError::Transport {
            class: TransportErrorClass::Builder,
            detail: message,
        },
        TransportError::HttpStatus { status, .. } => AppDataError::Transport {
            class: TransportErrorClass::Other,
            detail: Redacted::new(format!("IPFS gateway returned HTTP status {status}")),
        },
        error => AppDataError::Transport {
            class: TransportErrorClass::Other,
            detail: Redacted::new(error.to_string()),
        },
    }
}
