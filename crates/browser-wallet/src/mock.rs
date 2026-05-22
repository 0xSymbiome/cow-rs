//! Deterministic mock EIP-1193 transport used by tests, examples, and proof-oriented reviews.

use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

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

/// Recorded mock wallet request.
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "the `params: Option<serde_json::Value>` field cannot participate in `Eq` because `serde_json::Value` does not implement `Eq`"
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MockRequestRecord {
    /// Requested RPC method.
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// JSON parameters supplied to the request, when present.
    pub params: Option<Value>,
}

struct MockState {
    connected: bool,
    chain_id: ChainId,
    accounts: Vec<Address>,
    switch_applies_requested_chain: bool,
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
    added_chains: BTreeSet<ChainId>,
    next_listener_id: usize,
    session_listeners: BTreeMap<usize, Rc<dyn Fn(WalletProviderEvent)>>,
}

impl Default for MockState {
    /// Creates the deterministic mock wallet state.
    ///
    /// # Panics
    ///
    /// Panics only if the crate-owned static mock account literal stops being
    /// a valid EVM address.
    fn default() -> Self {
        Self {
            connected: false,
            chain_id: u64::from(SupportedChainId::Sepolia),
            accounts: vec![
                // SAFETY: the mock account is a reviewed static literal used
                // only to seed deterministic browser-wallet fixtures.
                Address::new("0x4444444444444444444444444444444444444444")
                    .expect("static mock address must remain valid"),
            ],
            switch_applies_requested_chain: true,
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
            added_chains: SupportedChainId::ALL
                .into_iter()
                .map(u64::from)
                .collect::<BTreeSet<_>>(),
            next_listener_id: 0,
            session_listeners: BTreeMap::new(),
        }
    }
}

impl MockState {
    fn accounts_response(&self) -> Value {
        if self.connected {
            json!(
                self.accounts
                    .iter()
                    .map(Address::to_hex_string)
                    .collect::<Vec<_>>()
            )
        } else {
            json!([])
        }
    }

    fn request_accounts(&mut self) -> Value {
        self.connected = true;
        self.accounts_response()
    }

    fn chain_id_response(&self) -> Result<Value, BrowserWalletError> {
        Ok(Value::String(hex_quantity(&self.chain_id.to_string())?))
    }

    fn switch_chain(
        &mut self,
        method: &str,
        params: Option<&Value>,
    ) -> Result<Value, BrowserWalletError> {
        let requested_chain = requested_chain_id(
            method,
            params,
            "mock switch request must include a `chainId` field",
        )?;
        if !self.added_chains.contains(&requested_chain) {
            return Err(BrowserWalletError::ChainNotAdded {
                chain_id: requested_chain,
                method: method.to_owned().into(),
                code: 4902,
                message: format!("mock wallet does not know chain {requested_chain}").into(),
            });
        }
        if self.switch_applies_requested_chain {
            self.chain_id = requested_chain;
        }
        Ok(Value::Null)
    }

    fn add_chain(
        &mut self,
        method: &str,
        params: Option<&Value>,
    ) -> Result<Value, BrowserWalletError> {
        self.added_chains.insert(requested_chain_id(
            method,
            params,
            "mock add-chain request must include a `chainId` field",
        )?);
        Ok(Value::Null)
    }

    fn code_response(
        &self,
        method: &str,
        params: Option<&Value>,
    ) -> Result<Value, BrowserWalletError> {
        let address = first_param_str(method, params, "mock code request must include an address")?;
        Ok(Value::String(
            self.code_by_address
                .get(&address.to_ascii_lowercase())
                .cloned()
                .unwrap_or_else(|| "0x".to_owned()),
        ))
    }

    fn storage_response(
        &self,
        method: &str,
        params: Option<&Value>,
    ) -> Result<Value, BrowserWalletError> {
        let values = params.and_then(Value::as_array).ok_or_else(|| {
            BrowserWalletError::malformed_response(
                method,
                "mock storage request must include address and slot",
            )
        })?;
        let address = values
            .first()
            .and_then(Value::as_str)
            .ok_or_else(|| BrowserWalletError::malformed_response(method, "missing address"))?;
        let slot = values
            .get(1)
            .and_then(Value::as_str)
            .ok_or_else(|| BrowserWalletError::malformed_response(method, "missing slot"))?;
        Ok(Value::String(
            self.storage_by_key
                .get(&format!(
                    "{}:{}",
                    address.to_ascii_lowercase(),
                    slot.to_ascii_lowercase()
                ))
                .cloned()
                .unwrap_or_else(|| "0x0".to_owned()),
        ))
    }

    fn receipt_response(
        &self,
        method: &str,
        params: Option<&Value>,
    ) -> Result<Value, BrowserWalletError> {
        let hash = first_param_str(
            method,
            params,
            "mock receipt request must include a transaction hash",
        )?;
        Ok(self
            .receipt_by_hash
            .get(&hash.to_ascii_lowercase())
            .cloned()
            .unwrap_or(Value::Null))
    }

    fn block_response(&self) -> Result<Value, BrowserWalletError> {
        Ok(json!({
            "number": hex_quantity(&self.block_number.to_string())?,
            "hash": self.block_hash,
        }))
    }
}

/// Deterministic EIP-1193 transport for tests and non-browser proof flows.
#[derive(Clone)]
pub struct MockEip1193Transport {
    label: String,
    state: Rc<RefCell<MockState>>,
}

impl std::fmt::Debug for MockEip1193Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.state.borrow();
        f.debug_struct("MockEip1193Transport")
            .field("label", &self.label)
            .field("connected", &state.connected)
            .field("chain_id", &state.chain_id)
            .field("accounts", &state.accounts)
            .field("request_log_len", &state.request_log.len())
            .field("listener_count", &state.session_listeners.len())
            .finish()
    }
}

impl Default for MockEip1193Transport {
    fn default() -> Self {
        Self::sepolia()
    }
}

impl MockEip1193Transport {
    /// Creates a mock wallet configured for Sepolia with deterministic responses.
    #[must_use]
    pub fn sepolia() -> Self {
        Self {
            label: "Mock Wallet".to_owned(),
            state: Rc::new(RefCell::new(MockState::default())),
        }
    }

    /// Replaces the human-readable wallet label used by session state and diagnostics.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Sets whether the mock wallet reports itself as connected.
    pub fn set_connected(&self, connected: bool) {
        self.state.borrow_mut().connected = connected;
    }

    /// Sets the active chain id returned by the mock wallet.
    pub fn set_chain_id(&self, chain_id: SupportedChainId) {
        self.state.borrow_mut().chain_id = u64::from(chain_id);
    }

    /// Replaces the set of chains that `wallet_switchEthereumChain` can switch to directly.
    pub fn set_added_chains(&self, chains: Vec<SupportedChainId>) {
        self.state.borrow_mut().added_chains =
            chains.into_iter().map(u64::from).collect::<BTreeSet<_>>();
    }

    /// Controls whether a successful switch request updates the active chain.
    ///
    /// This is useful for proving that higher-level helpers verify the
    /// refreshed session chain instead of treating RPC acknowledgement alone as
    /// authoritative.
    pub fn set_switch_chain_updates_active_chain(&self, updates_chain: bool) {
        self.state.borrow_mut().switch_applies_requested_chain = updates_chain;
    }

    /// Sets the wallet accounts returned by account queries.
    pub fn set_accounts(&self, accounts: Vec<Address>) {
        self.state.borrow_mut().accounts = accounts;
    }

    /// Sets the fallback result returned by `eth_call`.
    pub fn set_default_call_result(&self, result: impl Into<String>) {
        self.state.borrow_mut().default_call_result = result.into();
    }

    /// Configures bytecode returned by `eth_getCode` for one address.
    pub fn set_code(&self, address: &Address, code_hex: impl Into<String>) {
        self.state
            .borrow_mut()
            .code_by_address
            .insert(address.to_hex_string(), code_hex.into());
    }

    /// Configures storage returned by `eth_getStorageAt` for one address and slot.
    pub fn set_storage(&self, address: &Address, slot: &str, value_hex: impl Into<String>) {
        self.state.borrow_mut().storage_by_key.insert(
            format!("{}:{}", address.to_hex_string(), slot.to_ascii_lowercase()),
            value_hex.into(),
        );
    }

    /// Configures a receipt returned by `eth_getTransactionReceipt`.
    pub fn set_receipt(&self, transaction_hash: &str, receipt: Value) {
        self.state
            .borrow_mut()
            .receipt_by_hash
            .insert(transaction_hash.to_ascii_lowercase(), receipt);
    }

    /// Configures one method to fail with the supplied browser-wallet error.
    pub fn fail_method(&self, method: &str, error: BrowserWalletError) {
        self.state
            .borrow_mut()
            .method_errors
            .insert(method.to_owned(), error);
    }

    /// Returns the recorded request log in call order.
    #[must_use]
    pub fn request_log(&self) -> Vec<MockRequestRecord> {
        self.state.borrow().request_log.clone()
    }

    /// Emits an `accountsChanged` provider event and updates the mock session state.
    pub fn emit_accounts_changed(&self, accounts: Vec<Address>) {
        {
            let mut state = self.state.borrow_mut();
            state.connected = !accounts.is_empty();
            state.accounts.clone_from(&accounts);
        }
        self.emit_provider_event(&WalletProviderEvent::AccountsChanged { accounts });
    }

    /// Emits a `chainChanged` provider event and updates the mock session state.
    pub fn emit_chain_changed(&self, chain_id: ChainId) {
        self.state.borrow_mut().chain_id = chain_id;
        self.emit_provider_event(&WalletProviderEvent::ChainChanged { chain_id });
    }

    /// Emits a `connect` provider event and updates the mock session state.
    pub fn emit_connected(&self, chain_id: Option<ChainId>) {
        {
            let mut state = self.state.borrow_mut();
            state.connected = true;
            if let Some(chain_id) = chain_id {
                state.chain_id = chain_id;
            }
        }
        self.emit_provider_event(&WalletProviderEvent::Connected { chain_id });
    }

    /// Emits a `disconnect` provider event and marks the mock wallet as disconnected.
    pub fn emit_disconnected(&self, message: Option<String>) {
        self.state.borrow_mut().connected = false;
        self.emit_provider_event(&WalletProviderEvent::Disconnected { message });
    }

    /// Returns the number of active session listeners currently attached to the mock transport.
    #[must_use]
    pub fn listener_count(&self) -> usize {
        self.state.borrow().session_listeners.len()
    }

    fn emit_provider_event(&self, event: &WalletProviderEvent) {
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

fn first_param_str<'a>(
    method: &str,
    params: Option<&'a Value>,
    missing_message: &'static str,
) -> Result<&'a str, BrowserWalletError> {
    params
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(Value::as_str)
        .ok_or_else(|| BrowserWalletError::malformed_response(method, missing_message))
}

fn requested_chain_id(
    method: &str,
    params: Option<&Value>,
    missing_message: &'static str,
) -> Result<ChainId, BrowserWalletError> {
    let requested = params
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|item| item.get("chainId"))
        .cloned()
        .ok_or_else(|| BrowserWalletError::malformed_response(method, missing_message))?;
    parse_chain_id_value(&requested, method)
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
            "eth_accounts" => Ok(state.accounts_response()),
            "eth_requestAccounts" => Ok(state.request_accounts()),
            "eth_chainId" => state.chain_id_response(),
            "wallet_switchEthereumChain" => state.switch_chain(method, params.as_ref()),
            "wallet_addEthereumChain" => state.add_chain(method, params.as_ref()),
            "personal_sign" => Ok(Value::String(state.message_signature.clone())),
            "eth_signTypedData_v4" => Ok(Value::String(state.typed_data_signature.clone())),
            "eth_signTransaction" => Ok(Value::String(state.signed_transaction.clone())),
            "eth_sendTransaction" => Ok(Value::String(state.transaction_hash.clone())),
            "eth_estimateGas" => Ok(Value::String(hex_quantity(&state.gas_estimate)?)),
            "eth_getCode" => state.code_response(method, params.as_ref()),
            "eth_getStorageAt" => state.storage_response(method, params.as_ref()),
            "eth_call" => Ok(Value::String(state.default_call_result.clone())),
            "eth_getTransactionReceipt" => state.receipt_response(method, params.as_ref()),
            "eth_getBlockByNumber" => state.block_response(),
            "web3_clientVersion" => Ok(Value::String(format!("{} / deterministic", self.label))),
            _ => Err(Self::rpc_error(
                method,
                RpcErrorPayload::new(
                    -32601,
                    format!("mock wallet does not implement `{method}`"),
                    None,
                ),
            )),
        }
    }
}
