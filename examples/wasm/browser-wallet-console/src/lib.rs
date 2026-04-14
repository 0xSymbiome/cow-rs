use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;
use wasm_bindgen::prelude::*;

use cow_sdk::browser_wallet::{
    BrowserWallet, BrowserWalletError, InjectedWalletDiscovery, InjectedWalletInfo,
    MockEip1193Transport,
};
use cow_sdk::core::{AppDataHash, wrapped_native_token};
use cow_sdk::orderbook::AppDataObject;
use cow_sdk::trading::OrderbookClient;
use cow_sdk::{
    Address, Amount, ApiContext, ApprovalParameters, AsyncSigner, CowEnv, OrderBookApi,
    OrderCancellations, OrderCreation, OrderQuoteRequest, OrderQuoteResponse, OrderTraderParameters,
    OrderUid, PartialTraderParameters, SupportedChainId, TradeParameters, TradingSdk,
    TradingSdkOptions, WalletEvent, WalletSession, approval_transaction, generate_order_id,
    sign_order_async,
};

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct BrowserWalletConsole {
    mock_transport: MockEip1193Transport,
    mock_wallet: BrowserWallet,
    injected_wallet: Mutex<Option<SelectedInjectedWallet>>,
    injected_discovery: Mutex<CachedInjectedWalletDiscovery>,
    confirmed_injected_selection: Mutex<Option<ConfirmedInjectedWalletSelection>>,
    last_live_order_uid: Mutex<Option<String>>,
}

#[derive(Clone)]
struct CachedInjectedWallet {
    wallet: BrowserWallet,
    info: Option<InjectedWalletInfo>,
}

#[derive(Clone, Default)]
struct CachedInjectedWalletDiscovery {
    generation: u64,
    timeout_ms: u32,
    used_window_ethereum_fallback: bool,
    wallets: Vec<CachedInjectedWallet>,
}

#[derive(Clone)]
struct ConfirmedInjectedWalletSelection {
    wallet_info: Option<InjectedWalletInfo>,
    selection_index: usize,
    discovery_generation: u64,
}

#[derive(Clone)]
struct SelectedInjectedWallet {
    wallet: BrowserWallet,
    info: Option<InjectedWalletInfo>,
    selection_index: Option<usize>,
    discovery_generation: Option<u64>,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
enum InjectedConnectSource {
    CachedDetection,
    SelectedWallet,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InjectedWalletDetectionReport {
    available: bool,
    wallets: Vec<InjectedWalletInfo>,
    wallet_count: usize,
    timeout_ms: u32,
    used_window_ethereum_fallback: bool,
    requires_explicit_selection: bool,
    connect_ready: bool,
    selected_wallet_present: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    selected_index: Option<u32>,
    confirmed_selection_present: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    confirmed_selection_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    confirmed_wallet_info: Option<InjectedWalletInfo>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InjectedWalletConnectionReport {
    mode: &'static str,
    session: WalletSession,
    #[serde(skip_serializing_if = "Option::is_none")]
    wallet_info: Option<InjectedWalletInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selection_index: Option<u32>,
    connection_source: InjectedConnectSource,
    events: Vec<WalletEvent>,
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
            injected_discovery: Mutex::new(CachedInjectedWalletDiscovery::default()),
            confirmed_injected_selection: Mutex::new(None),
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
            TradingSdkOptions::new().with_orderbook_client(mock_orderbook.clone()),
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

    pub async fn injected_detection_json(&self) -> Result<String, JsValue> {
        let report = self
            .ensure_injected_detection(false)
            .await
            .map_err(js_string_error)?;
        pretty_json(&report)
    }

    pub async fn injected_rescan_json(&self) -> Result<String, JsValue> {
        let report = self
            .ensure_injected_detection(true)
            .await
            .map_err(js_string_error)?;
        pretty_json(&report)
    }

    pub async fn injected_connect_json(&self) -> Result<String, JsValue> {
        let report = self.connect_injected_wallet(None).await.map_err(js_string_error)?;
        pretty_json(&report)
    }

    pub fn injected_confirm_selection_json(&self, selection_index: u32) -> Result<String, JsValue> {
        let report = self
            .confirm_injected_selection(selection_index as usize)
            .map_err(js_string_error)?;
        pretty_json(&report)
    }

    pub async fn injected_connect_selected_json(
        &self,
        selection_index: u32,
    ) -> Result<String, JsValue> {
        let report = self
            .connect_injected_wallet(Some(selection_index as usize))
            .await
            .map_err(js_string_error)?;
        pretty_json(&report)
    }

    pub fn injected_status_json(&self) -> Result<String, JsValue> {
        let selected = self.injected_wallet()?;
        let confirmed = self.confirmed_injected_selection();
        pretty_json(&json!({
            "mode": "injected",
            "session": selected.wallet.session(),
            "walletInfo": selected.info,
            "selectionIndex": selected.selection_index.map(|index| index as u32),
            "confirmedSelectionPresent": confirmed.is_some(),
            "confirmedSelectionIndex": confirmed
                .as_ref()
                .map(|selection| selection.selection_index as u32),
            "confirmedWalletInfo": confirmed.and_then(|selection| selection.wallet_info),
            "events": selected.wallet.events(),
        }))
    }

    pub fn injected_reset_session_json(&self) -> Result<String, JsValue> {
        let selected = self.injected_wallet()?;
        let confirmed = self.confirmed_injected_selection();
        let session = selected.wallet.reset_session();
        let events = selected.wallet.take_events();
        *self.last_live_order_uid.lock().unwrap() = None;
        pretty_json(&json!({
            "mode": "injected",
            "session": session,
            "walletInfo": selected.info,
            "selectionIndex": selected.selection_index.map(|index| index as u32),
            "walletSelectionRetained": true,
            "confirmedSelectionRetained": confirmed.is_some(),
            "confirmedSelectionIndex": confirmed
                .as_ref()
                .map(|selection| selection.selection_index as u32),
            "confirmedWalletInfo": confirmed.and_then(|selection| selection.wallet_info),
            "note": "console session state cleared; selected wallet and confirmed provider remain available; wallet authorization remains managed by the extension",
            "events": events,
        }))
    }

    pub fn injected_forget_wallet_json(&self) -> Result<String, JsValue> {
        let forgotten_wallet = self.injected_wallet.lock().unwrap().take();
        let forgotten_confirmation = self.confirmed_injected_selection.lock().unwrap().take();
        let forgotten_wallet_info = forgotten_wallet
            .as_ref()
            .and_then(|wallet| wallet.info.clone());
        let forgotten_session = forgotten_wallet.as_ref().map(|wallet| wallet.wallet.session());
        let forgotten_selection_index = forgotten_wallet
            .as_ref()
            .and_then(|wallet| wallet.selection_index.map(|index| index as u32));
        let cleared_order_uid = self.last_live_order_uid.lock().unwrap().take();
        pretty_json(&json!({
            "mode": "injected",
            "walletSelectionCleared": forgotten_wallet.is_some(),
            "forgottenSession": forgotten_session,
            "forgottenWalletInfo": forgotten_wallet_info,
            "forgottenSelectionIndex": forgotten_selection_index,
            "confirmedSelectionCleared": forgotten_confirmation.is_some(),
            "forgottenConfirmedSelectionIndex": forgotten_confirmation
                .as_ref()
                .map(|selection| selection.selection_index as u32),
            "forgottenConfirmedWalletInfo": forgotten_confirmation
                .and_then(|selection| selection.wallet_info),
            "lastLiveOrderUidCleared": cleared_order_uid.is_some(),
            "note": "selected wallet and confirmed provider cleared from the console; wallet authorization remains managed by the extension",
        }))
    }

    pub async fn injected_refresh_json(&self) -> Result<String, JsValue> {
        let selected = self.injected_wallet()?;
        let session = selected
            .wallet
            .refresh_session()
            .await
            .map_err(js_string_error)?;
        pretty_json(&json!({
            "mode": "injected",
            "session": session,
            "walletInfo": selected.info,
            "selectionIndex": selected.selection_index.map(|index| index as u32),
            "events": selected.wallet.take_events(),
        }))
    }

    pub async fn injected_switch_chain_json(&self, chain_id: u32) -> Result<String, JsValue> {
        let selected = self.injected_wallet()?;
        let chain_id = parse_chain_id(chain_id)?;
        let session = selected
            .wallet
            .switch_chain(chain_id)
            .await
            .map_err(js_string_error)?;
        pretty_json(&json!({
            "mode": "injected",
            "session": session,
            "walletInfo": selected.info,
            "selectionIndex": selected.selection_index.map(|index| index as u32),
            "events": selected.wallet.take_events(),
        }))
    }

    pub async fn injected_sign_message_json(&self, message: &str) -> Result<String, JsValue> {
        let selected = self.injected_wallet()?;
        let signer = selected.wallet.signer();
        let signature = signer
            .sign_message(message.as_bytes())
            .await
            .map_err(js_string_error)?;
        pretty_json(&json!({
            "mode": "injected",
            "message": message,
            "signature": signature,
            "walletInfo": selected.info,
            "selectionIndex": selected.selection_index.map(|index| index as u32),
            "events": selected.wallet.take_events(),
        }))
    }

    pub async fn injected_sign_order_json(
        &self,
        chain_id: u32,
        order_json: &str,
    ) -> Result<String, JsValue> {
        let selected = self.injected_wallet()?;
        let chain_id = self.live_chain_id_for_selected_wallet(&selected, chain_id)?;
        let order = parse_order(order_json)?;
        let signer = selected
            .wallet
            .signer_for_chain(chain_id)
            .await
            .map_err(js_string_error)?;
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
            "walletInfo": selected.info,
            "selectionIndex": selected.selection_index.map(|index| index as u32),
            "events": selected.wallet.take_events(),
        }))
    }

    pub async fn injected_quote_json(
        &self,
        chain_id: u32,
        env: &str,
        app_code: &str,
        trade_json: &str,
    ) -> Result<String, JsValue> {
        let selected = self.injected_wallet()?;
        let chain_id = self.live_chain_id_for_selected_wallet(&selected, chain_id)?;
        let env = parse_env(env)?;
        let trade = parse_trade_parameters(trade_json)?;
        let sdk = live_sdk(chain_id, env, app_code.trim());
        let signer = selected
            .wallet
            .signer_for_chain(chain_id)
            .await
            .map_err(js_string_error)?;
        let quote = sdk
            .get_quote_results_async(trade, &signer, None)
            .await
            .map_err(js_string_error)?;

        pretty_json(&json!({
            "mode": "injected",
            "quote": quote,
            "walletInfo": selected.info,
            "selectionIndex": selected.selection_index.map(|index| index as u32),
            "events": selected.wallet.take_events(),
        }))
    }

    pub async fn injected_submit_order_json(
        &self,
        chain_id: u32,
        env: &str,
        app_code: &str,
        trade_json: &str,
    ) -> Result<String, JsValue> {
        let selected = self.injected_wallet()?;
        let chain_id = self.live_chain_id_for_selected_wallet(&selected, chain_id)?;
        let env = parse_env(env)?;
        let trade = parse_trade_parameters(trade_json)?;
        let sdk = live_sdk(chain_id, env, app_code.trim());
        let signer = selected
            .wallet
            .signer_for_chain(chain_id)
            .await
            .map_err(js_string_error)?;
        let posting = sdk
            .post_swap_order_async(trade, &signer, None)
            .await
            .map_err(js_string_error)?;
        *self.last_live_order_uid.lock().unwrap() = Some(posting.order_id.as_str().to_owned());

        pretty_json(&json!({
            "mode": "injected",
            "posting": posting,
            "walletInfo": selected.info,
            "selectionIndex": selected.selection_index.map(|index| index as u32),
            "events": selected.wallet.take_events(),
        }))
    }

    pub async fn injected_cancel_order_json(
        &self,
        chain_id: u32,
        env: &str,
        app_code: &str,
        order_uid: &str,
    ) -> Result<String, JsValue> {
        let selected = self.injected_wallet()?;
        let chain_id = self.live_chain_id_for_selected_wallet(&selected, chain_id)?;
        let env = parse_env(env)?;
        let order_uid = parse_order_uid(order_uid)?;
        let sdk = live_sdk(chain_id, env, app_code.trim());
        let signer = selected
            .wallet
            .signer_for_chain(chain_id)
            .await
            .map_err(js_string_error)?;
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
            "walletInfo": selected.info,
            "selectionIndex": selected.selection_index.map(|index| index as u32),
            "events": selected.wallet.take_events(),
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
        TradingSdkOptions::new().with_orderbook_client(Arc::new(OrderBookApi::new(ApiContext {
                chain_id,
                env,
                base_urls: None,
                api_key: None,
            }))),
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
        amount: Amount::new("10000000000000000").unwrap(),
        env: None,
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
        sell_amount: Amount::new("10000000000000000").unwrap(),
        buy_amount: Amount::new("2500000000000000000").unwrap(),
        valid_to: 1_900_000_000,
        app_data: cow_sdk::AppDataHex::new(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .unwrap(),
        fee_amount: Amount::new("0").unwrap(),
        kind: cow_sdk::OrderKind::Sell,
        partially_fillable: false,
        sell_token_balance: cow_sdk::OrderBalance::Erc20,
        buy_token_balance: cow_sdk::OrderBalance::Erc20,
    }
}

fn sample_approval_parameters(chain_id: SupportedChainId) -> ApprovalParameters {
    ApprovalParameters {
        token_address: wrapped_native_token(chain_id).address,
        amount: Amount::new("100000000000000000").unwrap(),
        chain_id: Some(chain_id),
        env: None,
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
    async fn ensure_injected_detection(
        &self,
        force_rescan: bool,
    ) -> Result<InjectedWalletDetectionReport, String> {
        if !force_rescan {
            let cached = self.injected_discovery.lock().unwrap().clone();
            if !cached.is_empty() {
                let confirmed = self.revalidate_confirmed_injected_selection(&cached);
                return Ok(cached.report(
                    self.selected_injected_wallet().as_ref(),
                    confirmed.as_ref(),
                ));
            }
        }

        let discovery = BrowserWallet::discover().await.map_err(|error| error.to_string())?;
        let next_generation = self.injected_discovery.lock().unwrap().generation + 1;
        let cached =
            CachedInjectedWalletDiscovery::from_discovery(discovery, next_generation)?;
        let confirmed = self.revalidate_confirmed_injected_selection(&cached);
        let report = cached.report(
            self.selected_injected_wallet().as_ref(),
            confirmed.as_ref(),
        );
        *self.injected_discovery.lock().unwrap() = cached;
        Ok(report)
    }

    async fn connect_injected_wallet(
        &self,
        requested_index: Option<usize>,
    ) -> Result<InjectedWalletConnectionReport, String> {
        let cached = self.injected_discovery.lock().unwrap().clone();
        let selected = self.selected_injected_wallet();
        let confirmed = self.confirmed_injected_selection();

        let (mut selected_wallet, connection_source) = match requested_index {
            Some(index) => {
                let confirmed_selection = if cached.is_empty() {
                    None
                } else {
                    let confirmed_selection = cached.confirmed_selection_at(index)?;
                    *self.confirmed_injected_selection.lock().unwrap() =
                        Some(confirmed_selection.clone());
                    Some(confirmed_selection)
                };

                if let Some(current) = selected.clone()
                    && (current.selection_index == Some(index)
                        || confirmed_selection
                            .as_ref()
                            .is_some_and(|selection| selection.matches_selected_wallet(&current)))
                {
                    (current, InjectedConnectSource::SelectedWallet)
                } else if cached.is_empty() {
                    return Err("detect injected wallets before connecting".to_owned());
                } else {
                    (cached.wallet_at(index)?, InjectedConnectSource::CachedDetection)
                }
            }
            None => {
                if let Some(selection) = confirmed {
                    if let Some(current) = selected.clone()
                        && selection.matches_selected_wallet(&current)
                    {
                        (current, InjectedConnectSource::SelectedWallet)
                    } else if cached.is_empty() {
                        return Err("detect injected wallets before connecting".to_owned());
                    } else {
                        (
                            cached.wallet_at(selection.selection_index)?,
                            InjectedConnectSource::CachedDetection,
                        )
                    }
                } else if let Some(current) = selected.clone() {
                    (current, InjectedConnectSource::SelectedWallet)
                } else if cached.is_empty() {
                    return Err("detect injected wallets before connecting".to_owned());
                } else if cached.requires_explicit_selection() {
                    return Err("confirm a detected wallet before connecting".to_owned());
                } else {
                    (
                        cached.single_wallet()?.ok_or_else(|| {
                            "detect injected wallets before connecting".to_owned()
                        })?,
                        InjectedConnectSource::CachedDetection,
                    )
                }
            }
        };

        let session = selected_wallet
            .wallet
            .connect()
            .await
            .map_err(|error| error.to_string())?;
        let events = selected_wallet.wallet.take_events();
        selected_wallet.info = selected_wallet
            .info
            .clone()
            .or_else(|| selected_wallet.wallet.injected_info());

        if !cached.is_empty()
            && let Some(index) = selected_wallet.selection_index
            && let Ok(confirmed_selection) = cached.confirmed_selection_at(index)
        {
            *self.confirmed_injected_selection.lock().unwrap() = Some(confirmed_selection);
        }

        *self.injected_wallet.lock().unwrap() = Some(selected_wallet.clone());

        Ok(InjectedWalletConnectionReport {
            mode: "injected",
            session,
            wallet_info: selected_wallet.info,
            selection_index: selected_wallet.selection_index.map(|index| index as u32),
            connection_source,
            events,
        })
    }

    fn confirm_injected_selection(
        &self,
        selection_index: usize,
    ) -> Result<InjectedWalletDetectionReport, String> {
        let cached = self.injected_discovery.lock().unwrap().clone();
        if cached.is_empty() {
            return Err("detect injected wallets before confirming a wallet".to_owned());
        }

        let confirmed = cached.confirmed_selection_at(selection_index)?;
        *self.confirmed_injected_selection.lock().unwrap() = Some(confirmed.clone());
        Ok(cached.report(
            self.selected_injected_wallet().as_ref(),
            Some(&confirmed),
        ))
    }

    fn revalidate_confirmed_injected_selection(
        &self,
        cached: &CachedInjectedWalletDiscovery,
    ) -> Option<ConfirmedInjectedWalletSelection> {
        let revalidated = self
            .confirmed_injected_selection()
            .and_then(|selection| selection.revalidated(cached))
            .or_else(|| {
                (!cached.requires_explicit_selection() && cached.wallets.len() == 1)
                    .then(|| cached.confirmed_selection_at(0).ok())
                    .flatten()
            });
        *self.confirmed_injected_selection.lock().unwrap() = revalidated.clone();
        revalidated
    }

    fn confirmed_injected_selection(&self) -> Option<ConfirmedInjectedWalletSelection> {
        self.confirmed_injected_selection.lock().unwrap().clone()
    }

    fn selected_injected_wallet(&self) -> Option<SelectedInjectedWallet> {
        self.injected_wallet.lock().unwrap().clone()
    }

    fn injected_wallet(&self) -> Result<SelectedInjectedWallet, JsValue> {
        self.injected_wallet
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| to_js_error("connect an injected wallet first"))
    }

    fn live_chain_id_for_selected_wallet(
        &self,
        selected: &SelectedInjectedWallet,
        chain_id: u32,
    ) -> Result<SupportedChainId, JsValue> {
        let chain_id = parse_chain_id(chain_id)?;
        let session = selected.wallet.session();
        if !session.connected {
            return Err(to_js_error("connect the injected wallet before live quote, signing, submission, or cancellation"));
        }

        let requested_chain_id = u64::from(chain_id);
        let session_chain_id = session.chain_id.ok_or_else(|| {
            to_js_error(
                "connected wallet session does not expose a chain id; use Switch Chain or Refresh before live actions",
            )
        })?;

        if session_chain_id != requested_chain_id {
            return Err(to_js_error(&format!(
                "connected wallet chain {session_chain_id} does not match the selected console chain {requested_chain_id}; use Switch Chain before live quote, signing, submission, or cancellation"
            )));
        }

        Ok(chain_id)
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[doc(hidden)]
impl BrowserWalletConsole {
    pub fn testing_set_injected_wallet(&self, wallet: BrowserWallet) {
        *self.injected_wallet.lock().unwrap() = Some(SelectedInjectedWallet {
            wallet,
            info: None,
            selection_index: None,
            discovery_generation: None,
        });
        *self.confirmed_injected_selection.lock().unwrap() = None;
    }

    pub fn testing_set_last_live_order_uid(&self, order_uid: Option<String>) {
        *self.last_live_order_uid.lock().unwrap() = order_uid;
    }

    pub fn testing_has_injected_wallet(&self) -> bool {
        self.injected_wallet.lock().unwrap().is_some()
    }

    pub fn testing_selected_wallet_index(&self) -> Option<usize> {
        self.injected_wallet
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|wallet| wallet.selection_index)
    }

    pub fn testing_confirmed_wallet_index(&self) -> Option<usize> {
        self.confirmed_injected_selection
            .lock()
            .unwrap()
            .as_ref()
            .map(|selection| selection.selection_index)
    }

    pub fn testing_confirm_injected_selection_json(
        &self,
        selection_index: usize,
    ) -> Result<String, String> {
        let report = self.confirm_injected_selection(selection_index)?;
        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())
    }

    pub async fn testing_injected_connect_json(&self) -> Result<String, String> {
        let report = self.connect_injected_wallet(None).await?;
        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())
    }

    pub fn testing_set_cached_injected_wallets(
        &self,
        wallets: Vec<(BrowserWallet, Option<InjectedWalletInfo>)>,
        timeout_ms: u32,
        used_window_ethereum_fallback: bool,
    ) {
        let generation = self.injected_discovery.lock().unwrap().generation + 1;
        let cached = CachedInjectedWalletDiscovery {
            generation,
            timeout_ms,
            used_window_ethereum_fallback,
            wallets: wallets
                .into_iter()
                .map(|(wallet, info)| CachedInjectedWallet { wallet, info })
                .collect(),
        };
        *self.injected_discovery.lock().unwrap() = cached.clone();
        let _ = self.revalidate_confirmed_injected_selection(&cached);
    }

    pub fn testing_cached_wallet_count(&self) -> usize {
        self.injected_discovery.lock().unwrap().wallets.len()
    }

    pub fn testing_cached_detection_json(&self) -> String {
        let cached = self.injected_discovery.lock().unwrap().clone();
        let confirmed = self.revalidate_confirmed_injected_selection(&cached);
        serde_json::to_string_pretty(&cached.report(
            self.selected_injected_wallet().as_ref(),
            confirmed.as_ref(),
        ))
        .expect("cached detection snapshot must remain serializable")
    }
}

impl ConfirmedInjectedWalletSelection {
    fn matches_selected_wallet(&self, selected: &SelectedInjectedWallet) -> bool {
        if selected.selection_index == Some(self.selection_index)
            && selected.discovery_generation == Some(self.discovery_generation)
        {
            return true;
        }

        injected_wallet_identity_matches(self.wallet_info.as_ref(), selected.info.as_ref())
    }

    fn revalidated(self, cached: &CachedInjectedWalletDiscovery) -> Option<Self> {
        if self.discovery_generation == cached.generation && self.selection_index < cached.wallets.len()
        {
            return Some(Self {
                wallet_info: cached.wallets[self.selection_index].info.clone(),
                selection_index: self.selection_index,
                discovery_generation: cached.generation,
            });
        }

        cached
            .wallets
            .iter()
            .enumerate()
            .find(|(_, wallet)| {
                injected_wallet_identity_matches(self.wallet_info.as_ref(), wallet.info.as_ref())
            })
            .map(|(selection_index, wallet)| Self {
                wallet_info: wallet.info.clone(),
                selection_index,
                discovery_generation: cached.generation,
            })
    }
}

impl CachedInjectedWalletDiscovery {
    fn from_discovery(
        discovery: InjectedWalletDiscovery,
        generation: u64,
    ) -> Result<Self, String> {
        let wallet_infos = discovery.wallets();
        let mut wallets = Vec::with_capacity(wallet_infos.len());
        for (index, info) in wallet_infos.into_iter().enumerate() {
            wallets.push(CachedInjectedWallet {
                wallet: discovery.wallet_at(index).map_err(|error| error.to_string())?,
                info: Some(info),
            });
        }

        Ok(Self {
            generation,
            timeout_ms: discovery.timeout_ms(),
            used_window_ethereum_fallback: discovery.used_legacy_fallback(),
            wallets,
        })
    }

    fn is_empty(&self) -> bool {
        self.wallets.is_empty()
    }

    fn requires_explicit_selection(&self) -> bool {
        self.wallets.len() > 1
    }

    fn report(
        &self,
        selected: Option<&SelectedInjectedWallet>,
        confirmed: Option<&ConfirmedInjectedWalletSelection>,
    ) -> InjectedWalletDetectionReport {
        InjectedWalletDetectionReport {
            available: !self.wallets.is_empty(),
            wallets: self
                .wallets
                .iter()
                .filter_map(|wallet| wallet.info.clone())
                .collect(),
            wallet_count: self.wallets.len(),
            timeout_ms: self.timeout_ms,
            used_window_ethereum_fallback: self.used_window_ethereum_fallback,
            requires_explicit_selection: self.requires_explicit_selection(),
            connect_ready: selected.is_some() || !self.requires_explicit_selection() || confirmed.is_some(),
            selected_wallet_present: selected.is_some(),
            selected_index: selected.and_then(|wallet| wallet.selection_index.map(|index| index as u32)),
            confirmed_selection_present: confirmed.is_some(),
            confirmed_selection_index: confirmed.map(|selection| selection.selection_index as u32),
            confirmed_wallet_info: confirmed.and_then(|selection| selection.wallet_info.clone()),
        }
    }

    fn wallet_at(&self, index: usize) -> Result<SelectedInjectedWallet, String> {
        let wallet = self.wallets.get(index).cloned().ok_or_else(|| {
            BrowserWalletError::DiscoverySelectionOutOfRange {
                index,
                candidates: self.wallets.len(),
            }
            .to_string()
        })?;
        Ok(SelectedInjectedWallet {
            wallet: wallet.wallet,
            info: wallet.info,
            selection_index: Some(index),
            discovery_generation: Some(self.generation),
        })
    }

    fn confirmed_selection_at(
        &self,
        index: usize,
    ) -> Result<ConfirmedInjectedWalletSelection, String> {
        let wallet = self.wallets.get(index).ok_or_else(|| {
            BrowserWalletError::DiscoverySelectionOutOfRange {
                index,
                candidates: self.wallets.len(),
            }
            .to_string()
        })?;

        Ok(ConfirmedInjectedWalletSelection {
            wallet_info: wallet.info.clone(),
            selection_index: index,
            discovery_generation: self.generation,
        })
    }

    fn single_wallet(&self) -> Result<Option<SelectedInjectedWallet>, String> {
        match self.wallets.len() {
            0 => Ok(None),
            1 => self.wallet_at(0).map(Some),
            candidates => Err(
                BrowserWalletError::DiscoverySelectionRequired { candidates }.to_string(),
            ),
        }
    }
}

fn injected_wallet_identity_matches(
    left: Option<&InjectedWalletInfo>,
    right: Option<&InjectedWalletInfo>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => {
            if let (Some(left_uuid), Some(right_uuid)) =
                (left.provider_uuid.as_deref(), right.provider_uuid.as_deref())
            {
                return left_uuid == right_uuid;
            }

            if let (Some(left_rdns), Some(right_rdns)) =
                (left.provider_rdns.as_deref(), right.provider_rdns.as_deref())
            {
                return left_rdns == right_rdns
                    && left.provider_label == right.provider_label
                    && left.discovery_source == right.discovery_source;
            }

            left == right
        }
        _ => false,
    }
}
