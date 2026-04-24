//! Browser [`HttpTransport`] implementation backed by `web-sys::fetch`.
//!
//! [`FetchTransport`] dispatches REST requests through the browser's native
//! `fetch` API and bridges the returned `Promise` to a `Future` via
//! [`wasm_bindgen_futures::JsFuture`]. Every failure surfaces through the
//! shared [`TransportError`] enum with the same [`TransportErrorClass`]
//! taxonomy that the native [`cow_sdk_core::ReqwestTransport`] uses, so
//! consumers that partition telemetry or shape retry policy on the class
//! value observe identical behavior across runtimes.
//!
//! Non-2xx responses surface through [`TransportError::HttpStatus`] with the
//! numeric status code, response headers, and raw response body so
//! downstream crates receive the HTTP-status context through the typed error
//! channel instead of through an `Ok(String)` success path.
//!
//! # Per-call header and timeout contract
//!
//! Per-call headers are merged onto the [`web_sys::Request`] header set
//! before the browser dispatches the request. An `Option<Duration>` per-call
//! timeout overrides the transport's constructor-configured default; a
//! `Some` timeout wires an [`web_sys::AbortController`] into the in-flight
//! request so the resulting `AbortError` surfaces as
//! [`TransportErrorClass::Timeout`].
//!
//! # URL redaction
//!
//! The configured base URL is held in [`cow_sdk_core::Redacted`] so it never
//! appears in [`std::fmt::Debug`], [`std::fmt::Display`], or serde output,
//! matching the native default.
//!
//! # Redirect handling
//!
//! The transport uses the browser's default `redirect: "follow"` fetch mode,
//! so the `fetch` call resolves to the final destination response after the
//! browser has walked every intermediate redirect. Redirect-chain failures
//! surface as `TypeError`-shaped DOMExceptions classified through
//! [`TransportErrorClass::Connect`], consistent with the browser platform
//! contract. Callers that need manual redirect inspection run the request
//! through their own fetch bridge rather than through this default adapter.

use std::time::Duration;

use async_trait::async_trait;
use cow_sdk_core::{HttpTransport, Redacted, TransportError, TransportErrorClass};
use js_sys::{Array, Object, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{AbortController, Headers, Request, RequestInit, Response, Window};

/// Configuration bundle for [`FetchTransport`].
///
/// The base URL is wrapped in [`Redacted`] so it is never emitted through
/// debug, display, or serde representations of the configuration value.
#[derive(Debug, Clone)]
pub struct FetchTransportConfig {
    base_url: Redacted<String>,
    timeout: Option<Duration>,
}

impl FetchTransportConfig {
    /// Creates a configuration bundle with the supplied base URL and no
    /// request timeout.
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: Redacted::new(base_url.into()),
            timeout: None,
        }
    }

    /// Returns a copy of this configuration with an explicit timeout.
    ///
    /// A non-zero timeout wires an [`AbortController`] into the in-flight
    /// request. The resulting `AbortError` surfaces as
    /// [`TransportErrorClass::Timeout`].
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Returns a copy of this configuration with the supplied timeout in
    /// milliseconds.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError::Configuration`] when `timeout_ms` exceeds
    /// the browser-side `setTimeout` range of a signed 32-bit integer.
    pub fn try_with_timeout_ms(mut self, timeout_ms: u64) -> Result<Self, TransportError> {
        if timeout_ms > i32::MAX as u64 {
            return Err(TransportError::Configuration {
                message: format!(
                    "timeout {timeout_ms} ms exceeds the supported browser setTimeout range"
                ),
            });
        }
        self.timeout = Some(Duration::from_millis(timeout_ms));
        Ok(self)
    }

    /// Returns the configured base URL for deliberate inspection.
    #[must_use]
    pub fn base_url(&self) -> &str {
        self.base_url.as_inner()
    }

    /// Returns the configured request timeout if one is set.
    #[must_use]
    pub const fn timeout(&self) -> Option<Duration> {
        self.timeout
    }
}

/// Browser fetch-based [`HttpTransport`] implementation.
///
/// The transport is cheap to clone: the base URL and timeout are stored
/// alongside each handle and every dispatch call re-reads `window.fetch`
/// from the current realm, so consumers can cache the instance per client
/// without worrying about cross-realm retention.
#[derive(Debug, Clone)]
pub struct FetchTransport {
    base_url: Redacted<String>,
    timeout: Option<Duration>,
}

impl FetchTransport {
    /// Builds a transport from the supplied configuration.
    #[must_use]
    pub fn new(config: &FetchTransportConfig) -> Self {
        let trimmed = config.base_url.as_inner().trim_end_matches('/').to_owned();
        Self {
            base_url: Redacted::new(trimmed),
            timeout: config.timeout,
        }
    }

    /// Returns the configured base URL for deliberate inspection.
    #[must_use]
    pub fn base_url(&self) -> &str {
        self.base_url.as_inner()
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

    async fn dispatch(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        let url = self.resolve_url(path);
        let window = window_or_configuration_error()?;
        let init = build_request_init(method, body, headers)?;
        let effective_timeout = timeout.or(self.timeout);
        let abort_timeout = if let Some(timeout) = effective_timeout {
            Some(install_abort_timeout(&window, &init, timeout)?)
        } else {
            None
        };
        let request = Request::new_with_str_and_init(&url, &init)
            .map_err(|error| configuration_error("could not build fetch request", &error))?;
        let response_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|error| classify_fetch_rejection(&error))?;
        if let Some(handle) = abort_timeout {
            handle.cancel(&window);
        }
        let response: Response = response_value
            .dyn_into()
            .map_err(|_| decode_error("fetch returned a value that was not a Response"))?;
        let status = response.status();
        let headers = response_headers(&response.headers());
        let text_promise = response
            .text()
            .map_err(|error| body_error("could not read response body", &error))?;
        let text_value = JsFuture::from(text_promise)
            .await
            .map_err(|error| body_error("could not decode response body", &error))?;
        let body_text = text_value
            .as_string()
            .ok_or_else(|| decode_error("response body was not a string"))?;
        if (200..300).contains(&status) {
            Ok(body_text)
        } else {
            Err(TransportError::HttpStatus {
                status,
                headers,
                body: body_text,
            })
        }
    }
}

#[async_trait(?Send)]
impl HttpTransport for FetchTransport {
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

fn window_or_configuration_error() -> Result<Window, TransportError> {
    web_sys::window().ok_or_else(|| TransportError::Configuration {
        message: "fetch requires a browser window; no global window is available".to_owned(),
    })
}

fn build_request_init(
    method: &str,
    body: Option<&str>,
    headers: &[(String, String)],
) -> Result<RequestInit, TransportError> {
    let init = RequestInit::new();
    init.set_method(method);
    let header_object = Headers::new()
        .map_err(|error| configuration_error("could not build request headers", &error))?;
    let mut has_content_type = false;
    for (name, value) in headers {
        if name.eq_ignore_ascii_case("content-type") {
            has_content_type = true;
        }
        header_object
            .set(name, value)
            .map_err(|error| configuration_error("could not set request header", &error))?;
    }
    if let Some(body) = body {
        if !has_content_type {
            header_object
                .set("Content-Type", "application/json")
                .map_err(|error| configuration_error("could not set Content-Type", &error))?;
        }
        init.set_body(&JsValue::from_str(body));
    }
    init.set_headers(&header_object);
    Ok(init)
}

struct AbortTimeoutHandle {
    controller: AbortController,
    timeout_id: i32,
}

impl AbortTimeoutHandle {
    fn cancel(self, window: &Window) {
        window.clear_timeout_with_handle(self.timeout_id);
        drop(self.controller);
    }
}

fn install_abort_timeout(
    window: &Window,
    init: &RequestInit,
    timeout: Duration,
) -> Result<AbortTimeoutHandle, TransportError> {
    let controller = AbortController::new().map_err(|error| {
        configuration_error("could not build an AbortController for the timeout", &error)
    })?;
    init.set_signal(Some(&controller.signal()));
    let ms = timeout.as_millis();
    let ms = i32::try_from(ms).map_err(|_| TransportError::Configuration {
        message: format!("timeout {ms} ms exceeds the supported browser setTimeout range"),
    })?;
    let controller_clone = controller.clone();
    let abort_callback = wasm_bindgen::closure::Closure::once_into_js(move || {
        controller_clone.abort();
    });
    let timeout_id = window
        .set_timeout_with_callback_and_timeout_and_arguments_0(abort_callback.unchecked_ref(), ms)
        .map_err(|error| configuration_error("could not schedule the timeout callback", &error))?;
    Ok(AbortTimeoutHandle {
        controller,
        timeout_id,
    })
}

fn classify_fetch_rejection(error: &JsValue) -> TransportError {
    let (class, detail) = classify_dom_exception(error);
    TransportError::Transport { class, detail }
}

fn classify_dom_exception(error: &JsValue) -> (TransportErrorClass, String) {
    let name = reflect_string(error, "name").unwrap_or_default();
    let message = reflect_string(error, "message").unwrap_or_else(|| redacted_error_render(error));
    let class = match name.as_str() {
        "AbortError" | "TimeoutError" => TransportErrorClass::Timeout,
        "NetworkError" | "TypeError" => TransportErrorClass::Connect,
        "SyntaxError" => TransportErrorClass::Decode,
        _ => TransportErrorClass::Other,
    };
    (
        class,
        format!("{name}: {message}")
            .trim_start_matches(": ")
            .to_owned(),
    )
}

fn configuration_error(context: &str, error: &JsValue) -> TransportError {
    TransportError::Configuration {
        message: format!("{context}: {}", redacted_error_render(error)),
    }
}

fn body_error(context: &str, error: &JsValue) -> TransportError {
    TransportError::Transport {
        class: TransportErrorClass::Body,
        detail: format!("{context}: {}", redacted_error_render(error)),
    }
}

fn decode_error(context: &str) -> TransportError {
    TransportError::Transport {
        class: TransportErrorClass::Decode,
        detail: context.to_owned(),
    }
}

fn redacted_error_render(error: &JsValue) -> String {
    reflect_string(error, "message")
        .or_else(|| error.as_string())
        .unwrap_or_else(|| "<opaque JsValue>".to_owned())
}

fn reflect_string(source: &JsValue, key: &str) -> Option<String> {
    let key_value = JsValue::from_str(key);
    Reflect::get(source, &key_value)
        .ok()
        .and_then(|value| value.as_string())
        .or_else(|| {
            source
                .dyn_ref::<Object>()
                .and_then(|object| Reflect::get(object, &key_value).ok())
                .and_then(|value| value.as_string())
        })
}

fn response_headers(headers: &Headers) -> Vec<(String, String)> {
    let entries = Array::from(headers.as_ref());
    let mut collected = Vec::with_capacity(entries.length() as usize);

    for index in 0..entries.length() {
        let pair = Array::from(&entries.get(index));
        let Some(name) = pair.get(0).as_string() else {
            continue;
        };
        let Some(value) = pair.get(1).as_string() else {
            continue;
        };
        collected.push((name, value));
    }

    collected
}
