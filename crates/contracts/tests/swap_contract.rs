mod common;

use alloy_primitives::Bytes;
use cow_sdk_contracts::{BatchSwapStep, Signature, Swap, SwapEncoder, encode_swap_step};
use cow_sdk_core::{Address, Amount, OrderData, OrderKind, TypedDataDomain};

use common::fixture_case;

fn sample_domain() -> TypedDataDomain {
    cow_sdk_test_utils::builders::sample_domain()
}

fn sample_order(kind: OrderKind) -> OrderData {
    cow_sdk_test_utils::builders::OrderBuilder::weth_dai()
        .kind(kind)
        .build()
}

fn sample_signature() -> Signature {
    Signature::PreSign {
        owner: Address::new("0x1111111111111111111111111111111111111111").unwrap(),
    }
}

fn bytes_from_hex_literal(literal: &str) -> Bytes {
    let stripped = literal
        .strip_prefix("0x")
        .expect("hex literal must start with 0x");
    Bytes::from(alloy_primitives::hex::decode(stripped).expect("hex literal must decode"))
}

fn hex_prefixed(bytes: &Bytes) -> String {
    format!("0x{}", alloy_primitives::hex::encode(bytes))
}

#[test]
fn swap_step_encoding_defaults_user_data_and_indexes_tokens() {
    let fixture = fixture_case("contracts-swap-default-user-data");
    let mut encoder = SwapEncoder::new(sample_domain());

    let swap = Swap::new(
        format!("0x{}", "11".repeat(32)),
        Address::new("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap(),
        Address::new("0x6b175474e89094c44da98b954eedeac495271d0f").unwrap(),
        Amount::new("42").unwrap(),
        None,
    );
    encoder.encode_swap_step(std::slice::from_ref(&swap));
    let encoded_steps = encoder.swaps();
    assert_eq!(
        encoded_steps,
        vec![BatchSwapStep::new(
            swap.pool_id.clone(),
            0,
            1,
            Amount::new("42").unwrap(),
            Bytes::new(),
        )]
    );
    assert_eq!(
        hex_prefixed(&encoded_steps[0].user_data),
        fixture["expected"]["user_data"].as_str().unwrap(),
        "default user data must serialize as the fixture hex form"
    );

    let step = encode_swap_step(&mut cow_sdk_contracts::TokenRegistry::new(), &swap);
    assert!(
        step.user_data.is_empty(),
        "missing user data must normalize to an empty byte buffer"
    );
}

#[test]
fn swap_encoder_uses_contract_default_limit_amounts() {
    let mut sell_encoder = SwapEncoder::new(sample_domain());
    sell_encoder
        .encode_trade(&sample_order(OrderKind::Sell), &sample_signature(), None)
        .unwrap();
    assert_eq!(
        sell_encoder.trade().unwrap().executed_amount,
        Amount::new("2000000000000000000000").unwrap()
    );

    let mut buy_encoder = SwapEncoder::new(sample_domain());
    buy_encoder
        .encode_trade(&sample_order(OrderKind::Buy), &sample_signature(), None)
        .unwrap();
    assert_eq!(
        buy_encoder.trade().unwrap().executed_amount,
        Amount::new("1000000000000000000").unwrap()
    );

    assert!(SwapEncoder::new(sample_domain()).encoded_swap().is_err());
}

#[test]
fn swap_encoder_tokens_preserve_unique_registry_order() {
    let mut encoder = SwapEncoder::new(sample_domain());
    let weth = Address::new("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap();
    let dai = Address::new("0x6b175474e89094c44da98b954eedeac495271d0f").unwrap();
    let usdc = Address::new("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap();

    encoder.encode_swap_step(&[
        Swap::new(
            format!("0x{}", "11".repeat(32)),
            weth,
            dai,
            Amount::new("1").unwrap(),
            None,
        ),
        Swap::new(
            format!("0x{}", "22".repeat(32)),
            dai,
            usdc,
            Amount::new("2").unwrap(),
            Some(bytes_from_hex_literal("0x1234")),
        ),
    ]);

    assert_eq!(encoder.tokens(), vec![weth, dai, usdc]);
}

#[test]
fn swap_step_user_data_round_trips_byte_equal_through_the_encoder() {
    let swap = Swap::new(
        format!("0x{}", "33".repeat(32)),
        Address::new("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap(),
        Address::new("0x6b175474e89094c44da98b954eedeac495271d0f").unwrap(),
        Amount::new("100").unwrap(),
        Some(bytes_from_hex_literal("0xdeadbeefcafef00d")),
    );

    let step = encode_swap_step(&mut cow_sdk_contracts::TokenRegistry::new(), &swap);
    assert_eq!(
        step.user_data.as_ref(),
        &[0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xf0, 0x0d][..],
        "user data must preserve the input bytes byte-for-byte through the encoder"
    );

    let cloned = step.user_data.clone();
    assert_eq!(
        cloned.as_ptr(),
        step.user_data.as_ptr(),
        "alloy_primitives::Bytes clone must reference the same backing allocation"
    );
}
