mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use cow_sdk_core::{AddressPerChain, Amount, CowEnv, SupportedChainId};
use cow_sdk_trading::{
    ApprovalParameters, OrderTraderParameters, PartialTraderParameters, TradingSdk,
    TradingSdkOptions,
};
use num_bigint::BigUint;

use crate::common::{
    ALT_RECEIVER, COW, CUSTOM_ETHFLOW, CUSTOM_SETTLEMENT, MockOrderbook, MockProvider, MockSigner,
    OWNER, address, ethflow_order, order_uid, sample_trade_parameters, sell_quote_response,
};

fn calldata_word(data: &str, index: usize) -> String {
    let stripped = data
        .strip_prefix("0x")
        .expect("encoded call data must include 0x prefix");
    let start = 8 + (index * 64);
    stripped[start..start + 64].to_owned()
}

fn uint256_word(value: &BigUint) -> String {
    format!("{value:064x}")
}

#[tokio::test]
async fn sdk_quote_only_works_without_signer_and_uses_owner_as_from() {
    let orderbook = Arc::new(MockOrderbook::new(
        SupportedChainId::Sepolia,
        sell_quote_response(),
    ));
    let sdk = TradingSdk::new(
        PartialTraderParameters::new()
            .with_chain_id(SupportedChainId::Sepolia)
            .with_app_code("0x007".to_owned())
            .with_env(CowEnv::Prod),
        TradingSdkOptions::new().with_orderbook_client(orderbook.clone()),
    )
    .expect("sdk construction should succeed");
    let mut trade = sample_trade_parameters(cow_sdk_core::OrderKind::Sell);
    trade.owner = Some(address(OWNER));

    let result = sdk
        .get_quote_only(trade, None)
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
fn sdk_ready_construction_requires_chain_authority_and_app_code() {
    let missing_chain = TradingSdk::builder()
        .with_app_code("0x007")
        .build()
        .expect_err("ready builder must reject missing chain authority");
    assert!(matches!(
        missing_chain,
        cow_sdk_trading::TradingError::MissingTraderParameters(ref fields) if fields == "chainId"
    ));

    let missing_app = TradingSdk::builder()
        .with_chain_id(SupportedChainId::Sepolia)
        .build()
        .expect_err("ready builder must reject missing appCode");
    assert!(matches!(
        missing_app,
        cow_sdk_trading::TradingError::MissingTraderParameters(ref fields) if fields == "appCode"
    ));

    let missing_ready_defaults = TradingSdk::new(
        PartialTraderParameters::new().with_chain_id(SupportedChainId::Sepolia),
        TradingSdkOptions::default(),
    )
    .expect_err("ready constructor must reject missing appCode");
    assert!(matches!(
        missing_ready_defaults,
        cow_sdk_trading::TradingError::MissingTraderParameters(ref fields) if fields == "appCode"
    ));
}

#[tokio::test]
async fn sdk_builder_validates_injected_orderbook_context_and_client_context_can_supply_chain_and_env()
 {
    let orderbook = Arc::new(MockOrderbook::new_with_env(
        SupportedChainId::Sepolia,
        CowEnv::Staging,
        sell_quote_response(),
    ));
    let error = TradingSdk::builder()
        .with_chain_id(SupportedChainId::Mainnet)
        .with_app_code("0x007")
        .with_orderbook_client(orderbook.clone())
        .build()
        .expect_err("mismatched injected orderbook chain must fail validation");
    assert!(matches!(
        error,
        cow_sdk_trading::TradingError::InjectedOrderbookContextConflict {
            field: "chainId",
            ..
        }
    ));

    let sdk = TradingSdk::builder()
        .with_app_code("0x007")
        .with_orderbook_client(orderbook)
        .build()
        .expect("builder should accept injected client when defaults do not conflict");
    let mut trade = sample_trade_parameters(cow_sdk_core::OrderKind::Sell);
    trade.env = Some(CowEnv::Staging);

    let result = sdk
        .get_quote_only(trade, None)
        .await
        .expect("injected client context should supply chain and env");

    assert_eq!(result.trade_parameters.env, Some(CowEnv::Staging));
    assert_eq!(
        result.order_typed_data.domain.verifying_contract,
        cow_sdk_core::settlement_contract_address(SupportedChainId::Sepolia, CowEnv::Staging)
    );
}

#[test]
fn sdk_new_validates_injected_orderbook_context_with_the_same_contract_as_the_builder() {
    let orderbook = Arc::new(MockOrderbook::new_with_env(
        SupportedChainId::Sepolia,
        CowEnv::Prod,
        sell_quote_response(),
    ));

    let error = TradingSdk::new(
        PartialTraderParameters::new()
            .with_chain_id(SupportedChainId::Mainnet)
            .with_app_code("0x007".to_owned())
            .with_env(CowEnv::Prod),
        TradingSdkOptions::new().with_orderbook_client(orderbook),
    )
    .expect_err("direct constructor must reject injected orderbook conflicts");

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
    let sdk = TradingSdk::builder()
        .with_app_code("0x007")
        .with_orderbook_client(orderbook)
        .build()
        .expect("builder should accept compatible config");
    let mut trade = sample_trade_parameters(cow_sdk_core::OrderKind::Sell);
    trade.env = Some(CowEnv::Staging);

    let error = sdk
        .get_quote_only(trade, None)
        .await
        .expect_err("conflicting env must fail before quoting");

    assert!(matches!(
        error,
        cow_sdk_trading::TradingError::InjectedOrderbookContextConflict { field: "env", .. }
    ));
}

#[tokio::test]
async fn sdk_partial_construction_still_reports_missing_chainid_and_appcode_for_quote_only() {
    let trade = sample_trade_parameters(cow_sdk_core::OrderKind::Sell);

    let missing_chain = TradingSdk::new_partial(
        PartialTraderParameters::new().with_app_code("0x007".to_owned()),
        TradingSdkOptions::default(),
    )
    .expect("partial sdk construction without injected orderbook should succeed");
    let chain_error = missing_chain
        .get_quote_only(trade.clone(), None)
        .await
        .expect_err("missing chainId must fail")
        .to_string();
    assert!(chain_error.contains("Missing quoter parameters: chainId"));

    let missing_app = TradingSdk::new_partial(
        PartialTraderParameters::new().with_chain_id(SupportedChainId::Sepolia),
        TradingSdkOptions::default(),
    )
    .expect("partial sdk construction without injected orderbook should succeed");
    let app_error = missing_app
        .get_quote_only(trade, None)
        .await
        .expect_err("missing appCode must fail")
        .to_string();
    assert!(app_error.contains("Missing quoter parameters: appCode"));
}

#[test]
fn sdk_allowance_and_approval_use_call_level_chain_resolution() {
    let provider = MockProvider::default();
    let signer = MockSigner::default();
    let sdk = TradingSdk::new_partial(
        PartialTraderParameters::new()
            .with_chain_id(SupportedChainId::Sepolia)
            .with_env(CowEnv::Prod),
        TradingSdkOptions::default(),
    )
    .expect("partial sdk construction should succeed");

    let allowance = sdk
        .get_cow_protocol_allowance(
            &provider,
            &cow_sdk_trading::AllowanceParameters::new(address(COW), address(OWNER))
                .with_chain_id(SupportedChainId::Mainnet)
                .with_env(CowEnv::Prod),
        )
        .expect("allowance read should succeed");
    assert_eq!(
        allowance,
        Amount::new("1000000000000000000").expect("test allowance literal must be valid")
    );

    let approval_hash = sdk
        .approve_cow_protocol(
            &signer,
            &ApprovalParameters::new(
                address(COW),
                Amount::new("1000").expect("test approval amount literal must be valid"),
            )
            .with_chain_id(SupportedChainId::Mainnet)
            .with_env(CowEnv::Prod)
            .with_vault_relayer_address(address(ALT_RECEIVER)),
        )
        .expect("approval should succeed");
    let sent = signer
        .state()
        .sent_transactions
        .last()
        .cloned()
        .expect("approval transaction must be sent");

    assert_eq!(approval_hash.as_str(), crate::common::TX_HASH);
    assert!(
        sent.data
            .as_ref()
            .map(cow_sdk_core::HexData::as_str)
            .unwrap_or_default()
            .to_lowercase()
            .contains(
                address(ALT_RECEIVER)
                    .as_str()
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
    let sdk = TradingSdk::new_partial(
        PartialTraderParameters::new()
            .with_chain_id(SupportedChainId::Sepolia)
            .with_env(CowEnv::Prod),
        TradingSdkOptions::default(),
    )
    .expect("partial sdk construction should succeed");

    let allowance = sdk
        .get_cow_protocol_allowance_async(
            &provider,
            &cow_sdk_trading::AllowanceParameters::new(address(COW), address(OWNER))
                .with_chain_id(SupportedChainId::Mainnet)
                .with_env(CowEnv::Prod),
        )
        .await
        .expect("async allowance read should succeed");
    assert_eq!(
        allowance,
        Amount::new("1000000000000000000").expect("test allowance literal must be valid")
    );

    let approval_hash = sdk
        .approve_cow_protocol_async(
            &signer,
            &ApprovalParameters::new(
                address(COW),
                Amount::new("1000").expect("test approval amount literal must be valid"),
            )
            .with_chain_id(SupportedChainId::Mainnet)
            .with_env(CowEnv::Prod)
            .with_vault_relayer_address(address(ALT_RECEIVER)),
        )
        .await
        .expect("async approval should succeed");
    assert_eq!(approval_hash.as_str(), crate::common::TX_HASH);
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
    let sdk = TradingSdk::new_partial(
        PartialTraderParameters::new()
            .with_chain_id(SupportedChainId::Sepolia)
            .with_env(CowEnv::Staging)
            .with_settlement_contract_override(AddressPerChain::from([(
                u64::from(SupportedChainId::Sepolia),
                address("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
            )]))
            .with_eth_flow_contract_override(AddressPerChain::from([(
                u64::from(SupportedChainId::Sepolia),
                address("0xcccccccccccccccccccccccccccccccccccccccc"),
            )])),
        TradingSdkOptions::new().with_orderbook_client(orderbook.clone()),
    )
    .expect("partial sdk construction should succeed");

    let pre_sign_tx = sdk
        .get_pre_sign_transaction(
            &OrderTraderParameters::new(order_uid())
                .with_chain_id(SupportedChainId::Sepolia)
                .with_env(CowEnv::Staging)
                .with_settlement_contract_override(AddressPerChain::from([(
                    u64::from(SupportedChainId::Sepolia),
                    address(CUSTOM_SETTLEMENT),
                )])),
            &signer,
        )
        .expect("pre-sign transaction should succeed");
    assert_eq!(pre_sign_tx.to, Some(address(CUSTOM_SETTLEMENT)));

    let tx_hash = sdk
        .on_chain_cancel_order(
            &OrderTraderParameters::new(order_uid())
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

    assert_eq!(tx_hash.as_str(), crate::common::TX_HASH);
    assert_eq!(sent.to, Some(address(CUSTOM_ETHFLOW)));
}

#[tokio::test]
async fn sdk_onchain_cancel_order_routes_regular_orders_through_settlement_when_not_ethflow() {
    let orderbook = Arc::new(MockOrderbook::new(
        SupportedChainId::Sepolia,
        sell_quote_response(),
    ));
    orderbook.push_order(crate::common::regular_order());
    let signer = MockSigner::default();
    let sdk = TradingSdk::new(
        PartialTraderParameters::new()
            .with_chain_id(SupportedChainId::Sepolia)
            .with_app_code("0x007".to_owned())
            .with_env(CowEnv::Prod)
            .with_settlement_contract_override(AddressPerChain::from([(
                u64::from(SupportedChainId::Sepolia),
                address(CUSTOM_SETTLEMENT),
            )]))
            .with_eth_flow_contract_override(AddressPerChain::from([(
                u64::from(SupportedChainId::Sepolia),
                address(CUSTOM_ETHFLOW),
            )])),
        TradingSdkOptions::new().with_orderbook_client(orderbook),
    )
    .expect("sdk construction should succeed");

    sdk.on_chain_cancel_order(
        &OrderTraderParameters::new(order_uid())
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
    let high_sell: BigUint = BigUint::from(1u8) << 255u32;
    let high_buy = &high_sell + BigUint::from(1u8);
    let mut order = ethflow_order();
    order.sell_amount = high_sell.to_str_radix(10);
    order.buy_amount = high_buy.to_str_radix(10);
    orderbook.push_order(order);

    let signer = MockSigner::default();
    let sdk = TradingSdk::new(
        PartialTraderParameters::new()
            .with_chain_id(SupportedChainId::Sepolia)
            .with_app_code("0x007".to_owned())
            .with_env(CowEnv::Prod)
            .with_settlement_contract_override(AddressPerChain::from([(
                u64::from(SupportedChainId::Sepolia),
                address(CUSTOM_SETTLEMENT),
            )]))
            .with_eth_flow_contract_override(AddressPerChain::from([(
                u64::from(SupportedChainId::Sepolia),
                address(CUSTOM_ETHFLOW),
            )])),
        TradingSdkOptions::new().with_orderbook_client(orderbook),
    )
    .expect("sdk construction should succeed");

    sdk.on_chain_cancel_order(
        &OrderTraderParameters::new(order_uid())
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
    assert_eq!(calldata_word(data.as_str(), 2), uint256_word(&high_sell));
    assert_eq!(calldata_word(data.as_str(), 3), uint256_word(&high_buy));
    // The canonical upstream EthFlowOrder.Data tuple places the `appData`
    // bytes32 at word index 4; the fixture order pins that hash, so confirm
    // the ABI layout carries it through the cancellation call-data unmodified.
    let app_data_without_prefix = crate::common::APP_DATA_HASH.trim_start_matches("0x");
    assert_eq!(calldata_word(data.as_str(), 4), app_data_without_prefix);
}

#[test]
fn typestate_build_ready_produces_a_ready_mode_sdk_without_runtime_default_checks() {
    let sdk = cow_sdk_trading::TradingSdkBuilder::new()
        .with_chain_id(SupportedChainId::Sepolia)
        .with_app_code("typestate-ready")
        .build_ready()
        .expect("typestate build_ready must succeed when the prerequisites are set");

    assert_eq!(sdk.mode(), cow_sdk_trading::TradingSdkMode::Ready);
    assert_eq!(
        sdk.trader_defaults().chain_id,
        Some(SupportedChainId::Sepolia)
    );
    assert_eq!(
        sdk.trader_defaults().app_code.as_deref(),
        Some("typestate-ready")
    );
}

#[test]
fn typestate_build_helper_only_produces_a_helper_mode_sdk_from_a_chain_only_state() {
    let sdk = cow_sdk_trading::TradingSdkBuilder::new()
        .with_chain_id(SupportedChainId::Sepolia)
        .build_helper_only()
        .expect("typestate build_helper_only must succeed when chain id is set");

    assert_eq!(sdk.mode(), cow_sdk_trading::TradingSdkMode::HelperOnly);
    assert_eq!(
        sdk.trader_defaults().chain_id,
        Some(SupportedChainId::Sepolia)
    );
    assert!(sdk.trader_defaults().app_code.is_none());
}

#[tokio::test]
async fn get_quote_only_returns_cancelled_when_combinator_token_fires_before_call() {
    use cow_sdk_core::Cancellable;

    let orderbook = Arc::new(MockOrderbook::new(
        SupportedChainId::Sepolia,
        sell_quote_response(),
    ));
    let sdk = TradingSdk::new(
        PartialTraderParameters::new()
            .with_chain_id(SupportedChainId::Sepolia)
            .with_app_code("cancellation-test".to_owned())
            .with_owner(address(OWNER))
            .with_env(CowEnv::Prod),
        TradingSdkOptions::new().with_orderbook_client(orderbook),
    )
    .expect("trading sdk must construct for the cancellation test");

    let trade = sample_trade_parameters(cow_sdk_core::OrderKind::Sell);
    let token = cow_sdk_core::CancellationToken::new();
    token.cancel();

    let error = sdk
        .get_quote_only(trade, None)
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
    let sdk = TradingSdk::new(
        PartialTraderParameters::new()
            .with_chain_id(SupportedChainId::Sepolia)
            .with_app_code("cancellation-test".to_owned())
            .with_owner(address(OWNER))
            .with_env(CowEnv::Prod),
        TradingSdkOptions::new().with_orderbook_client(orderbook),
    )
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
        sdk.get_quote_only(trade, None)
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

#[tokio::test]
async fn helper_only_sdk_refuses_quote_post_and_off_chain_cancel_flows() {
    let sdk = cow_sdk_trading::TradingSdkBuilder::new()
        .with_chain_id(SupportedChainId::Sepolia)
        .with_owner(address(OWNER))
        .build_helper_only()
        .expect("helper-only builder must succeed when chain id is set");

    let trade = sample_trade_parameters(cow_sdk_core::OrderKind::Sell);
    let quote_error = sdk
        .get_quote_only(trade, None)
        .await
        .expect_err("helper-only sdk must refuse the quote flow");
    assert!(matches!(
        quote_error,
        cow_sdk_trading::TradingError::HelperOnlyMode
    ));

    let cancel_error = sdk
        .off_chain_cancel_order_async(
            &OrderTraderParameters::new(order_uid())
                .with_chain_id(SupportedChainId::Sepolia)
                .with_env(CowEnv::Prod),
            &MockSigner::default(),
        )
        .await
        .expect_err("helper-only sdk must refuse the off-chain cancellation flow");
    assert!(matches!(
        cancel_error,
        cow_sdk_trading::TradingError::HelperOnlyMode
    ));
}
