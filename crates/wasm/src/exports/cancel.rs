use std::{future::Future, time::Duration};

#[cfg(feature = "transport-policy")]
use cow_sdk_core::transport::policy::TransportPolicy;
use cow_sdk_core::{Cancellable, CancellationToken};
use js_sys::Reflect;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};

use crate::exports::errors::WasmError;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "SdkClientOptions")]
    pub type SdkClientOptions;

    #[wasm_bindgen(typescript_type = "SigningOptions")]
    pub type SigningOptions;
}

/// Per-call bridge from a JavaScript `AbortSignal` to the SDK cancellation token.
pub struct AbortBridge {
    token: CancellationToken,
    signal: Option<web_sys::AbortSignal>,
    closure: Option<Closure<dyn FnMut()>>,
}

impl AbortBridge {
    /// Creates a bridge for one async call.
    #[must_use]
    pub fn new(signal: Option<web_sys::AbortSignal>) -> Self {
        let token = CancellationToken::new();
        let mut bridge = Self {
            token: token.clone(),
            signal: None,
            closure: None,
        };

        let Some(signal) = signal else {
            return bridge;
        };

        if signal.aborted() {
            token.cancel();
            return bridge;
        }

        let token_for_abort = token.clone();
        let closure = Closure::wrap(Box::new(move || {
            token_for_abort.cancel();
        }) as Box<dyn FnMut()>);

        match signal.add_event_listener_with_callback("abort", closure.as_ref().unchecked_ref()) {
            Ok(()) => {
                bridge.signal = Some(signal);
                bridge.closure = Some(closure);
            }
            Err(error) => {
                web_sys::console::warn_1(
                    &format!(
                        "AbortSignal listener registration failed; cancellation degraded: {}",
                        js_message(&error)
                    )
                    .into(),
                );
            }
        }

        bridge
    }

    /// Returns the cancellation token mirrored by this bridge.
    #[must_use]
    pub fn token(&self) -> CancellationToken {
        self.token.clone()
    }

    /// Returns whether the mirrored token has already been cancelled.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }
}

impl Drop for AbortBridge {
    fn drop(&mut self) {
        if let (Some(signal), Some(closure)) = (&self.signal, &self.closure) {
            let _ = signal
                .remove_event_listener_with_callback("abort", closure.as_ref().unchecked_ref());
        }
        drop(self.closure.take());
    }
}

/// Parsed per-call options plus the owned cancellation bridge for the call.
pub(crate) struct ClientCallScope {
    bridge: AbortBridge,
    #[cfg_attr(not(feature = "transport-policy"), allow(dead_code))]
    timeout: Option<Duration>,
}

impl ClientCallScope {
    pub(crate) fn new(options: Option<&JsValue>) -> Result<Self, JsValue> {
        let signal = options.map(optional_signal).transpose()?.flatten();
        let bridge = AbortBridge::new(signal);
        if bridge.is_cancelled() {
            return Err(WasmError::cancelled().into_js());
        }

        let timeout = options
            .map(|options| optional_timeout_ms(options, "timeoutMs"))
            .transpose()?
            .flatten();
        let timeout = duration_from_timeout_ms(timeout)?;

        Ok(Self { bridge, timeout })
    }

    #[cfg_attr(not(feature = "transport-policy"), allow(dead_code))]
    pub(crate) const fn timeout(&self) -> Option<Duration> {
        self.timeout
    }

    fn token(&self) -> CancellationToken {
        self.bridge.token()
    }
}

pub(crate) async fn run_with_client_options<F>(
    scope: ClientCallScope,
    future: F,
) -> Result<JsValue, JsValue>
where
    F: Future<Output = Result<JsValue, JsValue>>,
{
    let token = scope.token();
    let result = async { future.await.map_err(JsCallError) }
        .cancel_with(&token)
        .await;
    drop(scope);
    result.map_err(JsCallError::into_js)
}

#[cfg(feature = "transport-policy")]
pub(crate) fn transport_policy_with_timeout(
    policy: &TransportPolicy,
    timeout: Option<Duration>,
) -> TransportPolicy {
    match timeout {
        Some(timeout) => policy
            .clone()
            .with_client_policy(policy.client_policy().clone().with_timeout(timeout)),
        None => policy.clone(),
    }
}

pub(crate) fn signing_wallet_timeout_ms(options: Option<&JsValue>) -> Result<Option<u32>, JsValue> {
    let Some(options) = options else {
        return Ok(None);
    };
    let Some(wallet_config) = optional_value(options, "walletConfig")? else {
        return Ok(None);
    };
    if !wallet_config.is_object() {
        return Err(WasmError::invalid("walletConfig", "expected an object").into_js());
    }
    optional_timeout_ms(&wallet_config, "timeoutMs")
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

pub(crate) fn optional_timeout_ms(
    value: &JsValue,
    field: &'static str,
) -> Result<Option<u32>, JsValue> {
    optional_value(value, field)?
        .map(|value| parse_u32_field(value, field))
        .transpose()
}

fn optional_signal(value: &JsValue) -> Result<Option<web_sys::AbortSignal>, JsValue> {
    optional_value(value, "signal")?
        .map(|value| {
            value
                .dyn_into::<web_sys::AbortSignal>()
                .map_err(|_| WasmError::invalid("signal", "expected an AbortSignal").into_js())
        })
        .transpose()
}

fn optional_value(value: &JsValue, field: &'static str) -> Result<Option<JsValue>, JsValue> {
    let value = Reflect::get(value, &JsValue::from_str(field))
        .map_err(|error| WasmError::invalid(field, js_message(&error)).into_js())?;
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

fn js_message(value: &JsValue) -> String {
    Reflect::get(value, &JsValue::from_str("message"))
        .ok()
        .and_then(|message| message.as_string())
        .or_else(|| value.as_string())
        .unwrap_or_else(|| "JavaScript operation failed".to_owned())
}

struct JsCallError(JsValue);

impl JsCallError {
    fn into_js(self) -> JsValue {
        self.0
    }
}

impl From<cow_sdk_core::Cancelled> for JsCallError {
    fn from(_: cow_sdk_core::Cancelled) -> Self {
        Self(WasmError::cancelled().into_js())
    }
}
