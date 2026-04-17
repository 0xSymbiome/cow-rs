mod common;

use bytes::Bytes;
use cow_sdk_contracts::{BatchSwapStep, Order, Signature, Swap, SwapEncoder, encode_swap_step};
use cow_sdk_core::{Address, Amount, AppDataHex, OrderBalance, OrderKind, TypedDataDomain};

use common::fixture_case;

fn sample_domain() -> TypedDataDomain {
    TypedDataDomain {
        name: "Gnosis Protocol".to_owned(),
        version: "v2".to_owned(),
        chain_id: 1,
        verifying_contract: Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap(),
    }
}

fn sample_order(kind: OrderKind) -> Order {
    Order {
        sell_token: Address::new("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap(),
        buy_token: Address::new("0x6b175474e89094c44da98b954eedeac495271d0f").unwrap(),
        receiver: None,
        sell_amount: Amount::new("1000000000000000000").unwrap(),
        buy_amount: Amount::new("2000000000000000000000").unwrap(),
        valid_to: 1_709_990_000,
        app_data: AppDataHex::new(
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap(),
        fee_amount: Amount::new("5000000000000000").unwrap(),
        kind,
        partially_fillable: false,
        sell_token_balance: Some(OrderBalance::Erc20),
        buy_token_balance: Some(OrderBalance::Erc20),
    }
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
    Bytes::from(hex::decode(stripped).expect("hex literal must decode"))
}

fn hex_prefixed(bytes: &Bytes) -> String {
    format!("0x{}", hex::encode(bytes))
}

#[test]
fn swap_step_encoding_defaults_user_data_and_indexes_tokens() {
    let fixture = fixture_case("contracts-swap-default-user-data");
    let mut encoder = SwapEncoder::new(sample_domain());

    let swap = Swap {
        pool_id: format!("0x{}", "11".repeat(32)),
        asset_in: Address::new("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap(),
        asset_out: Address::new("0x6b175474e89094c44da98b954eedeac495271d0f").unwrap(),
        amount: Amount::new("42").unwrap(),
        user_data: None,
    };
    encoder.encode_swap_step(std::slice::from_ref(&swap));
    let encoded_steps = encoder.swaps();
    assert_eq!(
        encoded_steps,
        vec![BatchSwapStep {
            pool_id: swap.pool_id.clone(),
            asset_in_index: 0,
            asset_out_index: 1,
            amount: Amount::new("42").unwrap(),
            user_data: Bytes::new(),
        }]
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
        Swap {
            pool_id: format!("0x{}", "11".repeat(32)),
            asset_in: weth.clone(),
            asset_out: dai.clone(),
            amount: Amount::new("1").unwrap(),
            user_data: None,
        },
        Swap {
            pool_id: format!("0x{}", "22".repeat(32)),
            asset_in: dai.clone(),
            asset_out: usdc.clone(),
            amount: Amount::new("2").unwrap(),
            user_data: Some(bytes_from_hex_literal("0x1234")),
        },
    ]);

    assert_eq!(encoder.tokens(), vec![weth, dai, usdc]);
}

#[test]
fn swap_step_user_data_round_trips_byte_equal_through_the_encoder() {
    let swap = Swap {
        pool_id: format!("0x{}", "33".repeat(32)),
        asset_in: Address::new("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap(),
        asset_out: Address::new("0x6b175474e89094c44da98b954eedeac495271d0f").unwrap(),
        amount: Amount::new("100").unwrap(),
        user_data: Some(bytes_from_hex_literal("0xdeadbeefcafef00d")),
    };

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
        "bytes::Bytes clone must reference the same backing allocation"
    );
}
