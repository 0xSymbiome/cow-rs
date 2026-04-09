mod common;

use sha3::{Digest, Keccak256};

use cow_sdk_contracts::{
    InteractionLike, InteractionStage, Order, OrderFlags, OrderRefunds, SettlementEncoder,
    Signature, SigningScheme, TokenRegistry, Trade, TradeExecution, TradeFlags, decode_order,
    decode_order_flags, decode_trade_flags, encode_order_flags, encode_trade_flags,
};
use cow_sdk_core::{Address, AppDataHex, OrderBalance, OrderKind, OrderUid, TypedDataDomain};

use common::fixture_case;

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
        sell_amount: "1000000000000000000".to_owned(),
        buy_amount: "2000000000000000000000".to_owned(),
        valid_to: 1_709_990_000,
        app_data: AppDataHex::new(
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap(),
        fee_amount: "5000000000000000".to_owned(),
        kind,
        partially_fillable,
        sell_token_balance: Some(OrderBalance::Internal),
        buy_token_balance: Some(OrderBalance::External),
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

#[test]
fn settlement_flag_encoding_matches_fixture_values() {
    let default_flags = fixture_case("contracts-order-flags-default-sell");
    assert_eq!(
        encode_order_flags(&OrderFlags {
            kind: OrderKind::Sell,
            partially_fillable: false,
            sell_token_balance: OrderBalance::Erc20,
            buy_token_balance: OrderBalance::Erc20,
        })
        .unwrap(),
        default_flags["expected"]["encoded_flags"].as_u64().unwrap() as u8
    );

    let buy_partial_internal = fixture_case("contracts-order-flags-buy-partial-internal");
    let encoded_buy_partial = encode_order_flags(&OrderFlags {
        kind: OrderKind::Buy,
        partially_fillable: true,
        sell_token_balance: OrderBalance::Internal,
        buy_token_balance: OrderBalance::Internal,
    })
    .unwrap();
    assert_eq!(
        encoded_buy_partial,
        buy_partial_internal["expected"]["encoded_flags"]
            .as_u64()
            .unwrap() as u8
    );

    let presign = fixture_case("contracts-trade-flags-presign");
    let encoded_trade = encode_trade_flags(&TradeFlags {
        kind: OrderKind::Sell,
        partially_fillable: false,
        sell_token_balance: OrderBalance::Erc20,
        buy_token_balance: OrderBalance::Erc20,
        signing_scheme: SigningScheme::PreSign,
    })
    .unwrap();
    assert_eq!(
        encoded_trade,
        presign["expected"]["encoded_flags"].as_u64().unwrap() as u8
    );

    let decoded_order = decode_order_flags(encoded_buy_partial).unwrap();
    assert_eq!(decoded_order.kind, OrderKind::Buy);
    assert_eq!(decoded_order.sell_token_balance, OrderBalance::Internal);
    assert_eq!(decoded_order.buy_token_balance, OrderBalance::Internal);

    let decoded_trade = decode_trade_flags(encoded_trade).unwrap();
    assert_eq!(decoded_trade.signing_scheme, SigningScheme::PreSign);
    assert_eq!(
        decode_order_flags(0b0100).unwrap().sell_token_balance,
        OrderBalance::Erc20
    );
    assert!(decode_order_flags(1 << 7).is_err());
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
            call_data: Some("0x12345678".to_owned()),
        },
        InteractionStage::Pre,
    );
    encoder
        .encode_trade(
            &order,
            &sample_signature(),
            Some(TradeExecution {
                executed_amount: "1000000000000000000".to_owned(),
            }),
        )
        .unwrap();

    let prices =
        serde_json::from_value::<std::collections::BTreeMap<String, String>>(serde_json::json!({
            "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2": "1000000000000000000",
            "0x6b175474e89094c44da98b954eedeac495271d0f": "500000000000000",
        }))
        .unwrap();

    let encoded = encoder.encoded_settlement(&prices).unwrap();
    assert_eq!(encoded.0.len(), 2);
    assert_eq!(
        encoded.1,
        vec![
            "1000000000000000000".to_owned(),
            "500000000000000".to_owned()
        ]
    );
    assert_eq!(encoded.2.len(), 1);
    assert_eq!(encoded.3[InteractionStage::Pre as usize].len(), 1);
    assert_eq!(encoded.3[InteractionStage::Intra as usize].len(), 0);
    assert_eq!(encoded.3[InteractionStage::Post as usize].len(), 0);

    let missing = serde_json::from_value(serde_json::json!({
        "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2": "1000000000000000000"
    }))
    .unwrap();
    assert!(encoder.clearing_prices(&missing).is_err());

    let setup = SettlementEncoder::encoded_setup(&[
        InteractionLike {
            target: Address::new("0x1234567890123456789012345678901234567890").unwrap(),
            value: None,
            call_data: Some("0x87654321".to_owned()),
        },
        InteractionLike {
            target: Address::new("0xabcdef0123456789abcdef0123456789abcdef01").unwrap(),
            value: Some("1".to_owned()),
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
    assert!(post[0].call_data.starts_with(&selector(&format!(
        "{}(bytes[])",
        fixture["expected"]["methods"][0].as_str().unwrap()
    ))));
    assert!(post[1].call_data.starts_with(&selector(&format!(
        "{}(bytes[])",
        fixture["expected"]["methods"][1].as_str().unwrap()
    ))));
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
        sell_amount: "10".to_owned(),
        buy_amount: "20".to_owned(),
        valid_to: 123,
        app_data: AppDataHex::new(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .unwrap(),
        fee_amount: "1".to_owned(),
        flags: encode_order_flags(&OrderFlags {
            kind: OrderKind::Sell,
            partially_fillable: false,
            sell_token_balance: OrderBalance::Erc20,
            buy_token_balance: OrderBalance::Erc20,
        })
        .unwrap(),
        executed_amount: "0".to_owned(),
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
