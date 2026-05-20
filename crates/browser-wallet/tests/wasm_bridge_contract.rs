#![cfg(target_arch = "wasm32")]

use cow_sdk_browser_wallet::{
    BrowserWallet, BrowserWalletError, InjectedWalletDetectionOptions,
    InjectedWalletDiscoverySource, InjectedWalletInfo, MockRequestRecord, Origin, WalletEvent,
};
use cow_sdk_core::{
    Address, AsyncSigner, SupportedChainId, TypedDataDomain, TypedDataField, TypedDataPayload,
    TypedDataTypes,
};
use js_sys::{Array, Object, Reflect};
use serde::Serialize;
use serde_json::{Value, json};
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

const ACCOUNT: &str = "0x4444444444444444444444444444444444444444";
const ALTERNATE_ACCOUNT: &str = "0x5555555555555555555555555555555555555555";

#[wasm_bindgen(inline_js = r#"
const deepClone = (value) => value === undefined ? null : JSON.parse(JSON.stringify(value));

function normalizeParams(params) {
  if (params === undefined) {
    return null;
  }
  if (Array.isArray(params)) {
    return deepClone(params);
  }
  if (params && typeof params === "object") {
    const numericKeys = Object.keys(params)
      .filter((key) => /^\\d+$/.test(key))
      .sort((left, right) => Number(left) - Number(right));
    if (numericKeys.length > 0) {
      return numericKeys.map((key) => deepClone(params[key]));
    }
  }
  return deepClone(params);
}

function firstParam(payload) {
  const params = normalizeParams(payload.params);
  return Array.isArray(params) ? params[0] : undefined;
}

function simpleRequestError(message) {
  return Promise.reject({ message });
}

export function bw_create_provider(config) {
  const settings = config || {};
  const listeners = new Map();
  const requestLog = [];
  const responses = settings.responses || {};
  const errors = settings.errors || {};
  const initialChainId = settings.chainId || "0xaa36a7";
  const state = {
    connected: Boolean(settings.connected),
    chainId: initialChainId,
    accounts: Array.isArray(settings.accounts) ? [...settings.accounts] : [],
    addedChains: new Set(
      Array.isArray(settings.addedChains) && settings.addedChains.length > 0
        ? settings.addedChains
        : [initialChainId]
    ),
  };

  const provider = {
    isMetaMask: Boolean(settings.flags && settings.flags.isMetaMask),
    isCoinbaseWallet: Boolean(settings.flags && settings.flags.isCoinbaseWallet),
    isRabby: Boolean(settings.flags && settings.flags.isRabby),
    request(payload) {
      const method = payload.method;
      requestLog.push({
        method,
        params: normalizeParams(payload.params),
      });

      if (Object.prototype.hasOwnProperty.call(errors, method)) {
        const error = errors[method];
        return Promise.reject({
          code: error.code,
          message: error.message,
          data: error.data === undefined ? undefined : deepClone(error.data),
        });
      }

      if (Object.prototype.hasOwnProperty.call(responses, method)) {
        return Promise.resolve(deepClone(responses[method]));
      }

      switch (method) {
        case "eth_accounts":
          return Promise.resolve(state.connected ? [...state.accounts] : []);
        case "eth_requestAccounts":
          state.connected = true;
          return Promise.resolve([...state.accounts]);
        case "eth_chainId":
          return Promise.resolve(state.chainId);
        case "wallet_switchEthereumChain": {
          const first = firstParam(payload);
          const requested = first && first.chainId;
          if (!requested) {
            return simpleRequestError("fixture switch request must include a chainId");
          }
          if (!state.addedChains.has(requested)) {
            return Promise.reject({
              code: 4902,
              message: `fixture wallet does not know chain ${requested}`,
            });
          }
          if (!settings.switchKeepsChain) {
            state.chainId = requested;
          }
          return Promise.resolve(null);
        }
        case "wallet_addEthereumChain": {
          const first = firstParam(payload);
          const requested = first && first.chainId;
          if (!requested) {
            return simpleRequestError("fixture add-chain request must include a chainId");
          }
          state.addedChains.add(requested);
          return Promise.resolve(null);
        }
        case "personal_sign":
          return Promise.resolve(
            settings.messageSignature || "0x111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111b"
          );
        case "eth_signTypedData_v4":
          return Promise.resolve(
            settings.typedDataSignature || "0x222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222221c"
          );
        case "eth_signTransaction":
          return Promise.resolve(settings.signedTransaction || "0xsigned-fixture-transaction");
        case "eth_sendTransaction":
          return Promise.resolve(
            settings.transactionHash || "0x3333333333333333333333333333333333333333333333333333333333333333"
          );
        case "web3_clientVersion":
          return Promise.resolve(settings.clientVersion || "Fixture Wallet / deterministic");
        default:
          return Promise.reject({
            code: -32601,
            message: `fixture wallet does not implement ${method}`,
          });
      }
    },
    on(eventName, callback) {
      let callbacks = listeners.get(eventName);
      if (!callbacks) {
        callbacks = new Set();
        listeners.set(eventName, callbacks);
      }
      callbacks.add(callback);
      return undefined;
    },
    removeListener(eventName, callback) {
      const callbacks = listeners.get(eventName);
      if (callbacks) {
        callbacks.delete(callback);
        if (callbacks.size === 0) {
          listeners.delete(eventName);
        }
      }
      return undefined;
    },
  };

  Object.defineProperty(provider, "__fixture", {
    value: {
      emit(eventName, payload) {
        const callbacks = Array.from(listeners.get(eventName) || []);
        for (const callback of callbacks) {
          callback(payload);
        }
      },
      listenerCount(eventName) {
        const callbacks = listeners.get(eventName);
        return callbacks ? callbacks.size : 0;
      },
      requestLog,
    },
  });

  return provider;
}

export function bw_install_legacy_provider(provider) {
  window.ethereum = provider;
}

export function bw_clear_legacy_provider() {
  delete window.ethereum;
}

export function bw_emit_provider_event(provider, eventName, payload) {
  provider.__fixture.emit(eventName, payload);
}

export function bw_provider_listener_count(provider, eventName) {
  return provider.__fixture.listenerCount(eventName);
}

export function bw_provider_request_log(provider) {
  return deepClone(provider.__fixture.requestLog);
}

let announcementHandler = null;

export function bw_install_eip6963_announcements(announcements) {
  bw_clear_eip6963_announcements();
  announcementHandler = () => {
    for (const announcement of announcements) {
      window.dispatchEvent(
        new CustomEvent("eip6963:announceProvider", {
          detail: {
            info: announcement.info,
            provider: announcement.provider,
          },
        }),
      );
    }
  };
  window.addEventListener("eip6963:requestProvider", announcementHandler);
}

export function bw_clear_eip6963_announcements() {
  if (announcementHandler) {
    window.removeEventListener("eip6963:requestProvider", announcementHandler);
    announcementHandler = null;
  }
}
"#)]
extern "C" {
    fn bw_create_provider(config: &JsValue) -> JsValue;
    fn bw_install_legacy_provider(provider: &JsValue);
    fn bw_clear_legacy_provider();
    fn bw_emit_provider_event(provider: &JsValue, event_name: &str, payload: &JsValue);
    fn bw_provider_listener_count(provider: &JsValue, event_name: &str) -> u32;
    fn bw_provider_request_log(provider: &JsValue) -> JsValue;
    fn bw_install_eip6963_announcements(announcements: &JsValue);
    fn bw_clear_eip6963_announcements();
}

struct LegacyProviderFixture {
    provider: JsValue,
}

impl LegacyProviderFixture {
    fn install(config: Value) -> Self {
        let provider = bw_create_provider(&to_js(&config));
        bw_install_legacy_provider(&provider);
        Self { provider }
    }

    fn emit(&self, event_name: &str, payload: Value) {
        bw_emit_provider_event(&self.provider, event_name, &to_js(&payload));
    }

    fn listener_count(&self, event_name: &str) -> u32 {
        bw_provider_listener_count(&self.provider, event_name)
    }

    fn request_log(&self) -> Vec<MockRequestRecord> {
        serde_wasm_bindgen::from_value(bw_provider_request_log(&self.provider))
            .expect("fixture request log must stay serializable")
    }
}

impl Drop for LegacyProviderFixture {
    fn drop(&mut self) {
        bw_clear_legacy_provider();
    }
}

struct DiscoveryAnnouncements;

impl DiscoveryAnnouncements {
    fn install(announcements: &[(JsValue, &str, &str, &str)]) -> Self {
        let entries = Array::new();
        for (provider, name, uuid, rdns) in announcements {
            let entry = Object::new();
            let info = Object::new();
            set_field(&info, "name", JsValue::from_str(name));
            set_field(&info, "uuid", JsValue::from_str(uuid));
            set_field(&info, "rdns", JsValue::from_str(rdns));
            set_field(
                &info,
                "icon",
                JsValue::from_str("data:image/svg+xml,<svg/>"),
            );
            set_field(&entry, "provider", provider.clone());
            set_field(&entry, "info", info.into());
            entries.push(&entry);
        }
        bw_install_eip6963_announcements(entries.as_ref());
        Self
    }
}

impl Drop for DiscoveryAnnouncements {
    fn drop(&mut self) {
        bw_clear_eip6963_announcements();
    }
}

#[wasm_bindgen_test(async)]
async fn legacy_detect_connect_and_signer_requests_cross_the_typed_promise_bridge() {
    let fixture = LegacyProviderFixture::install(json!({
        "accounts": [ACCOUNT],
        "chainId": "0xaa36a7",
        "flags": {
            "isMetaMask": true,
        },
        "messageSignature": repeated_signature("11", "1b"),
        "typedDataSignature": repeated_signature("22", "1c"),
    }));

    let wallet = BrowserWallet::detect_with_trusted_origin(
        Origin::new("test://window.ethereum").expect("test origin should parse"),
    )
    .expect("legacy provider detection should succeed")
    .expect("legacy provider should be present");
    let info = wallet
        .injected_info()
        .expect("detected wallet should include injected metadata");
    assert_eq!(
        info.discovery_source,
        InjectedWalletDiscoverySource::LegacyWindowEthereum
    );
    assert!(!info.provider_label.trim().is_empty());
    assert_eq!(wallet.session().wallet_label, info.provider_label);

    let session = wallet.connect().await.expect("connect should succeed");
    assert!(session.connected);
    assert_eq!(session.chain_id, Some(u64::from(SupportedChainId::Sepolia)));
    assert_eq!(session.accounts.len(), 1);
    assert_eq!(
        session
            .selected_account
            .as_ref()
            .map(Address::to_hex_string)
            .as_deref(),
        Some(ACCOUNT)
    );

    let signer = wallet.signer();
    assert_eq!(
        signer
            .sign_message(b"cow-rs bridge")
            .await
            .expect("personal_sign should succeed"),
        repeated_signature("11", "1b")
    );
    assert_eq!(
        signer
            .sign_typed_data_payload(&order_payload(SupportedChainId::Sepolia))
            .await
            .expect("typed-data signing should succeed"),
        repeated_signature("22", "1c")
    );

    let request_log = fixture.request_log();
    let methods = request_log
        .iter()
        .map(|record| record.method.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        methods,
        vec![
            "eth_requestAccounts",
            "eth_chainId",
            "personal_sign",
            "eth_signTypedData_v4",
        ]
    );

    let personal_sign = &request_log[2];
    assert_eq!(
        personal_sign.params,
        Some(json!([
            format!("0x{}", hex::encode(b"cow-rs bridge")),
            ACCOUNT
        ]))
    );

    let typed_data = request_log[3]
        .params
        .as_ref()
        .and_then(Value::as_array)
        .and_then(|items| items.get(1))
        .and_then(Value::as_str)
        .expect("typed-data request must serialize the payload as JSON");
    let typed_data: Value =
        serde_json::from_str(typed_data).expect("typed-data request must be valid JSON");
    assert_eq!(typed_data["primaryType"], json!("Order"));
    assert_eq!(
        typed_data["domain"]["chainId"],
        json!(u64::from(SupportedChainId::Sepolia))
    );
}

#[wasm_bindgen_test(async)]
async fn provider_events_keep_session_synchronized_and_listener_cleanup_tracks_rust_owners() {
    let fixture = LegacyProviderFixture::install(json!({
        "accounts": [ACCOUNT],
        "chainId": "0xaa36a7",
        "flags": {
            "isMetaMask": true,
        },
    }));

    let wallet = BrowserWallet::detect_with_trusted_origin(
        Origin::new("test://window.ethereum").expect("test origin should parse"),
    )
    .expect("legacy provider detection should succeed")
    .expect("legacy provider should be present");
    assert_eq!(fixture.listener_count("accountsChanged"), 1);
    assert_eq!(fixture.listener_count("chainChanged"), 1);
    assert_eq!(fixture.listener_count("connect"), 1);
    assert_eq!(fixture.listener_count("disconnect"), 1);

    fixture.emit("connect", json!({ "chainId": "0x1" }));
    fixture.emit("accountsChanged", json!([ALTERNATE_ACCOUNT]));
    fixture.emit("chainChanged", json!("0x1"));
    fixture.emit("disconnect", json!({ "message": "fixture disconnected" }));

    let session = wallet.session();
    assert!(!session.connected);
    assert!(session.accounts.is_empty());
    assert!(session.selected_account.is_none());
    assert_eq!(session.chain_id, None);

    let events = wallet.take_events();
    assert!(events.iter().any(|event| matches!(
        event,
        WalletEvent::Connected { chain_id } if *chain_id == Some(1)
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        WalletEvent::AccountsChanged { accounts }
            if accounts.len() == 1 && accounts[0].to_hex_string() == ALTERNATE_ACCOUNT
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        WalletEvent::Disconnected { message }
            if message.as_deref() == Some("fixture disconnected")
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        WalletEvent::SessionUpdated { current, .. } if !current.connected && current.chain_id.is_none()
    )));

    let provider = wallet.provider();
    let wallet_clone = wallet.clone();

    drop(wallet);
    assert_eq!(fixture.listener_count("accountsChanged"), 1);

    drop(provider);
    assert_eq!(fixture.listener_count("accountsChanged"), 1);

    drop(wallet_clone);
    assert_eq!(fixture.listener_count("accountsChanged"), 0);
    assert_eq!(fixture.listener_count("chainChanged"), 0);
    assert_eq!(fixture.listener_count("connect"), 0);
    assert_eq!(fixture.listener_count("disconnect"), 0);
}

#[wasm_bindgen_test(async)]
async fn eip6963_discovery_preserves_metadata_and_requires_explicit_selection() {
    let first = bw_create_provider(&to_js(&json!({
        "flags": {
            "isMetaMask": true,
        },
    })));
    let second = bw_create_provider(&to_js(&json!({
        "flags": {
            "isRabby": true,
        },
    })));
    let _announcements = DiscoveryAnnouncements::install(&[
        (first, "MetaMask", "uuid-1", "io.metamask"),
        (second, "Rabby", "uuid-2", "io.rabby"),
    ]);

    let discovery = BrowserWallet::discover_with(InjectedWalletDetectionOptions::new(0))
        .await
        .expect("EIP-6963 discovery should succeed");

    assert_eq!(discovery.timeout_ms(), 0);
    assert_eq!(discovery.len(), 2);
    assert!(!discovery.used_legacy_fallback());
    assert!(discovery.requires_explicit_selection());
    assert_eq!(
        discovery
            .single_wallet()
            .expect_err("multiple candidates must require selection"),
        BrowserWalletError::DiscoverySelectionRequired { candidates: 2 }
    );

    let wallets = discovery.wallets();
    assert_eq!(wallets[0].provider_label, "MetaMask");
    assert_eq!(wallets[0].provider_uuid.as_deref(), Some("uuid-1"));
    assert_eq!(wallets[1].provider_label, "Rabby");
    assert_eq!(
        wallets[1].discovery_source,
        InjectedWalletDiscoverySource::Eip6963
    );

    let selected = discovery
        .wallet_at(1)
        .expect("wallet selection by index should succeed");
    assert_eq!(selected.session().wallet_label, "Rabby");
    assert_eq!(
        selected
            .injected_info()
            .and_then(|info| info.provider_rdns)
            .as_deref(),
        Some("io.rabby")
    );
}

#[wasm_bindgen_test(async)]
async fn rejected_chain_switch_requests_map_to_typed_browser_wallet_errors() {
    let fixture = LegacyProviderFixture::install(json!({
        "accounts": [ACCOUNT],
        "chainId": "0xaa36a7",
        "errors": {
            "wallet_switchEthereumChain": {
                "code": 4902,
                "message": "fixture wallet does not know the requested chain",
            },
        },
    }));

    let wallet = BrowserWallet::detect_with_trusted_origin(
        Origin::new("test://window.ethereum").expect("test origin should parse"),
    )
    .expect("legacy provider detection should succeed")
    .expect("legacy provider should be present");
    let error = wallet
        .switch_chain(SupportedChainId::Base)
        .await
        .expect_err("unknown chain should fail through the typed bridge");
    assert_eq!(
        error,
        BrowserWalletError::ChainNotAdded {
            chain_id: u64::from(SupportedChainId::Base),
            method: "wallet_switchEthereumChain".to_owned().into(),
            code: 4902,
            message: "fixture wallet does not know the requested chain"
                .to_owned()
                .into(),
        }
    );

    let request_log = fixture.request_log();
    assert_eq!(request_log.len(), 1);
    assert_eq!(request_log[0].method, "wallet_switchEthereumChain");
    assert_eq!(
        request_log[0]
            .params
            .as_ref()
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(1)
    );
}

#[wasm_bindgen_test(async)]
async fn successful_switch_requests_fail_when_the_refreshed_session_stays_on_a_different_chain() {
    let fixture = LegacyProviderFixture::install(json!({
        "accounts": [ACCOUNT],
        "chainId": "0xaa36a7",
        "addedChains": ["0xaa36a7", "0x1"],
        "switchKeepsChain": true,
    }));

    let wallet = BrowserWallet::detect_with_trusted_origin(
        Origin::new("test://window.ethereum").expect("test origin should parse"),
    )
    .expect("legacy provider detection should succeed")
    .expect("legacy provider should be present");
    wallet.connect().await.expect("connect should succeed");

    let error = wallet
        .switch_chain(SupportedChainId::Mainnet)
        .await
        .expect_err("stale session chain should fail after a successful switch request");

    assert_eq!(
        error,
        BrowserWalletError::SessionChainMismatch {
            expected_chain_id: u64::from(SupportedChainId::Mainnet),
            session_chain_id: u64::from(SupportedChainId::Sepolia),
        }
    );

    let request_log = fixture.request_log();
    let methods = request_log
        .into_iter()
        .map(|record| record.method)
        .collect::<Vec<_>>();
    assert_eq!(
        methods,
        vec![
            "eth_requestAccounts".to_owned(),
            "eth_chainId".to_owned(),
            "wallet_switchEthereumChain".to_owned(),
            "eth_accounts".to_owned(),
            "eth_chainId".to_owned(),
            "eth_chainId".to_owned(),
        ]
    );
}

#[wasm_bindgen_test(async)]
async fn mock_wallet_console_state_machine_is_deterministic() {
    let fixture = LegacyProviderFixture::install(json!({
        "accounts": [ACCOUNT],
        "chainId": "0xaa36a7",
        "flags": {
            "isMetaMask": true,
        },
    }));

    let wallet = BrowserWallet::detect_with_trusted_origin(
        Origin::new("test://window.ethereum").expect("test origin should parse"),
    )
    .expect("legacy provider detection should succeed")
    .expect("legacy provider should be present");

    let connected = wallet.connect().await.expect("connect should succeed");
    assert!(connected.connected);
    assert_eq!(
        connected.chain_id,
        Some(u64::from(SupportedChainId::Sepolia))
    );

    let reset = wallet.reset_session();
    assert!(!reset.connected);
    assert_eq!(reset.chain_id, None);

    let refreshed = wallet
        .refresh_session()
        .await
        .expect("refresh should succeed");
    assert!(refreshed.connected);
    assert_eq!(
        refreshed.chain_id,
        Some(u64::from(SupportedChainId::Sepolia))
    );

    let methods = fixture
        .request_log()
        .into_iter()
        .map(|record| record.method)
        .collect::<Vec<_>>();
    assert_eq!(
        methods,
        vec![
            "eth_requestAccounts".to_owned(),
            "eth_chainId".to_owned(),
            "eth_accounts".to_owned(),
            "eth_chainId".to_owned(),
        ]
    );
}

#[wasm_bindgen_test]
fn eip6963_discovery_event_serde_roundtrip() {
    let info = InjectedWalletInfo::new(
        "Rabby",
        InjectedWalletDiscoverySource::Eip6963,
        Some("wallet-rabby".to_owned()),
        Some("io.rabby".to_owned()),
        Some("data:text/plain,rabby".to_owned()),
        false,
        false,
        true,
    );

    let value = serde_json::to_value(&info).expect("EIP-6963 info must serialize");
    let reparsed: InjectedWalletInfo =
        serde_json::from_value(value).expect("EIP-6963 info must deserialize");

    assert_eq!(reparsed, info);
}

fn order_payload(chain_id: SupportedChainId) -> TypedDataPayload {
    fn typed_field(name: &str, kind: &str) -> TypedDataField {
        TypedDataField::new(name.to_owned(), kind.to_owned())
    }

    let mut types = TypedDataTypes::new();
    types.insert(
        "Order".to_owned(),
        [
            ("sellToken", "address"),
            ("buyToken", "address"),
            ("receiver", "address"),
            ("sellAmount", "uint256"),
            ("buyAmount", "uint256"),
            ("validTo", "uint32"),
            ("appData", "bytes32"),
            ("feeAmount", "uint256"),
            ("kind", "string"),
            ("partiallyFillable", "bool"),
            ("sellTokenBalance", "string"),
            ("buyTokenBalance", "string"),
        ]
        .into_iter()
        .map(|(name, kind)| typed_field(name, kind))
        .collect(),
    );
    types.insert(
        "EIP712Domain".to_owned(),
        [
            ("name", "string"),
            ("version", "string"),
            ("chainId", "uint256"),
            ("verifyingContract", "address"),
        ]
        .into_iter()
        .map(|(name, kind)| typed_field(name, kind))
        .collect(),
    );

    let verifying_contract =
        cow_sdk_core::Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41")
            .expect("static contract address must stay valid");
    TypedDataPayload::new(
        TypedDataDomain {
            name: Some("Gnosis Protocol".into()),
            version: Some("v2".into()),
            chain_id: Some(alloy_primitives::U256::from(u64::from(chain_id))),
            verifying_contract: Some(*verifying_contract.as_alloy()),
            salt: None,
        },
        "Order".to_owned(),
        types,
        r#"{"sellToken":"0x1111111111111111111111111111111111111111","buyToken":"0x2222222222222222222222222222222222222222","receiver":"0x3333333333333333333333333333333333333333","sellAmount":"1","buyAmount":"2","validTo":1,"appData":"0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","feeAmount":"0","kind":"sell","partiallyFillable":false,"sellTokenBalance":"erc20","buyTokenBalance":"erc20"}"#.to_owned(),
    )
}

fn repeated_signature(byte: &str, suffix: &str) -> String {
    format!("0x{}{}", byte.repeat(64), suffix)
}

fn to_js<T>(value: &T) -> JsValue
where
    T: Serialize,
{
    value
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .expect("fixture values must stay serializable")
}

fn set_field(target: &Object, field: &str, value: JsValue) {
    Reflect::set(target, &JsValue::from_str(field), &value)
        .expect("fixture objects must accept direct field assignment");
}
