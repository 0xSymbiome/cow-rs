use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::json;

use cow_sdk::core::{
    BlockInfo, ContractCall, ContractHandle, Provider, Signer, TransactionReceipt,
    TransactionRequest, TypedDataDomain, TypedDataField,
};
use cow_sdk::orderbook::{
    ApiContext, AppDataHash, AppDataObject, Order, OrderCancellations, OrderCreation,
    OrderQuoteRequest, OrderQuoteResponse, OrderbookError,
};
use cow_sdk::trading::OrderbookClient;
use cow_sdk::{
    Address, AppDataHex, CowEnv, OrderBalance, OrderKind, OrderUid, SupportedChainId,
    TradeParameters, TraderParameters, UnsignedOrder,
};

pub const WETH: &str = "0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14";
pub const COW: &str = "0x0625aFB445C3B6B7B929342a04A22599fd5dBB59";
pub const OWNER: &str = "0xc8c753Ee51E8Fc80e199AB297fB575634a1aC1d3";
pub const ALT_RECEIVER: &str = "0x974cAa59E49682CdA0aD2BbE82983419A2ECC400";
pub const ORDER_UID: &str = "0xd64389693b6cf89ad6c140a113b10df08073e5ef3063d05a02f3f42e1a42f0ad0b7795e18767259cc253a2af471dbc4c72b49516ffffffff";
pub const APP_DATA_HASH: &str =
    "0xe269b09f45b1d3c98d8e4e841b99a0779fbd3b77943d069b91ddc4fd9789e27e";
pub const TYPED_SIGNATURE: &str = "0x1111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111b";
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

pub fn sample_unsigned_order() -> UnsignedOrder {
    UnsignedOrder {
        sell_token: sample_sell_token(),
        buy_token: sample_buy_token(),
        receiver: address(ALT_RECEIVER),
        sell_amount: "100000000000000000".to_owned(),
        buy_amount: "250000000000000000".to_owned(),
        valid_to: 1_700_000_000,
        app_data: AppDataHex::new(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .expect("example app-data hex must remain valid"),
        fee_amount: "0".to_owned(),
        kind: OrderKind::Sell,
        partially_fillable: false,
        sell_token_balance: OrderBalance::Erc20,
        buy_token_balance: OrderBalance::Erc20,
    }
}

pub fn sample_trade_parameters() -> TradeParameters {
    TradeParameters {
        kind: OrderKind::Sell,
        owner: Some(sample_owner()),
        sell_token: sample_sell_token(),
        sell_token_decimals: 18,
        buy_token: sample_buy_token(),
        buy_token_decimals: 18,
        amount: "100000000000000000".to_owned(),
        env: None,
        settlement_contract_override: None,
        eth_flow_contract_override: None,
        partially_fillable: false,
        slippage_bps: Some(50),
        receiver: None,
        valid_for: None,
        valid_to: None,
        partner_fee: None,
    }
}

pub fn sample_limit_parameters() -> cow_sdk::LimitTradeParameters {
    let quote = sample_quote_response();

    cow_sdk::LimitTradeParameters {
        kind: OrderKind::Sell,
        owner: Some(sample_owner()),
        sell_token: sample_sell_token(),
        sell_token_decimals: 18,
        buy_token: sample_buy_token(),
        buy_token_decimals: 18,
        sell_amount: quote.quote.sell_amount,
        buy_amount: quote.quote.buy_amount,
        quote_id: quote.id,
        env: None,
        settlement_contract_override: None,
        eth_flow_contract_override: None,
        partially_fillable: false,
        slippage_bps: Some(0),
        receiver: None,
        valid_for: None,
        valid_to: None,
        partner_fee: None,
    }
}

pub fn sample_trader_parameters() -> TraderParameters {
    TraderParameters {
        chain_id: SupportedChainId::Sepolia,
        app_code: "cow-rs-native-examples".to_owned(),
        env: Some(CowEnv::Prod),
        settlement_contract_override: None,
        eth_flow_contract_override: None,
    }
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
            context: ApiContext {
                chain_id,
                env: CowEnv::Prod,
                base_urls: None,
                api_key: None,
            },
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
        _order_uid: &OrderUid,
    ) -> Result<cow_sdk::orderbook::Order, OrderbookError> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
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
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .uploads
            .push((app_data_hash.clone(), full_app_data.to_owned()));
        Ok(AppDataObject {
            full_app_data: full_app_data.to_owned(),
        })
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
    pub estimated_gas: Result<String, String>,
    pub tx_hash: String,
}

impl Default for MockSignerState {
    fn default() -> Self {
        Self {
            sent_transactions: Vec::new(),
            estimated_gas: Ok("125000".to_owned()),
            tx_hash: TX_HASH.to_owned(),
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
        _domain: &TypedDataDomain,
        _fields: &[TypedDataField],
        _value_json: &str,
    ) -> Result<String, Self::Error> {
        Ok(TYPED_SIGNATURE.to_owned())
    }

    fn send_transaction(&self, tx: &TransactionRequest) -> Result<TransactionReceipt, Self::Error> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.sent_transactions.push(tx.clone());
        Ok(TransactionReceipt {
            transaction_hash: state.tx_hash.clone(),
        })
    }

    fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
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
    type Signer = MockSigner;
    type Error = String;

    fn signer_or_null(&self) -> Option<&Self::Signer> {
        self.signer.as_ref()
    }

    fn get_chain_id(&self) -> Result<u64, Self::Error> {
        Ok(SupportedChainId::Sepolia.into())
    }

    fn get_code(&self, _address: &Address) -> Result<Option<String>, Self::Error> {
        Ok(None)
    }

    fn get_transaction_receipt(
        &self,
        _transaction_hash: &str,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        Ok(None)
    }

    fn create_signer(&self, _signer_hint: &str) -> Result<Self::Signer, Self::Error> {
        Ok(self.signer.clone().unwrap_or_default())
    }

    fn get_storage_at(&self, _address: &Address, _slot: &str) -> Result<String, Self::Error> {
        Ok(String::new())
    }

    fn call(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
        Ok(String::new())
    }

    fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.last_contract_call = Some(request.clone());
        Ok(state.allowance.clone())
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
