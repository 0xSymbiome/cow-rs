use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use cow_sdk_app_data::{AppDataError, IpfsFetchTransport};
use cow_sdk_core::{HttpTransport, Redacted, TransportError, TransportErrorClass};
use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::exports::{
    dto::{AppDataDocDto, to_js_value},
    errors::WasmError,
    transport::{
        callback_fetch_transport, callback_fetch_transport_from_handle, default_fetch_transport,
    },
};

/// Adapter that lets app-data IPFS reads flow through an HTTP transport.
#[wasm_bindgen]
pub struct HttpToIpfsAdapter {
    inner: Arc<dyn HttpTransport + Send + Sync>,
    timeout: Option<Duration>,
    _handle: Option<crate::exports::registry::FetchCallbackHandle>,
}

#[wasm_bindgen]
impl HttpToIpfsAdapter {
    /// Creates an adapter that owns a registered fetch callback.
    #[wasm_bindgen(constructor)]
    pub fn new(
        fetch_callback: Function,
        timeout_ms: Option<u32>,
    ) -> Result<HttpToIpfsAdapter, JsValue> {
        let timeout = duration_from_timeout_ms(timeout_ms)?;
        let (transport, handle) = callback_fetch_transport(fetch_callback, timeout)?;
        Ok(Self::from_parts(transport, timeout, Some(handle)))
    }

    /// Creates an adapter from an existing fetch-callback handle id.
    #[wasm_bindgen(js_name = "fromHandle")]
    pub fn from_handle(
        fetch_callback_id: u32,
        timeout_ms: Option<u32>,
    ) -> Result<HttpToIpfsAdapter, JsValue> {
        let timeout = duration_from_timeout_ms(timeout_ms)?;
        let transport = callback_fetch_transport_from_handle(fetch_callback_id, timeout)?;
        Ok(Self::from_parts(transport, timeout, None))
    }

    /// Fetches and parses an app-data document by CID.
    #[wasm_bindgen(js_name = "fetchAppDataFromCid")]
    pub async fn fetch_app_data_from_cid(
        &self,
        cid: String,
        ipfs_uri: Option<String>,
    ) -> Result<JsValue, JsValue> {
        fetch_doc_from_cid_with_adapter(&cid, ipfs_uri.as_deref(), self).await
    }

    /// Fetches and parses an app-data document by app-data hash.
    #[wasm_bindgen(js_name = "fetchAppDataFromHex")]
    pub async fn fetch_app_data_from_hex(
        &self,
        app_data_hex: String,
        ipfs_uri: Option<String>,
    ) -> Result<JsValue, JsValue> {
        fetch_doc_from_hex_with_adapter(&app_data_hex, ipfs_uri.as_deref(), self).await
    }
}

impl HttpToIpfsAdapter {
    fn from_parts(
        inner: Arc<dyn HttpTransport + Send + Sync>,
        timeout: Option<Duration>,
        handle: Option<crate::exports::registry::FetchCallbackHandle>,
    ) -> Self {
        Self {
            inner,
            timeout,
            _handle: handle,
        }
    }
}

#[async_trait(?Send)]
impl IpfsFetchTransport for HttpToIpfsAdapter {
    async fn get(&self, uri: &str) -> Result<String, AppDataError> {
        self.inner
            .get(uri, &[], self.timeout)
            .await
            .map_err(transport_to_app_data_error)
    }
}

/// IPFS client backed by the browser fetch transport.
#[wasm_bindgen]
pub struct IpfsClient {
    adapter: HttpToIpfsAdapter,
    ipfs_uri: Option<String>,
}

#[wasm_bindgen]
impl IpfsClient {
    /// Creates an IPFS client with the default browser fetch transport.
    #[wasm_bindgen(constructor)]
    pub fn new(ipfs_uri: Option<String>, timeout_ms: Option<u32>) -> Result<IpfsClient, JsValue> {
        let timeout = duration_from_timeout_ms(timeout_ms)?;
        Ok(Self {
            adapter: HttpToIpfsAdapter::from_parts(default_fetch_transport(timeout), timeout, None),
            ipfs_uri,
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

/// IPFS client backed by a JavaScript fetch callback.
#[wasm_bindgen]
pub struct IpfsClientWithFetch {
    adapter: HttpToIpfsAdapter,
    ipfs_uri: Option<String>,
}

#[wasm_bindgen]
impl IpfsClientWithFetch {
    /// Creates an IPFS client that owns a registered fetch callback.
    #[wasm_bindgen(constructor)]
    pub fn new(
        ipfs_uri: Option<String>,
        timeout_ms: Option<u32>,
        fetch_callback: Function,
    ) -> Result<IpfsClientWithFetch, JsValue> {
        let timeout = duration_from_timeout_ms(timeout_ms)?;
        let (transport, handle) = callback_fetch_transport(fetch_callback, timeout)?;
        Ok(Self {
            adapter: HttpToIpfsAdapter::from_parts(transport, timeout, Some(handle)),
            ipfs_uri,
        })
    }

    /// Creates an IPFS client from an existing fetch-callback handle id.
    #[wasm_bindgen(js_name = "fromHandle")]
    pub fn from_handle(
        ipfs_uri: Option<String>,
        timeout_ms: Option<u32>,
        fetch_callback_id: u32,
    ) -> Result<IpfsClientWithFetch, JsValue> {
        let timeout = duration_from_timeout_ms(timeout_ms)?;
        let transport = callback_fetch_transport_from_handle(fetch_callback_id, timeout)?;
        Ok(Self {
            adapter: HttpToIpfsAdapter::from_parts(transport, timeout, None),
            ipfs_uri,
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

/// Fetches and parses an app-data document by CID.
#[wasm_bindgen(js_name = "fetchAppDataFromCid")]
pub async fn fetch_app_data_from_cid(
    cid: String,
    ipfs_uri: Option<String>,
    timeout_ms: Option<u32>,
) -> Result<JsValue, JsValue> {
    let timeout = duration_from_timeout_ms(timeout_ms)?;
    let adapter = HttpToIpfsAdapter::from_parts(default_fetch_transport(timeout), timeout, None);
    fetch_doc_from_cid_with_adapter(&cid, ipfs_uri.as_deref(), &adapter).await
}

/// Fetches and parses an app-data document by app-data hash.
#[wasm_bindgen(js_name = "fetchAppDataFromHex")]
pub async fn fetch_app_data_from_hex(
    app_data_hex: String,
    ipfs_uri: Option<String>,
    timeout_ms: Option<u32>,
) -> Result<JsValue, JsValue> {
    let timeout = duration_from_timeout_ms(timeout_ms)?;
    let adapter = HttpToIpfsAdapter::from_parts(default_fetch_transport(timeout), timeout, None);
    fetch_doc_from_hex_with_adapter(&app_data_hex, ipfs_uri.as_deref(), &adapter).await
}

fn duration_from_timeout_ms(timeout_ms: Option<u32>) -> Result<Option<Duration>, JsValue> {
    match timeout_ms {
        Some(ms) if ms > i32::MAX as u32 => Err(WasmError::invalid(
            "timeoutMs",
            format!("timeout {ms} ms exceeds the supported setTimeout range"),
        )
        .into_js()),
        Some(ms) => Ok(Some(Duration::from_millis(u64::from(ms)))),
        None => Ok(None),
    }
}

async fn fetch_doc_from_cid_with_adapter(
    cid: &str,
    ipfs_uri: Option<&str>,
    adapter: &HttpToIpfsAdapter,
) -> Result<JsValue, JsValue> {
    let document = crate::pure::app_data::fetch_doc_from_cid(cid, adapter, ipfs_uri)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&AppDataDocDto::from(document))
}

async fn fetch_doc_from_hex_with_adapter(
    app_data_hex: &str,
    ipfs_uri: Option<&str>,
    adapter: &HttpToIpfsAdapter,
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
