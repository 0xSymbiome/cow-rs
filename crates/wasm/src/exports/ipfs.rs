use std::{
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};

use async_trait::async_trait;
use cow_sdk_app_data::{AppDataError, IpfsFetchTransport};
use cow_sdk_core::{
    CancellationToken, HttpTransport, Redacted, TransportError, TransportErrorClass,
};
use cow_sdk_pure_helpers as pure;
use cow_sdk_transport_policy::{NetworkErrorKind, TransportPolicy, sleep};
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
        let retry = self.transport_policy.retry();
        let rate_limiter = self.transport_policy.rate_limit();

        for attempt_index in 1..=retry.max_attempts() {
            let cancellation_token = CancellationToken::new();
            rate_limiter
                .acquire_global(&cancellation_token)
                .await
                .map_err(|_| AppDataError::Cancelled)?;

            match self
                .inner
                .get(uri, &[], self.transport_policy.timeout())
                .await
            {
                Ok(body) => return Ok(body),
                Err(error) => {
                    let Some(delay) =
                        retry_delay_for_error(&self.transport_policy, &error, attempt_index)
                    else {
                        return Err(transport_to_app_data_error(error));
                    };
                    sleep(delay).await;
                }
            }
        }

        Err(AppDataError::Transport {
            class: TransportErrorClass::Other,
            detail: Redacted::new("IPFS request attempts exhausted".to_owned()),
        })
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IpfsClientConfig")]
    pub type IpfsClientConfig;
}

/// IPFS client backed by an explicitly configured HTTP transport.
#[wasm_bindgen]
pub struct IpfsClient {
    adapter: IpfsHttpAdapter,
    ipfs_uri: Option<String>,
    _callback_guard: crate::exports::registry::FetchCallbackGuard,
}

#[wasm_bindgen]
impl IpfsClient {
    /// Creates an IPFS client from a single config object.
    #[wasm_bindgen(constructor)]
    pub fn new(config: IpfsClientConfig) -> Result<IpfsClient, JsValue> {
        let config = config.as_ref();
        let timeout = optional_timeout(config)?;
        let ipfs_uri = optional_string(config, "ipfsUri")?;
        let transport_policy =
            transport_policy_from_config(config, TransportPolicy::default_ipfs(), timeout)?;
        let (transport, callback_guard) = configured_fetch_transport(config, timeout)?;
        Ok(Self {
            adapter: IpfsHttpAdapter::from_parts(transport, transport_policy),
            ipfs_uri,
            _callback_guard: callback_guard,
        })
    }

    /// Fetches and parses an app-data document by CID.
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

fn retry_delay_for_error(
    transport_policy: &TransportPolicy,
    error: &TransportError,
    attempt_index: usize,
) -> Option<Duration> {
    let retry = transport_policy.retry();
    if attempt_index >= retry.max_attempts() {
        return None;
    }

    match error {
        TransportError::HttpStatus {
            status, headers, ..
        } if retry.should_retry_status(*status) => {
            let headers = headers
                .iter()
                .map(|(name, value)| (name.clone(), value.as_inner().clone()))
                .collect::<Vec<_>>();
            Some(retry.delay_for_status(attempt_index, *status, &headers, UNIX_EPOCH))
        }
        TransportError::Transport { class, .. }
            if retry.should_retry_network(NetworkErrorKind::from_transport_error_class(*class)) =>
        {
            Some(retry.delay_for_attempt(attempt_index))
        }
        _ => None,
    }
}
