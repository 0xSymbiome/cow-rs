#![allow(dead_code)]

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use serde_json::json;

use cow_sdk_core::{
    Address, Amount, ApiBaseUrls, ApiContext, AppDataHash, BlockInfo, ContractCall, ContractHandle,
    CowEnv, Hash32, HexData, OrderKind, OrderUid, Provider, Signer, SupportedChainId,
    TransactionReceipt, TransactionRequest, TypedDataDomain, TypedDataField,
};
use cow_sdk_orderbook::{
    AppDataObject, Order, OrderCancellations, OrderCreation, OrderQuoteRequest, OrderQuoteResponse,
    OrderbookError,
};
use cow_sdk_trading::{
    Eip1271SignatureProvider, EthFlowOrderExistsChecker, OrderbookClient,
    SlippageSuggestionProvider, SlippageToleranceRequest, SlippageToleranceResponse, TradingError,
};

pub const WETH: &str = "0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14";
pub const COW: &str = "0x0625aFB445C3B6B7B929342a04A22599fd5dBB59";
pub const OWNER: &str = "0xc8c753Ee51E8Fc80e199AB297fB575634a1aC1d3";
pub const ALT_RECEIVER: &str = "0x974cAa59E49682CdA0aD2BbE82983419A2ECC400";
pub const CUSTOM_SETTLEMENT: &str = "0x13579bdf2468ace013579bdf2468ace013579bdf";
pub const CUSTOM_ETHFLOW: &str = "0x2468ace013579bdf2468ace013579bdf2468ace0";
pub const TX_HASH: &str = "0x13579bdf2468ace013579bdf2468ace013579bdf2468ace013579bdf2468ace0";
pub const ORDER_UID: &str = "0xd64389693b6cf89ad6c140a113b10df08073e5ef3063d05a02f3f42e1a42f0ad0b7795e18767259cc253a2af471dbc4c72b49516ffffffff";
pub const APP_DATA_HASH: &str =
    "0xe269b09f45b1d3c98d8e4e841b99a0779fbd3b77943d069b91ddc4fd9789e27e";
pub const TYPED_SIGNATURE: &str = "0x1111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111b";
pub const MESSAGE_SIGNATURE: &str = "0x222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222221c";

pub fn trading_fixture() -> serde_json::Value {
    serde_json::from_str(include_str!("../../../../parity/fixtures/trading.json"))
        .expect("trading parity fixture must remain valid json")
}

pub fn address(value: &str) -> Address {
    Address::new(value).expect("test address literal must be valid")
}

pub fn order_uid() -> OrderUid {
    OrderUid::new(ORDER_UID).expect("test order uid literal must be valid")
}

pub fn app_data_hash() -> AppDataHash {
    AppDataHash::new(APP_DATA_HASH).expect("test app-data hash literal must be valid")
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

pub fn sample_trade_parameters(kind: OrderKind) -> cow_sdk_trading::TradeParameters {
    cow_sdk_trading::TradeParameters {
        kind,
        owner: Some(address(OWNER)),
        sell_token: address(WETH),
        sell_token_decimals: 18,
        buy_token: address(COW),
        buy_token_decimals: 18,
        amount: if kind == OrderKind::Sell {
            Amount::new("100000000000000000").expect("test sell amount literal must be valid")
        } else {
            Amount::new("400000000000000000000").expect("test buy amount literal must be valid")
        },
        env: None,
        settlement_contract_override: None,
        eth_flow_contract_override: None,
        partially_fillable: false,
        sell_token_balance: cow_sdk_core::OrderBalance::Erc20,
        buy_token_balance: cow_sdk_core::OrderBalance::Erc20,
        slippage_bps: Some(50),
        receiver: None,
        valid_for: None,
        valid_to: None,
        partner_fee: None,
    }
}

pub fn sample_trader_parameters() -> cow_sdk_trading::TraderParameters {
    cow_sdk_trading::TraderParameters {
        chain_id: SupportedChainId::Sepolia,
        app_code: "0x007".to_owned(),
        env: Some(CowEnv::Prod),
        settlement_contract_override: None,
        eth_flow_contract_override: None,
    }
}

pub fn sample_limit_parameters(kind: OrderKind) -> cow_sdk_trading::LimitTradeParameters {
    let quote = if kind == OrderKind::Sell {
        sell_quote_response()
    } else {
        buy_quote_response()
    };

    cow_sdk_trading::LimitTradeParameters {
        kind,
        owner: Some(address(OWNER)),
        sell_token: address(WETH),
        sell_token_decimals: 18,
        buy_token: address(COW),
        buy_token_decimals: 18,
        sell_amount: Amount::new(quote.quote.sell_amount.clone())
            .expect("quote sell amount literal must be valid"),
        buy_amount: Amount::new(quote.quote.buy_amount.clone())
            .expect("quote buy amount literal must be valid"),
        quote_id: quote.id,
        env: None,
        settlement_contract_override: None,
        eth_flow_contract_override: None,
        partially_fillable: false,
        sell_token_balance: quote.quote.sell_token_balance,
        buy_token_balance: quote.quote.buy_token_balance,
        slippage_bps: Some(50),
        receiver: None,
        valid_for: None,
        valid_to: None,
        partner_fee: None,
    }
}

#[derive(Clone)]
pub struct MockOrderbook {
    context: ApiContext,
    quote_response: OrderQuoteResponse,
    state: Arc<Mutex<MockOrderbookState>>,
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
        }
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

    async fn get_quote(
        &self,
        request: &OrderQuoteRequest,
    ) -> Result<OrderQuoteResponse, OrderbookError> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .quote_requests
            .push(request.clone());
        Ok(self.quote_response.clone())
    }

    async fn send_order(&self, request: &OrderCreation) -> Result<OrderUid, OrderbookError> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.sent_orders.push(request.clone());
        Ok(state
            .order_id
            .clone()
            .expect("test order id remains configured"))
    }

    async fn send_signed_order_cancellations(
        &self,
        request: &OrderCancellations,
    ) -> Result<(), OrderbookError> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .cancellations
            .push(request.clone());
        Ok(())
    }

    async fn get_order(&self, _order_uid: &OrderUid) -> Result<Order, OrderbookError> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .orders
            .first()
            .cloned()
            .ok_or_else(|| OrderbookError::InvalidTransform("missing mock order".to_owned()))
    }

    async fn upload_app_data(
        &self,
        app_data_hash: &AppDataHash,
        full_app_data: &str,
    ) -> Result<AppDataObject, OrderbookError> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .uploads
            .push((app_data_hash.clone(), full_app_data.to_owned()));
        Ok(AppDataObject {
            full_app_data: full_app_data.to_owned(),
        })
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
            state: Arc::new(Mutex::new(MockSignerState::default())),
        }
    }

    pub fn state(&self) -> MockSignerState {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl Signer for MockSigner {
    type Provider = ();
    type Error = String;

    fn connect(&mut self, _provider: Self::Provider) {}

    fn get_address(&self) -> Result<Address, Self::Error> {
        Ok(self.address.clone())
    }

    fn sign_message(&self, _message: &[u8]) -> Result<String, Self::Error> {
        Ok(MESSAGE_SIGNATURE.to_owned())
    }

    fn sign_transaction(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
        Ok(TX_HASH.to_owned())
    }

    fn sign_typed_data(
        &self,
        domain: &TypedDataDomain,
        _fields: &[TypedDataField],
        _value_json: &str,
    ) -> Result<String, Self::Error> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .last_typed_data_domain = Some(domain.clone());
        Ok(TYPED_SIGNATURE.to_owned())
    }

    fn send_transaction(&self, tx: &TransactionRequest) -> Result<TransactionReceipt, Self::Error> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.sent_transactions.push(tx.clone());
        Ok(TransactionReceipt {
            transaction_hash: state.tx_hash.clone(),
        })
    }

    fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
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
    pub signer: Option<MockSigner>,
    pub state: Arc<Mutex<MockProviderState>>,
}

impl Default for MockProvider {
    fn default() -> Self {
        Self {
            signer: None,
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
                address.normalized_key(),
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
    type Signer = MockSigner;
    type Error = String;

    fn signer_or_null(&self) -> Option<&Self::Signer> {
        self.signer.as_ref()
    }

    fn get_chain_id(&self) -> Result<u64, Self::Error> {
        Ok(u64::from(SupportedChainId::Mainnet))
    }

    fn get_code(&self, address: &Address) -> Result<Option<HexData>, Self::Error> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(message) = state.get_code_error.clone() {
            return Err(message);
        }
        Ok(state
            .code_by_address
            .get(&address.normalized_key())
            .cloned())
    }

    fn get_transaction_receipt(
        &self,
        _transaction_hash: &Hash32,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        Ok(None)
    }

    fn create_signer(&self, _signer_hint: &str) -> Result<Self::Signer, Self::Error> {
        Ok(self.signer.clone().unwrap_or_default())
    }

    fn get_storage_at(&self, _address: &Address, _slot: &str) -> Result<HexData, Self::Error> {
        Ok(HexData::empty())
    }

    fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        Ok(HexData::empty())
    }

    fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
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

    fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Ok(BlockInfo {
            number: 0,
            hash: None,
        })
    }

    fn set_signer(&mut self, signer: Self::Signer) {
        self.signer = Some(signer);
    }

    fn set_provider(&mut self, _provider_hint: String) {}

    fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle {
            address: address.clone(),
            abi_json: abi_json.to_owned(),
        })
    }
}

pub struct MockSlippageProvider {
    pub response: Option<u32>,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl SlippageSuggestionProvider for MockSlippageProvider {
    async fn get_slippage_suggestion(
        &self,
        _request: SlippageToleranceRequest,
    ) -> Result<SlippageToleranceResponse, TradingError> {
        Ok(SlippageToleranceResponse {
            slippage_bps: self.response,
        })
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
impl Eip1271SignatureProvider for MockEip1271Provider {
    async fn sign(
        &self,
        _order_to_sign: &cow_sdk_core::UnsignedOrder,
    ) -> Result<String, TradingError> {
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
        "invalidated": false,
        "status": "open",
        "totalFee": "0"
    }))
    .expect("regular order fixture must deserialize")
}

pub fn ethflow_order() -> Order {
    let mut order = regular_order();
    order.ethflow_data = Some(cow_sdk_orderbook::EthflowData {
        refund_tx_hash: None,
        user_valid_to: order.valid_to,
    });
    order
}
