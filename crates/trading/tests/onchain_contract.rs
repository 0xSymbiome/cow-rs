#![allow(
    clippy::missing_const_for_fn,
    clippy::must_use_candidate,
    clippy::option_if_let_else,
    reason = "small integration-test helpers do not need public API lint polish"
)]

mod common;

use alloy_primitives::U256;
use cow_sdk_core::{
    AddressPerChain, Amount, CowEnv, NATIVE_CURRENCY_ADDRESS, OrderKind, SupportedChainId,
    TransactionRequest,
};
use cow_sdk_trading::{
    DEFAULT_GAS_LIMIT, LimitTradeParamsFromQuote, PartnerFeePolicy, PostTradeAdditionalParams,
    eth_flow_transaction, onchain_cancel_order, onchain_cancellation_transaction,
    pre_sign_transaction,
};

use crate::common::{
    ALT_RECEIVER, CUSTOM_ETHFLOW, CUSTOM_SETTLEMENT, MockSigner, address, app_data_hash,
    ethflow_order, order_uid, regular_order, sample_limit_parameters, sample_trader_parameters,
};

fn calldata_word(data: &str, index: usize) -> String {
    let stripped = data
        .strip_prefix("0x")
        .expect("encoded call data must include 0x prefix");
    let start = 8 + (index * 64);
    stripped[start..start + 64].to_owned()
}

fn uint256_word(value: &U256) -> String {
    // Test oracle helper: format the cow uint256 value as the 32-byte
    // big-endian ABI word that the production encoder emits. The
    // `to_be_bytes::<32>()` byte stream is hex-encoded to the canonical
    // 64-character zero-padded lowercase form.
    alloy_primitives::hex::encode(value.to_be_bytes::<32>())
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

#[tokio::test]
async fn presign_transaction_uses_zero_value_margin_and_settlement_override() {
    let signer = MockSigner::default();
    let options = cow_sdk_core::ProtocolOptions::new()
        .with_env(CowEnv::Staging)
        .with_settlement_contract_override(AddressPerChain::from([(
            u64::from(SupportedChainId::Sepolia),
            address(CUSTOM_SETTLEMENT),
        )]));

    let tx = pre_sign_transaction(
        &signer,
        SupportedChainId::Sepolia,
        &order_uid(),
        Some(&options),
    )
    .await
    .expect("pre-sign transaction should build");

    assert_eq!(tx.to, address(CUSTOM_SETTLEMENT));
    assert_eq!(tx.value, Amount::ZERO);
    assert_eq!(
        tx.gas_limit,
        Amount::new("150000").expect("test gas literal must be valid")
    );

    // The submission-side conversion mirrors every prepared field into the
    // optional-field wire shape.
    let request = TransactionRequest::from(tx.clone());
    assert_eq!(request.to, Some(tx.to));
    assert_eq!(request.data.as_ref(), Some(&tx.data));
    assert_eq!(request.value, Some(tx.value));
    assert_eq!(request.gas_limit, Some(tx.gas_limit));
}

#[tokio::test]
async fn pre_sign_gas_estimate_applies_documented_floor_overhead() {
    for estimate in gas_overhead_cases() {
        let signer = MockSigner::default();
        set_estimated_gas(&signer, estimate);

        let tx = pre_sign_transaction(&signer, SupportedChainId::Sepolia, &order_uid(), None)
            .await
            .expect("pre-sign transaction should build");

        assert_eq!(
            tx.gas_limit,
            expected_gas_with_floor_overhead(estimate),
            "estimate={estimate}"
        );
    }
}

#[tokio::test]
async fn ethflow_transaction_uses_wrapped_native_value_margin_and_ethflow_override() {
    let signer = MockSigner::default();
    let mut params = sample_limit_parameters(cow_sdk_core::OrderKind::Sell);
    params.sell_token = NATIVE_CURRENCY_ADDRESS;
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

    let from_quote =
        LimitTradeParamsFromQuote::try_from_limit(params).expect("test params carry a quote id");
    let transaction = eth_flow_transaction(
        &app_data_hash(),
        &from_quote,
        &PostTradeAdditionalParams::default(),
        &trader,
        &signer,
    )
    .await
    .expect("ethflow transaction should build");

    assert_eq!(transaction.transaction.to, address(CUSTOM_ETHFLOW));
    assert_eq!(
        transaction.order_to_sign.sell_token,
        cow_sdk_core::wrapped_native_token(SupportedChainId::Sepolia).address
    );
    assert_eq!(
        transaction.transaction.value,
        transaction.order_to_sign.sell_amount
    );
    assert_eq!(
        transaction.transaction.gas_limit,
        Amount::new("150000").expect("test gas literal must be valid")
    );
}

#[tokio::test]
async fn ethflow_create_value_equals_encoded_sell_plus_fee_amounts() {
    // Pin CoWSwapEthFlow.createOrder's `msg.value == sellAmount + feeAmount`
    // rule: decode sellAmount (word 2) and feeAmount (word 5) from the encoded
    // calldata and assert the tx value equals their sum. Fails closed if a fee
    // is ever threaded in without raising the sent value.
    let signer = MockSigner::default();
    let trader = sample_trader_parameters();
    let mut params = sample_limit_parameters(OrderKind::Sell);
    params.sell_token = NATIVE_CURRENCY_ADDRESS;
    params.quote_id = Some(7);
    params.slippage_bps = Some(50);

    let from_quote =
        LimitTradeParamsFromQuote::try_from_limit(params).expect("test params carry a quote id");
    let transaction = eth_flow_transaction(
        &app_data_hash(),
        &from_quote,
        &PostTradeAdditionalParams::default(),
        &trader,
        &signer,
    )
    .await
    .expect("ethflow transaction should build");

    let data = transaction.transaction.data.to_hex_string();
    let encoded_sell_amount = U256::from_be_slice(
        &alloy_primitives::hex::decode(calldata_word(&data, 2))
            .expect("encoded sell amount word must decode"),
    );
    let encoded_fee_amount = U256::from_be_slice(
        &alloy_primitives::hex::decode(calldata_word(&data, 5))
            .expect("encoded fee amount word must decode"),
    );

    assert_eq!(
        encoded_fee_amount,
        U256::ZERO,
        "eth-flow signed order feeAmount must be zero on the live protocol",
    );
    assert_eq!(
        *transaction.transaction.value.as_u256(),
        encoded_sell_amount + encoded_fee_amount,
        "eth-flow tx value must equal encoded sellAmount + feeAmount per the \
         CoWSwapEthFlow.createOrder msg.value check",
    );
}

async fn eth_flow_signed_buy_amount(protocol_fee_bps: Option<f64>) -> String {
    // Native-currency sell, sell 1e18 -> buy 2e18, zero network cost, partner
    // fee 100 bps, slippage 50 — the upstream composition vector adapted to the
    // eth-flow lane. Returns the signed buy amount so the caller can assert the
    // protocol-fee golden against the no-protocol-fee golden.
    let signer = MockSigner::default();
    let trader = sample_trader_parameters();
    let mut params = sample_limit_parameters(OrderKind::Sell);
    params.sell_token = NATIVE_CURRENCY_ADDRESS;
    params.sell_amount = Amount::new("1000000000000000000").expect("sell literal must be valid");
    params.buy_amount = Amount::new("2000000000000000000").expect("buy literal must be valid");
    params.slippage_bps = Some(50);
    params.partner_fee = Some(
        PartnerFeePolicy::volume(100, address(ALT_RECEIVER))
            .expect("volume policy must validate")
            .into(),
    );
    let additional = match protocol_fee_bps {
        Some(bps) => PostTradeAdditionalParams::new().with_protocol_fee_bps(bps),
        None => PostTradeAdditionalParams::new(),
    };
    let from_quote =
        LimitTradeParamsFromQuote::try_from_limit(params).expect("test params carry a quote id");

    eth_flow_transaction(&app_data_hash(), &from_quote, &additional, &trader, &signer)
        .await
        .expect("eth-flow transaction should build")
        .order_to_sign
        .buy_amount
        .to_string()
}

#[tokio::test]
async fn eth_flow_transaction_threads_protocol_fee_into_the_signed_amounts() {
    // The native-currency lane applies `protocolFeeBps` to the signed amounts the
    // same way the orderbook lane does: with a partner fee configured, the
    // protocol fee enlarges the partner-fee base and lowers the signed buy
    // amount (the eth-flow lane dropped the value at HEAD).
    assert_eq!(
        eth_flow_signed_buy_amount(Some(5.0)).await,
        "1970090045022511257"
    );
    assert_eq!(
        eth_flow_signed_buy_amount(None).await,
        "1970100000000000000"
    );
}

#[tokio::test]
async fn eth_flow_gas_estimate_applies_documented_floor_overhead() {
    for estimate in gas_overhead_cases() {
        let signer = MockSigner::default();
        set_estimated_gas(&signer, estimate);
        let trader = sample_trader_parameters();
        let mut params = sample_limit_parameters(OrderKind::Sell);
        params.sell_token = NATIVE_CURRENCY_ADDRESS;
        params.quote_id = Some(3);

        let from_quote = LimitTradeParamsFromQuote::try_from_limit(params)
            .expect("test params carry a quote id");
        let transaction = eth_flow_transaction(
            &app_data_hash(),
            &from_quote,
            &PostTradeAdditionalParams::default(),
            &trader,
            &signer,
        )
        .await
        .expect("eth-flow transaction should build");

        assert_eq!(
            transaction.transaction.gas_limit,
            expected_gas_with_floor_overhead(estimate),
            "estimate={estimate}"
        );
    }
}

#[tokio::test]
async fn ethflow_transaction_encodes_high_bit_uint256_amounts_as_unsigned_words() {
    let signer = MockSigner::default();
    let trader = sample_trader_parameters();
    let high_sell: U256 = U256::from(1u8) << 255usize;
    let high_buy = high_sell + U256::from(1u8);
    let mut params = sample_limit_parameters(OrderKind::Sell);
    params.sell_token = NATIVE_CURRENCY_ADDRESS;
    params.sell_amount = Amount::from_u256(high_sell);
    params.buy_amount = Amount::from_u256(high_buy);
    params.quote_id = Some(3);
    params.valid_to = Some(1_234_567_890);

    let from_quote =
        LimitTradeParamsFromQuote::try_from_limit(params).expect("test params carry a quote id");
    let transaction = eth_flow_transaction(
        &app_data_hash(),
        &from_quote,
        &PostTradeAdditionalParams::new().with_apply_costs_slippage_and_fees(false),
        &trader,
        &signer,
    )
    .await
    .expect("ethflow transaction should encode high-bit amounts");
    let data = &transaction.transaction.data;

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
    params.sell_token = NATIVE_CURRENCY_ADDRESS;
    params.quote_id = Some(-1);
    params.valid_to = Some(1_234_567_890);

    let from_quote =
        LimitTradeParamsFromQuote::try_from_limit(params).expect("test params carry a quote id");
    let transaction = eth_flow_transaction(
        &app_data_hash(),
        &from_quote,
        &PostTradeAdditionalParams::new().with_apply_costs_slippage_and_fees(false),
        &trader,
        &signer,
    )
    .await
    .expect("signed int64 quote id must round-trip through the ABI boundary");

    let data = &transaction.transaction.data;

    assert_eq!(
        calldata_word(&data.to_hex_string(), 8),
        "f".repeat(64),
        "negative int64 quote id must sign-extend to a full 256-bit two's-complement word",
    );
}

#[tokio::test]
async fn onchain_cancellation_routes_regular_orders_to_settlement_and_ethflow_orders_to_ethflow() {
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
    .await
    .expect("regular cancellation should build");
    let ethflow_tx = onchain_cancellation_transaction(
        &signer,
        SupportedChainId::Sepolia,
        &ethflow_order(),
        Some(&options),
    )
    .await
    .expect("ethflow cancellation should build");

    assert_eq!(regular_tx.to, Some(address(CUSTOM_SETTLEMENT)));
    assert_eq!(ethflow_tx.to, Some(address(CUSTOM_ETHFLOW)));
}

#[tokio::test]
async fn onchain_cancellation_uses_fallback_gas_when_estimation_fails() {
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
    .await
    .expect("fallback cancellation should build");

    let expected = DEFAULT_GAS_LIMIT.to_string();
    assert_eq!(
        tx.gas_limit,
        Some(Amount::new(expected).expect("fallback gas literal must be valid"))
    );
}

#[tokio::test]
async fn cancel_order_onchain_sends_transaction_and_returns_hash() {
    let signer = MockSigner::default();

    let tx_hash = onchain_cancel_order(&signer, SupportedChainId::Sepolia, &regular_order(), None)
        .await
        .expect("onchain cancellation should send");

    assert_eq!(tx_hash.to_hex_string(), crate::common::TX_HASH);
}
