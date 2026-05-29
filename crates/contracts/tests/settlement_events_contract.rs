//! Integration tests for the `GPv2Settlement` event decoder.
//!
//! Correctness anchors:
//!
//! * Event topic-0 byte-locks cross-checked against an independent keccak-256 of
//!   the canonical event signature.
//! * A full encode -> decode round-trip for each of the five settlement events.
//! * Fail-closed rejection of unknown topics, wrong indexed arity, and an
//!   order-UID byte length other than 56.

use alloy_primitives::{Bytes, FixedBytes, LogData, U256, b256, keccak256};
use alloy_sol_types::SolEvent;
use cow_sdk_contracts::{ContractsError, IGPv2SettlementEvents, SettlementEvent, decode_settlement_log};
use cow_sdk_core::{Address, Amount, OrderUid};

fn evm(byte: u8) -> alloy_primitives::Address {
    alloy_primitives::Address::from([byte; 20])
}

fn uid_bytes() -> Bytes {
    Bytes::from(vec![0x5a_u8; 56])
}

#[test]
fn settlement_event_topic0_byte_locks_match_canonical_keccak() {
    // (literal lock, canonical signature) for each event.
    let cases: [(FixedBytes<32>, &str); 5] = [
        (
            b256!("0xa07a543ab8a018198e99ca0184c93fe9050a79400a0a723441f84de1d972cc17"),
            "Trade(address,address,address,uint256,uint256,uint256,bytes)",
        ),
        (
            b256!("0xed99827efb37016f2275f98c4bcf71c7551c75d59e9b450f79fa32e60be672c2"),
            "Interaction(address,uint256,bytes4)",
        ),
        (
            b256!("0x40338ce1a7c49204f0099533b1e9a7ee0a3d261f84974ab7af36105b8c4e9db4"),
            "Settlement(address)",
        ),
        (
            b256!("0x875b6cb035bbd4ac6500fabc6d1e4ca5bdc58a3e2b424ccb5c24cdbebeb009a9"),
            "OrderInvalidated(address,bytes)",
        ),
        (
            b256!("0x01bf7c8b0ca55deecbea89d7e58295b7ffbf685fd0d96801034ba8c6ffe1c68d"),
            "PreSignature(address,bytes,bool)",
        ),
    ];
    let generated: [FixedBytes<32>; 5] = [
        IGPv2SettlementEvents::Trade::SIGNATURE_HASH,
        IGPv2SettlementEvents::Interaction::SIGNATURE_HASH,
        IGPv2SettlementEvents::Settlement::SIGNATURE_HASH,
        IGPv2SettlementEvents::OrderInvalidated::SIGNATURE_HASH,
        IGPv2SettlementEvents::PreSignature::SIGNATURE_HASH,
    ];
    for (gen_hash, (literal, signature)) in generated.into_iter().zip(cases) {
        assert_eq!(gen_hash, literal, "topic0 must equal the locked literal");
        assert_eq!(
            gen_hash,
            keccak256(signature.as_bytes()),
            "topic0 must equal keccak256 of the canonical signature `{signature}`",
        );
    }
}

#[test]
fn trade_round_trips() {
    let log = IGPv2SettlementEvents::Trade {
        owner: evm(0x11),
        sellToken: evm(0xc0),
        buyToken: evm(0xab),
        sellAmount: U256::from(1_000_000_000_000_000_000_u128),
        buyAmount: U256::from(3_000_000_000_u128),
        feeAmount: U256::from(7_000_u128),
        orderUid: uid_bytes(),
    }
    .encode_log_data();

    match decode_settlement_log(&log).expect("decode Trade") {
        SettlementEvent::Trade {
            owner,
            sell_token,
            buy_token,
            sell_amount,
            buy_amount,
            fee_amount,
            order_uid,
        } => {
            assert_eq!(owner, Address::from_bytes([0x11; 20]));
            assert_eq!(sell_token, Address::from_bytes([0xc0; 20]));
            assert_eq!(buy_token, Address::from_bytes([0xab; 20]));
            assert_eq!(sell_amount, Amount::from(1_000_000_000_000_000_000_u128));
            assert_eq!(buy_amount, Amount::from(3_000_000_000_u128));
            assert_eq!(fee_amount, Amount::from(7_000_u128));
            assert_eq!(order_uid, OrderUid::from_bytes([0x5a; 56]));
        }
        other => panic!("expected Trade, got {other:?}"),
    }
}

#[test]
fn interaction_round_trips_including_bytes4_selector() {
    let log = IGPv2SettlementEvents::Interaction {
        target: evm(0x33),
        value: U256::from(42_u64),
        selector: FixedBytes::<4>::from([0xde, 0xad, 0xbe, 0xef]),
    }
    .encode_log_data();

    match decode_settlement_log(&log).expect("decode Interaction") {
        SettlementEvent::Interaction { target, value, selector } => {
            assert_eq!(target, Address::from_bytes([0x33; 20]));
            assert_eq!(value, Amount::from(42_u64));
            assert_eq!(selector, [0xde, 0xad, 0xbe, 0xef]);
        }
        other => panic!("expected Interaction, got {other:?}"),
    }
}

#[test]
fn settlement_invalidated_and_presignature_round_trip() {
    let settlement = IGPv2SettlementEvents::Settlement { solver: evm(0x09) }.encode_log_data();
    assert!(matches!(
        decode_settlement_log(&settlement).expect("decode Settlement"),
        SettlementEvent::Settlement { solver } if solver == Address::from_bytes([0x09; 20])
    ));

    let invalidated = IGPv2SettlementEvents::OrderInvalidated {
        owner: evm(0x11),
        orderUid: uid_bytes(),
    }
    .encode_log_data();
    assert!(matches!(
        decode_settlement_log(&invalidated).expect("decode OrderInvalidated"),
        SettlementEvent::OrderInvalidated { owner, order_uid }
            if owner == Address::from_bytes([0x11; 20]) && order_uid == OrderUid::from_bytes([0x5a; 56])
    ));

    let presignature = IGPv2SettlementEvents::PreSignature {
        owner: evm(0x11),
        orderUid: uid_bytes(),
        signed: true,
    }
    .encode_log_data();
    assert!(matches!(
        decode_settlement_log(&presignature).expect("decode PreSignature"),
        SettlementEvent::PreSignature { signed: true, .. }
    ));
}

#[test]
fn unknown_topic0_is_rejected() {
    let log = LogData::new_unchecked(vec![FixedBytes::<32>::ZERO], Bytes::new());
    assert!(matches!(
        decode_settlement_log(&log),
        Err(ContractsError::UnexpectedEventTopics { event: "settlement" })
    ));
}

#[test]
fn missing_indexed_topic_is_rejected() {
    let log = IGPv2SettlementEvents::Trade {
        owner: evm(0x11),
        sellToken: evm(0xc0),
        buyToken: evm(0xab),
        sellAmount: U256::ZERO,
        buyAmount: U256::ZERO,
        feeAmount: U256::ZERO,
        orderUid: uid_bytes(),
    }
    .encode_log_data();
    // Drop the indexed `owner` topic -> only topic0 remains.
    let broken = LogData::new_unchecked(vec![log.topics()[0]], log.data.clone());
    assert!(matches!(
        decode_settlement_log(&broken),
        Err(ContractsError::UnexpectedEventTopics { event: "Trade" })
    ));
}

#[test]
fn wrong_order_uid_length_is_rejected() {
    let log = IGPv2SettlementEvents::OrderInvalidated {
        owner: evm(0x11),
        orderUid: Bytes::from(vec![0x5a_u8; 55]), // one byte short of 56
    }
    .encode_log_data();
    assert!(matches!(
        decode_settlement_log(&log),
        Err(ContractsError::InvalidOrderUidLength { actual: 55 })
    ));
}
