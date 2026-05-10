use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use cow_sdk_app_data::{AppDataError, IpfsFetchTransport};
use cow_sdk_core::{HttpTransport, Redacted, TransportError, TransportErrorClass};
use cow_sdk_pure_helpers as pure;
use wasm_bindgen::prelude::*;

use crate::exports::{
    dto::{AppDataDocDto, to_js_value},
    errors::WasmError,
    transport::{configured_fetch_transport, optional_string, optional_timeout},
};

/// Adapter that lets app-data IPFS reads flow through an HTTP transport.
pub(crate) struct IpfsHttpAdapter {
    inner: Arc<dyn HttpTransport + Send + Sync>,
    timeout: Option<Duration>,
}

impl IpfsHttpAdapter {
    fn from_parts(inner: Arc<dyn HttpTransport + Send + Sync>, timeout: Option<Duration>) -> Self {
        Self { inner, timeout }
    }
}

#[async_trait(?Send)]
impl IpfsFetchTransport for IpfsHttpAdapter {
    async fn get(&self, uri: &str) -> Result<String, AppDataError> {
        self.inner
            .get(uri, &[], self.timeout)
            .await
            .map_err(transport_to_app_data_error)
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
        let (transport, callback_guard) = configured_fetch_transport(config, timeout)?;
        Ok(Self {
            adapter: IpfsHttpAdapter::from_parts(transport, timeout),
            ipfs_uri,
            _callback_guard: callback_guard,
        })
    }

    /// Fetches and parses an app-data document by CID.
    #[wasm_bindgen(js_name = "fetchAppDataFromCid")]
    pub async fn fetch_app_data_from_cid(&self, cid: String) -> Result<JsValue, JsValue> {
        fetch_doc_from_cid_with_adapter(&cid, self.ipfs_uri.as_deref(), &self.adapter).await
    }

    /// Fetches and parses an app-data document by app-data hash.
    #[wasm_bindgen(js_name = "fetchAppDataFromHex")]
    pub async fn fetch_app_data_from_hex(&self, app_data_hex: String) -> Result<JsValue, JsValue> {
        fetch_doc_from_hex_with_adapter(&app_data_hex, self.ipfs_uri.as_deref(), &self.adapter)
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
    to_js_value(&AppDataDocDto::from(document))
}

async fn fetch_doc_from_hex_with_adapter(
    app_data_hex: &str,
    ipfs_uri: Option<&str>,
    adapter: &IpfsHttpAdapter,
) -> Result<JsValue, JsValue> {
    let document = cow_sdk_app_data::fetch_doc_from_app_data_hex(app_data_hex, adapter, ipfs_uri)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&AppDataDocDto::from(document))
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
