mod common;

use cow_sdk_core::{AddressPerChain, CowEnv, EVM_NATIVE_CURRENCY_ADDRESS, SupportedChainId};
use cow_sdk_trading::{
    GAS_LIMIT_DEFAULT, PostTradeAdditionalParams, TraderParameters, cancel_order_onchain,
    get_eth_flow_transaction, get_pre_sign_transaction, onchain_cancellation_transaction,
};

use crate::common::{
    CUSTOM_ETHFLOW, CUSTOM_SETTLEMENT, MockSigner, address, app_data_hash, ethflow_order,
    order_uid, regular_order, sample_limit_parameters, sample_trader_parameters,
};

#[test]
fn presign_transaction_uses_zero_value_margin_and_settlement_override() {
    let signer = MockSigner::default();
    let options = cow_sdk_core::ProtocolOptions {
        env: Some(CowEnv::Staging),
        settlement_contract_override: Some(AddressPerChain::from([(
            u64::from(SupportedChainId::Sepolia),
            address(CUSTOM_SETTLEMENT),
        )])),
        eth_flow_contract_override: None,
    };

    let tx = get_pre_sign_transaction(
        &signer,
        SupportedChainId::Sepolia,
        &order_uid(),
        Some(&options),
    )
    .expect("pre-sign transaction should build");

    assert_eq!(tx.to, Some(address(CUSTOM_SETTLEMENT)));
    assert_eq!(tx.value.as_deref(), Some("0"));
    assert_eq!(tx.gas_limit.as_deref(), Some("150000"));
}

#[tokio::test]
async fn ethflow_transaction_uses_wrapped_native_value_margin_and_ethflow_override() {
    let signer = MockSigner::default();
    let mut params = sample_limit_parameters(cow_sdk_core::OrderKind::Sell);
    params.sell_token = address(EVM_NATIVE_CURRENCY_ADDRESS);
    params.quote_id = Some(3);
    params.slippage_bps = Some(50);
    params.eth_flow_contract_override = Some(AddressPerChain::from([(
        u64::from(SupportedChainId::Sepolia),
        address(CUSTOM_ETHFLOW),
    )]));
    let trader = TraderParameters {
        eth_flow_contract_override: Some(AddressPerChain::from([(
            u64::from(SupportedChainId::Sepolia),
            address("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        )])),
        ..sample_trader_parameters()
    };

    let transaction = get_eth_flow_transaction(
        &app_data_hash(),
        &params,
        SupportedChainId::Sepolia,
        &PostTradeAdditionalParams::default(),
        &trader,
        &signer,
    )
    .await
    .expect("ethflow transaction should build");

    assert_eq!(transaction.transaction.to, Some(address(CUSTOM_ETHFLOW)));
    assert_eq!(
        transaction.order_to_sign.sell_token,
        cow_sdk_core::wrapped_native_token(SupportedChainId::Sepolia).address
    );
    assert_eq!(
        transaction.transaction.value.as_deref(),
        Some(transaction.order_to_sign.sell_amount.as_str())
    );
    assert_eq!(transaction.transaction.gas_limit.as_deref(), Some("150000"));
}

#[test]
fn onchain_cancellation_routes_regular_orders_to_settlement_and_ethflow_orders_to_ethflow() {
    let signer = MockSigner::default();
    let options = cow_sdk_core::ProtocolOptions {
        env: Some(CowEnv::Staging),
        settlement_contract_override: Some(AddressPerChain::from([(
            u64::from(SupportedChainId::Sepolia),
            address(CUSTOM_SETTLEMENT),
        )])),
        eth_flow_contract_override: Some(AddressPerChain::from([(
            u64::from(SupportedChainId::Sepolia),
            address(CUSTOM_ETHFLOW),
        )])),
    };
    let regular_tx = onchain_cancellation_transaction(
        &signer,
        SupportedChainId::Sepolia,
        &regular_order(),
        Some(&options),
    )
    .expect("regular cancellation should build");
    let ethflow_tx = onchain_cancellation_transaction(
        &signer,
        SupportedChainId::Sepolia,
        &ethflow_order(),
        Some(&options),
    )
    .expect("ethflow cancellation should build");

    assert_eq!(regular_tx.to, Some(address(CUSTOM_SETTLEMENT)));
    assert_eq!(ethflow_tx.to, Some(address(CUSTOM_ETHFLOW)));
}

#[test]
fn onchain_cancellation_uses_fallback_gas_when_estimation_fails() {
    let signer = MockSigner::default();
    signer
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .estimated_gas = Err("estimation failed".to_owned());

    let tx = onchain_cancellation_transaction(
        &signer,
        SupportedChainId::Sepolia,
        &regular_order(),
        None,
    )
    .expect("fallback cancellation should build");

    let expected = GAS_LIMIT_DEFAULT.to_string();
    assert_eq!(tx.gas_limit.as_deref(), Some(expected.as_str()));
}

#[test]
fn cancel_order_onchain_sends_transaction_and_returns_hash() {
    let signer = MockSigner::default();

    let tx_hash = cancel_order_onchain(&signer, SupportedChainId::Sepolia, &regular_order(), None)
        .expect("onchain cancellation should send");

    assert_eq!(tx_hash, crate::common::TX_HASH);
}
