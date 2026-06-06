#![allow(
    clippy::missing_const_for_fn,
    clippy::too_many_lines,
    clippy::type_complexity,
    reason = "table-driven cancellation tests keep shared harness code close to the cases"
)]

mod common;

use core::future::Future;
use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use cow_sdk_core::{
    Address, Amount, ApiContext, AppDataHash, BlockInfo, Cancellable, ContractCall, ContractHandle,
    CowEnv, Hash32, HexData, OrderKind, OrderUid, Provider, Signer, SupportedChainId,
    TransactionBroadcast, TransactionHash, TransactionReceipt, TransactionRequest, TypedDataDomain,
    TypedDataField, TypedDataPayload,
};
use cow_sdk_orderbook::{
    Order, OrderCancellations, OrderCreation, OrderQuoteRequest, OrderQuoteResponse, OrderbookError,
};
use cow_sdk_trading::{
    AllowanceParameters, ApprovalParameters, OrderPostingResult, OrderTraderParameters,
    OrderbookClient, QuoteResults, Trading, TradingError, TradingOptions,
};

use crate::common::{
    COW, MESSAGE_SIGNATURE, MockOrderbook, OWNER, TX_HASH, TYPED_SIGNATURE, address, order_uid,
    regular_order, sample_limit_parameters, sample_trade_parameters, sell_quote_response,
};

// Every public async method on `Trading` composes with
// `cancel_with(&token)`. The table below pins one case per method.

type CaseFuture<'a> = Pin<Box<dyn Future<Output = Result<(), TradingError>> + 'a>>;

struct CancellationCase {
    method_name: &'static str,
    invoke: for<'a> fn(&'a TradingHarness) -> CaseFuture<'a>,
}

const TESTED_METHODS: &[CancellationCase] = &[
    CancellationCase {
        method_name: "quote_results",
        invoke: invoke_get_quote_results,
    },
    CancellationCase {
        method_name: "post_swap_order",
        invoke: invoke_post_swap_order,
    },
    CancellationCase {
        method_name: "post_swap_order_from_quote",
        invoke: invoke_post_swap_order_from_quote,
    },
    CancellationCase {
        method_name: "post_limit_order",
        invoke: invoke_post_limit_order,
    },
    CancellationCase {
        method_name: "pre_sign_transaction",
        invoke: invoke_get_pre_sign_transaction,
    },
    CancellationCase {
        method_name: "order",
        invoke: invoke_get_order,
    },
    CancellationCase {
        method_name: "off_chain_cancel_order",
        invoke: invoke_off_chain_cancel_order,
    },
    CancellationCase {
        method_name: "on_chain_cancel_order",
        invoke: invoke_on_chain_cancel_order,
    },
    CancellationCase {
        method_name: "cow_protocol_allowance",
        invoke: invoke_get_cow_protocol_allowance,
    },
    CancellationCase {
        method_name: "approve_cow_protocol",
        invoke: invoke_approve_cow_protocol,
    },
];

#[tokio::test]
async fn every_remaining_trading_method_returns_cancelled_when_token_is_pre_cancelled() {
    for case in TESTED_METHODS {
        let harness = TradingHarness::new(Duration::from_secs(30)).await;
        let token = cow_sdk_core::CancellationToken::new();
        token.cancel();

        let error = match (case.invoke)(&harness).cancel_with(&token).await {
            Ok(()) => panic!(
                "{} must return an error for the pre-cancelled token branch",
                case.method_name,
            ),
            Err(error) => error,
        };

        assert!(
            matches!(error, TradingError::Cancelled),
            "{} must lift pre-cancelled tokens into TradingError::Cancelled, got {error:?}",
            case.method_name,
        );
    }
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn every_remaining_trading_method_aborts_an_in_flight_operation() {
    for case in TESTED_METHODS {
        let harness = TradingHarness::new(Duration::from_secs(30)).await;
        let token = cow_sdk_core::CancellationToken::new();
        let token_for_call = token.clone();
        let dropped = Arc::new(AtomicBool::new(false));
        let spy = DropSpy(Arc::clone(&dropped));

        let started = Instant::now();
        let call = async {
            let _spy = spy;
            (case.invoke)(&harness).cancel_with(&token_for_call).await
        };
        let trigger = async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            token.cancel();
        };

        let (result, ()) = tokio::join!(call, trigger);
        let elapsed = started.elapsed();

        assert!(
            matches!(result, Err(TradingError::Cancelled)),
            "{} must lift in-flight aborts into TradingError::Cancelled, got {result:?}",
            case.method_name,
        );
        assert!(
            elapsed < Duration::from_secs(5),
            "{} must abort before the delayed operation completes; elapsed = {elapsed:?}",
            case.method_name,
        );
        assert!(
            dropped.load(Ordering::SeqCst),
            "{} must drop the inner trading future when the token fires",
            case.method_name,
        );
    }
}

struct DropSpy(Arc<AtomicBool>);

impl Drop for DropSpy {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

struct TradingHarness {
    trading: Trading,
    quote_results: QuoteResults,
    signer: SlowSigner,
    provider: SlowProvider,
}

impl TradingHarness {
    async fn new(delay: Duration) -> Self {
        let quote_results = quote_results_fixture().await;
        let orderbook = Arc::new(DelayedOrderbook::new(delay));
        let trading = Trading::builder()
            .chain_id(SupportedChainId::Sepolia)
            .app_code("cancellation-composition")
            .env(CowEnv::Prod)
            .options(TradingOptions::new().with_orderbook_client(orderbook))
            .build()
            .expect("trading sdk must construct for cancellation composition tests");

        Self {
            trading,
            quote_results,
            signer: SlowSigner::new(delay),
            provider: SlowProvider::new(delay),
        }
    }
}

async fn quote_results_fixture() -> QuoteResults {
    let orderbook = Arc::new(DelayedOrderbook::new(Duration::ZERO));
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("cancellation-composition")
        .env(CowEnv::Prod)
        .options(TradingOptions::new().with_orderbook_client(orderbook))
        .build()
        .expect("fixture sdk must construct");

    trading
        .quote_only(sample_trade_parameters(OrderKind::Sell), None)
        .await
        .expect("quote fixture must build")
}

#[derive(Clone)]
struct DelayedOrderbook {
    inner: MockOrderbook,
    delay: Duration,
}

impl DelayedOrderbook {
    fn new(delay: Duration) -> Self {
        let inner = MockOrderbook::new(SupportedChainId::Sepolia, sell_quote_response());
        inner.push_order(regular_order());
        Self { inner, delay }
    }

    async fn wait(&self) {
        tokio::time::sleep(self.delay).await;
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl OrderbookClient for DelayedOrderbook {
    fn context(&self) -> &ApiContext {
        OrderbookClient::context(&self.inner)
    }

    fn runtime_binding(&self) -> cow_sdk_trading::OrderbookRuntimeBinding {
        OrderbookClient::runtime_binding(&self.inner)
    }

    async fn quote(
        &self,
        request: &OrderQuoteRequest,
    ) -> Result<OrderQuoteResponse, OrderbookError> {
        self.wait().await;
        OrderbookClient::quote(&self.inner, request).await
    }

    async fn send_order(&self, request: &OrderCreation) -> Result<OrderUid, OrderbookError> {
        self.wait().await;
        OrderbookClient::send_order(&self.inner, request).await
    }

    async fn send_signed_order_cancellations(
        &self,
        request: &OrderCancellations,
    ) -> Result<(), OrderbookError> {
        self.wait().await;
        OrderbookClient::send_signed_order_cancellations(&self.inner, request).await
    }

    async fn order(&self, order_uid: &OrderUid) -> Result<Order, OrderbookError> {
        self.wait().await;
        OrderbookClient::order(&self.inner, order_uid).await
    }

    async fn upload_app_data(
        &self,
        app_data_hash: &AppDataHash,
        full_app_data: &str,
    ) -> Result<(), OrderbookError> {
        self.wait().await;
        OrderbookClient::upload_app_data(&self.inner, app_data_hash, full_app_data).await
    }
}

struct SlowSigner {
    delay: Duration,
}

impl SlowSigner {
    const fn new(delay: Duration) -> Self {
        Self { delay }
    }

    async fn wait(&self) {
        tokio::time::sleep(self.delay).await;
    }
}

impl Signer for SlowSigner {
    type Error = String;

    async fn address(&self) -> Result<Address, Self::Error> {
        self.wait().await;
        Ok(address(OWNER))
    }

    async fn sign_message(&self, _message: &[u8]) -> Result<String, Self::Error> {
        self.wait().await;
        Ok(MESSAGE_SIGNATURE.to_owned())
    }

    async fn sign_transaction(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
        self.wait().await;
        Ok(TX_HASH.to_owned())
    }

    async fn sign_typed_data_payload(
        &self,
        payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        self.sign_typed_data(
            &payload.domain,
            payload.primary_type_fields().unwrap_or_default(),
            payload.message_json(),
        )
        .await
    }

    async fn sign_typed_data(
        &self,
        _domain: &TypedDataDomain,
        _fields: &[TypedDataField],
        _value_json: &str,
    ) -> Result<String, Self::Error> {
        self.wait().await;
        Ok(TYPED_SIGNATURE.to_owned())
    }

    async fn send_transaction(
        &self,
        _tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        self.wait().await;
        Ok(TransactionBroadcast::new(
            Hash32::new(TX_HASH).expect("test transaction hash literal must be valid"),
        ))
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        self.wait().await;
        Ok(Amount::new("125000").expect("test gas literal must be valid"))
    }
}

struct SlowProvider {
    delay: Duration,
}

impl SlowProvider {
    const fn new(delay: Duration) -> Self {
        Self { delay }
    }

    async fn wait(&self) {
        tokio::time::sleep(self.delay).await;
    }
}

impl Provider for SlowProvider {
    type Error = String;

    async fn get_chain_id(&self) -> Result<u64, Self::Error> {
        self.wait().await;
        Ok(u64::from(SupportedChainId::Sepolia))
    }

    async fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
        self.wait().await;
        Ok(None)
    }

    async fn get_transaction_receipt(
        &self,
        _transaction_hash: &TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        self.wait().await;
        Ok(None)
    }

    async fn get_storage_at(
        &self,
        _address: &Address,
        _slot: &str,
    ) -> Result<HexData, Self::Error> {
        self.wait().await;
        Ok(HexData::empty())
    }

    async fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        self.wait().await;
        Ok(HexData::empty())
    }

    async fn read_contract(&self, _request: &ContractCall) -> Result<String, Self::Error> {
        self.wait().await;
        Ok("1000000000000000000".to_owned())
    }

    async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        self.wait().await;
        Ok(BlockInfo::new(0, None))
    }

    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        self.wait().await;
        Ok(ContractHandle::new(*address, abi_json.to_owned()))
    }
}

fn invoke_get_quote_results(harness: &TradingHarness) -> CaseFuture<'_> {
    Box::pin(async move {
        harness
            .trading
            .quote_results(
                sample_trade_parameters(OrderKind::Sell),
                &harness.signer,
                None,
            )
            .await
            .map(|_: QuoteResults| ())
    })
}

fn invoke_post_swap_order(harness: &TradingHarness) -> CaseFuture<'_> {
    Box::pin(async move {
        harness
            .trading
            .post_swap_order(
                sample_trade_parameters(OrderKind::Sell),
                &harness.signer,
                None,
            )
            .await
            .map(|_: OrderPostingResult| ())
    })
}

fn invoke_post_swap_order_from_quote(harness: &TradingHarness) -> CaseFuture<'_> {
    Box::pin(async move {
        harness
            .trading
            .post_swap_order_from_quote(&harness.quote_results, &harness.signer, None)
            .await
            .map(|_: OrderPostingResult| ())
    })
}

fn invoke_post_limit_order(harness: &TradingHarness) -> CaseFuture<'_> {
    Box::pin(async move {
        harness
            .trading
            .post_limit_order(
                sample_limit_parameters(OrderKind::Sell),
                &harness.signer,
                None,
            )
            .await
            .map(|_: OrderPostingResult| ())
    })
}

fn invoke_get_pre_sign_transaction(harness: &TradingHarness) -> CaseFuture<'_> {
    Box::pin(async move {
        harness
            .trading
            .pre_sign_transaction(&order_params(), &harness.signer)
            .await
            .map(|_: TransactionRequest| ())
    })
}

fn invoke_get_order(harness: &TradingHarness) -> CaseFuture<'_> {
    Box::pin(async move {
        harness
            .trading
            .order(&order_params())
            .await
            .map(|_: Order| ())
    })
}

fn invoke_off_chain_cancel_order(harness: &TradingHarness) -> CaseFuture<'_> {
    Box::pin(async move {
        harness
            .trading
            .off_chain_cancel_order(&order_params(), &harness.signer)
            .await
            .map(|_: bool| ())
    })
}

fn invoke_on_chain_cancel_order(harness: &TradingHarness) -> CaseFuture<'_> {
    Box::pin(async move {
        harness
            .trading
            .on_chain_cancel_order(&order_params(), &harness.signer)
            .await
            .map(|_: TransactionHash| ())
    })
}

fn invoke_get_cow_protocol_allowance(harness: &TradingHarness) -> CaseFuture<'_> {
    Box::pin(async move {
        harness
            .trading
            .cow_protocol_allowance(&harness.provider, &allowance_params())
            .await
            .map(|_: Amount| ())
    })
}

fn invoke_approve_cow_protocol(harness: &TradingHarness) -> CaseFuture<'_> {
    Box::pin(async move {
        harness
            .trading
            .approve_cow_protocol(&harness.signer, &approval_params())
            .await
            .map(|_: TransactionHash| ())
    })
}

fn order_params() -> OrderTraderParameters {
    OrderTraderParameters::new(order_uid())
        .with_chain_id(SupportedChainId::Sepolia)
        .with_env(CowEnv::Prod)
}

fn allowance_params() -> AllowanceParameters {
    AllowanceParameters::new(address(COW), address(OWNER))
        .with_chain_id(SupportedChainId::Sepolia)
        .with_env(CowEnv::Prod)
}

fn approval_params() -> ApprovalParameters {
    ApprovalParameters::new(
        address(COW),
        Amount::new("1000").expect("test approval amount literal must be valid"),
    )
    .with_chain_id(SupportedChainId::Sepolia)
    .with_env(CowEnv::Prod)
}

#[test]
fn tested_method_table_documents_expected_surface() {
    let names = TESTED_METHODS
        .iter()
        .map(|case| case.method_name)
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        [
            "quote_results",
            "post_swap_order",
            "post_swap_order_from_quote",
            "post_limit_order",
            "pre_sign_transaction",
            "order",
            "off_chain_cancel_order",
            "on_chain_cancel_order",
            "cow_protocol_allowance",
            "approve_cow_protocol",
        ]
    );
}
