//! Signed order end-to-end journey against in-process mocks.
//!
//! This native example shows the public `cow-sdk-trading` builder API plus
//! the `post_swap_order` entry point that drives the full quote → sign
//! → post journey against an injected orderbook client and an injected
//! signer. It runs without RPC credentials because every external seam
//! (orderbook HTTP, signer, quote fixture) is supplied locally by the example.
//!
//! Run with:
//!
//! ```text
//! cargo run -p cow-sdk-trading --example signed_order_end_to_end
//! ```
//!
//! Expected output:
//!
//! - the posted order UID returned by the mock orderbook
//! - the signing scheme selected by the trading flow
//! - the prefix of the signature returned by the example signer

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    native::run().await
}

#[cfg(target_arch = "wasm32")]
fn main() {
    panic!(
        "signed_order_end_to_end is a native example; use examples/wasm for browser-runtime flows"
    );
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use cow_sdk_core::{
        Address, Amount, ApiContext, BlockInfo, BuyTokenDestination, ContractCall, ContractHandle,
        CowEnv, HexData, OrderKind, Provider, SellTokenSource, Signer, SupportedChainId,
        TransactionBroadcast, TransactionHash, TransactionReceipt, TransactionRequest,
        TypedDataDomain, TypedDataField, ValidationReason,
    };
    use cow_sdk_orderbook::{
        AppDataHash, Order, OrderCancellations, OrderCreation, OrderQuoteRequest,
        OrderQuoteResponse, OrderUid, OrderbookError,
    };
    use cow_sdk_trading::{OrderbookClient, TradeParameters, Trading, TradingError};
    use serde_json::json;

    const OWNER: &str = "0xc8c753ee51e8fc80e199ab297fb575634a1ac1d3";
    const WETH: &str = "0xfff9976782d46cc05630d1f6ebab18b2324d6b14";
    const COW_TOKEN: &str = "0x0625afb445c3b6b7b929342a04a22599fd5dbb59";
    const APP_DATA_HASH_HEX: &str =
        "0xe269b09f45b1d3c98d8e4e841b99a0779fbd3b77943d069b91ddc4fd9789e27e";
    const ORDER_UID_HEX: &str = "0xd64389693b6cf89ad6c140a113b10df08073e5ef3063d05a02f3f42e1a42f0ad0b7795e18767259cc253a2af471dbc4c72b49516ffffffff";
    const TYPED_SIGNATURE: &str = "0x111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111b";

    pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
        let owner = Address::new(OWNER)?;

        let orderbook: Arc<dyn OrderbookClient> = Arc::new(ExampleOrderbook::new(
            SupportedChainId::Sepolia,
            CowEnv::Prod,
            sell_quote_response(),
        ));

        let signer = ExampleSigner::new(owner);

        // Builder path: every builder method shown below is public. Calling
        // `.build()` is only possible after `appCode` and chain authority
        // are set, then validates that the injected orderbook runtime agrees
        // with the trader defaults.
        let trading: Trading = Trading::builder()
            .chain_id(SupportedChainId::Sepolia)
            .app_code("cow-rs-signed-order-example")
            .orderbook_client(orderbook)
            .build()
            .map_err(TradingErrorReport::from)?;

        // Full journey: quote fetch, order signing, and order submission all run
        // inside `post_swap_order` against the injected orderbook and signer.
        let posting = trading
            .post_swap_order(
                sample_trade_parameters(OrderKind::Sell, &owner),
                &signer,
                None,
            )
            .await
            .map_err(TradingErrorReport::from)?;

        println!("order_id={}", posting.order_id.to_hex_string());
        println!("signing_scheme={:?}", posting.signing_scheme);
        println!(
            "signature_prefix={}",
            posting.signature.chars().take(10).collect::<String>()
        );

        Ok(())
    }

    fn sample_trade_parameters(kind: OrderKind, owner: &Address) -> TradeParameters {
        TradeParameters::new(
            kind,
            Address::new(WETH).expect("example WETH literal must be valid"),
            Address::new(COW_TOKEN).expect("example COW token literal must be valid"),
            Amount::new("100000000000000000").expect("example amount literal must be valid"),
        )
        .with_owner(*owner)
        .with_sell_token_balance(SellTokenSource::Erc20)
        .with_buy_token_balance(BuyTokenDestination::Erc20)
        .with_slippage_bps(50)
    }

    fn sell_quote_response() -> OrderQuoteResponse {
        serde_json::from_value(json!({
            "quote": {
                "sellToken": WETH,
                "buyToken": COW_TOKEN,
                "receiver": OWNER,
                "sellAmount": "98646335338956442",
                "buyAmount": "30000000000000000000",
                "validTo": 1_737_464_594_u32,
                "appData": APP_DATA_HASH_HEX,
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
        .expect("example quote fixture must deserialize")
    }

    /// Minimal in-process [`OrderbookClient`] stand-in that returns a fixed
    /// quote, records submitted orders, and hands back a stable order UID so
    /// the example can run end-to-end without a live orderbook.
    struct ExampleOrderbook {
        context: ApiContext,
        quote_response: OrderQuoteResponse,
        state: Mutex<OrderbookState>,
    }

    #[derive(Default)]
    struct OrderbookState {
        quote_requests: Vec<OrderQuoteRequest>,
        sent_orders: Vec<OrderCreation>,
        uploads: Vec<(AppDataHash, String)>,
    }

    impl ExampleOrderbook {
        fn new(
            chain_id: SupportedChainId,
            env: CowEnv,
            quote_response: OrderQuoteResponse,
        ) -> Self {
            Self {
                context: ApiContext::new(chain_id, env),
                quote_response,
                state: Mutex::new(OrderbookState::default()),
            }
        }
    }

    #[async_trait]
    impl OrderbookClient for ExampleOrderbook {
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
            self.state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .sent_orders
                .push(request.clone());
            Ok(OrderUid::new(ORDER_UID_HEX).expect("example order uid literal must be valid"))
        }

        async fn send_signed_order_cancellations(
            &self,
            _request: &OrderCancellations,
        ) -> Result<(), OrderbookError> {
            Ok(())
        }

        async fn get_order(&self, _order_uid: &OrderUid) -> Result<Order, OrderbookError> {
            Err(OrderbookError::InvalidTransform {
                field: "orderUid",
                reason: ValidationReason::Precondition {
                    details: "order lookup is outside this example's mock orderbook surface",
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

    /// Minimal in-process [`Signer`] that returns a fixed address and a fixed
    /// typed-data signature. Real consumers wire in a hardware wallet, keystore,
    /// or alloy-backed signer through the same trait surface.
    #[derive(Clone)]
    struct ExampleSigner {
        address: Address,
    }

    impl ExampleSigner {
        const fn new(address: Address) -> Self {
            Self { address }
        }
    }

    impl Signer for ExampleSigner {
        type Error = ExampleSignerError;

        async fn get_address(&self) -> Result<Address, Self::Error> {
            Ok(self.address)
        }

        async fn sign_message(&self, _message: &[u8]) -> Result<String, Self::Error> {
            Ok(TYPED_SIGNATURE.to_owned())
        }

        async fn sign_transaction(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
            Ok(TYPED_SIGNATURE.to_owned())
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
            _tx: &TransactionRequest,
        ) -> Result<TransactionBroadcast, Self::Error> {
            Err(ExampleSignerError::Unsupported("send_transaction"))
        }

        async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
            Err(ExampleSignerError::Unsupported("estimate_gas"))
        }
    }

    /// Optional [`Provider`] implementation kept for completeness; the trading
    /// flow used by this example does not require it.
    #[allow(
        dead_code,
        reason = "example provider scaffold is kept beside the signer that the trading example actually exercises so readers can see the symmetric Provider shape"
    )]
    struct ExampleProvider;

    impl Provider for ExampleProvider {
        type Error = ExampleSignerError;

        async fn get_chain_id(&self) -> Result<cow_sdk_core::ChainId, Self::Error> {
            Err(ExampleSignerError::Unsupported("get_chain_id"))
        }

        async fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
            Err(ExampleSignerError::Unsupported("get_code"))
        }

        async fn get_transaction_receipt(
            &self,
            _transaction_hash: &TransactionHash,
        ) -> Result<Option<TransactionReceipt>, Self::Error> {
            Err(ExampleSignerError::Unsupported("get_transaction_receipt"))
        }

        async fn get_storage_at(
            &self,
            _address: &Address,
            _slot: &str,
        ) -> Result<HexData, Self::Error> {
            Err(ExampleSignerError::Unsupported("get_storage_at"))
        }

        async fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
            Err(ExampleSignerError::Unsupported("call"))
        }

        async fn read_contract(&self, _request: &ContractCall) -> Result<String, Self::Error> {
            Err(ExampleSignerError::Unsupported("read_contract"))
        }

        async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
            Err(ExampleSignerError::Unsupported("get_block"))
        }

        async fn get_contract(
            &self,
            _address: &Address,
            _abi_json: &str,
        ) -> Result<ContractHandle, Self::Error> {
            Err(ExampleSignerError::Unsupported("get_contract"))
        }
    }

    #[derive(Debug, thiserror::Error)]
    enum ExampleSignerError {
        #[error("example signer does not implement {0}")]
        Unsupported(&'static str),
    }

    impl cow_sdk_core::SignerError for ExampleSignerError {}

    /// Thin wrapper around [`TradingError`] used to plug the trading error type
    /// into `Box<dyn Error>` without adding an explicit `impl From` on the
    /// upstream type.
    struct TradingErrorReport(String);

    impl std::fmt::Display for TradingErrorReport {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&self.0)
        }
    }

    impl std::fmt::Debug for TradingErrorReport {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&self.0)
        }
    }

    impl std::error::Error for TradingErrorReport {}

    impl From<TradingError> for TradingErrorReport {
        fn from(error: TradingError) -> Self {
            Self(error.to_string())
        }
    }
}
