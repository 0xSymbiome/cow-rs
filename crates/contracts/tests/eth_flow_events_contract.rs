//! Integration tests for the eth-flow event decoders.
//!
//! Correctness anchors:
//!
//! * `OrderRefund` topic-0 byte-lock cross-checked against an independent
//!   keccak-256 of the canonical signature.
//! * A full `OrderRefund` encode -> decode round-trip.
//! * `decode_eth_flow_log` dispatch across `OrderPlacement`, `OrderInvalidation`,
//!   and `OrderRefund`.
//! * Fail-closed rejection of unknown topics, wrong indexed arity, and an
//!   order-UID byte length other than 56.

use alloy_primitives::{Bytes, FixedBytes, LogData, U256, b256, keccak256};
use alloy_sol_types::SolEvent;
use cow_sdk_contracts::{
    ContractsError, EthFlowEvent, ICoWSwapEthFlowEvents, ICoWSwapOnchainOrders,
    decode_eth_flow_log, decode_order_refund,
};
use cow_sdk_core::{Address, OrderUid};

fn evm(byte: u8) -> alloy_primitives::Address {
    alloy_primitives::Address::from([byte; 20])
}

fn marker(label: &[u8]) -> FixedBytes<32> {
    keccak256(label)
}

fn uid_bytes() -> Bytes {
    Bytes::from(vec![0x5a_u8; 56])
}

fn refund_log() -> LogData {
    ICoWSwapEthFlowEvents::OrderRefund {
        orderUid: uid_bytes(),
        refunder: evm(0x44),
    }
    .encode_log_data()
}

fn placement_log() -> LogData {
    ICoWSwapOnchainOrders::OrderPlacement {
        sender: evm(0x11),
        order: ICoWSwapOnchainOrders::GPv2OrderData {
            sellToken: evm(0xc0),
            buyToken: evm(0xab),
            receiver: evm(0x22),
            sellAmount: U256::from(1_000_000_000_000_000_000_u128),
            buyAmount: U256::from(3_000_000_000_u128),
            validTo: 0xffff_ffff,
            appData: FixedBytes::<32>::repeat_byte(0xaa),
            feeAmount: U256::ZERO,
            kind: marker(b"sell"),
            partiallyFillable: false,
            sellTokenBalance: marker(b"erc20"),
            buyTokenBalance: marker(b"erc20"),
        },
        signature: ICoWSwapOnchainOrders::OnchainSignature {
            scheme: 1, // PreSign
            data: Bytes::new(),
        },
        data: Bytes::from(vec![0u8; 12]),
    }
    .encode_log_data()
}

fn invalidation_log() -> LogData {
    ICoWSwapOnchainOrders::OrderInvalidation {
        orderUid: uid_bytes(),
    }
    .encode_log_data()
}

#[test]
fn order_refund_topic0_matches_canonical_hash() {
    assert_eq!(
        ICoWSwapEthFlowEvents::OrderRefund::SIGNATURE_HASH,
        b256!("0x195271068a288191e4b265c641a56b9832919f69e9e7d6c2f31ba40278aeb85a"),
    );
    assert_eq!(
        ICoWSwapEthFlowEvents::OrderRefund::SIGNATURE_HASH,
        keccak256(b"OrderRefund(bytes,address)"),
        "OrderRefund topic-0 must equal keccak256 of the canonical signature",
    );
}

#[test]
fn order_refund_round_trips() {
    let decoded = decode_order_refund(&refund_log()).expect("decode OrderRefund");
    assert_eq!(decoded.order_uid, OrderUid::from_bytes([0x5a; 56]));
    assert_eq!(decoded.refunder, Address::from_bytes([0x44; 20]));
}

#[test]
fn decode_eth_flow_log_dispatches_all_three_events() {
    assert!(matches!(
        decode_eth_flow_log(&placement_log()).expect("placement"),
        EthFlowEvent::OrderPlacement(_)
    ));
    assert!(matches!(
        decode_eth_flow_log(&invalidation_log()).expect("invalidation"),
        EthFlowEvent::OrderInvalidation(_)
    ));
    assert!(matches!(
        decode_eth_flow_log(&refund_log()).expect("refund"),
        EthFlowEvent::OrderRefund(refund) if refund.refunder == Address::from_bytes([0x44; 20])
    ));
}

#[test]
fn unknown_topic0_is_rejected() {
    let log = LogData::new_unchecked(vec![FixedBytes::<32>::ZERO], Bytes::new());
    assert!(matches!(
        decode_eth_flow_log(&log),
        Err(ContractsError::UnexpectedEventTopics { event: "eth-flow" })
    ));
}

#[test]
fn order_refund_missing_indexed_topic_is_rejected() {
    let log = refund_log();
    // Drop the indexed `refunder` topic -> only topic-0 remains.
    let broken = LogData::new_unchecked(vec![log.topics()[0]], log.data.clone());
    assert!(matches!(
        decode_order_refund(&broken),
        Err(ContractsError::UnexpectedEventTopics {
            event: "OrderRefund"
        })
    ));
}

#[test]
fn order_refund_wrong_uid_length_is_rejected() {
    let log = ICoWSwapEthFlowEvents::OrderRefund {
        orderUid: Bytes::from(vec![0x5a_u8; 55]), // one byte short of 56
        refunder: evm(0x44),
    }
    .encode_log_data();
    assert!(matches!(
        decode_order_refund(&log),
        Err(ContractsError::InvalidOrderUidLength { actual: 55 })
    ));
}
