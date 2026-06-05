#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::derive_partial_eq_without_eq,
    clippy::iter_on_single_items,
    clippy::missing_const_for_fn,
    clippy::option_if_let_else,
    clippy::redundant_clone,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::unnested_or_patterns,
    reason = "pedantic, nursery, style, and perf lints acceptable in test helper code"
)]

use sha3::{Digest, Keccak256};

use cow_sdk_contracts::{
    InteractionLike, InteractionStage, OrderFlags, OrderRefunds, SettlementEncoder, SigningScheme,
    TokenRegistry, Trade, TradeExecution, TradeFlags, decode_order, decode_order_flags,
    decode_trade_flags, encode_order_flags, encode_trade_flags,
};
use cow_sdk_core::{
    Address, Amount, AppDataHex, BuyTokenDestination, OrderData, OrderKind, OrderUid,
    SellTokenSource, TypedDataDomain,
};

mod common;
use common::{bytes_from_hex_literal, sample_presign};

fn sample_domain() -> TypedDataDomain {
    cow_sdk_test_utils::builders::sample_domain()
}

fn sample_order(kind: OrderKind, partially_fillable: bool) -> OrderData {
    cow_sdk_test_utils::builders::OrderBuilder::weth_dai()
        .kind(kind)
        .partially_fillable(partially_fillable)
        .sell_balance(SellTokenSource::Internal)
        .buy_balance(BuyTokenDestination::Internal)
        .build()
}

fn u256_word(value: u64) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[24..].copy_from_slice(&value.to_be_bytes());
    out
}

fn expected_bytes_array_call_data(selector_signature: &str, uid: &[u8; 56]) -> Vec<u8> {
    let digest = Keccak256::digest(selector_signature.as_bytes());
    let selector = [digest[0], digest[1], digest[2], digest[3]];
    let mut expected = Vec::new();
    expected.extend_from_slice(&selector);
    expected.extend_from_slice(&u256_word(32));
    expected.extend_from_slice(&u256_word(1));
    expected.extend_from_slice(&u256_word(32));
    expected.extend_from_slice(&u256_word(56));
    expected.extend_from_slice(uid);
    expected.extend_from_slice(&[0u8; 8]);
    expected
}

#[test]
fn flag_encode_decode_round_trips_and_rejects_the_reserved_bit() {
    let encoded_buy_partial = encode_order_flags(&OrderFlags::new(
        OrderKind::Buy,
        true,
        SellTokenSource::Internal,
        BuyTokenDestination::Internal,
    ))
    .unwrap();

    let encoded_trade = encode_trade_flags(&TradeFlags::new(
        OrderKind::Sell,
        false,
        SellTokenSource::Erc20,
        BuyTokenDestination::Erc20,
        SigningScheme::PreSign,
    ))
    .unwrap();

    let decoded_order = decode_order_flags(encoded_buy_partial).unwrap();
    assert_eq!(decoded_order.kind, OrderKind::Buy);
    assert_eq!(decoded_order.sell_token_balance, SellTokenSource::Internal);
    assert_eq!(
        decoded_order.buy_token_balance,
        BuyTokenDestination::Internal
    );

    let decoded_trade = decode_trade_flags(encoded_trade).unwrap();
    assert_eq!(decoded_trade.signing_scheme, SigningScheme::PreSign);
    assert_eq!(
        decode_order_flags(0b0100).unwrap().sell_token_balance,
        SellTokenSource::Erc20
    );
    assert!(decode_order_flags(1 << 7).is_err());
}

#[test]
fn trade_flag_encoding_keeps_order_and_signing_bits_partitioned() {
    let order_flags = OrderFlags::new(
        OrderKind::Buy,
        true,
        SellTokenSource::Internal,
        BuyTokenDestination::Internal,
    );
    let encoded_order = encode_order_flags(&order_flags).unwrap();
    assert_eq!(encoded_order & 0b1110_0000, 0);

    for signing_scheme in [
        SigningScheme::Eip712,
        SigningScheme::EthSign,
        SigningScheme::Eip1271,
        SigningScheme::PreSign,
    ] {
        let encoded_trade = encode_trade_flags(&TradeFlags::new(
            order_flags.kind,
            order_flags.partially_fillable,
            order_flags.sell_token_balance,
            order_flags.buy_token_balance,
            signing_scheme,
        ))
        .unwrap();

        assert_eq!(encoded_trade & 0b1_1111, encoded_order);
        assert_eq!((encoded_trade >> 5) & 0b11, signing_scheme.as_u8());
        assert_eq!(encoded_trade, encoded_order + (signing_scheme.as_u8() << 5));
    }
}

#[test]
fn settlement_encoder_tracks_tokens_prices_and_interactions() {
    let domain = sample_domain();
    let order = sample_order(OrderKind::Sell, false);
    let mut encoder = SettlementEncoder::new(domain.clone());

    encoder
        .encode_interaction(
            &InteractionLike::new(
                Address::new("0xdef1c0ded9bec7f1a1670819833240f027b25eff").unwrap(),
                None,
                Some(bytes_from_hex_literal("0x12345678")),
            ),
            InteractionStage::Pre,
        )
        .unwrap();
    encoder
        .encode_trade(
            &order,
            &sample_presign(),
            Some(TradeExecution::new(
                Amount::new("1000000000000000000").unwrap(),
            )),
        )
        .unwrap();

    let prices = serde_json::from_value::<cow_sdk_contracts::Prices>(serde_json::json!({
        "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2": "1000000000000000000",
        "0x6b175474e89094c44da98b954eedeac495271d0f": "500000000000000",
    }))
    .unwrap();

    let settlement = encoder.encoded_settlement(&prices).unwrap();
    assert_eq!(settlement.0.len(), 2);
    assert_eq!(
        settlement.1,
        vec![
            Amount::new("1000000000000000000").unwrap(),
            Amount::new("500000000000000").unwrap()
        ]
    );
    assert_eq!(settlement.2.len(), 1);
    assert_eq!(settlement.3[InteractionStage::Pre as usize].len(), 1);
    assert_eq!(settlement.3[InteractionStage::Intra as usize].len(), 0);
    assert_eq!(settlement.3[InteractionStage::Post as usize].len(), 0);

    let missing = serde_json::from_value::<cow_sdk_contracts::Prices>(serde_json::json!({
        "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2": "1000000000000000000"
    }))
    .unwrap();
    assert!(encoder.clearing_prices(&missing).is_err());

    let setup = SettlementEncoder::encoded_setup(&[
        InteractionLike::new(
            Address::new("0x1234567890123456789012345678901234567890").unwrap(),
            None,
            Some(bytes_from_hex_literal("0x87654321")),
        ),
        InteractionLike::new(
            Address::new("0xabcdef0123456789abcdef0123456789abcdef01").unwrap(),
            Some(Amount::new("1").unwrap()),
            None,
        ),
    ])
    .unwrap();
    assert!(setup.0.is_empty());
    assert!(setup.1.is_empty());
    assert!(setup.2.is_empty());
    assert!(setup.3[InteractionStage::Pre as usize].is_empty());
    assert_eq!(setup.3[InteractionStage::Intra as usize].len(), 2);
    assert!(setup.3[InteractionStage::Post as usize].is_empty());
}

#[test]
fn order_refunds_and_trade_decoding_follow_contract_rules() {
    let domain = sample_domain();
    let mut encoder = SettlementEncoder::new(domain.clone());
    let uids = [
        OrderUid::new(format!(
            "0x{}{}{}",
            "01".repeat(32),
            "02".repeat(20),
            "00000000"
        ))
        .unwrap(),
        OrderUid::new(format!(
            "0x{}{}{}",
            "03".repeat(32),
            "04".repeat(20),
            "00000000"
        ))
        .unwrap(),
    ];

    encoder
        .encode_order_refunds(&OrderRefunds::new(vec![uids[0]], vec![uids[1]]))
        .unwrap();

    let post = encoder.interactions().unwrap()[InteractionStage::Post as usize].clone();
    assert_eq!(post.len(), 2);
    assert_eq!(post[0].target, domain.verifying_contract);
    let invalid = serde_json::from_value::<OrderRefunds>(serde_json::json!({
        "filledAmounts": ["0x1234"],
        "preSignatures": []
    }));
    assert!(invalid.is_err());

    let partially_fillable = sample_order(OrderKind::Buy, true);
    assert!(
        SettlementEncoder::new(sample_domain())
            .encode_trade(&partially_fillable, &sample_presign(), None)
            .is_err()
    );

    let mut tokens = TokenRegistry::new();
    let first = tokens.index(&Address::new("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap());
    let second = tokens.index(&Address::new("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap());
    assert_eq!(first, second);

    let trade = Trade::new(
        0,
        1,
        Address::new("0x3333333333333333333333333333333333333333").unwrap(),
        Amount::new("10").unwrap(),
        Amount::new("20").unwrap(),
        123,
        AppDataHex::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap(),
        Amount::new("1").unwrap(),
        encode_order_flags(&OrderFlags::new(
            OrderKind::Sell,
            false,
            SellTokenSource::Erc20,
            BuyTokenDestination::Erc20,
        ))
        .unwrap(),
        Amount::ZERO,
        "0x".to_owned(),
    );
    let decoded = decode_order(
        &trade,
        &[
            Address::new("0x1111111111111111111111111111111111111111").unwrap(),
            Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        ],
    )
    .unwrap();
    assert_eq!(
        decoded.sell_token.to_hex_string(),
        "0x1111111111111111111111111111111111111111"
    );
    assert_eq!(
        decoded.buy_token.to_hex_string(),
        "0x2222222222222222222222222222222222222222"
    );
    assert!(decode_order(&trade, &[]).is_err());
}

#[test]
fn order_refund_call_data_matches_the_canonical_abi_byte_layout() {
    // Manual reference encoding for `freeFilledAmountStorage(bytes[] orderUids)`
    // and `freePreSignatureStorage(bytes[] orderUids)` for a single 56-byte
    // orderUid argument. The layout is the canonical Solidity ABI:
    //
    //   4-byte function selector
    //   32-byte pointer to the head of `bytes[]`              (0x20)
    //   32-byte array length                                  (0x01)
    //   32-byte offset of the first element within the array  (0x20)
    //   32-byte length of the first element                   (56 = 0x38)
    //   56-byte payload, right-padded to 64 bytes
    let uid_bytes = [
        0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
        0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
        0x11, 0x11, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
        0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x00, 0x00, 0x00, 0x00,
    ];
    let uid_hex = format!("0x{}", alloy_primitives::hex::encode(uid_bytes));
    let uid = OrderUid::new(uid_hex).unwrap();

    let mut encoder = SettlementEncoder::new(sample_domain());
    encoder
        .encode_order_refunds(&OrderRefunds::new(vec![uid], vec![uid]))
        .unwrap();

    let post = encoder.interactions().unwrap()[InteractionStage::Post as usize].clone();
    assert_eq!(post.len(), 2);

    assert_eq!(
        post[0].call_data.as_ref(),
        expected_bytes_array_call_data("freeFilledAmountStorage(bytes[])", &uid_bytes).as_slice(),
        "freeFilledAmountStorage call-data must match the canonical Solidity ABI byte layout",
    );
    assert_eq!(
        post[1].call_data.as_ref(),
        expected_bytes_array_call_data("freePreSignatureStorage(bytes[])", &uid_bytes).as_slice(),
        "freePreSignatureStorage call-data must match the canonical Solidity ABI byte layout",
    );
}

#[test]
fn encoded_settlement_calldata_starts_with_the_settle_selector() {
    let domain = sample_domain();
    let mut encoder = SettlementEncoder::new(domain);

    let prices = serde_json::from_value::<cow_sdk_contracts::Prices>(serde_json::json!({
        "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2": "1000000000000000000",
        "0x6b175474e89094c44da98b954eedeac495271d0f": "500000000000000",
    }))
    .unwrap();
    encoder
        .encode_trade(
            &sample_order(OrderKind::Sell, false),
            &sample_presign(),
            Some(TradeExecution::new(
                Amount::new("1000000000000000000").unwrap(),
            )),
        )
        .unwrap();
    encoder
        .encode_interaction(
            &InteractionLike::new(
                Address::new("0xdef1c0ded9bec7f1a1670819833240f027b25eff").unwrap(),
                None,
                Some(bytes_from_hex_literal("0xdeadbeef")),
            ),
            InteractionStage::Intra,
        )
        .unwrap();

    let calldata = encoder.encoded_settlement_calldata(&prices).unwrap();

    let expected_selector = {
        let signature = "settle(address[],uint256[],(uint256,uint256,address,uint256,uint256,uint32,bytes32,uint256,uint256,uint256,bytes)[],(address,uint256,bytes)[][3])";
        let digest = Keccak256::digest(signature.as_bytes());
        [digest[0], digest[1], digest[2], digest[3]]
    };
    assert_eq!(
        &calldata[..4],
        expected_selector,
        "encoded_settlement_calldata must start with the GPv2Settlement settle(...) selector",
    );
    assert!(
        calldata.len() > 4,
        "encoded settle calldata must carry ABI arguments beyond the selector",
    );
}

#[test]
fn settlement_encoder_stage_order_pre_intra_post() {
    let mut encoder = SettlementEncoder::new(sample_domain());

    for (target, stage) in [
        (
            "0x3333333333333333333333333333333333333333",
            InteractionStage::Post,
        ),
        (
            "0x1111111111111111111111111111111111111111",
            InteractionStage::Pre,
        ),
        (
            "0x2222222222222222222222222222222222222222",
            InteractionStage::Intra,
        ),
    ] {
        encoder
            .encode_interaction(
                &InteractionLike::new(Address::new(target).unwrap(), None, None),
                stage,
            )
            .unwrap();
    }

    let grouped = encoder.interactions().unwrap();
    assert_eq!(
        grouped[InteractionStage::Pre as usize][0]
            .target
            .to_hex_string(),
        "0x1111111111111111111111111111111111111111"
    );
    assert_eq!(
        grouped[InteractionStage::Intra as usize][0]
            .target
            .to_hex_string(),
        "0x2222222222222222222222222222222222222222"
    );
    assert_eq!(
        grouped[InteractionStage::Post as usize][0]
            .target
            .to_hex_string(),
        "0x3333333333333333333333333333333333333333"
    );
}
