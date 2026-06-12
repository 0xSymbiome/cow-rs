use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use cow_sdk_core::{
    HttpTransport, Redacted, TransportError, TransportErrorClass, TransportResponse,
};
use js_sys::{Array, Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::JsFuture;

use crate::exports::{
    dto::CowFetchResponse,
    errors::WasmError,
    registry::{
        FetchCallbackGuard, FetchCallbackKey, lookup_fetch_callback, register_fetch_adapter,
        register_fetch_callback,
    },
};

/// HTTP transport that dispatches requests through a registered JS callback.
#[derive(Debug, Clone)]
pub struct JsCallbackHttpTransport {
    base_url: Redacted<String>,
    timeout: Option<Duration>,
    callback_id: FetchCallbackKey,
    max_response_bytes: usize,
}

impl JsCallbackHttpTransport {
    /// Creates a callback-backed HTTP transport.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError::Configuration`] when the callback id is invalid.
    pub(crate) fn new(
        base_url: String,
        callback_id: FetchCallbackKey,
        timeout: Option<Duration>,
        max_response_bytes: usize,
    ) -> Result<Self, TransportError> {
        if callback_id.raw() == 0 {
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
            max_response_bytes,
        })
    }

    async fn dispatch(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError> {
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
        parse_callback_response(response_value, self.max_response_bytes)
    }

    fn resolve_url(&self, path: &str) -> String {
        cow_sdk_core::transport::join_request_url(self.base_url.as_inner(), path)
    }
}

#[async_trait(?Send)]
impl HttpTransport for JsCallbackHttpTransport {
    async fn get(
        &self,
        path: &str,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError> {
        self.dispatch("GET", path, None, headers, timeout).await
    }

    async fn post(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError> {
        self.dispatch("POST", path, Some(body), headers, timeout)
            .await
    }

    async fn put(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError> {
        self.dispatch("PUT", path, Some(body), headers, timeout)
            .await
    }

    async fn delete(
        &self,
        path: &str,
        body: &str,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError> {
        self.dispatch("DELETE", path, Some(body), headers, timeout)
            .await
    }
}

pub(crate) fn callback_fetch_transport(
    callback: Function,
    timeout: Option<Duration>,
    max_response_bytes: usize,
) -> Result<(Arc<dyn HttpTransport + Send + Sync>, FetchCallbackGuard), JsValue> {
    let guard = register_fetch_callback(callback)?;
    let transport =
        callback_fetch_transport_from_handle_id(guard.id(), timeout, max_response_bytes)?;
    Ok((transport, guard))
}

pub(crate) fn fetch_adapter_transport(
    fetch: Function,
    timeout: Option<Duration>,
    max_response_bytes: usize,
) -> Result<(Arc<dyn HttpTransport + Send + Sync>, FetchCallbackGuard), JsValue> {
    let guard = register_fetch_adapter(fetch)?;
    let transport =
        callback_fetch_transport_from_handle_id(guard.id(), timeout, max_response_bytes)?;
    Ok((transport, guard))
}

fn callback_fetch_transport_from_handle_id(
    handle_id: FetchCallbackKey,
    timeout: Option<Duration>,
    max_response_bytes: usize,
) -> Result<Arc<dyn HttpTransport + Send + Sync>, JsValue> {
    let transport =
        JsCallbackHttpTransport::new(String::new(), handle_id, timeout, max_response_bytes)
            .map_err(|error| WasmError::from(error).into_js())?;
    Ok(Arc::new(transport))
}

pub(crate) fn configured_fetch_transport(
    config: &JsValue,
    timeout: Option<Duration>,
    max_response_bytes: usize,
) -> Result<(Arc<dyn HttpTransport + Send + Sync>, FetchCallbackGuard), JsValue> {
    let transport = required_object(config, "transport")?;
    let kind = required_string(&transport, "kind")?;

    match kind.as_str() {
        "callback" => {
            let callback = required_function(&transport, "callback")?;
            callback_fetch_transport(callback, timeout, max_response_bytes)
        }
        "fetch" => {
            let fetch = optional_function(&transport, "fetch")?
                .or_else(global_fetch_function)
                .ok_or_else(|| {
                    WasmError::invalid(
                        "transport.fetch",
                        "globalThis.fetch is unavailable; pass an explicit fetch function",
                    )
                    .into_js()
                })?;
            fetch_adapter_transport(fetch, timeout, max_response_bytes)
        }
        other => Err(WasmError::invalid(
            "transport.kind",
            format!("unsupported transport kind `{other}`"),
        )
        .into_js()),
    }
}

pub(crate) fn duration_from_timeout_ms(
    timeout_ms: Option<u32>,
) -> Result<Option<Duration>, JsValue> {
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

pub(crate) fn optional_timeout(config: &JsValue) -> Result<Option<Duration>, JsValue> {
    duration_from_timeout_ms(optional_u32(config, "timeoutMs")?)
}

pub(crate) fn required_u32(config: &JsValue, field: &'static str) -> Result<u32, JsValue> {
    let value = required_value(config, field)?;
    parse_u32_field(value, field)
}

pub(crate) fn optional_u32(config: &JsValue, field: &'static str) -> Result<Option<u32>, JsValue> {
    optional_value(config, field)?
        .map(|value| parse_u32_field(value, field))
        .transpose()
}

pub(crate) fn required_string(config: &JsValue, field: &'static str) -> Result<String, JsValue> {
    required_value(config, field)?
        .as_string()
        .ok_or_else(|| WasmError::invalid(field, "expected a string").into_js())
}

pub(crate) fn optional_string(
    config: &JsValue,
    field: &'static str,
) -> Result<Option<String>, JsValue> {
    optional_value(config, field)?
        .map(|value| {
            value
                .as_string()
                .ok_or_else(|| WasmError::invalid(field, "expected a string").into_js())
        })
        .transpose()
}

fn required_object(config: &JsValue, field: &'static str) -> Result<JsValue, JsValue> {
    let value = required_value(config, field)?;
    if value.is_object() {
        Ok(value)
    } else {
        Err(WasmError::invalid(field, "expected an object").into_js())
    }
}

fn required_function(config: &JsValue, field: &'static str) -> Result<Function, JsValue> {
    let value = required_value(config, field)?;
    value
        .dyn_into::<Function>()
        .map_err(|_| WasmError::invalid(field, "expected a function").into_js())
}

fn optional_function(config: &JsValue, field: &'static str) -> Result<Option<Function>, JsValue> {
    optional_value(config, field)?
        .map(|value| {
            value
                .dyn_into::<Function>()
                .map_err(|_| WasmError::invalid(field, "expected a function").into_js())
        })
        .transpose()
}

fn required_value(config: &JsValue, field: &'static str) -> Result<JsValue, JsValue> {
    optional_value(config, field)?
        .ok_or_else(|| WasmError::invalid(field, "missing required field").into_js())
}

fn optional_value(config: &JsValue, field: &'static str) -> Result<Option<JsValue>, JsValue> {
    let value = Reflect::get(config, &JsValue::from_str(field))
        .map_err(|error| WasmError::from(map_reflect_error(error)).into_js())?;
    if value.is_undefined() || value.is_null() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

fn parse_u32_field(value: JsValue, field: &'static str) -> Result<u32, JsValue> {
    let number = value
        .as_f64()
        .ok_or_else(|| WasmError::invalid(field, "expected a number").into_js())?;
    if !number.is_finite() || number.fract() != 0.0 || number < 0.0 || number > f64::from(u32::MAX)
    {
        return Err(WasmError::invalid(field, "expected an unsigned 32-bit integer").into_js());
    }
    Ok(number as u32)
}

fn global_fetch_function() -> Option<Function> {
    Reflect::get(&js_sys::global(), &JsValue::from_str("fetch"))
        .ok()
        .and_then(|value| value.dyn_into::<Function>().ok())
}

pub(crate) fn dispatch_fetch_adapter(fetch: &Function, request: JsValue) -> JsValue {
    let fetch = fetch.clone();
    wasm_bindgen_futures::future_to_promise(
        async move { fetch_adapter_response(fetch, request).await },
    )
    .into()
}

async fn fetch_adapter_response(fetch: Function, request: JsValue) -> Result<JsValue, JsValue> {
    let url = required_string(&request, "url")?;
    let init = Object::new();
    set_if_present(&init, "method", &request, "method")?;
    set_if_present(&init, "headers", &request, "headers")?;
    set_if_present(&init, "body", &request, "body")?;
    set_if_present(&init, "signal", &request, "signal")?;

    let fetch_result = fetch.call2(&JsValue::NULL, &JsValue::from_str(&url), init.as_ref())?;
    let fetch_promise = Promise::resolve(&fetch_result);
    let response = JsFuture::from(fetch_promise).await?;
    let response: web_sys::Response = response.dyn_into().map_err(|_| {
        JsValue::from_str("fetch transport returned a value that is not a Response")
    })?;
    let status = response.status();
    let status_text = response.status_text();
    let headers = headers_to_object(response.headers())?;
    let text = response.text()?;
    let body = JsFuture::from(text).await?;
    build_callback_response_object(status, &status_text, &headers, body)
}

fn build_callback_response_object(
    status: u16,
    status_text: &str,
    headers: &Object,
    body: JsValue,
) -> Result<JsValue, JsValue> {
    let dto = Object::new();
    reflect_set(&dto, "status", &JsValue::from_f64(f64::from(status)))
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    if !status_text.is_empty() {
        reflect_set(&dto, "statusText", &JsValue::from_str(status_text))
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
    }
    reflect_set(&dto, "headers", headers.as_ref())
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    reflect_set(&dto, "body", &body).map_err(|error| JsValue::from_str(&error.to_string()))?;
    Ok(dto.into())
}

fn headers_to_object(headers: web_sys::Headers) -> Result<Object, JsValue> {
    let object = Object::new();
    for entry in Array::from(headers.as_ref()).iter() {
        let pair = Array::from(&entry);
        if pair.length() < 2 {
            continue;
        }
        let Some(name) = pair.get(0).as_string() else {
            continue;
        };
        let Some(value) = pair.get(1).as_string() else {
            continue;
        };
        reflect_set(&object, &name, &JsValue::from_str(&value)).map_err(|error| {
            JsValue::from_str(&format!("could not copy fetch response header: {error}"))
        })?;
    }
    Ok(object)
}

fn set_if_present(
    target: &Object,
    target_key: &str,
    source: &JsValue,
    source_key: &'static str,
) -> Result<(), JsValue> {
    if let Some(value) = optional_value(source, source_key)? {
        reflect_set(target, target_key, &value)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
    }
    Ok(())
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

fn parse_callback_response(
    value: JsValue,
    max_response_bytes: usize,
) -> Result<TransportResponse, TransportError> {
    let response: CowFetchResponse =
        serde_wasm_bindgen::from_value(value).map_err(|error| TransportError::Transport {
            class: TransportErrorClass::Decode,
            detail: Redacted::new(format!(
                "fetch callback returned malformed response: {error}"
            )),
        })?;

    // The JS callback has already materialized the full body into a string
    // before the SDK sees it, so this bound refuses to process an oversized
    // body rather than capping the read mid-stream; the JS layer owns its own
    // pre-materialization bound. The limit bounds decoded bytes.
    if response.body.len() > max_response_bytes {
        return Err(TransportError::Transport {
            class: TransportErrorClass::ResponseTooLarge,
            detail: Redacted::new(format!(
                "response body exceeded {max_response_bytes} byte limit"
            )),
        });
    }

    if (200..300).contains(&response.status) {
        Ok(TransportResponse::new(
            response.status,
            redact_response_headers(response.headers),
            response.body,
        ))
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
