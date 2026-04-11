mod common;

use std::sync::Arc;

use cow_sdk_core::{AddressPerChain, Amount, CowEnv, SupportedChainId};
use cow_sdk_trading::{
    ApprovalParameters, OrderTraderParameters, PartialTraderParameters, TradingSdk,
    TradingSdkOptions,
};

use crate::common::{
    ALT_RECEIVER, COW, CUSTOM_ETHFLOW, CUSTOM_SETTLEMENT, MockOrderbook, MockProvider, MockSigner,
    OWNER, address, ethflow_order, order_uid, sample_trade_parameters, sell_quote_response,
};

#[tokio::test]
async fn sdk_quote_only_works_without_signer_and_uses_owner_as_from() {
    let orderbook = Arc::new(MockOrderbook::new(
        SupportedChainId::Sepolia,
        sell_quote_response(),
    ));
    let sdk = TradingSdk::new(
        PartialTraderParameters {
            chain_id: Some(SupportedChainId::Sepolia),
            app_code: Some("0x007".to_owned()),
            owner: None,
            env: Some(CowEnv::Prod),
            settlement_contract_override: None,
            eth_flow_contract_override: None,
        },
        TradingSdkOptions::new().with_orderbook_client(orderbook.clone()),
    );
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
    assert_eq!(result.quote_response.id, Some(575401));
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
        .err()
        .expect("mismatched injected orderbook chain must fail validation");
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
async fn sdk_quote_only_reports_missing_chainid_and_appcode_explicitly() {
    let trade = sample_trade_parameters(cow_sdk_core::OrderKind::Sell);

    let missing_chain = TradingSdk::new(
        PartialTraderParameters {
            chain_id: None,
            app_code: Some("0x007".to_owned()),
            owner: None,
            env: None,
            settlement_contract_override: None,
            eth_flow_contract_override: None,
        },
        TradingSdkOptions::default(),
    );
    let chain_error = missing_chain
        .get_quote_only(trade.clone(), None)
        .await
        .expect_err("missing chainId must fail")
        .to_string();
    assert!(chain_error.contains("Missing quoter parameters: chainId"));

    let missing_app = TradingSdk::new(
        PartialTraderParameters {
            chain_id: Some(SupportedChainId::Sepolia),
            app_code: None,
            owner: None,
            env: None,
            settlement_contract_override: None,
            eth_flow_contract_override: None,
        },
        TradingSdkOptions::default(),
    );
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
    let sdk = TradingSdk::new(
        PartialTraderParameters {
            chain_id: Some(SupportedChainId::Sepolia),
            app_code: Some("0x007".to_owned()),
            owner: None,
            env: Some(CowEnv::Prod),
            settlement_contract_override: None,
            eth_flow_contract_override: None,
        },
        TradingSdkOptions::default(),
    );

    let allowance = sdk
        .get_cow_protocol_allowance(
            &provider,
            &cow_sdk_trading::AllowanceParameters {
                token_address: address(COW),
                owner: address(OWNER),
                chain_id: Some(SupportedChainId::Mainnet),
                env: Some(CowEnv::Prod),
                vault_relayer_address: None,
            },
        )
        .expect("allowance read should succeed");
    assert_eq!(
        allowance,
        Amount::new("1000000000000000000").expect("test allowance literal must be valid")
    );

    let approval_hash = sdk
        .approve_cow_protocol(
            &signer,
            &ApprovalParameters {
                token_address: address(COW),
                amount: Amount::new("1000").expect("test approval amount literal must be valid"),
                chain_id: Some(SupportedChainId::Mainnet),
                env: Some(CowEnv::Prod),
                vault_relayer_address: Some(address(ALT_RECEIVER)),
            },
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
            .map(|value| value.as_str())
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
    let sdk = TradingSdk::new(
        PartialTraderParameters {
            chain_id: Some(SupportedChainId::Sepolia),
            app_code: Some("0x007".to_owned()),
            owner: None,
            env: Some(CowEnv::Prod),
            settlement_contract_override: None,
            eth_flow_contract_override: None,
        },
        TradingSdkOptions::default(),
    );

    let allowance = sdk
        .get_cow_protocol_allowance_async(
            &provider,
            &cow_sdk_trading::AllowanceParameters {
                token_address: address(COW),
                owner: address(OWNER),
                chain_id: Some(SupportedChainId::Mainnet),
                env: Some(CowEnv::Prod),
                vault_relayer_address: None,
            },
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
            &ApprovalParameters {
                token_address: address(COW),
                amount: Amount::new("1000").expect("test approval amount literal must be valid"),
                chain_id: Some(SupportedChainId::Mainnet),
                env: Some(CowEnv::Prod),
                vault_relayer_address: Some(address(ALT_RECEIVER)),
            },
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
    let sdk = TradingSdk::new(
        PartialTraderParameters {
            chain_id: Some(SupportedChainId::Sepolia),
            app_code: Some("0x007".to_owned()),
            owner: None,
            env: Some(CowEnv::Prod),
            settlement_contract_override: Some(AddressPerChain::from([(
                u64::from(SupportedChainId::Sepolia),
                address("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
            )])),
            eth_flow_contract_override: Some(AddressPerChain::from([(
                u64::from(SupportedChainId::Sepolia),
                address("0xcccccccccccccccccccccccccccccccccccccccc"),
            )])),
        },
        TradingSdkOptions::new().with_orderbook_client(orderbook.clone()),
    );

    let pre_sign_tx = sdk
        .get_pre_sign_transaction(
            &OrderTraderParameters {
                order_uid: order_uid(),
                chain_id: Some(SupportedChainId::Sepolia),
                env: Some(CowEnv::Staging),
                settlement_contract_override: Some(AddressPerChain::from([(
                    u64::from(SupportedChainId::Sepolia),
                    address(CUSTOM_SETTLEMENT),
                )])),
                eth_flow_contract_override: None,
            },
            &signer,
        )
        .expect("pre-sign transaction should succeed");
    assert_eq!(pre_sign_tx.to, Some(address(CUSTOM_SETTLEMENT)));

    let tx_hash = sdk
        .on_chain_cancel_order(
            &OrderTraderParameters {
                order_uid: order_uid(),
                chain_id: Some(SupportedChainId::Sepolia),
                env: Some(CowEnv::Staging),
                settlement_contract_override: None,
                eth_flow_contract_override: Some(AddressPerChain::from([(
                    u64::from(SupportedChainId::Sepolia),
                    address(CUSTOM_ETHFLOW),
                )])),
            },
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
        PartialTraderParameters {
            chain_id: Some(SupportedChainId::Sepolia),
            app_code: Some("0x007".to_owned()),
            owner: None,
            env: Some(CowEnv::Prod),
            settlement_contract_override: Some(AddressPerChain::from([(
                u64::from(SupportedChainId::Sepolia),
                address(CUSTOM_SETTLEMENT),
            )])),
            eth_flow_contract_override: Some(AddressPerChain::from([(
                u64::from(SupportedChainId::Sepolia),
                address(CUSTOM_ETHFLOW),
            )])),
        },
        TradingSdkOptions::new().with_orderbook_client(orderbook),
    );

    sdk.on_chain_cancel_order(
        &OrderTraderParameters {
            order_uid: order_uid(),
            chain_id: Some(SupportedChainId::Sepolia),
            env: Some(CowEnv::Prod),
            settlement_contract_override: None,
            eth_flow_contract_override: None,
        },
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
