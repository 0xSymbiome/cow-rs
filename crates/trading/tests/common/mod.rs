#![allow(
    dead_code,
    clippy::missing_const_for_fn,
    reason = "shared test-helper module aggregates fixtures, constants, and adapters that not every integration test binary exercises; an integration test may use only a subset of the shared helpers without leaving the others permanently unused, and forcing const on builder-style test helpers adds no value"
)]

use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use async_trait::async_trait;
use serde_json::json;

use cow_sdk_core::{
    Address, Amount, ApiBaseUrls, ApiContext, AppCode, AppDataHash, BlockHash, BlockInfo,
    ContractCall, ContractHandle, CowEnv, Hash32, HexData, OrderKind, OrderUid, Provider, Signer,
    SupportedChainId, TransactionBroadcast, TransactionHash, TransactionReceipt,
    TransactionRequest, TransactionStatus, TypedDataDomain, TypedDataPayload,
};
use cow_sdk_orderbook::{
    Order, OrderCancellations, OrderCreation, OrderQuoteRequest, OrderQuoteResponse, OrderbookError,
};
use cow_sdk_signing::eip1271::{Eip1271SignatureError, Eip1271Signer};
use cow_sdk_trading::{
    EthFlowOrderExistsChecker, OrderbookClient, SlippageSuggester, SlippageToleranceRequest,
    SlippageToleranceResponse, TradingError,
};

// Canonical lowercase 0x-prefixed wire form per PROP-WB-004; cow Address
// canonicalizes input casing at construction (ADR 0052).
pub const WETH: &str = "0xfff9976782d46cc05630d1f6ebab18b2324d6b14";
pub const COW: &str = "0x0625afb445c3b6b7b929342a04a22599fd5dbb59";
// The default owner/signer is a well-known deterministic test account (Anvil
// account 0) so the post-sign owner-recovery gate (ADR 0015) recovers the real
// signer from `MockSigner`'s genuine ECDSA signature. The signing keys for the
// addresses that sign successfully live in `test_signing_key_for` below.
pub const OWNER: &str = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";
pub const ALT_RECEIVER: &str = "0x974caa59e49682cda0ad2bbe82983419a2ecc400";
pub const CUSTOM_SETTLEMENT: &str = "0x13579bdf2468ace013579bdf2468ace013579bdf";
pub const CUSTOM_ETHFLOW: &str = "0x2468ace013579bdf2468ace013579bdf2468ace0";
pub const TX_HASH: &str = "0x13579bdf2468ace013579bdf2468ace013579bdf2468ace013579bdf2468ace0";
pub const ORDER_UID: &str = "0xd64389693b6cf89ad6c140a113b10df08073e5ef3063d05a02f3f42e1a42f0ad0b7795e18767259cc253a2af471dbc4c72b49516ffffffff";
pub const APP_DATA_HASH: &str =
    "0xe269b09f45b1d3c98d8e4e841b99a0779fbd3b77943d069b91ddc4fd9789e27e";
pub const TYPED_SIGNATURE: &str = "0x111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111b";
pub const MESSAGE_SIGNATURE: &str = "0x222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222221c";

pub use cow_sdk_test_utils::builders::address;

pub fn order_uid() -> OrderUid {
    OrderUid::new(ORDER_UID).expect("test order uid literal must be valid")
}

pub fn app_data_hash() -> AppDataHash {
    AppDataHash::new(APP_DATA_HASH).expect("test app-data hash literal must be valid")
}

pub fn test_app_code() -> AppCode {
    AppCode::new("0x007").expect("fixture appCode must validate")
}

pub fn sell_quote_response() -> OrderQuoteResponse {
    serde_json::from_value(json!({
        "quote": {
            "sellToken": WETH,
            "buyToken": COW,
            "receiver": OWNER,
            "sellAmount": "98646335338956442",
            "buyAmount": "30000000000000000000",
            "validTo": 1_737_464_594_u32,
            "appData": APP_DATA_HASH,
            "feeAmount": "1353664661043558",
            "kind": "sell",
            "partiallyFillable": false,
            "sellTokenBalance": "erc20",
            "buyTokenBalance": "erc20"
        },
        "from": OWNER,
        "expiration": "2025-01-21T12:55:14.799709609Z",
        "id": 575_401,
        "verified": true
    }))
    .expect("sell quote fixture must deserialize")
}

pub fn buy_quote_response() -> OrderQuoteResponse {
    serde_json::from_value(json!({
        "quote": {
            "sellToken": WETH,
            "buyToken": COW,
            "receiver": OWNER,
            "sellAmount": "1005456782512030400",
            "buyAmount": "400000000000000000000",
            "validTo": 1_737_468_944_u32,
            "appData": APP_DATA_HASH,
            "feeAmount": "1112955650440102",
            "kind": "buy",
            "partiallyFillable": false,
            "sellTokenBalance": "erc20",
            "buyTokenBalance": "erc20"
        },
        "from": OWNER,
        "expiration": "2025-01-21T14:07:44.176194885Z",
        "id": 575_498,
        "verified": true
    }))
    .expect("buy quote fixture must deserialize")
}

pub fn sample_trade_parameters(kind: OrderKind) -> cow_sdk_trading::TradeParams {
    let amount = if kind == OrderKind::Sell {
        Amount::new("100000000000000000").expect("test sell amount literal must be valid")
    } else {
        Amount::new("400000000000000000000").expect("test buy amount literal must be valid")
    };
    cow_sdk_trading::TradeParams::new(kind, address(WETH), address(COW), amount)
        .with_owner(address(OWNER))
        .with_slippage_bps(50)
}

pub fn sample_trader_parameters() -> cow_sdk_trading::TraderParams {
    cow_sdk_trading::TraderParams::new(SupportedChainId::Sepolia, "0x007")
        .expect("app code should validate")
        .with_env(CowEnv::Prod)
}

pub fn sample_limit_parameters(kind: OrderKind) -> cow_sdk_trading::LimitTradeParams {
    let quote = if kind == OrderKind::Sell {
        sell_quote_response()
    } else {
        buy_quote_response()
    };

    let sell_amount = quote.quote.sell_amount;
    let buy_amount = quote.quote.buy_amount;
    let mut params = cow_sdk_trading::LimitTradeParams::new(
        kind,
        address(WETH),
        address(COW),
        sell_amount,
        buy_amount,
    )
    .with_owner(address(OWNER))
    .with_sell_token_balance(quote.quote.sell_token_balance)
    .with_buy_token_balance(quote.quote.buy_token_balance)
    .with_slippage_bps(50);
    if let Some(id) = quote.id {
        params = params.with_quote_id(id);
    }
    params
}

#[derive(Clone)]
pub struct MockOrderbook {
    context: ApiContext,
    quote_response: OrderQuoteResponse,
    state: Arc<Mutex<MockOrderbookState>>,
    quote_delay: Option<std::time::Duration>,
}

#[derive(Clone, Default)]
pub struct MockOrderbookState {
    pub quote_requests: Vec<OrderQuoteRequest>,
    pub sent_orders: Vec<OrderCreation>,
    pub uploads: Vec<(AppDataHash, String)>,
    pub cancellations: Vec<OrderCancellations>,
    pub orders: Vec<Order>,
    pub order_id: Option<OrderUid>,
}

impl MockOrderbook {
    pub fn new(chain_id: SupportedChainId, quote_response: OrderQuoteResponse) -> Self {
        Self::new_with_env(chain_id, CowEnv::Prod, quote_response)
    }

    pub fn new_with_env(
        chain_id: SupportedChainId,
        env: CowEnv,
        quote_response: OrderQuoteResponse,
    ) -> Self {
        Self {
            context: ApiContext::new(chain_id, env),
            quote_response,
            state: Arc::new(Mutex::new(MockOrderbookState {
                order_id: Some(order_uid()),
                ..MockOrderbookState::default()
            })),
            quote_delay: None,
        }
    }

    pub const fn with_quote_delay(mut self, delay: std::time::Duration) -> Self {
        self.quote_delay = Some(delay);
        self
    }

    pub fn new_with_base_url(
        chain_id: SupportedChainId,
        env: CowEnv,
        base_url: &str,
        quote_response: OrderQuoteResponse,
    ) -> Self {
        let mut orderbook = Self::new_with_env(chain_id, env, quote_response);
        let mut base_urls = ApiBaseUrls::new();
        base_urls.insert(chain_id.into(), base_url.trim_end_matches('/').to_owned());
        orderbook.context.base_urls = Some(base_urls);
        orderbook
    }

    pub fn state(&self) -> MockOrderbookState {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn push_order(&self, order: Order) {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .orders
            .push(order);
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl OrderbookClient for MockOrderbook {
    fn context(&self) -> &ApiContext {
        &self.context
    }

    async fn quote(
        &self,
        request: &OrderQuoteRequest,
    ) -> Result<OrderQuoteResponse, OrderbookError> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .quote_requests
            .push(request.clone());
        if let Some(delay) = self.quote_delay {
            tokio::time::sleep(delay).await;
        }
        Ok(self.quote_response.clone())
    }

    async fn send_order(&self, request: &OrderCreation) -> Result<OrderUid, OrderbookError> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.sent_orders.push(request.clone());
        Ok(state.order_id.expect("test order id remains configured"))
    }

    async fn send_cancellations(&self, request: &OrderCancellations) -> Result<(), OrderbookError> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .cancellations
            .push(request.clone());
        Ok(())
    }

    async fn order(&self, _order_uid: &OrderUid) -> Result<Order, OrderbookError> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .orders
            .first()
            .cloned()
            .ok_or_else(|| OrderbookError::InvalidTransform {
                field: "mockOrder",
                reason: cow_sdk_core::ValidationReason::Precondition {
                    details: "fixture must register a mock order before dispatch",
                },
            })
    }

    async fn upload_app_data(
        &self,
        app_data_hash: &AppDataHash,
        full_app_data: &str,
    ) -> Result<(), OrderbookError> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .uploads
            .push((*app_data_hash, full_app_data.to_owned()));
        Ok(())
    }
}

pub struct CountingSigner {
    address: Address,
    sign_calls: AtomicUsize,
}

impl CountingSigner {
    pub const fn new(address: Address) -> Self {
        Self {
            address,
            sign_calls: AtomicUsize::new(0),
        }
    }

    pub fn sign_calls(&self) -> usize {
        self.sign_calls.load(Ordering::Relaxed)
    }

    fn record_sign_call(&self) {
        self.sign_calls.fetch_add(1, Ordering::Relaxed);
    }
}

impl Signer for CountingSigner {
    type Error = String;

    async fn address(&self) -> Result<Address, Self::Error> {
        Ok(self.address)
    }

    async fn sign_message(&self, _message: &[u8]) -> Result<String, Self::Error> {
        self.record_sign_call();
        Err(
            "CountingSigner::sign_message must not be reached under validator-first invariant"
                .into(),
        )
    }

    async fn sign_transaction(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
        Err(
            "CountingSigner::sign_transaction must not be reached under validator-first invariant"
                .into(),
        )
    }

    async fn sign_typed_data_payload(
        &self,
        _payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        self.record_sign_call();
        Err(
            "CountingSigner::sign_typed_data_payload must not be reached under validator-first invariant"
                .into(),
        )
    }

    async fn send_transaction(
        &self,
        _tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        Err(
            "CountingSigner::send_transaction must not be reached under validator-first invariant"
                .into(),
        )
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Err(
            "CountingSigner::estimate_gas must not be reached under validator-first invariant"
                .into(),
        )
    }
}

#[derive(Clone)]
pub struct MockSignerState {
    pub sent_transactions: Vec<TransactionRequest>,
    pub estimated_gas: Result<Amount, String>,
    pub tx_hash: Hash32,
    pub last_typed_data_domain: Option<TypedDataDomain>,
}

impl Default for MockSignerState {
    fn default() -> Self {
        Self {
            sent_transactions: Vec::new(),
            estimated_gas: Ok(Amount::new("125000").expect("test gas literal must be valid")),
            tx_hash: Hash32::new(TX_HASH).expect("test transaction hash literal must be valid"),
            last_typed_data_domain: None,
        }
    }
}

#[derive(Clone)]
pub struct MockSigner {
    pub address: Address,
    /// The address whose key actually produces the ECDSA signature. Defaults to
    /// `address` (an honest signer); a test sets it to a different known
    /// address to model a signer that reports one identity but signs with
    /// another, exercising the post-sign owner-recovery gate.
    pub sign_key_address: Address,
    pub state: Arc<Mutex<MockSignerState>>,
}

impl Default for MockSigner {
    fn default() -> Self {
        Self::new(address(OWNER))
    }
}

impl MockSigner {
    pub fn new(address: Address) -> Self {
        Self {
            address,
            sign_key_address: address,
            state: Arc::new(Mutex::new(MockSignerState::default())),
        }
    }

    /// Reports `self.address` but signs with the key for `sign_key_address`,
    /// modelling a signer that lies about its identity.
    #[must_use]
    pub fn with_sign_key_address(mut self, sign_key_address: Address) -> Self {
        self.sign_key_address = sign_key_address;
        self
    }

    pub fn state(&self) -> MockSignerState {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

/// The private key for a known deterministic test address (Anvil accounts),
/// used by `MockSigner` to produce a genuine ECDSA signature that recovers to
/// that address. `None` for addresses that never sign successfully (their
/// posts are rejected before the owner-recovery gate), which keep the canned
/// signature.
#[cfg(not(target_arch = "wasm32"))]
fn test_signing_key_for(signer: &Address) -> Option<&'static str> {
    match signer.to_hex_string().as_str() {
        // Anvil account 0 == OWNER.
        "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266" => {
            Some("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
        }
        // Anvil account 1 == THIRD_OWNER (app_data_merge override identity).
        "0x70997970c51812dc3a010c7d01b50e0d17dc79c8" => {
            Some("0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d")
        }
        _ => None,
    }
}

/// Produces a genuine EIP-712 signature over `payload` with the given test key,
/// so the post-sign owner-recovery gate recovers the key's address.
#[cfg(not(target_arch = "wasm32"))]
async fn real_sign_typed_data(key: &str, payload: &TypedDataPayload) -> Result<String, String> {
    let signer = cow_sdk_alloy_signer::LocalAlloySigner::builder()
        .private_key(key)
        .map_err(|error| error.to_string())?
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .map_err(|error| error.to_string())?;
    Signer::sign_typed_data_payload(&signer, payload)
        .await
        .map_err(|error| error.to_string())
}

impl Signer for MockSigner {
    type Error = String;

    async fn address(&self) -> Result<Address, Self::Error> {
        Ok(self.address)
    }

    async fn sign_message(&self, _message: &[u8]) -> Result<String, Self::Error> {
        Ok(MESSAGE_SIGNATURE.to_owned())
    }

    async fn sign_transaction(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
        Ok(TX_HASH.to_owned())
    }

    async fn sign_typed_data_payload(
        &self,
        payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .last_typed_data_domain = Some(payload.domain.clone());
        // On native targets sign for real with the test key for the signing
        // identity, so the produced signature recovers to that address and the
        // post-sign owner-recovery gate accepts (or rejects) it on its merits.
        // Addresses without a known key (and the wasm lane, which never reaches
        // the gate) keep the canned signature.
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(key) = test_signing_key_for(&self.sign_key_address) {
            return real_sign_typed_data(key, payload).await;
        }
        Ok(TYPED_SIGNATURE.to_owned())
    }

    async fn send_transaction(
        &self,
        tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.sent_transactions.push(tx.clone());
        Ok(TransactionBroadcast::new(state.tx_hash))
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .estimated_gas
            .clone()
    }
}

#[derive(Clone, Default)]
pub struct MockProviderState {
    pub last_contract_call: Option<ContractCall>,
    pub allowance: String,
    pub contract_responses: BTreeMap<String, String>,
    pub code_by_address: BTreeMap<String, HexData>,
    pub read_contract_error: Option<String>,
    pub get_code_error: Option<String>,
}

#[derive(Clone)]
pub struct MockProvider {
    pub state: Arc<Mutex<MockProviderState>>,
}

impl Default for MockProvider {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(MockProviderState {
                allowance: "1000000000000000000".to_owned(),
                contract_responses: BTreeMap::new(),
                code_by_address: BTreeMap::new(),
                read_contract_error: None,
                get_code_error: None,
                ..MockProviderState::default()
            })),
        }
    }
}

impl MockProvider {
    pub fn state(&self) -> MockProviderState {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn set_code(&self, address: &Address, code: &str) {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .code_by_address
            .insert(
                address.to_hex_string(),
                HexData::new(code).expect("mock code must be valid hex"),
            );
    }

    pub fn set_contract_response(&self, method: &str, response: &str) {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contract_responses
            .insert(method.to_owned(), response.to_owned());
    }

    pub fn set_read_contract_error(&self, message: Option<&str>) {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .read_contract_error = message.map(str::to_owned);
    }

    pub fn set_get_code_error(&self, message: Option<&str>) {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_code_error = message.map(str::to_owned);
    }
}

impl Provider for MockProvider {
    type Error = String;

    async fn get_chain_id(&self) -> Result<u64, Self::Error> {
        Ok(u64::from(SupportedChainId::Mainnet))
    }

    async fn get_code(&self, address: &Address) -> Result<Option<HexData>, Self::Error> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(message) = state.get_code_error.clone() {
            return Err(message);
        }
        Ok(state.code_by_address.get(&address.to_hex_string()).cloned())
    }

    async fn get_transaction_receipt(
        &self,
        _transaction_hash: &Hash32,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        Ok(None)
    }

    async fn get_storage_at(
        &self,
        _address: &Address,
        _slot: &str,
    ) -> Result<HexData, Self::Error> {
        Ok(HexData::empty())
    }

    async fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        Ok(HexData::empty())
    }

    async fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.last_contract_call = Some(request.clone());
        if let Some(message) = state.read_contract_error.clone() {
            return Err(message);
        }
        if request.method == "allowance" {
            Ok(state.allowance.clone())
        } else {
            state
                .contract_responses
                .get(&request.method)
                .cloned()
                .ok_or_else(|| format!("missing mock contract response for {}", request.method))
        }
    }

    async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Ok(BlockInfo::new(0, None))
    }

    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle::new(*address, abi_json.to_owned()))
    }
}

pub struct MockSlippageProvider {
    pub response: Option<u32>,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl SlippageSuggester for MockSlippageProvider {
    async fn slippage_suggestion(
        &self,
        _request: SlippageToleranceRequest,
    ) -> Result<SlippageToleranceResponse, TradingError> {
        let mut response = SlippageToleranceResponse::new();
        if let Some(bps) = self.response {
            response = response.with_slippage_bps(bps);
        }
        Ok(response)
    }
}

pub struct MockEthFlowChecker {
    pub results: Arc<Mutex<Vec<bool>>>,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl EthFlowOrderExistsChecker for MockEthFlowChecker {
    async fn order_exists(
        &self,
        _order_id: &OrderUid,
        _order_digest: &cow_sdk_core::OrderDigest,
    ) -> Result<bool, TradingError> {
        let mut results = self
            .results
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(if results.is_empty() {
            false
        } else {
            results.remove(0)
        })
    }
}

pub struct MockEip1271Provider;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Eip1271Signer for MockEip1271Provider {
    async fn sign(
        &self,
        _order_to_sign: &cow_sdk_core::OrderData,
    ) -> Result<String, Eip1271SignatureError> {
        Ok("0x7e57c0de".to_owned())
    }
}

pub fn regular_order() -> Order {
    serde_json::from_value(json!({
        "sellToken": WETH,
        "buyToken": COW,
        "receiver": OWNER,
        "sellAmount": "1000000000000000000",
        "buyAmount": "500000000000000000",
        "validTo": 1_234_567_890_u32,
        "appData": APP_DATA_HASH,
        "feeAmount": "10000000000000000",
        "kind": "sell",
        "partiallyFillable": false,
        "sellTokenBalance": "erc20",
        "buyTokenBalance": "erc20",
        "signingScheme": "eip712",
        "signature": TYPED_SIGNATURE,
        "class": "market",
        "owner": OWNER,
        "uid": ORDER_UID,
        "executedSellAmount": "0",
        "executedBuyAmount": "0",
        "settlementContract": CUSTOM_SETTLEMENT,
        "invalidated": false,
        "status": "open",
        "totalFee": "0"
    }))
    .expect("regular order fixture must deserialize")
}

pub fn ethflow_order() -> Order {
    let mut order = regular_order();
    order.ethflow_data = Some(cow_sdk_orderbook::EthflowData::new(order.valid_to));
    order
}

pub const BLOCK_HASH: &str = "0x2468ace013579bdf2468ace013579bdf2468ace013579bdf2468ace013579bdf";
pub const TO: &str = "0x70997970c51812dc3a010c7d01b50e0d17dc79c8";

pub fn test_hash() -> TransactionHash {
    TransactionHash::new(TX_HASH).expect("test transaction hash literal must be valid")
}

pub fn test_block_hash() -> BlockHash {
    BlockHash::new(BLOCK_HASH).expect("test block hash literal must be valid")
}

pub fn test_from_address() -> Address {
    address(OWNER)
}

pub fn test_to_address() -> Address {
    address(TO)
}

pub fn rich_receipt_fixture() -> TransactionReceipt {
    TransactionReceipt::from_parts(
        test_hash(),
        Some(TransactionStatus::Success),
        Some(1_234),
        Some(test_block_hash()),
        Some(Amount::from(21_000u64)),
        Some(test_from_address()),
        Some(test_to_address()),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FakeSignerError {
    Boom,
}

impl std::fmt::Display for FakeSignerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Boom => f.write_str("fake signer boom"),
        }
    }
}

impl std::error::Error for FakeSignerError {}

#[derive(Clone)]
pub struct FakeSigner {
    outcome: Arc<FakeSignerOutcome>,
}

#[derive(Clone)]
enum FakeSignerOutcome {
    Broadcast(TransactionHash),
    Error(FakeSignerError),
}

impl FakeSigner {
    pub fn with_broadcast(transaction_hash: TransactionHash) -> Self {
        Self {
            outcome: Arc::new(FakeSignerOutcome::Broadcast(transaction_hash)),
        }
    }

    pub fn with_error(error: FakeSignerError) -> Self {
        Self {
            outcome: Arc::new(FakeSignerOutcome::Error(error)),
        }
    }
}

impl Signer for FakeSigner {
    type Error = FakeSignerError;

    async fn address(&self) -> Result<Address, Self::Error> {
        Ok(test_from_address())
    }

    async fn sign_message(&self, _message: &[u8]) -> Result<String, Self::Error> {
        Ok(MESSAGE_SIGNATURE.to_owned())
    }

    async fn sign_transaction(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
        Ok(TX_HASH.to_owned())
    }

    async fn sign_typed_data_payload(
        &self,
        _payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        Ok(TYPED_SIGNATURE.to_owned())
    }

    async fn send_transaction(
        &self,
        _tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        match self.outcome.as_ref() {
            FakeSignerOutcome::Broadcast(transaction_hash) => {
                Ok(TransactionBroadcast::new(*transaction_hash))
            }
            FakeSignerOutcome::Error(error) => Err(*error),
        }
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Ok(Amount::from(21_000u64))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FakeProviderError {
    LookupFailed,
}

impl std::fmt::Display for FakeProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LookupFailed => f.write_str("fake provider lookup failed"),
        }
    }
}

impl std::error::Error for FakeProviderError {}

#[derive(Clone)]
pub struct FakeProvider {
    state: Arc<Mutex<FakeProviderState>>,
}

#[derive(Clone)]
struct FakeProviderState {
    poll_count: usize,
    receipt_after_poll: Option<usize>,
    receipt: Option<TransactionReceipt>,
    fail_first_poll: bool,
}

impl FakeProvider {
    pub fn with_receipt_after_polls(polls: usize, receipt: TransactionReceipt) -> Self {
        Self {
            state: Arc::new(Mutex::new(FakeProviderState {
                poll_count: 0,
                receipt_after_poll: Some(polls.max(1)),
                receipt: Some(receipt),
                fail_first_poll: false,
            })),
        }
    }

    pub fn with_receipt_immediately_available(receipt: TransactionReceipt) -> Self {
        Self::with_receipt_after_polls(1, receipt)
    }

    pub fn never_yields_receipt() -> Self {
        Self {
            state: Arc::new(Mutex::new(FakeProviderState {
                poll_count: 0,
                receipt_after_poll: None,
                receipt: None,
                fail_first_poll: false,
            })),
        }
    }

    pub fn never_polled() -> Self {
        Self::never_yields_receipt()
    }

    pub fn with_lookup_error_on_first_poll() -> Self {
        Self {
            state: Arc::new(Mutex::new(FakeProviderState {
                poll_count: 0,
                receipt_after_poll: None,
                receipt: None,
                fail_first_poll: true,
            })),
        }
    }

    pub fn poll_count(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .poll_count
    }
}

impl Provider for FakeProvider {
    type Error = FakeProviderError;

    async fn get_chain_id(&self) -> Result<u64, Self::Error> {
        Ok(u64::from(SupportedChainId::Mainnet))
    }

    async fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
        Ok(None)
    }

    async fn get_transaction_receipt(
        &self,
        _transaction_hash: &TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.poll_count += 1;
        if state.fail_first_poll && state.poll_count == 1 {
            return Err(FakeProviderError::LookupFailed);
        }
        Ok(state
            .receipt_after_poll
            .filter(|poll| state.poll_count >= *poll)
            .and_then(|_| state.receipt.clone()))
    }

    async fn get_storage_at(
        &self,
        _address: &Address,
        _slot: &str,
    ) -> Result<HexData, Self::Error> {
        Ok(HexData::empty())
    }

    async fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        Ok(HexData::empty())
    }

    async fn read_contract(&self, _request: &ContractCall) -> Result<String, Self::Error> {
        Ok("null".to_owned())
    }

    async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Ok(BlockInfo::new(0, None))
    }

    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle::new(*address, abi_json.to_owned()))
    }
}
