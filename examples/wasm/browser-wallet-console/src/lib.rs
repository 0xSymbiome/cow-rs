use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;
use wasm_bindgen::prelude::*;

use cow_sdk::browser_wallet::{BrowserWallet, MockEip1193Transport};
use cow_sdk::core::{AppDataHash, wrapped_native_token};
use cow_sdk::orderbook::AppDataObject;
use cow_sdk::trading::OrderbookClient;
use cow_sdk::{
    Address, ApiContext, ApprovalParameters, AsyncSigner, CowEnv, OrderBookApi,
    OrderCancellations, OrderCreation, OrderQuoteRequest, OrderQuoteResponse, OrderTraderParameters,
    OrderUid, PartialTraderParameters, SupportedChainId, TradeParameters, TradingSdk,
    TradingSdkOptions, approval_transaction, generate_order_id, sign_order_async,
};

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct BrowserWalletConsole {
    mock_transport: MockEip1193Transport,
    mock_wallet: BrowserWallet,
    injected_wallet: Mutex<Option<BrowserWallet>>,
    last_live_order_uid: Mutex<Option<String>>,
}

#[wasm_bindgen]
impl BrowserWalletConsole {
    #[wasm_bindgen(constructor)]
    pub fn new() -> BrowserWalletConsole {
        let mock_transport = MockEip1193Transport::sepolia();
        let mock_wallet = BrowserWallet::from_transport(mock_transport.clone());
        Self {
            mock_transport,
            mock_wallet,
            injected_wallet: Mutex::new(None),
            last_live_order_uid: Mutex::new(None),
        }
    }

    pub fn sample_trade_json(&self, chain_id: u32) -> Result<String, JsValue> {
        let chain_id = parse_chain_id(chain_id)?;
        pretty_json(&sample_trade_parameters(chain_id))
    }

    pub fn sample_order_json(&self, chain_id: u32) -> Result<String, JsValue> {
        let chain_id = parse_chain_id(chain_id)?;
        let order = sample_unsigned_order(chain_id);
        pretty_json(&order)
    }

    pub fn sample_approval_json(&self, chain_id: u32) -> Result<String, JsValue> {
        let chain_id = parse_chain_id(chain_id)?;
        let approval = sample_approval_parameters(chain_id);
        pretty_json(&approval)
    }

    pub fn mock_status_json(&self) -> Result<String, JsValue> {
        pretty_json(&json!({
            "wallet": self.mock_wallet.session(),
            "requestLog": self.mock_transport.request_log(),
        }))
    }

    pub async fn mock_connect_json(&self) -> Result<String, JsValue> {
        let session = self.mock_wallet.connect().await.map_err(js_string_error)?;
        pretty_json(&json!({
            "mode": "mock",
            "session": session,
            "events": self.mock_wallet.take_events(),
        }))
    }

    pub async fn mock_sign_message_json(&self, message: &str) -> Result<String, JsValue> {
        let signer = self.mock_wallet.signer();
        let signature = signer
            .sign_message(message.as_bytes())
            .await
            .map_err(js_string_error)?;
        pretty_json(&json!({
            "mode": "mock",
            "message": message,
            "signature": signature,
            "events": self.mock_wallet.take_events(),
            "requestLog": self.mock_transport.request_log(),
        }))
    }

    pub async fn mock_approval_flow_json(
        &self,
        chain_id: u32,
        env: &str,
        approval_json: &str,
    ) -> Result<String, JsValue> {
        let chain_id = parse_chain_id(chain_id)?;
        let env = parse_env(env)?;
        let approval: ApprovalParameters = parse_json(approval_json, "approvalParameters")?;
        let tx = approval_transaction(&approval, chain_id, env).map_err(js_string_error)?;
        let signer = self.mock_wallet.signer();
        let estimated_gas = signer.estimate_gas(&tx).await.map_err(js_string_error)?;
        let receipt = signer.send_transaction(&tx).await.map_err(js_string_error)?;

        pretty_json(&json!({
            "mode": "mock",
            "transaction": tx,
            "estimatedGas": estimated_gas,
            "transactionHash": receipt.transaction_hash,
            "events": self.mock_wallet.take_events(),
            "requestLog": self.mock_transport.request_log(),
        }))
    }

    pub async fn mock_trading_flow_json(
        &self,
        chain_id: u32,
        env: &str,
        app_code: &str,
        trade_json: &str,
    ) -> Result<String, JsValue> {
        let chain_id = parse_chain_id(chain_id)?;
        let env = parse_env(env)?;
        let trade = parse_trade_parameters(trade_json)?;
        let mock_orderbook = Arc::new(MockBrowserOrderbook::new(chain_id, env));
        let sdk = TradingSdk::new(
            PartialTraderParameters {
                chain_id: Some(chain_id),
                app_code: Some(app_code.trim().to_owned()),
                owner: None,
                env: Some(env),
                ..Default::default()
            },
            TradingSdkOptions {
                order_book_api: Some(mock_orderbook.clone()),
            },
        );
        let signer = self.mock_wallet.signer();
        let posting = sdk
            .post_swap_order_async(trade, &signer, None)
            .await
            .map_err(js_string_error)?;
        let cancellation = sdk
            .off_chain_cancel_order_async(
            &OrderTraderParameters {
                order_uid: posting.order_id.clone(),
                chain_id: Some(chain_id),
                env: Some(env),
                settlement_contract_override: None,
                eth_flow_contract_override: None,
            },
            &signer,
        )
        .await
        .map_err(js_string_error)?;

        pretty_json(&json!({
            "mode": "mock",
            "posting": posting,
            "cancellationAccepted": cancellation,
            "orderbookState": mock_orderbook.snapshot(),
            "walletEvents": self.mock_wallet.take_events(),
            "walletRequestLog": self.mock_transport.request_log(),
        }))
    }

    pub fn injected_detection_json(&self) -> Result<String, JsValue> {
        let wallet = BrowserWallet::detect().map_err(js_string_error)?;
        pretty_json(&json!({
            "available": wallet.is_some(),
            "walletInfo": wallet.as_ref().and_then(BrowserWallet::injected_info),
        }))
    }

    pub async fn injected_connect_json(&self) -> Result<String, JsValue> {
        let wallet = BrowserWallet::detect()
            .map_err(js_string_error)?
            .ok_or_else(|| to_js_error("no injected wallet detected"))?;
        let session = wallet.connect().await.map_err(js_string_error)?;
        *self.injected_wallet.lock().unwrap() = Some(wallet.clone());
        pretty_json(&json!({
            "mode": "injected",
            "session": session,
            "walletInfo": wallet.injected_info(),
            "events": wallet.take_events(),
        }))
    }

    pub fn injected_reset_session_json(&self) -> Result<String, JsValue> {
        let wallet = self.injected_wallet()?;
        let session = wallet.reset_session();
        let events = wallet.take_events();
        *self.injected_wallet.lock().unwrap() = None;
        *self.last_live_order_uid.lock().unwrap() = None;
        pretty_json(&json!({
            "mode": "injected",
            "session": session,
            "note": "local session cleared; extension authorization remains managed by the wallet",
            "events": events,
        }))
    }

    pub async fn injected_refresh_json(&self) -> Result<String, JsValue> {
        let wallet = self.injected_wallet()?;
        let session = wallet.refresh_session().await.map_err(js_string_error)?;
        pretty_json(&json!({
            "mode": "injected",
            "session": session,
            "events": wallet.take_events(),
        }))
    }

    pub async fn injected_switch_chain_json(&self, chain_id: u32) -> Result<String, JsValue> {
        let wallet = self.injected_wallet()?;
        let chain_id = parse_chain_id(chain_id)?;
        let session = wallet.switch_chain(chain_id).await.map_err(js_string_error)?;
        pretty_json(&json!({
            "mode": "injected",
            "session": session,
            "events": wallet.take_events(),
        }))
    }

    pub async fn injected_sign_message_json(&self, message: &str) -> Result<String, JsValue> {
        let wallet = self.injected_wallet()?;
        let signer = wallet.signer();
        let signature = signer
            .sign_message(message.as_bytes())
            .await
            .map_err(js_string_error)?;
        pretty_json(&json!({
            "mode": "injected",
            "message": message,
            "signature": signature,
            "events": wallet.take_events(),
        }))
    }

    pub async fn injected_sign_order_json(
        &self,
        chain_id: u32,
        order_json: &str,
    ) -> Result<String, JsValue> {
        let wallet = self.injected_wallet()?;
        let chain_id = parse_chain_id(chain_id)?;
        let order = parse_order(order_json)?;
        let signer = wallet.signer();
        let owner = signer.get_address().await.map_err(js_string_error)?;
        let signing = sign_order_async(&order, chain_id, &signer, None)
            .await
            .map_err(js_string_error)?;
        let generated = generate_order_id(chain_id, &order, &owner, None).map_err(js_string_error)?;

        pretty_json(&json!({
            "mode": "injected",
            "owner": owner,
            "signing": signing,
            "orderId": generated.order_id,
            "orderDigest": generated.order_digest,
            "events": wallet.take_events(),
        }))
    }

    pub async fn injected_quote_json(
        &self,
        chain_id: u32,
        env: &str,
        app_code: &str,
        trade_json: &str,
    ) -> Result<String, JsValue> {
        let wallet = self.injected_wallet()?;
        let chain_id = parse_chain_id(chain_id)?;
        let env = parse_env(env)?;
        let trade = parse_trade_parameters(trade_json)?;
        let sdk = live_sdk(chain_id, env, app_code.trim());
        let signer = wallet.signer();
        let quote = sdk
            .get_quote_results_async(trade, &signer, None)
            .await
            .map_err(js_string_error)?;

        pretty_json(&json!({
            "mode": "injected",
            "quote": quote,
            "events": wallet.take_events(),
        }))
    }

    pub async fn injected_submit_order_json(
        &self,
        chain_id: u32,
        env: &str,
        app_code: &str,
        trade_json: &str,
    ) -> Result<String, JsValue> {
        let wallet = self.injected_wallet()?;
        let chain_id = parse_chain_id(chain_id)?;
        let env = parse_env(env)?;
        let trade = parse_trade_parameters(trade_json)?;
        let sdk = live_sdk(chain_id, env, app_code.trim());
        let signer = wallet.signer();
        let posting = sdk
            .post_swap_order_async(trade, &signer, None)
            .await
            .map_err(js_string_error)?;
        *self.last_live_order_uid.lock().unwrap() = Some(posting.order_id.as_str().to_owned());

        pretty_json(&json!({
            "mode": "injected",
            "posting": posting,
            "events": wallet.take_events(),
        }))
    }

    pub async fn injected_cancel_order_json(
        &self,
        chain_id: u32,
        env: &str,
        app_code: &str,
        order_uid: &str,
    ) -> Result<String, JsValue> {
        let wallet = self.injected_wallet()?;
        let chain_id = parse_chain_id(chain_id)?;
        let env = parse_env(env)?;
        let order_uid = parse_order_uid(order_uid)?;
        let sdk = live_sdk(chain_id, env, app_code.trim());
        let signer = wallet.signer();
        let cancelled = sdk
            .off_chain_cancel_order_async(
                &OrderTraderParameters {
                    order_uid,
                    chain_id: Some(chain_id),
                    env: Some(env),
                    settlement_contract_override: None,
                    eth_flow_contract_override: None,
                },
                &signer,
            )
            .await
            .map_err(js_string_error)?;

        pretty_json(&json!({
            "mode": "injected",
            "cancelled": cancelled,
            "events": wallet.take_events(),
        }))
    }

    pub fn last_live_order_uid(&self) -> Option<String> {
        self.last_live_order_uid.lock().unwrap().clone()
    }
}

#[derive(Clone)]
struct MockBrowserOrderbook {
    context: ApiContext,
    state: Arc<Mutex<MockBrowserOrderbookState>>,
}

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct MockBrowserOrderbookState {
    quote_requests: Vec<OrderQuoteRequest>,
    uploads: Vec<(String, String)>,
    sent_orders: Vec<OrderCreation>,
    cancellations: Vec<OrderCancellations>,
}

impl MockBrowserOrderbook {
    fn new(chain_id: SupportedChainId, env: CowEnv) -> Self {
        Self {
            context: ApiContext {
                chain_id,
                env,
                base_urls: None,
                api_key: None,
            },
            state: Arc::new(Mutex::new(MockBrowserOrderbookState::default())),
        }
    }

    fn snapshot(&self) -> MockBrowserOrderbookState {
        self.state.lock().unwrap().clone()
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl OrderbookClient for MockBrowserOrderbook {
    fn context(&self) -> &ApiContext {
        &self.context
    }

    async fn get_quote(
        &self,
        request: &OrderQuoteRequest,
    ) -> Result<OrderQuoteResponse, cow_sdk::OrderbookError> {
        self.state.lock().unwrap().quote_requests.push(request.clone());
        Ok(mock_quote_response(request))
    }

    async fn send_order(
        &self,
        request: &OrderCreation,
    ) -> Result<OrderUid, cow_sdk::OrderbookError> {
        self.state.lock().unwrap().sent_orders.push(request.clone());
        OrderUid::new(
            "0x9f0c29bfbafde4cf5f43f67ff6be7277e5a103ce3be3d05a02f3f42e1a42f0ad44444444444444444444444444444444444444446ff1d400",
        )
        .map_err(|error| cow_sdk::OrderbookError::InvalidTransform(error.to_string()))
    }

    async fn send_signed_order_cancellations(
        &self,
        request: &OrderCancellations,
    ) -> Result<(), cow_sdk::OrderbookError> {
        self.state.lock().unwrap().cancellations.push(request.clone());
        Ok(())
    }

    async fn get_order(
        &self,
        order_uid: &OrderUid,
    ) -> Result<cow_sdk::orderbook::Order, cow_sdk::OrderbookError> {
        Ok(serde_json::from_value(json!({
            "sellToken": wrapped_native_token(self.context.chain_id).address,
            "buyToken": sample_buy_token(),
            "receiver": sample_owner(),
            "sellAmount": "10000000000000000",
            "buyAmount": "2500000000000000000",
            "validTo": 1900000000u32,
            "appData": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "feeAmount": "1000000000000000",
            "kind": "sell",
            "partiallyFillable": false,
            "sellTokenBalance": "erc20",
            "buyTokenBalance": "erc20",
            "signingScheme": "eip712",
            "signature": format!("0x{}1c", "22".repeat(64)),
            "class": "market",
            "owner": sample_owner(),
            "uid": order_uid,
            "executedSellAmount": "0",
            "executedBuyAmount": "0",
            "invalidated": false,
            "status": "open",
            "totalFee": "0"
        }))
        .map_err(|error| cow_sdk::OrderbookError::InvalidTransform(error.to_string()))?)
    }

    async fn upload_app_data(
        &self,
        app_data_hash: &AppDataHash,
        full_app_data: &str,
    ) -> Result<AppDataObject, cow_sdk::OrderbookError> {
        self.state
            .lock()
            .unwrap()
            .uploads
            .push((app_data_hash.as_str().to_owned(), full_app_data.to_owned()));
        Ok(AppDataObject {
            full_app_data: full_app_data.to_owned(),
        })
    }
}

fn live_sdk(chain_id: SupportedChainId, env: CowEnv, app_code: &str) -> TradingSdk {
    TradingSdk::new(
        PartialTraderParameters {
            chain_id: Some(chain_id),
            app_code: Some(app_code.to_owned()),
            owner: None,
            env: Some(env),
            ..Default::default()
        },
        TradingSdkOptions {
            order_book_api: Some(Arc::new(OrderBookApi::new(ApiContext {
                chain_id,
                env,
                base_urls: None,
                api_key: None,
            }))),
        },
    )
}

fn mock_quote_response(request: &OrderQuoteRequest) -> OrderQuoteResponse {
    let (sell_amount, buy_amount, kind) = if request.side.is_sell() {
        (
            request
                .side
                .sell_amount_before_fee
                .clone()
                .unwrap_or_else(|| "10000000000000000".to_owned()),
            "2500000000000000000".to_owned(),
            cow_sdk::OrderKind::Sell,
        )
    } else {
        (
            "10000000000000000".to_owned(),
            request
                .side
                .buy_amount_after_fee
                .clone()
                .unwrap_or_else(|| "2500000000000000000".to_owned()),
            cow_sdk::OrderKind::Buy,
        )
    };

    serde_json::from_value(json!({
        "quote": {
            "sellToken": request.sell_token,
            "buyToken": request.buy_token,
            "receiver": request.receiver.clone().unwrap_or_else(|| request.from.clone()),
            "sellAmount": sell_amount,
            "buyAmount": buy_amount,
            "validTo": request.valid_to.unwrap_or(1900000000u32),
            "appData": request.app_data_hash.clone().unwrap_or_else(|| AppDataHash::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap()),
            "feeAmount": "1000000000000000",
            "kind": kind,
            "partiallyFillable": request.partially_fillable,
            "sellTokenBalance": request.sell_token_balance,
            "buyTokenBalance": request.buy_token_balance
        },
        "from": request.from,
        "expiration": "2030-01-01T00:00:00Z",
        "id": 991001,
        "verified": true,
        "protocolFeeBps": "12.5"
    }))
    .expect("mock quote response must remain valid")
}

fn sample_trade_parameters(chain_id: SupportedChainId) -> TradeParameters {
    TradeParameters {
        kind: cow_sdk::OrderKind::Sell,
        owner: None,
        sell_token: wrapped_native_token(chain_id).address,
        sell_token_decimals: 18,
        buy_token: sample_buy_token(),
        buy_token_decimals: 18,
        amount: "10000000000000000".to_owned(),
        env: Some(CowEnv::Prod),
        settlement_contract_override: None,
        eth_flow_contract_override: None,
        partially_fillable: false,
        slippage_bps: Some(50),
        receiver: None,
        valid_for: Some(1800),
        valid_to: None,
        partner_fee: None,
    }
}

fn sample_unsigned_order(chain_id: SupportedChainId) -> cow_sdk::UnsignedOrder {
    cow_sdk::UnsignedOrder {
        sell_token: wrapped_native_token(chain_id).address,
        buy_token: sample_buy_token(),
        receiver: sample_owner(),
        sell_amount: "10000000000000000".to_owned(),
        buy_amount: "2500000000000000000".to_owned(),
        valid_to: 1_900_000_000,
        app_data: cow_sdk::AppDataHex::new(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .unwrap(),
        fee_amount: "0".to_owned(),
        kind: cow_sdk::OrderKind::Sell,
        partially_fillable: false,
        sell_token_balance: cow_sdk::OrderBalance::Erc20,
        buy_token_balance: cow_sdk::OrderBalance::Erc20,
    }
}

fn sample_approval_parameters(chain_id: SupportedChainId) -> ApprovalParameters {
    ApprovalParameters {
        token_address: wrapped_native_token(chain_id).address,
        amount: "100000000000000000".to_owned(),
        chain_id: Some(chain_id),
        env: Some(CowEnv::Prod),
        vault_relayer_address: None,
    }
}

fn sample_owner() -> Address {
    Address::new("0x4444444444444444444444444444444444444444")
        .expect("static example owner must remain valid")
}

fn sample_buy_token() -> Address {
    Address::new("0x0625aFB445C3B6B7B929342a04A22599fd5dBB59")
        .expect("static example token must remain valid")
}

fn parse_chain_id(chain_id: u32) -> Result<SupportedChainId, JsValue> {
    SupportedChainId::try_from(u64::from(chain_id)).map_err(|error| to_js_error(error.to_string()))
}

fn parse_env(env: &str) -> Result<CowEnv, JsValue> {
    match env.trim().to_ascii_lowercase().as_str() {
        "prod" => Ok(CowEnv::Prod),
        "staging" => Ok(CowEnv::Staging),
        other => Err(to_js_error(format!(
            "unsupported env `{other}`; expected `prod` or `staging`"
        ))),
    }
}

fn parse_order_uid(value: &str) -> Result<OrderUid, JsValue> {
    OrderUid::new(value).map_err(|error| to_js_error(error.to_string()))
}

fn parse_order(order_json: &str) -> Result<cow_sdk::UnsignedOrder, JsValue> {
    parse_json(order_json, "unsignedOrder")
}

fn parse_trade_parameters(trade_json: &str) -> Result<TradeParameters, JsValue> {
    parse_json(trade_json, "tradeParameters")
}

fn parse_json<T>(json_text: &str, label: &str) -> Result<T, JsValue>
where
    T: DeserializeOwned,
{
    serde_json::from_str(json_text)
        .map_err(|error| to_js_error(format!("invalid {label} JSON: {error}")))
}

fn pretty_json<T>(value: &T) -> Result<String, JsValue>
where
    T: Serialize,
{
    serde_json::to_string_pretty(value).map_err(|error| to_js_error(error.to_string()))
}

fn js_string_error(error: impl ToString) -> JsValue {
    to_js_error(error.to_string())
}

fn to_js_error(message: impl Into<String>) -> JsValue {
    JsValue::from_str(&message.into())
}

impl BrowserWalletConsole {
    fn injected_wallet(&self) -> Result<BrowserWallet, JsValue> {
        self.injected_wallet
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| to_js_error("connect an injected wallet first"))
    }
}
