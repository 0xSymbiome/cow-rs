//! Browser [`HttpTransport`] implementation backed by the realm's global `fetch`.
//!
//! [`FetchTransport`] dispatches REST requests through the realm's global
//! `fetch` function — present on a `Window` or a worker global scope — and
//! bridges the returned `Promise` to a `Future` via
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
//! request and holds the abort handle across both the `fetch()` promise and
//! the response-body read. The configured [`Duration`] therefore bounds the
//! full request-response lifecycle, including headers and body bytes. A
//! stalled body that exceeds the configured timeout is aborted and surfaces as
//! [`TransportErrorClass::Timeout`]. Cancellation drops the owned callback
//! closure registered with `setTimeout`, so long-lived browser sessions do
//! not accumulate dead timeout callbacks.
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

// DO NOT SWAP for any alloy transport.
//
// alloy ships no browser-fetch transport. The alloy transport stack
// (`alloy_transport_http`, `alloy_transport`, etc.) wraps
// `tower::Service` over JSON-RPC packet types and hard-depends on
// `tokio` for `Service::poll_ready` — both incompatible with the
// `wasm32-unknown-unknown` target. Swapping would force a tokio
// runtime into the browser bundle and explode the bundle size
// budget pinned in ADR 0044.
//
// The `AbortController` lifecycle in this module (declared at the
// `use web_sys::AbortController` import below and wired through the
// dispatch path and the abort-timeout helper) is the cow-owned
// timeout-cancellation seam. The per-call timeout contract
// documented above is part of the cow public API; the alloy
// ecosystem does not own this seam.
//
// ADR: docs/adr/0010-runtime-neutral-async-and-transport-posture.md
// (lines 19-31),
// docs/adr/0046-transport-policy-js-exposure.md (lines 25-31).
// Doctrine: docs/alloy-doctrine.md, Bucket 2 row for Browser
// `FetchTransport` with `AbortController` lifecycle.
// Enforced by cargo check-source-fences (xtask/src/policy/fences.rs).

#[cfg(feature = "tracing")]
use std::borrow::Cow;
use std::time::Duration;

use async_trait::async_trait;
use cow_sdk_core::{
    DEFAULT_MAX_RESPONSE_BYTES, HttpTransport, Redacted, TransportError, TransportErrorClass,
    TransportResponse,
};
use js_sys::{Array, Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{AbortController, Headers, Request, RequestInit, Response};

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

/// Browser fetch-based [`HttpTransport`] implementation.
///
/// The transport is cheap to clone: the base URL and timeout are stored
/// alongside each handle and every dispatch call re-reads the global `fetch`
/// from the current realm, so consumers can cache the instance per client
/// without worrying about cross-realm retention. Reading `fetch` from the
/// global scope rather than `window()` lets the same transport run on a
/// `Window` or a worker, not the main thread alone.
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
        cow_sdk_core::transport::join_request_url(self.base_url.as_inner(), path)
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

            let endpoint = span_endpoint(path);
            let bytes_sent = body.map_or(0, str::len);
            let span = tracing::info_span!(
                "transport.dispatch",
                chain = "wasm32",
                method = method,
                endpoint = endpoint.as_ref(),
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
        let effective_timeout = timeout.or(self.timeout);
        let mut abort_timeout = match effective_timeout {
            Some(timeout) => Some(install_abort_timeout(&init, timeout)?),
            None => None,
        };
        let request = match Request::new_with_str_and_init(&url, &init) {
            Ok(request) => request,
            Err(error) => {
                if let Some(handle) = abort_timeout.take() {
                    handle.cancel();
                }
                return Err(configuration_error("could not build fetch request", &error));
            }
        };
        let fetch_invocation = match fetch.call1(&global, request.as_ref()) {
            Ok(value) => value,
            Err(error) => {
                if let Some(handle) = abort_timeout.take() {
                    handle.cancel();
                }
                return Err(classify_fetch_rejection(&error));
            }
        };
        let response_value = match JsFuture::from(Promise::resolve(&fetch_invocation)).await {
            Ok(value) => value,
            Err(error) => {
                if let Some(handle) = abort_timeout.take() {
                    handle.cancel();
                }
                return Err(classify_fetch_rejection(&error));
            }
        };
        let response: Response = match response_value.dyn_into() {
            Ok(response) => response,
            Err(_) => {
                if let Some(handle) = abort_timeout.take() {
                    handle.cancel();
                }
                return Err(decode_error(
                    "fetch returned a value that was not a Response",
                ));
            }
        };
        let status = response.status();
        let headers = response_headers(&response.headers());
        let text_promise = match response.text() {
            Ok(promise) => promise,
            Err(error) => {
                if let Some(handle) = abort_timeout.take() {
                    handle.cancel();
                }
                return Err(body_error("could not read response body", &error));
            }
        };
        let body_result = JsFuture::from(text_promise).await;
        if let Some(handle) = abort_timeout.take() {
            handle.cancel();
        }
        let text_value =
            body_result.map_err(|error| body_error("could not decode response body", &error))?;
        let body_text = text_value
            .as_string()
            .ok_or_else(|| decode_error("response body was not a string"))?;
        // The browser fetch has already materialized the full body into a JS
        // string by this point, so this bound refuses to hand an oversized body
        // to the rest of the SDK rather than capping the read mid-stream; the
        // browser's single-request model keeps the residual allocation small.
        // The limit bounds decoded bytes.
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

#[cfg(feature = "tracing")]
fn span_endpoint(path: &str) -> Cow<'_, str> {
    let has_authority = path.contains("://");
    let endpoint = path.find("://").map_or(path, |scheme_end| {
        let after_authority = &path[scheme_end + 3..];
        after_authority
            .find('/')
            .map_or("/", |path_start| &after_authority[path_start..])
    });
    let end = endpoint.find(['?', '#']).unwrap_or(endpoint.len());
    let endpoint = &endpoint[..end];
    if endpoint.is_empty() && has_authority {
        Cow::Borrowed("/")
    } else {
        Cow::Borrowed(endpoint)
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

/// Resolves the global `fetch` function from the active realm.
///
/// The lookup goes through [`js_sys::global`] rather than `web_sys::window()`
/// so the transport serves any realm that exposes `fetch` on its global scope
/// — a `Window` or a worker — not the main thread alone. The function is
/// invoked with the global object as `this`. A realm without a global `fetch`
/// returns a typed [`TransportError::Configuration`].
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

struct AbortTimeoutHandle {
    controller: AbortController,
    timer: gloo_timers::callback::Timeout,
}

impl AbortTimeoutHandle {
    fn cancel(self) {
        let Self { controller, timer } = self;
        drop(timer);
        drop(controller);
    }
}

fn install_abort_timeout(
    init: &RequestInit,
    timeout: Duration,
) -> Result<AbortTimeoutHandle, TransportError> {
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
    let controller_clone = controller.clone();
    // `gloo_timers::callback::Timeout::new` requires a `FnOnce() + 'static`
    // closure. `'static` is mandatory because the timer outlives the
    // enclosing stack frame; `Send` is irrelevant on wasm32 (single-
    // threaded) but the move-capture of `controller_clone` keeps the
    // closure naturally `Send`-shaped on any future native-thread port.
    let timer = gloo_timers::callback::Timeout::new(ms, move || {
        controller_clone.abort();
    });
    Ok(AbortTimeoutHandle { controller, timer })
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
