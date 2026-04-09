#[cfg(target_arch = "wasm32")]
use async_trait::async_trait;
#[cfg(target_arch = "wasm32")]
use js_sys::{Function, Promise, Reflect};
#[cfg(target_arch = "wasm32")]
use serde_json::Value;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

#[cfg(target_arch = "wasm32")]
use crate::{
    BrowserWalletError, Eip1193Transport, InjectedWalletInfo, RpcErrorPayload,
    provider::parse_chain_id_value,
};

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
pub struct InjectedProviderTransport {
    provider: JsValue,
    info: InjectedWalletInfo,
}

#[cfg(target_arch = "wasm32")]
impl InjectedProviderTransport {
    pub fn detect() -> Result<Option<Self>, BrowserWalletError> {
        let window = web_sys::window()
            .ok_or_else(|| BrowserWalletError::js("browser window is unavailable"))?;
        let provider = Reflect::get(window.as_ref(), &JsValue::from_str("ethereum"))
            .map_err(|error| BrowserWalletError::js(js_value_to_string(&error)))?;
        if provider.is_null() || provider.is_undefined() {
            return Ok(None);
        }
        Ok(Some(Self {
            info: detect_wallet_info(&provider),
            provider,
        }))
    }

    pub fn info(&self) -> InjectedWalletInfo {
        self.info.clone()
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl Eip1193Transport for InjectedProviderTransport {
    fn label(&self) -> &str {
        &self.info.provider_label
    }

    async fn request(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value, BrowserWalletError> {
        let payload = js_sys::Object::new();
        Reflect::set(
            &payload,
            &JsValue::from_str("method"),
            &JsValue::from_str(method),
        )
        .map_err(|error| BrowserWalletError::js(js_value_to_string(&error)))?;
        if let Some(params) = &params {
            let params = serde_wasm_bindgen::to_value(params)
                .map_err(|error| BrowserWalletError::serialization(error.to_string()))?;
            Reflect::set(&payload, &JsValue::from_str("params"), &params)
                .map_err(|error| BrowserWalletError::js(js_value_to_string(&error)))?;
        }

        let request = Reflect::get(&self.provider, &JsValue::from_str("request"))
            .map_err(|error| BrowserWalletError::js(js_value_to_string(&error)))?;
        let request = request.dyn_into::<Function>().map_err(|_| {
            BrowserWalletError::js("wallet provider does not expose a callable `request`")
        })?;
        let requested_chain = params
            .as_ref()
            .and_then(|value| value.as_array())
            .and_then(|items| items.first())
            .and_then(|item| item.get("chainId"))
            .and_then(|chain_id| parse_chain_id_value(chain_id, method).ok());
        let result = request
            .call1(&self.provider, &payload)
            .map_err(|error| map_js_error(method, error, requested_chain))?;
        let value = JsFuture::from(Promise::resolve(&result))
            .await
            .map_err(|error| map_js_error(method, error, requested_chain))?;
        serde_wasm_bindgen::from_value(value.clone()).map_err(|error| {
            BrowserWalletError::malformed_response(
                method,
                format!("failed to deserialize wallet response: {error}"),
            )
        })
    }
}

#[cfg(target_arch = "wasm32")]
fn detect_wallet_info(provider: &JsValue) -> InjectedWalletInfo {
    let is_meta_mask = get_flag(provider, "isMetaMask");
    let is_coinbase_wallet = get_flag(provider, "isCoinbaseWallet");
    let is_rabby = get_flag(provider, "isRabby");
    let provider_label = if is_rabby {
        "Rabby".to_owned()
    } else if is_coinbase_wallet {
        "Coinbase Wallet".to_owned()
    } else if is_meta_mask {
        "MetaMask".to_owned()
    } else {
        "Injected Wallet".to_owned()
    };

    InjectedWalletInfo {
        provider_label,
        is_meta_mask,
        is_coinbase_wallet,
        is_rabby,
    }
}

#[cfg(target_arch = "wasm32")]
fn get_flag(provider: &JsValue, flag: &str) -> bool {
    Reflect::get(provider, &JsValue::from_str(flag))
        .ok()
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

#[cfg(target_arch = "wasm32")]
fn map_js_error(method: &str, error: JsValue, requested_chain: Option<u64>) -> BrowserWalletError {
    let code = Reflect::get(&error, &JsValue::from_str("code"))
        .ok()
        .and_then(|value| value.as_f64())
        .map(|value| value as i32);
    let message = Reflect::get(&error, &JsValue::from_str("message"))
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_else(|| js_value_to_string(&error));
    let data = Reflect::get(&error, &JsValue::from_str("data"))
        .ok()
        .filter(|value| !value.is_null() && !value.is_undefined())
        .and_then(|value| serde_wasm_bindgen::from_value(value).ok());

    if let Some(code) = code {
        return BrowserWalletError::from_rpc(
            method,
            RpcErrorPayload {
                code,
                message,
                data,
            },
            requested_chain,
        );
    }

    BrowserWalletError::js(message)
}

#[cfg(target_arch = "wasm32")]
fn js_value_to_string(value: &JsValue) -> String {
    value.as_string().unwrap_or_else(|| format!("{value:?}"))
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
pub struct InjectedProviderTransport;

#[cfg(not(target_arch = "wasm32"))]
impl InjectedProviderTransport {
    pub fn detect() -> Result<Option<Self>, crate::BrowserWalletError> {
        Ok(None)
    }

    pub fn info(&self) -> crate::InjectedWalletInfo {
        crate::InjectedWalletInfo::default()
    }
}
