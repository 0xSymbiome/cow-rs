use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::json;

use cow_sdk::core::{
    Amount, AppDataHex, BlockInfo, BuyTokenDestination, ContractCall, ContractHandle, Hash32,
    HexData, OrderKind, Provider, SellTokenSource, Signer, TransactionBroadcast,
    TransactionReceipt, TransactionRequest, TypedDataDomain, TypedDataField, OrderData,
};
use cow_sdk::orderbook::{
    ApiContext, AppDataHash, Order, OrderCancellations, OrderCreation, OrderQuoteRequest,
    OrderQuoteResponse, OrderbookError,
};
use cow_sdk::prelude::{Address, CowEnv, OrderUid, SupportedChainId, TradeParameters};
use cow_sdk::trading::{LimitTradeParameters, OrderbookClient, TraderParameters};
use wiremock::ResponseTemplate;

pub const WETH: &str = "0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14";
pub const COW: &str = "0x0625aFB445C3B6B7B929342a04A22599fd5dBB59";
pub const OWNER: &str = "0xc8c753Ee51E8Fc80e199AB297fB575634a1aC1d3";
pub const ALT_RECEIVER: &str = "0x974cAa59E49682CdA0aD2BbE82983419A2ECC400";
pub const SETTLEMENT: &str = "0x9008D19f58AAbD9eD0D60971565AA8510560ab41";
pub const ORDER_UID: &str = "0xd64389693b6cf89ad6c140a113b10df08073e5ef3063d05a02f3f42e1a42f0ad0b7795e18767259cc253a2af471dbc4c72b49516ffffffff";
pub const APP_DATA_HASH: &str =
    "0xe269b09f45b1d3c98d8e4e841b99a0779fbd3b77943d069b91ddc4fd9789e27e";
pub const TYPED_SIGNATURE: &str = "0x111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111b";
pub const MESSAGE_SIGNATURE: &str = "0x222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222221c";
pub const TX_HASH: &str = "0x13579bdf2468ace013579bdf2468ace013579bdf2468ace013579bdf2468ace0";

pub fn address(value: &str) -> Address {
    Address::new(value).expect("example address literal must remain valid")
}

pub fn sample_owner() -> Address {
    address(OWNER)
}

pub fn sample_sell_token() -> Address {
    address(WETH)
}

pub fn sample_buy_token() -> Address {
    address(COW)
}

pub fn sample_order_uid() -> OrderUid {
    OrderUid::new(ORDER_UID).expect("example order uid literal must remain valid")
}

pub fn sample_app_data_hash() -> AppDataHash {
    AppDataHash::new(APP_DATA_HASH).expect("example app-data hash must remain valid")
}

pub fn text_preview(value: &str, max_chars: usize) -> &str {
    if max_chars == 0 {
        return "";
    }

    value
        .char_indices()
        .nth(max_chars)
        .map_or(value, |(index, _)| &value[..index])
}

pub fn orderbook_version_response(version: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_raw(version.as_bytes(), "text/plain; charset=utf-8")
}

pub fn sample_unsigned_order() -> OrderData {
    OrderData::new(
        sample_sell_token(),
        sample_buy_token(),
        address(ALT_RECEIVER),
        Amount::parse_units("0.1", 18).expect("example sell amount must remain valid"),
        Amount::parse_units("0.25", 18).expect("example buy amount must remain valid"),
        1_700_000_000,
        AppDataHex::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .expect("example app-data hex must remain valid"),
        Amount::ZERO,
        OrderKind::Sell,
        false,
        SellTokenSource::Erc20,
        BuyTokenDestination::Erc20,
    )
}

pub fn sample_trade_parameters() -> TradeParameters {
    TradeParameters::new(
        OrderKind::Sell,
        sample_sell_token(),
        sample_buy_token(),
        Amount::parse_units("0.1", 18).expect("example trade amount must remain valid"),
    )
    .with_owner(sample_owner())
    .with_slippage_bps(50)
}

pub fn sample_limit_parameters() -> LimitTradeParameters {
    let quote = sample_quote_response();
    let sell_token_balance = quote.quote.sell_token_balance;
    let buy_token_balance = quote.quote.buy_token_balance;
    let quote_id = quote.id;

    let mut params = LimitTradeParameters::new(
        OrderKind::Sell,
        sample_sell_token(),
        sample_buy_token(),
        quote.quote.sell_amount.clone(),
        quote.quote.buy_amount.clone(),
    )
    .with_owner(sample_owner())
    .with_sell_token_balance(sell_token_balance)
    .with_buy_token_balance(buy_token_balance)
    .with_slippage_bps(0);
    if let Some(id) = quote_id {
        params = params.with_quote_id(id);
    }
    params
}

pub fn sample_trader_parameters() -> TraderParameters {
    TraderParameters::new(SupportedChainId::Sepolia, "cow-rs-native-examples")
        .expect("app code should validate")
        .with_env(CowEnv::Prod)
}

pub fn sample_quote_response() -> OrderQuoteResponse {
    serde_json::from_value(sample_quote_response_json())
        .expect("example quote response fixture must deserialize")
}

pub fn sample_quote_response_json() -> serde_json::Value {
    json!({
        "quote": {
            "sellToken": WETH,
            "buyToken": COW,
            "receiver": OWNER,
            "sellAmount": "98646335338956442",
            "buyAmount": "30000000000000000000",
            "validTo": 1737464594u32,
            "appData": APP_DATA_HASH,
            "feeAmount": "1353664661043558",
            "kind": "sell",
            "partiallyFillable": false,
            "sellTokenBalance": "erc20",
            "buyTokenBalance": "erc20"
        },
        "from": OWNER,
        "expiration": "2025-01-21T12:55:14.799709609Z",
        "id": 575401,
        "verified": true
    })
}

pub fn sample_signature() -> &'static str {
    TYPED_SIGNATURE
}

pub fn sample_open_order() -> Order {
    serde_json::from_value(json!({
        "sellToken": WETH,
        "buyToken": COW,
        "receiver": OWNER,
        "sellAmount": "1000000000000000000",
        "buyAmount": "500000000000000000",
        "validTo": 1234567890u32,
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
        "settlementContract": SETTLEMENT,
        "executedSellAmount": "0",
        "executedBuyAmount": "0",
        "invalidated": false,
        "status": "open",
        "totalFee": "0"
    }))
    .expect("example order fixture must deserialize")
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
        Self {
            context: ApiContext::new(chain_id, CowEnv::Prod),
            quote_response,
            state: Arc::new(Mutex::new(MockOrderbookState {
                order_id: Some(sample_order_uid()),
                ..MockOrderbookState::default()
            })),
        }
    }

    pub fn state(&self) -> MockOrderbookState {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn push_order(&self, order: Order) {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
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
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .quote_requests
            .push(request.clone());
        Ok(self.quote_response.clone())
    }

    async fn send_order(&self, request: &OrderCreation) -> Result<OrderUid, OrderbookError> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.sent_orders.push(request.clone());
        Ok(state
            .order_id
            .clone()
            .expect("example order id must stay configured"))
    }

    async fn send_signed_order_cancellations(
        &self,
        request: &OrderCancellations,
    ) -> Result<(), OrderbookError> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .cancellations
            .push(request.clone());
        Ok(())
    }

    async fn get_order(
        &self,
        order_uid: &OrderUid,
    ) -> Result<cow_sdk::orderbook::Order, OrderbookError> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .orders
            .iter()
            .find(|order| &order.uid == order_uid)
            .cloned()
            .ok_or_else(|| OrderbookError::InvalidTransform {
                field: "orderUid",
                reason: cow_sdk::core::ValidationReason::Precondition {
                    details: "requested order uid is not registered with the mock orderbook",
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
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .uploads
            .push((app_data_hash.clone(), full_app_data.to_owned()));
        Ok(())
    }
}

#[derive(Clone)]
pub struct MockSigner {
    address: Address,
    state: Arc<Mutex<MockSignerState>>,
}

#[derive(Clone)]
pub struct MockSignerState {
    pub sent_transactions: Vec<TransactionRequest>,
    pub estimated_gas: Result<Amount, String>,
    pub tx_hash: Hash32,
}

impl Default for MockSignerState {
    fn default() -> Self {
        Self {
            sent_transactions: Vec::new(),
            estimated_gas: Ok(Amount::new("125000").expect("example gas amount must remain valid")),
            tx_hash: Hash32::new(TX_HASH).expect("example tx hash must remain valid"),
        }
    }
}

impl Default for MockSigner {
    fn default() -> Self {
        Self::new(sample_owner())
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
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

impl Signer for MockSigner {
    type Error = String;

    async fn get_address(&self) -> Result<Address, Self::Error> {
        Ok(self.address.clone())
    }

    async fn sign_message(&self, _message: &[u8]) -> Result<String, Self::Error> {
        Ok(MESSAGE_SIGNATURE.to_owned())
    }

    async fn sign_transaction(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
        Ok(TX_HASH.to_owned())
    }

    async fn sign_typed_data(
        &self,
        _domain: &TypedDataDomain,
        _fields: &[TypedDataField],
        _value_json: &str,
    ) -> Result<String, Self::Error> {
        Ok(TYPED_SIGNATURE.to_owned())
    }

    async fn send_transaction(
        &self,
        tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.sent_transactions.push(tx.clone());
        Ok(TransactionBroadcast::new(state.tx_hash.clone()))
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .estimated_gas
            .clone()
    }
}

#[derive(Clone)]
pub struct MockProvider {
    pub signer: Option<MockSigner>,
    state: Arc<Mutex<MockProviderState>>,
}

#[derive(Clone)]
pub struct MockProviderState {
    pub last_contract_call: Option<ContractCall>,
    pub allowance: String,
}

impl Default for MockProviderState {
    fn default() -> Self {
        Self {
            last_contract_call: None,
            allowance: "1000000000000000000".to_owned(),
        }
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self {
            signer: None,
            state: Arc::new(Mutex::new(MockProviderState::default())),
        }
    }
}

impl MockProvider {
    pub fn state(&self) -> MockProviderState {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

impl Provider for MockProvider {
    type Error = String;

    async fn get_chain_id(&self) -> Result<u64, Self::Error> {
        Ok(SupportedChainId::Sepolia.into())
    }

    async fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
        Ok(None)
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
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.last_contract_call = Some(request.clone());
        Ok(state.allowance.clone())
    }

    async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Ok(BlockInfo::new(0, None))
    }

    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle::new(address.clone(), abi_json.to_owned()))
    }
}

impl cow_sdk::core::SigningProvider for MockProvider {
    type Signer = MockSigner;

    async fn create_signer(&self, _signer_hint: &str) -> Result<Self::Signer, Self::Error> {
        Ok(self.signer.clone().unwrap_or_default())
    }
}
