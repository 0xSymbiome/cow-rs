#![allow(
    clippy::missing_const_for_fn,
    clippy::must_use_candidate,
    reason = "small integration-test helpers do not need public API lint polish"
)]

mod common;

use cow_sdk_core::{
    AddressPerChain, Amount, CowEnv, EVM_NATIVE_CURRENCY_ADDRESS, OrderKind, SupportedChainId,
};
use cow_sdk_trading::{
    GAS_LIMIT_DEFAULT, PostTradeAdditionalParams, cancel_order_onchain, get_eth_flow_transaction,
    get_pre_sign_transaction, onchain_cancellation_transaction,
};
use num_bigint::BigUint;

use crate::common::{
    CUSTOM_ETHFLOW, CUSTOM_SETTLEMENT, MockSigner, address, app_data_hash, ethflow_order,
    order_uid, regular_order, sample_limit_parameters, sample_trader_parameters,
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

fn set_estimated_gas(signer: &MockSigner, estimate: u64) {
    signer
        .state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .estimated_gas =
        Ok(Amount::new(estimate.to_string()).expect("test gas estimate must be valid"));
}

fn expected_gas_with_floor_overhead(estimate: u64) -> Amount {
    let estimate = u128::from(estimate);
    let expected = estimate + (estimate * 20) / 100;
    Amount::new(expected.to_string()).expect("expected gas estimate must be valid")
}

fn gas_overhead_cases() -> [u64; 6] {
    [1, 7, 100, 1_000, 100_000, u64::MAX / 2]
}

#[test]
fn presign_transaction_uses_zero_value_margin_and_settlement_override() {
    let signer = MockSigner::default();
    let options = cow_sdk_core::ProtocolOptions::new()
        .with_env(CowEnv::Staging)
        .with_settlement_contract_override(AddressPerChain::from([(
            u64::from(SupportedChainId::Sepolia),
            address(CUSTOM_SETTLEMENT),
        )]));

    let tx = get_pre_sign_transaction(
        &signer,
        SupportedChainId::Sepolia,
        &order_uid(),
        Some(&options),
    )
    .expect("pre-sign transaction should build");

    assert_eq!(tx.to, Some(address(CUSTOM_SETTLEMENT)));
    assert_eq!(tx.value, Some(Amount::zero()));
    assert_eq!(
        tx.gas_limit,
        Some(Amount::new("150000").expect("test gas literal must be valid"))
    );
}

#[test]
fn pre_sign_gas_estimate_applies_documented_floor_overhead() {
    for estimate in gas_overhead_cases() {
        let signer = MockSigner::default();
        set_estimated_gas(&signer, estimate);

        let tx = get_pre_sign_transaction(&signer, SupportedChainId::Sepolia, &order_uid(), None)
            .expect("pre-sign transaction should build");

        assert_eq!(
            tx.gas_limit,
            Some(expected_gas_with_floor_overhead(estimate)),
            "estimate={estimate}"
        );
    }
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
    let mut trader = sample_trader_parameters();
    trader.eth_flow_contract_override = Some(AddressPerChain::from([(
        u64::from(SupportedChainId::Sepolia),
        address("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
    )]));

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
        transaction.transaction.value,
        Some(transaction.order_to_sign.sell_amount.clone())
    );
    assert_eq!(
        transaction.transaction.gas_limit,
        Some(Amount::new("150000").expect("test gas literal must be valid"))
    );
}

#[tokio::test]
async fn eth_flow_gas_estimate_applies_documented_floor_overhead() {
    for estimate in gas_overhead_cases() {
        let signer = MockSigner::default();
        set_estimated_gas(&signer, estimate);
        let trader = sample_trader_parameters();
        let mut params = sample_limit_parameters(OrderKind::Sell);
        params.sell_token = address(EVM_NATIVE_CURRENCY_ADDRESS);
        params.quote_id = Some(3);

        let transaction = get_eth_flow_transaction(
            &app_data_hash(),
            &params,
            SupportedChainId::Sepolia,
            &PostTradeAdditionalParams::default(),
            &trader,
            &signer,
        )
        .await
        .expect("eth-flow transaction should build");

        assert_eq!(
            transaction.transaction.gas_limit,
            Some(expected_gas_with_floor_overhead(estimate)),
            "estimate={estimate}"
        );
    }
}

#[tokio::test]
async fn ethflow_transaction_encodes_high_bit_uint256_amounts_as_unsigned_words() {
    let signer = MockSigner::default();
    let trader = sample_trader_parameters();
    let high_sell: BigUint = BigUint::from(1u8) << 255u32;
    let high_buy = &high_sell + BigUint::from(1u8);
    let mut params = sample_limit_parameters(OrderKind::Sell);
    params.sell_token = address(EVM_NATIVE_CURRENCY_ADDRESS);
    params.sell_amount =
        Amount::new(high_sell.to_str_radix(10)).expect("2^255 amount literal must remain valid");
    params.buy_amount =
        Amount::new(high_buy.to_str_radix(10)).expect("2^255 + 1 amount literal must remain valid");
    params.quote_id = Some(3);
    params.valid_to = Some(1_234_567_890);

    let transaction = get_eth_flow_transaction(
        &app_data_hash(),
        &params,
        SupportedChainId::Sepolia,
        &PostTradeAdditionalParams::new().with_apply_costs_slippage_and_fees(false),
        &trader,
        &signer,
    )
    .await
    .expect("ethflow transaction should encode high-bit amounts");
    let data = transaction
        .transaction
        .data
        .as_ref()
        .expect("ethflow transaction must include call data");

    assert_eq!(
        calldata_word(&data.to_hex_string(), 2),
        uint256_word(&high_sell)
    );
    assert_eq!(
        calldata_word(&data.to_hex_string(), 3),
        uint256_word(&high_buy)
    );
}

#[tokio::test]
async fn ethflow_transaction_sign_extends_negative_quote_id_in_the_encoded_tuple() {
    // The canonical upstream `EthFlowOrder.Data.quoteId` field is a signed
    // `int64` value and must be sign-extended to a full 256-bit two's-complement
    // word in the ABI-encoded tuple. The quoteId sits at word index 8 of the
    // encoded struct.
    let signer = MockSigner::default();
    let trader = sample_trader_parameters();
    let mut params = sample_limit_parameters(OrderKind::Sell);
    params.sell_token = address(EVM_NATIVE_CURRENCY_ADDRESS);
    params.quote_id = Some(-1);
    params.valid_to = Some(1_234_567_890);

    let transaction = get_eth_flow_transaction(
        &app_data_hash(),
        &params,
        SupportedChainId::Sepolia,
        &PostTradeAdditionalParams::new().with_apply_costs_slippage_and_fees(false),
        &trader,
        &signer,
    )
    .await
    .expect("signed int64 quote id must round-trip through the ABI boundary");

    let data = transaction
        .transaction
        .data
        .as_ref()
        .expect("ethflow transaction must include call data");

    assert_eq!(
        calldata_word(&data.to_hex_string(), 8),
        "f".repeat(64),
        "negative int64 quote id must sign-extend to a full 256-bit two's-complement word",
    );
}

#[test]
fn onchain_cancellation_routes_regular_orders_to_settlement_and_ethflow_orders_to_ethflow() {
    let signer = MockSigner::default();
    let options = cow_sdk_core::ProtocolOptions::new()
        .with_env(CowEnv::Staging)
        .with_settlement_contract_override(AddressPerChain::from([(
            u64::from(SupportedChainId::Sepolia),
            address(CUSTOM_SETTLEMENT),
        )]))
        .with_eth_flow_contract_override(AddressPerChain::from([(
            u64::from(SupportedChainId::Sepolia),
            address(CUSTOM_ETHFLOW),
        )]));
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
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .estimated_gas = Err("estimation failed".to_owned());

    let tx = onchain_cancellation_transaction(
        &signer,
        SupportedChainId::Sepolia,
        &regular_order(),
        None,
    )
    .expect("fallback cancellation should build");

    let expected = GAS_LIMIT_DEFAULT.to_string();
    assert_eq!(
        tx.gas_limit,
        Some(Amount::new(expected).expect("fallback gas literal must be valid"))
    );
}

#[test]
fn cancel_order_onchain_sends_transaction_and_returns_hash() {
    let signer = MockSigner::default();

    let tx_hash = cancel_order_onchain(&signer, SupportedChainId::Sepolia, &regular_order(), None)
        .expect("onchain cancellation should send");

    assert_eq!(tx_hash.to_hex_string(), crate::common::TX_HASH);
}
