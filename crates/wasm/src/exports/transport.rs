use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use cow_sdk_core::{HttpTransport, Redacted, TransportError, TransportErrorClass};
use cow_sdk_transport_wasm::{FetchTransport, FetchTransportConfig};
use js_sys::{Function, Object, Reflect};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::JsFuture;

use crate::exports::{
    dto::CowFetchResponse,
    errors::WasmError,
    registry::{
        FetchCallbackHandle, FetchCallbackHandleId, HANDLE_ID_RESERVED_INVALID,
        lookup_fetch_callback, register_fetch_callback,
    },
};

/// HTTP transport that dispatches requests through a registered JS callback.
#[derive(Debug, Clone)]
pub struct JsCallbackHttpTransport {
    base_url: Redacted<String>,
    timeout: Option<Duration>,
    callback_id: FetchCallbackHandleId,
}

impl JsCallbackHttpTransport {
    /// Creates a callback-backed HTTP transport.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError::Configuration`] when the callback id is invalid.
    pub fn new(
        base_url: String,
        callback_id: FetchCallbackHandleId,
        timeout: Option<Duration>,
    ) -> Result<Self, TransportError> {
        if callback_id.0 == HANDLE_ID_RESERVED_INVALID {
            return Err(TransportError::Configuration {
                message: Redacted::new(
                    "fetch callback handle id 0 is reserved as invalid".to_owned(),
                ),
            });
        }

        Ok(Self {
            base_url: Redacted::new(base_url.trim_end_matches('/').to_owned()),
            timeout,
            callback_id,
        })
    }

    async fn dispatch(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        let callback = lookup_fetch_callback(self.callback_id).ok_or_else(|| {
            TransportError::Configuration {
                message: Redacted::new("fetch callback handle is disposed or invalid".to_owned()),
            }
        })?;
        let url = self.resolve_url(path);
        let effective_timeout = timeout.or(self.timeout);
        let abort_controller = GlobalAbortController::new();
        let signal = abort_controller.signal();
        let timer_ms = effective_timeout
            .map(timeout_ms_or_configuration_error)
            .transpose()?;
        let mut timer = timer_ms.map_or_else(TimerGuard::empty, |ms| {
            schedule_abort_timer(&abort_controller, ms)
        });
        let request_dto = build_request_dto(method, &url, body, headers, timer_ms, &signal)?;
        let value = callback
            .call1(&JsValue::NULL, &request_dto)
            .map_err(map_callback_throw_to_transport)?;
        let promise = js_sys::Promise::resolve(&value);
        let response_value = JsFuture::from(promise)
            .await
            .map_err(map_callback_reject_to_transport)?;
        timer.clear();
        parse_callback_response(response_value)
    }

    fn resolve_url(&self, path: &str) -> String {
        if path.starts_with("http://")
            || path.starts_with("https://")
            || self.base_url.as_inner().is_empty()
        {
            path.to_owned()
        } else if path.starts_with('/') {
            format!("{}{}", self.base_url.as_inner(), path)
        } else {
            format!("{}/{}", self.base_url.as_inner(), path)
        }
    }
}

#[async_trait(?Send)]
impl HttpTransport for JsCallbackHttpTransport {
    async fn get(
        &self,
        path: &str,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        self.dispatch("GET", path, None, headers, timeout).await
    }

    async fn post(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        self.dispatch("POST", path, Some(body), headers, timeout)
            .await
    }

    async fn put(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        self.dispatch("PUT", path, Some(body), headers, timeout)
            .await
    }

    async fn delete(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        self.dispatch("DELETE", path, Some(body), headers, timeout)
            .await
    }
}

pub(crate) fn default_fetch_transport(
    timeout: Option<Duration>,
) -> Arc<dyn HttpTransport + Send + Sync> {
    let mut config = FetchTransportConfig::new("");
    if let Some(timeout) = timeout {
        config = config.with_timeout(timeout);
    }
    Arc::new(FetchTransport::new(&config))
}

pub(crate) fn callback_fetch_transport(
    callback: Function,
    timeout: Option<Duration>,
) -> Result<(Arc<dyn HttpTransport + Send + Sync>, FetchCallbackHandle), JsValue> {
    let handle = register_fetch_callback(callback)?;
    let transport = callback_fetch_transport_from_handle_id(handle.handle_id(), timeout)?;
    Ok((transport, handle))
}

pub(crate) fn callback_fetch_transport_from_handle(
    handle_id: u32,
    timeout: Option<Duration>,
) -> Result<Arc<dyn HttpTransport + Send + Sync>, JsValue> {
    callback_fetch_transport_from_handle_id(FetchCallbackHandleId::new(handle_id)?, timeout)
}

fn callback_fetch_transport_from_handle_id(
    handle_id: FetchCallbackHandleId,
    timeout: Option<Duration>,
) -> Result<Arc<dyn HttpTransport + Send + Sync>, JsValue> {
    let transport = JsCallbackHttpTransport::new(String::new(), handle_id, timeout)
        .map_err(|error| WasmError::from(error).into_js())?;
    Ok(Arc::new(transport))
}

struct TimerGuard {
    handle: Option<JsValue>,
    on_timeout: Option<Closure<dyn FnMut()>>,
}

impl TimerGuard {
    const fn empty() -> Self {
        Self {
            handle: None,
            on_timeout: None,
        }
    }

    fn from_parts(handle: JsValue, on_timeout: Closure<dyn FnMut()>) -> Self {
        Self {
            handle: Some(handle),
            on_timeout: Some(on_timeout),
        }
    }

    fn clear(&mut self) {
        if let Some(handle) = self.handle.take() {
            global_clear_timeout_raw(&handle);
        }
        drop(self.on_timeout.take());
    }
}

impl Drop for TimerGuard {
    fn drop(&mut self) {
        self.clear();
    }
}

fn timeout_ms_or_configuration_error(duration: Duration) -> Result<u32, TransportError> {
    let millis = duration.as_millis();
    if millis > i32::MAX as u128 {
        return Err(TransportError::Configuration {
            message: Redacted::new(format!(
                "timeout {millis} ms exceeds the supported setTimeout range"
            )),
        });
    }
    Ok(millis as u32)
}

fn schedule_abort_timer(controller: &GlobalAbortController, ms: u32) -> TimerGuard {
    let controller_clone = controller.clone();
    let on_timeout = Closure::<dyn FnMut()>::new(move || {
        controller_clone.abort();
    });
    let handle = global_set_timeout_raw(on_timeout.as_ref().unchecked_ref(), ms);
    TimerGuard::from_parts(handle, on_timeout)
}

fn build_request_dto(
    method: &str,
    url: &str,
    body: Option<&str>,
    headers: &[(String, String)],
    timeout_ms: Option<u32>,
    signal: &web_sys::AbortSignal,
) -> Result<JsValue, TransportError> {
    let dto = Object::new();
    reflect_set(&dto, "method", &JsValue::from_str(method))?;
    reflect_set(&dto, "url", &JsValue::from_str(url))?;

    let headers_obj = Object::new();
    for (name, value) in headers {
        reflect_set(&headers_obj, name, &JsValue::from_str(value))?;
    }
    reflect_set(&dto, "headers", &headers_obj)?;

    if let Some(body) = body {
        reflect_set(&dto, "body", &JsValue::from_str(body))?;
    }
    if let Some(timeout_ms) = timeout_ms {
        reflect_set(&dto, "timeoutMs", &JsValue::from_f64(f64::from(timeout_ms)))?;
    }
    Reflect::set(&dto, &"signal".into(), signal.as_ref()).map_err(map_reflect_error)?;

    Ok(dto.into())
}

fn reflect_set(target: &Object, key: &str, value: &JsValue) -> Result<(), TransportError> {
    Reflect::set(target, &JsValue::from_str(key), value)
        .map(|_| ())
        .map_err(map_reflect_error)
}

fn map_reflect_error(error: JsValue) -> TransportError {
    TransportError::Configuration {
        message: Redacted::new(format!(
            "could not build fetch callback request: {}",
            js_message(&error)
        )),
    }
}

fn map_callback_throw_to_transport(error: JsValue) -> TransportError {
    TransportError::Transport {
        class: TransportErrorClass::Connect,
        detail: Redacted::new(js_message(&error)),
    }
}

fn map_callback_reject_to_transport(error: JsValue) -> TransportError {
    let class = if is_abort_error(&error) {
        TransportErrorClass::Timeout
    } else {
        TransportErrorClass::Connect
    };
    TransportError::Transport {
        class,
        detail: Redacted::new(js_message(&error)),
    }
}

fn parse_callback_response(value: JsValue) -> Result<String, TransportError> {
    let response: CowFetchResponse =
        serde_wasm_bindgen::from_value(value).map_err(|error| TransportError::Transport {
            class: TransportErrorClass::Decode,
            detail: Redacted::new(format!(
                "fetch callback returned malformed response: {error}"
            )),
        })?;

    if (200..300).contains(&response.status) {
        Ok(response.body)
    } else {
        Err(TransportError::HttpStatus {
            status: response.status,
            headers: redact_response_headers(response.headers),
            body: Redacted::new(response.body),
        })
    }
}

fn redact_response_headers(headers: HashMap<String, String>) -> Vec<(String, Redacted<String>)> {
    headers
        .into_iter()
        .map(|(name, value)| (name, Redacted::new(value)))
        .collect()
}

fn is_abort_error(value: &JsValue) -> bool {
    Reflect::get(value, &JsValue::from_str("name"))
        .ok()
        .and_then(|name| name.as_string())
        .is_some_and(|name| name == "AbortError")
}

fn js_message(value: &JsValue) -> String {
    if let Some(message) = Reflect::get(value, &JsValue::from_str("message"))
        .ok()
        .and_then(|message| message.as_string())
    {
        return message;
    }
    value
        .as_string()
        .unwrap_or_else(|| "JavaScript callback failed".to_owned())
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = globalThis, js_name = AbortController)]
    #[derive(Clone)]
    type GlobalAbortController;

    #[wasm_bindgen(constructor, js_namespace = globalThis, js_class = AbortController)]
    fn new() -> GlobalAbortController;

    #[wasm_bindgen(method, getter, js_class = "AbortController")]
    fn signal(this: &GlobalAbortController) -> web_sys::AbortSignal;

    #[wasm_bindgen(method, js_class = "AbortController")]
    fn abort(this: &GlobalAbortController);

    #[wasm_bindgen(js_namespace = globalThis, js_name = setTimeout)]
    fn global_set_timeout_raw(handler: &Function, ms: u32) -> JsValue;

    #[wasm_bindgen(js_namespace = globalThis, js_name = clearTimeout)]
    fn global_clear_timeout_raw(handle: &JsValue);
}
