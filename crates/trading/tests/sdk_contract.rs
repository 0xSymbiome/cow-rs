mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use alloy_primitives::U256;
use cow_sdk_core::{AddressPerChain, Amount, CowEnv, SupportedChainId};
#[cfg(target_arch = "wasm32")]
use cow_sdk_core::{FetchTransport, FetchTransportConfig};
#[cfg(target_arch = "wasm32")]
use cow_sdk_orderbook::OrderbookApi;
#[cfg(target_arch = "wasm32")]
use cow_sdk_trading::TradingError;
use cow_sdk_trading::{ApprovalParams, OrderTraderParams, TraderParams, Trading, TradingBuilder};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

use crate::common::{
    ALT_RECEIVER, COW, CUSTOM_ETHFLOW, CUSTOM_SETTLEMENT, MockOrderbook, MockProvider, MockSigner,
    OWNER, address, ethflow_order, order_uid, sample_limit_parameters, sample_trade_parameters,
    sell_quote_response,
};

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test_configure!(run_in_browser);

fn calldata_word(data: &str, index: usize) -> String {
    let stripped = data
        .strip_prefix("0x")
        .expect("encoded call data must include 0x prefix");
    let start = 8 + (index * 64);
    stripped[start..start + 64].to_owned()
}

fn uint256_word(value: &U256) -> String {
    // Test oracle helper: emit the canonical 32-byte big-endian ABI
    // word for the cow uint256 value as a 64-character zero-padded
    // lowercase hex string.
    alloy_primitives::hex::encode(value.to_be_bytes::<32>())
}

#[tokio::test]
async fn sdk_quote_only_works_without_signer_and_uses_owner_as_from() {
    let orderbook = Arc::new(MockOrderbook::new(
        SupportedChainId::Sepolia,
        sell_quote_response(),
    ));
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("0x007")
        .env(CowEnv::Prod)
        .orderbook_shared(orderbook.clone())
        .build()
        .expect("sdk construction should succeed");
    let mut trade = sample_trade_parameters(cow_sdk_core::OrderKind::Sell);
    trade.owner = Some(address(OWNER));

    let result = trading
        .quote_only(trade, None)
        .await
        .expect("quote-only should succeed without signer");
    let request = orderbook
        .state()
        .quote_requests
        .last()
        .cloned()
        .expect("quote request must be captured");

    assert_eq!(request.from, address(OWNER));
    assert_eq!(result.quote_response.id, Some(575_401));
}

#[test]
fn sdk_ready_shortcut_accepts_total_trader_parameters() {
    let trading = TradingBuilder::ready(
        TraderParams::new(SupportedChainId::Sepolia, "0x007")
            .expect("app code should validate")
            .with_env(CowEnv::Prod)
            .with_settlement_contract_override(AddressPerChain::from([(
                u64::from(SupportedChainId::Sepolia),
                address(CUSTOM_SETTLEMENT),
            )]))
            .with_eth_flow_contract_override(AddressPerChain::from([(
                u64::from(SupportedChainId::Sepolia),
                address(CUSTOM_ETHFLOW),
            )])),
    );

    assert_eq!(trading.chain_id(), Some(SupportedChainId::Sepolia));
    assert_eq!(
        trading.app_code().map(cow_sdk_core::AppCode::as_str),
        Some("0x007")
    );
    assert_eq!(trading.env(), Some(CowEnv::Prod));
    assert!(trading.settlement_contract_override().is_some());
    assert!(trading.eth_flow_contract_override().is_some());
}

#[tokio::test]
async fn sdk_builder_validates_injected_orderbook_context_and_client_context_can_supply_chain_and_env()
 {
    let orderbook = Arc::new(MockOrderbook::new_with_env(
        SupportedChainId::Sepolia,
        CowEnv::Staging,
        sell_quote_response(),
    ));
    let error = Trading::builder()
        .chain_id(SupportedChainId::Mainnet)
        .app_code("0x007")
        .orderbook_shared(orderbook.clone())
        .build()
        .expect_err("mismatched injected orderbook chain must fail validation");
    assert!(matches!(
        error,
        cow_sdk_trading::TradingError::InjectedOrderbookContextConflict {
            field: "chainId",
            ..
        }
    ));

    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("0x007")
        .orderbook_shared(orderbook)
        .build()
        .expect("builder should accept injected client when defaults do not conflict");
    let mut trade = sample_trade_parameters(cow_sdk_core::OrderKind::Sell);
    trade.env = Some(CowEnv::Staging);

    let result = trading
        .quote_only(trade, None)
        .await
        .expect("injected client context should supply chain and env");

    assert_eq!(result.trade_parameters.env, Some(CowEnv::Staging));
    assert_eq!(
        result.order_typed_data.domain.verifying_contract,
        cow_sdk_contracts::Registry::default()
            .address(
                cow_sdk_contracts::ContractId::Settlement,
                SupportedChainId::Sepolia,
                CowEnv::Staging
            )
            .expect("canonical settlement address is registered for sepolia staging")
    );
}

#[test]
fn sdk_builder_rejects_an_injected_orderbook_whose_chain_conflicts() {
    let orderbook = Arc::new(MockOrderbook::new_with_env(
        SupportedChainId::Sepolia,
        CowEnv::Prod,
        sell_quote_response(),
    ));

    let error = Trading::builder()
        .chain_id(SupportedChainId::Mainnet)
        .app_code("0x007")
        .env(CowEnv::Prod)
        .orderbook_shared(orderbook)
        .build()
        .expect_err("the builder must reject an injected orderbook whose chain conflicts");

    assert!(matches!(
        error,
        cow_sdk_trading::TradingError::InjectedOrderbookContextConflict {
            field: "chainId",
            ..
        }
    ));
}

#[tokio::test]
async fn sdk_orderbook_bound_calls_reject_env_conflicts_with_injected_client_context() {
    let orderbook = Arc::new(MockOrderbook::new_with_env(
        SupportedChainId::Sepolia,
        CowEnv::Prod,
        sell_quote_response(),
    ));
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("0x007")
        .orderbook_shared(orderbook)
        .build()
        .expect("builder should accept compatible config");
    let mut trade = sample_trade_parameters(cow_sdk_core::OrderKind::Sell);
    trade.env = Some(CowEnv::Staging);

    let error = trading
        .quote_only(trade, None)
        .await
        .expect_err("conflicting env must fail before quoting");

    assert!(matches!(
        error,
        cow_sdk_trading::TradingError::InjectedOrderbookContextConflict { field: "env", .. }
    ));
}

#[tokio::test]
async fn sdk_allowance_and_approval_use_call_level_chain_resolution() {
    let provider = MockProvider::default();
    let signer = MockSigner::default();
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .env(CowEnv::Prod)
        .app_code("test-app")
        .build()
        .expect("sdk construction should succeed");

    let allowance = trading
        .cow_protocol_allowance(
            &provider,
            &cow_sdk_trading::AllowanceParams::new(address(COW), address(OWNER))
                .with_chain_id(SupportedChainId::Mainnet)
                .with_env(CowEnv::Prod),
        )
        .await
        .expect("allowance read should succeed");
    assert_eq!(
        allowance,
        Amount::new("1000000000000000000").expect("test allowance literal must be valid")
    );

    let approval_hash = trading
        .approve_cow_protocol(
            &signer,
            &ApprovalParams::new(
                address(COW),
                Amount::new("1000").expect("test approval amount literal must be valid"),
            )
            .with_chain_id(SupportedChainId::Mainnet)
            .with_env(CowEnv::Prod)
            .with_vault_relayer_override(address(ALT_RECEIVER)),
        )
        .await
        .expect("approval should succeed");
    let sent = signer
        .state()
        .sent_transactions
        .last()
        .cloned()
        .expect("approval transaction must be sent");

    assert_eq!(approval_hash.to_hex_string(), crate::common::TX_HASH);
    assert!(
        sent.data
            .as_ref()
            .map(cow_sdk_core::HexData::to_hex_string)
            .unwrap_or_default()
            .to_lowercase()
            .contains(
                address(ALT_RECEIVER)
                    .to_hex_string()
                    .trim_start_matches("0x")
                    .to_lowercase()
                    .as_str()
            )
    );
}

#[tokio::test]
async fn sdk_async_allowance_and_approval_accept_async_runtime_contracts() {
    let provider = MockProvider::default();
    let signer = MockSigner::default();
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .env(CowEnv::Prod)
        .app_code("test-app")
        .build()
        .expect("sdk construction should succeed");

    let allowance = trading
        .cow_protocol_allowance(
            &provider,
            &cow_sdk_trading::AllowanceParams::new(address(COW), address(OWNER))
                .with_chain_id(SupportedChainId::Mainnet)
                .with_env(CowEnv::Prod),
        )
        .await
        .expect("async allowance read should succeed");
    assert_eq!(
        allowance,
        Amount::new("1000000000000000000").expect("test allowance literal must be valid")
    );

    let approval_hash = trading
        .approve_cow_protocol(
            &signer,
            &ApprovalParams::new(
                address(COW),
                Amount::new("1000").expect("test approval amount literal must be valid"),
            )
            .with_chain_id(SupportedChainId::Mainnet)
            .with_env(CowEnv::Prod)
            .with_vault_relayer_override(address(ALT_RECEIVER)),
        )
        .await
        .expect("async approval should succeed");
    assert_eq!(approval_hash.to_hex_string(), crate::common::TX_HASH);
}

#[tokio::test]
async fn sdk_call_level_overrides_beat_trader_level_overrides_for_settlement_and_ethflow() {
    let orderbook = Arc::new(MockOrderbook::new_with_env(
        SupportedChainId::Sepolia,
        CowEnv::Staging,
        sell_quote_response(),
    ));
    orderbook.push_order(ethflow_order());
    let signer = MockSigner::default();
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .env(CowEnv::Staging)
        .settlement_contract_override(AddressPerChain::from([(
            u64::from(SupportedChainId::Sepolia),
            address("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        )]))
        .eth_flow_contract_override(AddressPerChain::from([(
            u64::from(SupportedChainId::Sepolia),
            address("0xcccccccccccccccccccccccccccccccccccccccc"),
        )]))
        .orderbook_shared(orderbook.clone())
        .app_code("test-app")
        .build()
        .expect("sdk construction should succeed");

    let pre_sign_tx = trading
        .pre_sign_transaction(
            &OrderTraderParams::new(order_uid())
                .with_chain_id(SupportedChainId::Sepolia)
                .with_env(CowEnv::Staging)
                .with_settlement_contract_override(AddressPerChain::from([(
                    u64::from(SupportedChainId::Sepolia),
                    address(CUSTOM_SETTLEMENT),
                )])),
            &signer,
        )
        .await
        .expect("pre-sign transaction should succeed");
    assert_eq!(pre_sign_tx.to, address(CUSTOM_SETTLEMENT));

    let tx_hash = trading
        .onchain_cancel_order(
            &OrderTraderParams::new(order_uid())
                .with_chain_id(SupportedChainId::Sepolia)
                .with_env(CowEnv::Staging)
                .with_eth_flow_contract_override(AddressPerChain::from([(
                    u64::from(SupportedChainId::Sepolia),
                    address(CUSTOM_ETHFLOW),
                )])),
            &signer,
        )
        .await
        .expect("on-chain cancellation should succeed");
    let sent = signer
        .state()
        .sent_transactions
        .last()
        .cloned()
        .expect("cancellation transaction must be sent");

    assert_eq!(tx_hash.to_hex_string(), crate::common::TX_HASH);
    assert_eq!(sent.to, Some(address(CUSTOM_ETHFLOW)));
}

#[tokio::test]
async fn sdk_post_limit_order_presign_posts_without_a_signer_and_requires_an_explicit_owner() {
    let orderbook = Arc::new(MockOrderbook::new(
        SupportedChainId::Sepolia,
        sell_quote_response(),
    ));
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("test-app")
        .env(CowEnv::Prod)
        .orderbook_shared(orderbook.clone())
        .build()
        .expect("sdk construction should succeed");

    // `sample_limit_parameters` carries an explicit owner; no signer exists
    // anywhere in this test, so any signer consultation fails the placement.
    let result = trading
        .post_limit_order_presign(sample_limit_parameters(cow_sdk_core::OrderKind::Sell), None)
        .await
        .expect("pre-sign placement with an explicit owner should post");

    let state = orderbook.state();
    let sent = state
        .sent_orders
        .last()
        .expect("pre-sign placement must reach the orderbook");
    assert_eq!(
        result.signing_scheme,
        cow_sdk_orderbook::SigningScheme::PreSign
    );
    assert_eq!(
        sent.signing_scheme,
        cow_sdk_orderbook::SigningScheme::PreSign
    );
    assert_eq!(sent.from, address(OWNER));
    assert_eq!(
        sent.signature,
        address(OWNER).to_hex_string(),
        "the pre-sign wire signature carries the owner address",
    );
    assert_eq!(
        state.uploads.len(),
        1,
        "pre-sign placements upload app-data like any other posting",
    );

    let mut ownerless = sample_limit_parameters(cow_sdk_core::OrderKind::Sell);
    ownerless.owner = None;
    let error = trading
        .post_limit_order_presign(ownerless, None)
        .await
        .expect_err("pre-sign placement without an explicit owner must fail");
    assert!(matches!(
        error,
        cow_sdk_trading::TradingError::MissingSubmissionOwner
    ));
}

#[tokio::test]
async fn sdk_onchain_cancel_order_routes_regular_orders_through_settlement_when_not_ethflow() {
    let orderbook = Arc::new(MockOrderbook::new(
        SupportedChainId::Sepolia,
        sell_quote_response(),
    ));
    orderbook.push_order(crate::common::regular_order());
    let signer = MockSigner::default();
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("0x007")
        .env(CowEnv::Prod)
        .settlement_contract_override(AddressPerChain::from([(
            u64::from(SupportedChainId::Sepolia),
            address(CUSTOM_SETTLEMENT),
        )]))
        .eth_flow_contract_override(AddressPerChain::from([(
            u64::from(SupportedChainId::Sepolia),
            address(CUSTOM_ETHFLOW),
        )]))
        .orderbook_shared(orderbook)
        .build()
        .expect("sdk construction should succeed");

    trading
        .onchain_cancel_order(
            &OrderTraderParams::new(order_uid())
                .with_chain_id(SupportedChainId::Sepolia)
                .with_env(CowEnv::Prod),
            &signer,
        )
        .await
        .expect("regular cancellation should succeed");

    let sent = signer
        .state()
        .sent_transactions
        .last()
        .cloned()
        .expect("regular cancellation transaction must be sent");
    assert_eq!(sent.to, Some(address(CUSTOM_SETTLEMENT)));
}

#[tokio::test]
async fn sdk_onchain_cancel_order_preserves_full_uint256_range_for_ethflow_orders() {
    let orderbook = Arc::new(MockOrderbook::new(
        SupportedChainId::Sepolia,
        sell_quote_response(),
    ));
    let high_sell: U256 = U256::from(1u8) << 255usize;
    let high_buy = high_sell + U256::from(1u8);
    let mut order = ethflow_order();
    order.sell_amount = Amount::from_u256(high_sell);
    order.buy_amount = Amount::from_u256(high_buy);
    orderbook.push_order(order);

    let signer = MockSigner::default();
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("0x007")
        .env(CowEnv::Prod)
        .settlement_contract_override(AddressPerChain::from([(
            u64::from(SupportedChainId::Sepolia),
            address(CUSTOM_SETTLEMENT),
        )]))
        .eth_flow_contract_override(AddressPerChain::from([(
            u64::from(SupportedChainId::Sepolia),
            address(CUSTOM_ETHFLOW),
        )]))
        .orderbook_shared(orderbook)
        .build()
        .expect("sdk construction should succeed");

    trading
        .onchain_cancel_order(
            &OrderTraderParams::new(order_uid())
                .with_chain_id(SupportedChainId::Sepolia)
                .with_env(CowEnv::Prod),
            &signer,
        )
        .await
        .expect("ethflow cancellation should encode large uint256 values");

    let sent = signer
        .state()
        .sent_transactions
        .last()
        .cloned()
        .expect("ethflow cancellation transaction must be sent");
    let data = sent
        .data
        .as_ref()
        .expect("ethflow cancellation transaction must include call data");

    assert_eq!(sent.to, Some(address(CUSTOM_ETHFLOW)));
    assert_eq!(
        calldata_word(&data.to_hex_string(), 2),
        uint256_word(&high_sell)
    );
    assert_eq!(
        calldata_word(&data.to_hex_string(), 3),
        uint256_word(&high_buy)
    );
    // The canonical upstream EthFlowOrder.Data tuple places the `appData`
    // bytes32 at word index 4; the fixture order pins that hash, so confirm
    // the ABI layout carries it through the cancellation call-data unmodified.
    let app_data_without_prefix = crate::common::APP_DATA_HASH.trim_start_matches("0x");
    assert_eq!(
        calldata_word(&data.to_hex_string(), 4),
        app_data_without_prefix
    );
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn build_succeeds_on_wasm32_without_injected_orderbook_client() {
    let trading = TradingBuilder::new()
        .chain_id(SupportedChainId::Mainnet)
        .app_code("test-app")
        .build()
        .expect("wasm32 build must succeed without an injected orderbook client");

    assert_eq!(trading.chain_id(), Some(SupportedChainId::Mainnet));
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn build_succeeds_on_wasm32_with_injected_orderbook_client() {
    let transport = FetchTransport::new(&FetchTransportConfig::new("https://api.cow.fi"));
    let orderbook = OrderbookApi::builder()
        .chain(SupportedChainId::Mainnet)
        .env(CowEnv::Prod)
        .transport(Arc::new(transport))
        .build()
        .expect("wasm32 injected orderbook client must build with explicit transport");

    let trading = TradingBuilder::new()
        .chain_id(SupportedChainId::Mainnet)
        .app_code("test-app")
        .orderbook_shared(Arc::new(orderbook))
        .build()
        .expect("wasm32 build must accept an injected orderbook client");
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn build_succeeds_on_native_without_injected_orderbook_client() {
    let trading = TradingBuilder::new()
        .chain_id(SupportedChainId::Mainnet)
        .app_code("test-app")
        .build()
        .expect("native build must succeed when the typestate prerequisites are set");

    assert_eq!(trading.chain_id(), Some(SupportedChainId::Mainnet));
    assert_eq!(
        trading.app_code().map(cow_sdk_core::AppCode::as_str),
        Some("test-app")
    );
}

#[tokio::test]
async fn get_quote_only_returns_cancelled_when_combinator_token_fires_before_call() {
    use cow_sdk_core::Cancellable;

    let orderbook = Arc::new(MockOrderbook::new(
        SupportedChainId::Sepolia,
        sell_quote_response(),
    ));
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("cancellation-test")
        .env(CowEnv::Prod)
        .orderbook_shared(orderbook)
        .build()
        .expect("trading sdk must construct for the cancellation test");

    let trade = sample_trade_parameters(cow_sdk_core::OrderKind::Sell);
    let token = cow_sdk_core::CancellationToken::new();
    token.cancel();

    let error = trading
        .quote_only(trade, None)
        .cancel_with(&token)
        .await
        .expect_err("pre-cancelled token must produce a Cancelled error");
    assert!(matches!(error, cow_sdk_trading::TradingError::Cancelled));
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn get_quote_only_combinator_aborts_an_in_flight_quote() {
    use cow_sdk_core::Cancellable;

    struct DropSpy(Arc<AtomicBool>);

    impl Drop for DropSpy {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    let orderbook = Arc::new(
        MockOrderbook::new(SupportedChainId::Sepolia, sell_quote_response())
            .with_quote_delay(std::time::Duration::from_secs(30)),
    );
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("cancellation-test")
        .env(CowEnv::Prod)
        .orderbook_shared(orderbook)
        .build()
        .expect("trading sdk must construct for the cancellation test");

    let trade = sample_trade_parameters(cow_sdk_core::OrderKind::Sell);
    let token = cow_sdk_core::CancellationToken::new();
    let token_for_call = token.clone();
    let dropped = Arc::new(AtomicBool::new(false));
    let spy = DropSpy(Arc::clone(&dropped));

    let trigger_cancellation = async {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        token.cancel();
    };

    let quote_call = async {
        let _spy = spy;
        trading
            .quote_only(trade, None)
            .cancel_with(&token_for_call)
            .await
    };

    let started = std::time::Instant::now();
    let (result, ()) = tokio::join!(quote_call, trigger_cancellation);
    let elapsed = started.elapsed();

    assert!(matches!(
        result,
        Err(cow_sdk_trading::TradingError::Cancelled)
    ));
    assert!(
        elapsed < std::time::Duration::from_secs(5),
        "cancellation must drop the in-flight future within the quote deadline; elapsed = {elapsed:?}"
    );
    assert!(
        dropped.load(Ordering::SeqCst),
        "the inner quote future must be dropped when the cancellation token fires"
    );
}
