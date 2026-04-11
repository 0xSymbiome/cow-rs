use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use cow_sdk_core::{Address, ChainId, SupportedChainId};

use crate::{
    BrowserWalletError, Eip1193Transport, EventLog, RpcErrorPayload, WalletSession,
    events::{
        WalletProviderEvent, WalletRuntimeBinding, WalletRuntimeBindingHandle, apply_provider_event,
    },
    provider::{hex_quantity, parse_chain_id_value},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MockRequestRecord {
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

struct MockState {
    connected: bool,
    chain_id: ChainId,
    accounts: Vec<Address>,
    request_log: Vec<MockRequestRecord>,
    message_signature: String,
    typed_data_signature: String,
    signed_transaction: String,
    transaction_hash: String,
    gas_estimate: String,
    default_call_result: String,
    block_number: u64,
    block_hash: Option<String>,
    code_by_address: BTreeMap<String, String>,
    storage_by_key: BTreeMap<String, String>,
    receipt_by_hash: BTreeMap<String, Value>,
    method_errors: BTreeMap<String, BrowserWalletError>,
    next_listener_id: usize,
    session_listeners: BTreeMap<usize, Rc<dyn Fn(WalletProviderEvent)>>,
}

impl Default for MockState {
    fn default() -> Self {
        Self {
            connected: false,
            chain_id: u64::from(SupportedChainId::Sepolia),
            accounts: vec![
                Address::new("0x4444444444444444444444444444444444444444")
                    .expect("static mock address must remain valid"),
            ],
            request_log: Vec::new(),
            message_signature: format!("0x{}1b", "11".repeat(64)),
            typed_data_signature: format!("0x{}1c", "22".repeat(64)),
            signed_transaction: "0xsigned-mock-transaction".to_owned(),
            transaction_hash: format!("0x{}", "33".repeat(32)),
            gas_estimate: "21000".to_owned(),
            default_call_result: format!("0x{}2a", "0".repeat(62)),
            block_number: 12_345,
            block_hash: Some(format!("0x{}", "55".repeat(32))),
            code_by_address: BTreeMap::new(),
            storage_by_key: BTreeMap::new(),
            receipt_by_hash: BTreeMap::new(),
            method_errors: BTreeMap::new(),
            next_listener_id: 0,
            session_listeners: BTreeMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct MockEip1193Transport {
    label: String,
    state: Rc<RefCell<MockState>>,
}

impl Default for MockEip1193Transport {
    fn default() -> Self {
        Self::sepolia()
    }
}

impl MockEip1193Transport {
    pub fn sepolia() -> Self {
        Self {
            label: "Mock Wallet".to_owned(),
            state: Rc::new(RefCell::new(MockState::default())),
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn set_connected(&self, connected: bool) {
        self.state.borrow_mut().connected = connected;
    }

    pub fn set_chain_id(&self, chain_id: SupportedChainId) {
        self.state.borrow_mut().chain_id = u64::from(chain_id);
    }

    pub fn set_accounts(&self, accounts: Vec<Address>) {
        self.state.borrow_mut().accounts = accounts;
    }

    pub fn set_default_call_result(&self, result: impl Into<String>) {
        self.state.borrow_mut().default_call_result = result.into();
    }

    pub fn set_code(&self, address: &Address, code_hex: impl Into<String>) {
        self.state
            .borrow_mut()
            .code_by_address
            .insert(address.normalized_key(), code_hex.into());
    }

    pub fn set_storage(&self, address: &Address, slot: &str, value_hex: impl Into<String>) {
        self.state.borrow_mut().storage_by_key.insert(
            format!("{}:{}", address.normalized_key(), slot.to_ascii_lowercase()),
            value_hex.into(),
        );
    }

    pub fn set_receipt(&self, transaction_hash: &str, receipt: Value) {
        self.state
            .borrow_mut()
            .receipt_by_hash
            .insert(transaction_hash.to_ascii_lowercase(), receipt);
    }

    pub fn fail_method(&self, method: &str, error: BrowserWalletError) {
        self.state
            .borrow_mut()
            .method_errors
            .insert(method.to_owned(), error);
    }

    pub fn request_log(&self) -> Vec<MockRequestRecord> {
        self.state.borrow().request_log.clone()
    }

    pub fn emit_accounts_changed(&self, accounts: Vec<Address>) {
        {
            let mut state = self.state.borrow_mut();
            state.connected = !accounts.is_empty();
            state.accounts = accounts.clone();
        }
        self.emit_provider_event(WalletProviderEvent::AccountsChanged { accounts });
    }

    pub fn emit_chain_changed(&self, chain_id: ChainId) {
        self.state.borrow_mut().chain_id = chain_id;
        self.emit_provider_event(WalletProviderEvent::ChainChanged { chain_id });
    }

    pub fn emit_connected(&self, chain_id: Option<ChainId>) {
        {
            let mut state = self.state.borrow_mut();
            state.connected = true;
            if let Some(chain_id) = chain_id {
                state.chain_id = chain_id;
            }
        }
        self.emit_provider_event(WalletProviderEvent::Connected { chain_id });
    }

    pub fn emit_disconnected(&self, message: Option<String>) {
        self.state.borrow_mut().connected = false;
        self.emit_provider_event(WalletProviderEvent::Disconnected { message });
    }

    pub fn listener_count(&self) -> usize {
        self.state.borrow().session_listeners.len()
    }

    fn emit_provider_event(&self, event: WalletProviderEvent) {
        let listeners = self
            .state
            .borrow()
            .session_listeners
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for listener in listeners {
            listener(event.clone());
        }
    }

    fn register_session_listener(&self, listener: Rc<dyn Fn(WalletProviderEvent)>) -> usize {
        let mut state = self.state.borrow_mut();
        let listener_id = state.next_listener_id;
        state.next_listener_id += 1;
        state.session_listeners.insert(listener_id, listener);
        listener_id
    }

    fn rpc_error(method: &str, payload: RpcErrorPayload) -> BrowserWalletError {
        BrowserWalletError::from_rpc(method, payload, None)
    }
}

struct MockSessionBinding {
    state: Rc<RefCell<MockState>>,
    listener_id: usize,
}

impl Drop for MockSessionBinding {
    fn drop(&mut self) {
        self.state
            .borrow_mut()
            .session_listeners
            .remove(&self.listener_id);
    }
}

impl WalletRuntimeBinding for MockSessionBinding {}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait(?Send))]
impl Eip1193Transport for MockEip1193Transport {
    fn label(&self) -> &str {
        &self.label
    }

    fn attach_session_sync(
        &self,
        session: Rc<RefCell<WalletSession>>,
        events: EventLog,
    ) -> Option<WalletRuntimeBindingHandle> {
        let listener = Rc::new(move |provider_event: WalletProviderEvent| {
            apply_provider_event(&session, &events, provider_event);
        });
        let listener_id = self.register_session_listener(listener);
        Some(Rc::new(MockSessionBinding {
            state: self.state.clone(),
            listener_id,
        }))
    }

    async fn request(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value, BrowserWalletError> {
        let mut state = self.state.borrow_mut();
        state.request_log.push(MockRequestRecord {
            method: method.to_owned(),
            params: params.clone(),
        });

        if let Some(error) = state.method_errors.get(method).cloned() {
            return Err(error);
        }

        match method {
            "eth_accounts" => Ok(if state.connected {
                json!(
                    state
                        .accounts
                        .iter()
                        .map(Address::as_str)
                        .collect::<Vec<_>>()
                )
            } else {
                json!([])
            }),
            "eth_requestAccounts" => {
                state.connected = true;
                Ok(json!(
                    state
                        .accounts
                        .iter()
                        .map(Address::as_str)
                        .collect::<Vec<_>>()
                ))
            }
            "eth_chainId" => Ok(Value::String(hex_quantity(&state.chain_id.to_string())?)),
            "wallet_switchEthereumChain" => {
                let requested = params
                    .as_ref()
                    .and_then(Value::as_array)
                    .and_then(|items| items.first())
                    .and_then(|item| item.get("chainId"))
                    .cloned()
                    .ok_or_else(|| {
                        BrowserWalletError::malformed_response(
                            method,
                            "mock switch request must include a `chainId` field",
                        )
                    })?;
                state.chain_id = parse_chain_id_value(&requested, method)?;
                Ok(Value::Null)
            }
            "personal_sign" => Ok(Value::String(state.message_signature.clone())),
            "eth_signTypedData_v4" => Ok(Value::String(state.typed_data_signature.clone())),
            "eth_signTransaction" => Ok(Value::String(state.signed_transaction.clone())),
            "eth_sendTransaction" => Ok(Value::String(state.transaction_hash.clone())),
            "eth_estimateGas" => Ok(Value::String(hex_quantity(&state.gas_estimate)?)),
            "eth_getCode" => {
                let address = params
                    .as_ref()
                    .and_then(Value::as_array)
                    .and_then(|items| items.first())
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        BrowserWalletError::malformed_response(
                            method,
                            "mock code request must include an address",
                        )
                    })?;
                Ok(Value::String(
                    state
                        .code_by_address
                        .get(&address.to_ascii_lowercase())
                        .cloned()
                        .unwrap_or_else(|| "0x".to_owned()),
                ))
            }
            "eth_getStorageAt" => {
                let values = params.as_ref().and_then(Value::as_array).ok_or_else(|| {
                    BrowserWalletError::malformed_response(
                        method,
                        "mock storage request must include address and slot",
                    )
                })?;
                let address = values.first().and_then(Value::as_str).ok_or_else(|| {
                    BrowserWalletError::malformed_response(method, "missing address")
                })?;
                let slot = values.get(1).and_then(Value::as_str).ok_or_else(|| {
                    BrowserWalletError::malformed_response(method, "missing slot")
                })?;
                Ok(Value::String(
                    state
                        .storage_by_key
                        .get(&format!(
                            "{}:{}",
                            address.to_ascii_lowercase(),
                            slot.to_ascii_lowercase()
                        ))
                        .cloned()
                        .unwrap_or_else(|| "0x0".to_owned()),
                ))
            }
            "eth_call" => Ok(Value::String(state.default_call_result.clone())),
            "eth_getTransactionReceipt" => {
                let hash = params
                    .as_ref()
                    .and_then(Value::as_array)
                    .and_then(|items| items.first())
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        BrowserWalletError::malformed_response(
                            method,
                            "mock receipt request must include a transaction hash",
                        )
                    })?;
                Ok(state
                    .receipt_by_hash
                    .get(&hash.to_ascii_lowercase())
                    .cloned()
                    .unwrap_or(Value::Null))
            }
            "eth_getBlockByNumber" => Ok(json!({
                "number": hex_quantity(&state.block_number.to_string())?,
                "hash": state.block_hash,
            })),
            "web3_clientVersion" => Ok(Value::String(format!("{} / deterministic", self.label))),
            _ => Err(Self::rpc_error(
                method,
                RpcErrorPayload {
                    code: -32601,
                    message: format!("mock wallet does not implement `{method}`"),
                    data: None,
                },
            )),
        }
    }
}
