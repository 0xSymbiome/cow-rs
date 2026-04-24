//! Browser-only injected-provider discovery and runtime bindings.
//!
//! The JavaScript details in this module remain crate-local implementation support for the typed
//! browser-wallet APIs. They do not widen the public SDK into a generic raw wallet bridge.

#[cfg(target_arch = "wasm32")]
use async_trait::async_trait;
#[cfg(target_arch = "wasm32")]
use js_sys::{Function, Object, Promise, Reflect};
#[cfg(target_arch = "wasm32")]
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use serde_json::Value;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue, closure::Closure, prelude::wasm_bindgen};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

#[cfg(target_arch = "wasm32")]
use crate::{
    BrowserWalletError, Eip1193Transport, EventLog, InjectedWalletDetectionOptions,
    InjectedWalletDiscoverySource, InjectedWalletInfo, RpcErrorPayload, WalletSession,
    events::{
        WalletProviderEvent, WalletRuntimeBinding, WalletRuntimeBindingHandle, apply_provider_event,
    },
    provider::parse_chain_id_value,
};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Eip1193ProviderLike")]
    type Eip1193ProviderBinding;

    #[wasm_bindgen(method, catch, js_name = request)]
    fn request(this: &Eip1193ProviderBinding, payload: &Object) -> Result<Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name = on)]
    fn on(
        this: &Eip1193ProviderBinding,
        event_name: &str,
        callback: &Function,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch, js_name = removeListener)]
    fn remove_listener(
        this: &Eip1193ProviderBinding,
        event_name: &str,
        callback: &Function,
    ) -> Result<JsValue, JsValue>;
}

#[cfg(target_arch = "wasm32")]
/// Injected `window.ethereum` transport backed by one browser provider object.
#[derive(Debug, Clone)]
pub struct InjectedProviderTransport {
    provider: JsValue,
    info: InjectedWalletInfo,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
pub(crate) struct DiscoveredInjectedWallet {
    pub(crate) transport: InjectedProviderTransport,
    pub(crate) info: InjectedWalletInfo,
}

#[cfg(target_arch = "wasm32")]
pub(crate) struct InjectedWalletDiscoveryResult {
    pub(crate) used_legacy_fallback: bool,
    pub(crate) wallets: Vec<DiscoveredInjectedWallet>,
}

#[cfg(target_arch = "wasm32")]
struct ProviderListenerRegistration {
    event_name: &'static str,
    callback: Closure<dyn FnMut(JsValue)>,
}

#[cfg(target_arch = "wasm32")]
struct InjectedProviderSessionBinding {
    provider: JsValue,
    registrations: Vec<ProviderListenerRegistration>,
}

#[cfg(target_arch = "wasm32")]
impl Drop for InjectedProviderSessionBinding {
    fn drop(&mut self) {
        let provider = typed_provider(&self.provider);
        for registration in &self.registrations {
            let _ = provider.remove_listener(
                registration.event_name,
                registration.callback.as_ref().unchecked_ref(),
            );
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl WalletRuntimeBinding for InjectedProviderSessionBinding {}

#[cfg(target_arch = "wasm32")]
impl InjectedProviderTransport {
    pub(crate) fn detect_legacy() -> Result<Option<Self>, BrowserWalletError> {
        let window = browser_window()?;
        let provider = Reflect::get(window.as_ref(), &JsValue::from_str("ethereum"))
            .map_err(|error| BrowserWalletError::js(js_value_to_string(&error)))?;
        if provider.is_null() || provider.is_undefined() {
            return Ok(None);
        }
        Ok(Some(Self::from_provider(
            provider.clone(),
            detect_wallet_info(
                &provider,
                InjectedWalletDiscoverySource::LegacyWindowEthereum,
                None,
            ),
        )))
    }

    fn from_provider(provider: JsValue, info: InjectedWalletInfo) -> Self {
        Self { provider, info }
    }

    fn provider(&self) -> &JsValue {
        &self.provider
    }

    /// Returns discovery metadata for the injected provider.
    #[must_use]
    pub fn info(&self) -> InjectedWalletInfo {
        self.info.clone()
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) async fn discover_injected_wallets(
    options: InjectedWalletDetectionOptions,
) -> Result<InjectedWalletDiscoveryResult, BrowserWalletError> {
    let window = browser_window()?;
    let wallets = std::rc::Rc::new(std::cell::RefCell::new(
        Vec::<DiscoveredInjectedWallet>::new(),
    ));
    let listener_wallets = wallets.clone();

    let listener = Closure::<dyn FnMut(web_sys::Event)>::new(move |event: web_sys::Event| {
        capture_announcement(&listener_wallets, event);
    });

    window
        .add_event_listener_with_callback(
            "eip6963:announceProvider",
            listener.as_ref().unchecked_ref(),
        )
        .map_err(|error| BrowserWalletError::js(js_value_to_string(&error)))?;

    let dispatch_result = request_eip6963_providers(&window);
    let wait_result = if options.timeout_ms() == 0 {
        Ok(())
    } else {
        wait_for_detection_timeout(options.timeout_ms()).await
    };
    let remove_result = window.remove_event_listener_with_callback(
        "eip6963:announceProvider",
        listener.as_ref().unchecked_ref(),
    );
    drop(listener);

    dispatch_result?;
    wait_result?;
    remove_result.map_err(|error| BrowserWalletError::js(js_value_to_string(&error)))?;

    let announced = wallets.borrow().clone();
    if !announced.is_empty() {
        return Ok(InjectedWalletDiscoveryResult {
            used_legacy_fallback: false,
            wallets: announced,
        });
    }

    let Some(transport) = InjectedProviderTransport::detect_legacy()? else {
        return Ok(InjectedWalletDiscoveryResult {
            used_legacy_fallback: false,
            wallets: Vec::new(),
        });
    };

    Ok(InjectedWalletDiscoveryResult {
        used_legacy_fallback: true,
        wallets: vec![DiscoveredInjectedWallet {
            info: transport.info(),
            transport,
        }],
    })
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl Eip1193Transport for InjectedProviderTransport {
    fn label(&self) -> &str {
        &self.info.provider_label
    }

    fn attach_session_sync(
        &self,
        session: std::rc::Rc<std::cell::RefCell<WalletSession>>,
        events: EventLog,
    ) -> Option<WalletRuntimeBindingHandle> {
        let registrations =
            register_session_listeners(typed_provider(&self.provider), session, events).ok()?;
        Some(std::rc::Rc::new(InjectedProviderSessionBinding {
            provider: self.provider.clone(),
            registrations,
        }))
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
            let params = params
                .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
                .map_err(|error| BrowserWalletError::serialization(error.to_string()))?;
            Reflect::set(&payload, &JsValue::from_str("params"), &params)
                .map_err(|error| BrowserWalletError::js(js_value_to_string(&error)))?;
        }

        let requested_chain = params
            .as_ref()
            .and_then(|value| value.as_array())
            .and_then(|items| items.first())
            .and_then(|item| item.get("chainId"))
            .and_then(|chain_id| parse_chain_id_value(chain_id, method).ok());
        let promise = typed_provider(&self.provider)
            .request(&payload)
            .map_err(|error| map_js_error(method, error, requested_chain))?;
        let value = JsFuture::from(promise)
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
fn capture_announcement(
    wallets: &std::rc::Rc<std::cell::RefCell<Vec<DiscoveredInjectedWallet>>>,
    event: web_sys::Event,
) {
    let Ok(custom_event) = event.dyn_into::<web_sys::CustomEvent>() else {
        return;
    };
    let detail = custom_event.detail();
    if detail.is_null() || detail.is_undefined() {
        return;
    }

    let Ok(provider) = Reflect::get(&detail, &JsValue::from_str("provider")) else {
        return;
    };
    if provider.is_null() || provider.is_undefined() {
        return;
    }

    let info_value = Reflect::get(&detail, &JsValue::from_str("info")).ok();
    let info = detect_wallet_info(
        &provider,
        InjectedWalletDiscoverySource::Eip6963,
        info_value.as_ref(),
    );
    let transport = InjectedProviderTransport::from_provider(provider.clone(), info.clone());

    let mut wallets = wallets.borrow_mut();
    if wallets
        .iter()
        .any(|candidate| same_wallet(candidate, &info, &provider))
    {
        return;
    }
    wallets.push(DiscoveredInjectedWallet { transport, info });
}

#[cfg(target_arch = "wasm32")]
fn same_wallet(
    existing: &DiscoveredInjectedWallet,
    incoming_info: &InjectedWalletInfo,
    incoming_provider: &JsValue,
) -> bool {
    match (
        existing.info.provider_uuid.as_ref(),
        incoming_info.provider_uuid.as_ref(),
    ) {
        (Some(existing_uuid), Some(incoming_uuid)) if existing_uuid == incoming_uuid => true,
        _ => Object::is(existing.transport.provider(), incoming_provider),
    }
}

#[cfg(target_arch = "wasm32")]
fn register_session_listeners(
    provider: &Eip1193ProviderBinding,
    session: std::rc::Rc<std::cell::RefCell<WalletSession>>,
    events: EventLog,
) -> Result<Vec<ProviderListenerRegistration>, BrowserWalletError> {
    let mut registrations = Vec::new();

    let accounts_session = session.clone();
    let accounts_events = events.clone();
    let accounts_callback = Closure::<dyn FnMut(JsValue)>::new(move |payload: JsValue| {
        if let Some(event) = parse_accounts_changed_event(payload) {
            apply_provider_event(&accounts_session, &accounts_events, event);
        }
    });
    register_listener(provider, "accountsChanged", &accounts_callback)?;
    registrations.push(ProviderListenerRegistration {
        event_name: "accountsChanged",
        callback: accounts_callback,
    });

    let chain_session = session.clone();
    let chain_events = events.clone();
    let chain_callback = Closure::<dyn FnMut(JsValue)>::new(move |payload: JsValue| {
        if let Some(event) = parse_chain_changed_event(payload) {
            apply_provider_event(&chain_session, &chain_events, event);
        }
    });
    register_listener(provider, "chainChanged", &chain_callback)?;
    registrations.push(ProviderListenerRegistration {
        event_name: "chainChanged",
        callback: chain_callback,
    });

    let connect_session = session.clone();
    let connect_events = events.clone();
    let connect_callback = Closure::<dyn FnMut(JsValue)>::new(move |payload: JsValue| {
        apply_provider_event(
            &connect_session,
            &connect_events,
            WalletProviderEvent::Connected {
                chain_id: parse_connect_chain_id(payload),
            },
        );
    });
    register_listener(provider, "connect", &connect_callback)?;
    registrations.push(ProviderListenerRegistration {
        event_name: "connect",
        callback: connect_callback,
    });

    let disconnect_callback = Closure::<dyn FnMut(JsValue)>::new(move |payload: JsValue| {
        apply_provider_event(
            &session,
            &events,
            WalletProviderEvent::Disconnected {
                message: parse_disconnect_message(payload),
            },
        );
    });
    register_listener(provider, "disconnect", &disconnect_callback)?;
    registrations.push(ProviderListenerRegistration {
        event_name: "disconnect",
        callback: disconnect_callback,
    });

    Ok(registrations)
}

#[cfg(target_arch = "wasm32")]
fn register_listener(
    provider: &Eip1193ProviderBinding,
    event_name: &'static str,
    callback: &Closure<dyn FnMut(JsValue)>,
) -> Result<(), BrowserWalletError> {
    provider
        .on(event_name, callback.as_ref().unchecked_ref())
        .map(|_| ())
        .map_err(|error| BrowserWalletError::js(js_value_to_string(&error)))
}

#[cfg(target_arch = "wasm32")]
fn parse_accounts_changed_event(payload: JsValue) -> Option<WalletProviderEvent> {
    let value: Value = serde_wasm_bindgen::from_value(payload).ok()?;
    let items = value.as_array()?;
    let mut accounts = Vec::with_capacity(items.len());
    for item in items {
        let raw = item.as_str()?;
        accounts.push(cow_sdk_core::Address::new(raw).ok()?);
    }
    Some(WalletProviderEvent::AccountsChanged { accounts })
}

#[cfg(target_arch = "wasm32")]
fn parse_chain_changed_event(payload: JsValue) -> Option<WalletProviderEvent> {
    let value: Value = serde_wasm_bindgen::from_value(payload).ok()?;
    let chain_id = parse_chain_id_value(&value, "chainChanged").ok()?;
    Some(WalletProviderEvent::ChainChanged { chain_id })
}

#[cfg(target_arch = "wasm32")]
fn parse_connect_chain_id(payload: JsValue) -> Option<u64> {
    let value: Value = serde_wasm_bindgen::from_value(payload).ok()?;
    match &value {
        Value::Object(fields) => fields
            .get("chainId")
            .and_then(|chain_id| parse_chain_id_value(chain_id, "connect").ok()),
        _ => parse_chain_id_value(&value, "connect").ok(),
    }
}

#[cfg(target_arch = "wasm32")]
fn parse_disconnect_message(payload: JsValue) -> Option<String> {
    let value: Value = serde_wasm_bindgen::from_value(payload).ok()?;
    match value {
        Value::String(message) if !message.trim().is_empty() => Some(message),
        Value::Object(fields) => fields
            .get("message")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|message| !message.is_empty())
            .map(str::to_owned),
        _ => None,
    }
}

#[cfg(target_arch = "wasm32")]
fn detect_wallet_info(
    provider: &JsValue,
    discovery_source: InjectedWalletDiscoverySource,
    announced_info: Option<&JsValue>,
) -> InjectedWalletInfo {
    let is_meta_mask = get_flag(provider, "isMetaMask");
    let is_coinbase_wallet = get_flag(provider, "isCoinbaseWallet");
    let is_rabby = get_flag(provider, "isRabby");
    let provider_label = announced_info
        .and_then(|info| get_string(info, "name"))
        .unwrap_or_else(|| provider_label_from_flags(is_meta_mask, is_coinbase_wallet, is_rabby));

    InjectedWalletInfo::new(
        provider_label,
        discovery_source,
        announced_info.and_then(|info| get_string(info, "uuid")),
        announced_info.and_then(|info| get_string(info, "rdns")),
        announced_info.and_then(|info| get_string(info, "icon")),
        is_meta_mask,
        is_coinbase_wallet,
        is_rabby,
    )
}

#[cfg(target_arch = "wasm32")]
fn provider_label_from_flags(
    is_meta_mask: bool,
    is_coinbase_wallet: bool,
    is_rabby: bool,
) -> String {
    if is_rabby {
        "Rabby".to_owned()
    } else if is_coinbase_wallet {
        "Coinbase Wallet".to_owned()
    } else if is_meta_mask {
        "MetaMask".to_owned()
    } else {
        "Injected Wallet".to_owned()
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
fn get_string(value: &JsValue, field: &str) -> Option<String> {
    Reflect::get(value, &JsValue::from_str(field))
        .ok()
        .and_then(|value| value.as_string())
        .filter(|value| !value.trim().is_empty())
}

#[cfg(target_arch = "wasm32")]
fn request_eip6963_providers(window: &web_sys::Window) -> Result<(), BrowserWalletError> {
    let event = web_sys::CustomEvent::new("eip6963:requestProvider")
        .map_err(|error| BrowserWalletError::js(js_value_to_string(&error)))?;
    window
        .dispatch_event(&event)
        .map_err(|error| BrowserWalletError::js(js_value_to_string(&error)))?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn typed_provider(provider: &JsValue) -> &Eip1193ProviderBinding {
    provider.unchecked_ref::<Eip1193ProviderBinding>()
}

#[cfg(target_arch = "wasm32")]
async fn wait_for_detection_timeout(timeout_ms: u32) -> Result<(), BrowserWalletError> {
    let window = browser_window()?;
    let promise = Promise::new(&mut move |resolve, reject| {
        let callback = Closure::once_into_js(move || {
            let _ = resolve.call0(&JsValue::UNDEFINED);
        });
        if let Err(error) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.unchecked_ref(),
            timeout_ms as i32,
        ) {
            let _ = reject.call1(&JsValue::UNDEFINED, &error);
        }
    });
    JsFuture::from(promise)
        .await
        .map(|_| ())
        .map_err(|error| BrowserWalletError::js(js_value_to_string(&error)))
}

#[cfg(target_arch = "wasm32")]
fn browser_window() -> Result<web_sys::Window, BrowserWalletError> {
    web_sys::window().ok_or_else(|| BrowserWalletError::js("browser window is unavailable"))
}

#[cfg(target_arch = "wasm32")]
fn map_js_error(method: &str, error: JsValue, requested_chain: Option<u64>) -> BrowserWalletError {
    // Provider error payloads remain explicitly dynamic because injected-wallet runtimes may add
    // vendor-specific fields beyond the standardized RPC code and message shape.
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
            RpcErrorPayload::new(code, message, data),
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
/// Non-WASM placeholder for the injected-provider transport type.
#[derive(Debug, Clone)]
pub struct InjectedProviderTransport;

#[cfg(not(target_arch = "wasm32"))]
impl InjectedProviderTransport {
    pub(crate) const fn detect_legacy() -> Option<Self> {
        None
    }

    /// Returns default injected-wallet metadata on non-WASM targets.
    #[must_use]
    pub fn info(&self) -> crate::InjectedWalletInfo {
        crate::InjectedWalletInfo::default()
    }
}
