//! Browser [`HttpTransport`] backed by the realm's global `fetch`.
//!
//! [`FetchTransport`] dispatches each request through the global `fetch` —
//! resolved from [`js_sys::global`], so it runs on a `Window` or a worker — and
//! bridges the returned `Promise` to a `Future`. Failures surface through
//! [`TransportError`] with the same [`TransportErrorClass`] taxonomy as the
//! native `ReqwestTransport`; a non-2xx response surfaces through
//! [`TransportError::HttpStatus`], carrying the status, headers, and body.
//!
//! A per-call timeout wires an [`web_sys::AbortController`] into the request and
//! bounds the whole request-response lifecycle, including the body read; an
//! exceeded timeout surfaces as [`TransportErrorClass::Timeout`]. The base URL
//! is held in [`crate::Redacted`] so it never reaches `Debug`, `Display`, or
//! serde output. Redirects follow the browser default.

use std::time::Duration;

use async_trait::async_trait;
use js_sys::{Array, Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{AbortController, Headers, Request, RequestInit, Response};

use crate::{
    DEFAULT_MAX_RESPONSE_BYTES, HttpTransport, Redacted, TransportError, TransportErrorClass,
    TransportResponse,
};

/// Configuration bundle for [`FetchTransport`].
///
/// The base URL is wrapped in [`Redacted`] so it is never emitted through
/// debug, display, or serde representations of the configuration value.
#[derive(Debug, Clone)]
pub struct FetchTransportConfig {
    base_url: Redacted<String>,
    timeout: Option<Duration>,
    max_response_bytes: usize,
}

impl FetchTransportConfig {
    /// Creates a configuration bundle with the supplied base URL and no
    /// request timeout.
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: Redacted::new(base_url.into()),
            timeout: None,
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
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

    /// Returns a copy of this configuration with an explicit maximum
    /// response-body size, in bytes.
    #[must_use]
    pub const fn with_max_response_bytes(mut self, max_response_bytes: usize) -> Self {
        self.max_response_bytes = max_response_bytes;
        self
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

    /// Returns the configured maximum response-body size, in bytes.
    #[must_use]
    pub const fn max_response_bytes(&self) -> usize {
        self.max_response_bytes
    }
}

/// Browser fetch-based [`HttpTransport`].
///
/// Cheap to clone: each dispatch re-reads the global `fetch` from the current
/// realm, so an instance can be cached per client and used on a `Window` or a
/// worker alike.
#[derive(Debug, Clone)]
pub struct FetchTransport {
    base_url: Redacted<String>,
    timeout: Option<Duration>,
    max_response_bytes: usize,
}

impl FetchTransport {
    /// Builds a transport from the supplied configuration.
    #[must_use]
    pub fn new(config: &FetchTransportConfig) -> Self {
        let trimmed = config.base_url.as_inner().trim_end_matches('/').to_owned();
        Self {
            base_url: Redacted::new(trimmed),
            timeout: config.timeout,
            max_response_bytes: config.max_response_bytes,
        }
    }

    /// Returns the configured base URL for deliberate inspection.
    #[must_use]
    pub fn base_url(&self) -> &str {
        self.base_url.as_inner()
    }

    fn resolve_url(&self, path: &str) -> String {
        super::join_request_url(self.base_url.as_inner(), path)
    }

    async fn dispatch(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError> {
        #[cfg(feature = "tracing")]
        {
            use tracing::Instrument as _;

            let endpoint = super::span_endpoint(path);
            let bytes_sent = body.map_or(0, str::len);
            let span = tracing::info_span!(
                target: "cow_sdk::transport",
                "transport.dispatch",
                chain = "wasm32",
                method = method,
                endpoint = endpoint,
                bytes_sent = bytes_sent as u64,
                bytes_received = tracing::field::Empty,
            );
            let recorder = span.clone();
            async move {
                let result = self
                    .dispatch_request(method, path, body, headers, timeout)
                    .await;
                if let Some(bytes_received) = bytes_received(&result) {
                    recorder.record("bytes_received", bytes_received as u64);
                }
                result
            }
            .instrument(span)
            .await
        }

        #[cfg(not(feature = "tracing"))]
        {
            self.dispatch_request(method, path, body, headers, timeout)
                .await
        }
    }

    async fn dispatch_request(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
        headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<TransportResponse, TransportError> {
        let url = self.resolve_url(path);
        let (global, fetch) = global_fetch_or_configuration_error()?;
        let init = build_request_init(method, body, headers)?;
        // RAII guard: dropping it cancels the timer (gloo calls `clearTimeout`)
        // on every exit path, so no timeout callback outlives its request.
        let _abort_timeout = match timeout.or(self.timeout) {
            Some(timeout) => Some(install_abort_timeout(&init, timeout)?),
            None => None,
        };
        let request = Request::new_with_str_and_init(&url, &init)
            .map_err(|error| configuration_error("could not build fetch request", &error))?;
        let fetch_invocation = fetch
            .call1(&global, request.as_ref())
            .map_err(|error| classify_fetch_rejection(&error))?;
        let response_value = JsFuture::from(Promise::resolve(&fetch_invocation))
            .await
            .map_err(|error| classify_fetch_rejection(&error))?;
        let response: Response = response_value
            .dyn_into()
            .map_err(|_| decode_error("fetch returned a value that was not a Response"))?;
        let status = response.status();
        let headers = response_headers(&response.headers());
        let text_promise = response
            .text()
            .map_err(|error| body_error("could not read response body", &error))?;
        let body_text = JsFuture::from(text_promise)
            .await
            .map_err(|error| body_error("could not decode response body", &error))?
            .as_string()
            .ok_or_else(|| decode_error("response body was not a string"))?;
        // The body is already fully materialized in JS here, so the cap rejects
        // an oversized body rather than capping mid-stream; it bounds decoded bytes.
        if body_text.len() > self.max_response_bytes {
            return Err(TransportError::Transport {
                class: TransportErrorClass::ResponseTooLarge,
                detail: Redacted::new(format!(
                    "response body exceeded {} byte limit",
                    self.max_response_bytes
                )),
            });
        }
        if (200..300).contains(&status) {
            Ok(TransportResponse::new(status, headers, body_text))
        } else {
            Err(TransportError::HttpStatus {
                status,
                headers,
                body: Redacted::new(body_text),
            })
        }
    }
}

#[cfg(feature = "tracing")]
fn bytes_received(result: &Result<TransportResponse, TransportError>) -> Option<usize> {
    match result {
        Ok(response) => Some(response.body().len()),
        Err(TransportError::HttpStatus { body, .. }) => Some(body.as_inner().len()),
        Err(_) => None,
    }
}

#[async_trait(?Send)]
impl HttpTransport for FetchTransport {
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

/// Resolves the global `fetch` from the active realm via [`js_sys::global`], so
/// it serves a `Window` or a worker alike, and invokes it with the global as
/// `this`. Returns a typed [`TransportError::Configuration`] when no global
/// `fetch` exists.
fn global_fetch_or_configuration_error() -> Result<(JsValue, Function), TransportError> {
    let global: JsValue = js_sys::global().into();
    let fetch = Reflect::get(&global, &JsValue::from_str("fetch"))
        .ok()
        .and_then(|value| value.dyn_into::<Function>().ok())
        .ok_or_else(|| TransportError::Configuration {
            message: Redacted::new(
                "no global `fetch` function is available in this JavaScript realm".to_owned(),
            ),
        })?;
    Ok((global, fetch))
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

fn install_abort_timeout(
    init: &RequestInit,
    timeout: Duration,
) -> Result<gloo_timers::callback::Timeout, TransportError> {
    let controller = AbortController::new().map_err(|error| {
        configuration_error("could not build an AbortController for the timeout", &error)
    })?;
    init.set_signal(Some(&controller.signal()));
    let ms_u128 = timeout.as_millis();
    let ms = u32::try_from(ms_u128).map_err(|_| TransportError::Configuration {
        message: Redacted::new(format!(
            "timeout {ms_u128} ms exceeds the supported browser setTimeout range"
        )),
    })?;
    // The `Timeout` owns the closure, which owns the controller, so both live
    // until the caller drops the guard; gloo's `Timeout::drop` then calls
    // `clearTimeout`, cancelling the pending abort.
    Ok(gloo_timers::callback::Timeout::new(ms, move || {
        controller.abort();
    }))
}

fn classify_fetch_rejection(error: &JsValue) -> TransportError {
    let (class, detail) = classify_dom_exception(error);
    TransportError::Transport {
        class,
        detail: Redacted::new(detail),
    }
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
        message: Redacted::new(format!("{context}: {}", redacted_error_render(error))),
    }
}

fn body_error(context: &str, error: &JsValue) -> TransportError {
    let (class, detail) = classify_dom_exception(error);
    let class = if matches!(class, TransportErrorClass::Timeout) {
        TransportErrorClass::Timeout
    } else {
        TransportErrorClass::Body
    };
    TransportError::Transport {
        class,
        detail: Redacted::new(format!("{context}: {detail}")),
    }
}

fn decode_error(context: &str) -> TransportError {
    TransportError::Transport {
        class: TransportErrorClass::Decode,
        detail: Redacted::new(context.to_owned()),
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

fn response_headers(headers: &Headers) -> Vec<(String, Redacted<String>)> {
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
        collected.push((name, Redacted::new(value)));
    }

    collected
}
