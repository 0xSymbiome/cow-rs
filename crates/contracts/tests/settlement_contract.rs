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

mod common;

use sha3::{Digest, Keccak256};

use bytes::Bytes;
use cow_sdk_contracts::{
    InteractionLike, InteractionStage, Order, OrderFlags, OrderRefunds, SettlementEncoder,
    Signature, SigningScheme, TokenRegistry, Trade, TradeExecution, TradeFlags, decode_order,
    decode_order_flags, decode_trade_flags, encode_order_flags, encode_trade_flags,
};
use cow_sdk_core::{
    Address, Amount, AppDataHex, BuyTokenDestination, OrderKind, OrderUid, SellTokenSource,
    TypedDataDomain,
};

use common::fixture_case;

fn expected_u8(value: &serde_json::Value) -> u8 {
    u8::try_from(value.as_u64().unwrap()).expect("fixture flag value must fit in u8")
}

fn sample_domain() -> TypedDataDomain {
    TypedDataDomain {
        name: "Gnosis Protocol".to_owned(),
        version: "v2".to_owned(),
        chain_id: 1,
        verifying_contract: Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap(),
    }
}

fn sample_order(kind: OrderKind, partially_fillable: bool) -> Order {
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
        partially_fillable,
        sell_token_balance: Some(SellTokenSource::Internal),
        buy_token_balance: Some(BuyTokenDestination::Internal),
    }
}

fn sample_signature() -> Signature {
    Signature::PreSign {
        owner: Address::new("0x1111111111111111111111111111111111111111").unwrap(),
    }
}

fn selector(signature: &str) -> String {
    let digest = Keccak256::digest(signature.as_bytes());
    format!("0x{}", hex::encode(&digest[..4]))
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
fn settlement_flag_encoding_matches_fixture_values() {
    let default_flags = fixture_case("contracts-order-flags-default-sell");
    assert_eq!(
        encode_order_flags(&OrderFlags {
            kind: OrderKind::Sell,
            partially_fillable: false,
            sell_token_balance: SellTokenSource::Erc20,
            buy_token_balance: BuyTokenDestination::Erc20,
        })
        .unwrap(),
        expected_u8(&default_flags["expected"]["encoded_flags"])
    );

    let buy_partial_internal = fixture_case("contracts-order-flags-buy-partial-internal");
    let encoded_buy_partial = encode_order_flags(&OrderFlags {
        kind: OrderKind::Buy,
        partially_fillable: true,
        sell_token_balance: SellTokenSource::Internal,
        buy_token_balance: BuyTokenDestination::Internal,
    })
    .unwrap();
    assert_eq!(
        encoded_buy_partial,
        expected_u8(&buy_partial_internal["expected"]["encoded_flags"])
    );

    let presign = fixture_case("contracts-trade-flags-presign");
    let encoded_trade = encode_trade_flags(&TradeFlags {
        kind: OrderKind::Sell,
        partially_fillable: false,
        sell_token_balance: SellTokenSource::Erc20,
        buy_token_balance: BuyTokenDestination::Erc20,
        signing_scheme: SigningScheme::PreSign,
    })
    .unwrap();
    assert_eq!(
        encoded_trade,
        expected_u8(&presign["expected"]["encoded_flags"])
    );

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
    let order_flags = OrderFlags {
        kind: OrderKind::Buy,
        partially_fillable: true,
        sell_token_balance: SellTokenSource::Internal,
        buy_token_balance: BuyTokenDestination::Internal,
    };
    let encoded_order = encode_order_flags(&order_flags).unwrap();
    assert_eq!(encoded_order & 0b1110_0000, 0);

    for signing_scheme in [
        SigningScheme::Eip712,
        SigningScheme::EthSign,
        SigningScheme::Eip1271,
        SigningScheme::PreSign,
    ] {
        let encoded_trade = encode_trade_flags(&TradeFlags {
            kind: order_flags.kind,
            partially_fillable: order_flags.partially_fillable,
            sell_token_balance: order_flags.sell_token_balance,
            buy_token_balance: order_flags.buy_token_balance,
            signing_scheme,
        })
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

    encoder.encode_interaction(
        &InteractionLike {
            target: Address::new("0xdef1c0ded9bec7f1a1670819833240f027b25eff").unwrap(),
            value: None,
            call_data: Some(bytes_from_hex_literal("0x12345678")),
        },
        InteractionStage::Pre,
    );
    encoder
        .encode_trade(
            &order,
            &sample_signature(),
            Some(TradeExecution {
                executed_amount: Amount::new("1000000000000000000").unwrap(),
            }),
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
        InteractionLike {
            target: Address::new("0x1234567890123456789012345678901234567890").unwrap(),
            value: None,
            call_data: Some(bytes_from_hex_literal("0x87654321")),
        },
        InteractionLike {
            target: Address::new("0xabcdef0123456789abcdef0123456789abcdef01").unwrap(),
            value: Some(Amount::new("1").unwrap()),
            call_data: None,
        },
    ]);
    assert!(setup.0.is_empty());
    assert!(setup.1.is_empty());
    assert!(setup.2.is_empty());
    assert!(setup.3[InteractionStage::Pre as usize].is_empty());
    assert_eq!(setup.3[InteractionStage::Intra as usize].len(), 2);
    assert!(setup.3[InteractionStage::Post as usize].is_empty());
}

#[test]
fn order_refunds_and_trade_decoding_follow_contract_rules() {
    let fixture = fixture_case("contracts-order-refund-method-names");
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
        .encode_order_refunds(&OrderRefunds {
            filled_amounts: vec![uids[0].clone()],
            pre_signatures: vec![uids[1].clone()],
        })
        .unwrap();

    let post = encoder.interactions().unwrap()[InteractionStage::Post as usize].clone();
    assert_eq!(post.len(), 2);
    assert_eq!(post[0].target, domain.verifying_contract);
    assert!(
        hex_prefixed(&post[0].call_data).starts_with(&selector(&format!(
            "{}(bytes[])",
            fixture["expected"]["methods"][0].as_str().unwrap()
        )))
    );
    assert!(
        hex_prefixed(&post[1].call_data).starts_with(&selector(&format!(
            "{}(bytes[])",
            fixture["expected"]["methods"][1].as_str().unwrap()
        )))
    );
    let invalid = serde_json::from_value::<OrderRefunds>(serde_json::json!({
        "filledAmounts": ["0x1234"],
        "preSignatures": []
    }));
    assert!(invalid.is_err());

    let partially_fillable = sample_order(OrderKind::Buy, true);
    assert!(
        SettlementEncoder::new(sample_domain())
            .encode_trade(&partially_fillable, &sample_signature(), None)
            .is_err()
    );

    let mut tokens = TokenRegistry::new();
    let first = tokens.index(&Address::new("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap());
    let second = tokens.index(&Address::new("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap());
    assert_eq!(first, second);

    let trade = Trade {
        sell_token_index: 0,
        buy_token_index: 1,
        receiver: Address::new("0x3333333333333333333333333333333333333333").unwrap(),
        sell_amount: Amount::new("10").unwrap(),
        buy_amount: Amount::new("20").unwrap(),
        valid_to: 123,
        app_data: AppDataHex::new(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .unwrap(),
        fee_amount: Amount::new("1").unwrap(),
        flags: encode_order_flags(&OrderFlags {
            kind: OrderKind::Sell,
            partially_fillable: false,
            sell_token_balance: SellTokenSource::Erc20,
            buy_token_balance: BuyTokenDestination::Erc20,
        })
        .unwrap(),
        executed_amount: Amount::zero(),
        signature: "0x".to_owned(),
    };
    let decoded = decode_order(
        &trade,
        &[
            Address::new("0x1111111111111111111111111111111111111111").unwrap(),
            Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        ],
    )
    .unwrap();
    assert_eq!(
        decoded.sell_token.as_str(),
        "0x1111111111111111111111111111111111111111"
    );
    assert_eq!(
        decoded.buy_token.as_str(),
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
    let uid_hex = format!("0x{}", hex::encode(uid_bytes));
    let uid = OrderUid::new(uid_hex).unwrap();

    let mut encoder = SettlementEncoder::new(sample_domain());
    encoder
        .encode_order_refunds(&OrderRefunds {
            filled_amounts: vec![uid.clone()],
            pre_signatures: vec![uid.clone()],
        })
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
            &sample_signature(),
            Some(TradeExecution {
                executed_amount: Amount::new("1000000000000000000").unwrap(),
            }),
        )
        .unwrap();
    encoder.encode_interaction(
        &InteractionLike {
            target: Address::new("0xdef1c0ded9bec7f1a1670819833240f027b25eff").unwrap(),
            value: None,
            call_data: Some(bytes_from_hex_literal("0xdeadbeef")),
        },
        InteractionStage::Intra,
    );

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
