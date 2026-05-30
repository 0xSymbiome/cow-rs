//! Integration tests for the `CoWSwapOnchainOrders` event decoder.
//!
//! Correctness anchors:
//!
//! * Event topic-0 byte-locks cross-checked against an independent keccak-256 of
//!   the canonical (flattened-tuple) event signature.
//! * A capstone EIP-712 order-hash vector reproduced bit-for-bit from the
//!   upstream `ethflowcontract` foundry suite, underpinning UID derivation.
//! * A full `OrderPlacement` encode -> decode -> owner -> UID round-trip for both
//!   on-chain signing schemes.
//! * Fail-closed rejection of malformed topics, signing schemes, signature
//!   payloads, and UID lengths.

use alloy_primitives::{B256, Bytes, LogData, U256, b256, keccak256};
use alloy_sol_types::SolEvent;
use cow_sdk_contracts::{
    ContractsError, ICoWSwapOnchainOrders, OnchainSigningScheme, compute_order_uid,
    decode_order_invalidation, decode_order_placement, hash_order, parse_eth_flow_onchain_data,
};
use cow_sdk_core::{
    Address, Amount, AppDataHash, BuyTokenDestination, OrderData, OrderKind, OrderUid,
    SellTokenSource, TypedDataDomain,
};
use sha3::{Digest, Keccak256};

fn evm(byte: u8) -> alloy_primitives::Address {
    alloy_primitives::Address::from([byte; 20])
}

fn marker(label: &[u8]) -> B256 {
    keccak256(label)
}

fn keccak32(preimage: &[u8]) -> [u8; 32] {
    Keccak256::digest(preimage).into()
}

const ORDER_PLACEMENT_SIGNATURE: &str = "OrderPlacement(address,(address,address,address,uint256,uint256,uint32,bytes32,uint256,bytes32,bool,bytes32,bytes32),(uint8,bytes),bytes)";

fn sample_order_data() -> ICoWSwapOnchainOrders::GPv2OrderData {
    ICoWSwapOnchainOrders::GPv2OrderData {
        sellToken: evm(0xc0),
        buyToken: evm(0xab),
        receiver: evm(0x11),
        sellAmount: U256::from(1_000_000_000_000_000_000_u128),
        buyAmount: U256::from(3_000_000_000_u128),
        validTo: 0xffff_ffff,
        appData: B256::repeat_byte(0xaa),
        feeAmount: U256::ZERO,
        kind: marker(b"sell"),
        partiallyFillable: false,
        sellTokenBalance: marker(b"erc20"),
        buyTokenBalance: marker(b"erc20"),
    }
}

fn placement_log(scheme: u8, signature_data: Bytes, sender: u8, data: Bytes) -> LogData {
    ICoWSwapOnchainOrders::OrderPlacement {
        sender: evm(sender),
        order: sample_order_data(),
        signature: ICoWSwapOnchainOrders::OnchainSignature {
            scheme,
            data: signature_data,
        },
        data,
    }
    .encode_log_data()
}

fn eth_flow_trailer(quote_id: i64, user_valid_to: u32) -> Bytes {
    let mut trailer = Vec::with_capacity(12);
    trailer.extend_from_slice(&quote_id.to_be_bytes());
    trailer.extend_from_slice(&user_valid_to.to_be_bytes());
    Bytes::from(trailer)
}

#[test]
fn order_placement_topic0_matches_canonical_hash() {
    assert_eq!(
        ICoWSwapOnchainOrders::OrderPlacement::SIGNATURE_HASH,
        b256!("0xcf5f9de2984132265203b5c335b25727702ca77262ff622e136baa7362bf1da9"),
    );
    assert_eq!(
        ICoWSwapOnchainOrders::OrderPlacement::SIGNATURE_HASH.as_slice(),
        keccak32(ORDER_PLACEMENT_SIGNATURE.as_bytes()),
        "OrderPlacement topic0 must equal keccak256 of the flattened-tuple signature",
    );
    assert_eq!(
        ICoWSwapOnchainOrders::OrderPlacement::SIGNATURE,
        ORDER_PLACEMENT_SIGNATURE,
    );
}

#[test]
fn order_invalidation_topic0_matches_canonical_hash() {
    assert_eq!(
        ICoWSwapOnchainOrders::OrderInvalidation::SIGNATURE_HASH,
        b256!("0xb8bad102ac8bbacfef31ff1c906ec6d951c230b4dce750bb0376b812ad35852a"),
    );
    assert_eq!(
        ICoWSwapOnchainOrders::OrderInvalidation::SIGNATURE_HASH.as_slice(),
        keccak32(b"OrderInvalidation(bytes)"),
    );
}

#[test]
fn order_hash_matches_canonical_ethflow_foundry_vector() {
    // Vector from cowprotocol/ethflowcontract test/CoWSwapOnchainOrders.t.sol:
    // dummyOrder() under the chain-31337 settlement domain separator.
    let settlement = Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap();
    let domain = TypedDataDomain::new(
        "Gnosis Protocol".to_owned(),
        "v2".to_owned(),
        31337,
        settlement,
    );
    let order = OrderData::new(
        Address::from_bytes([0x01; 20]),
        Address::from_bytes([0x02; 20]),
        Address::from_bytes([0x03; 20]),
        Amount::new("42000000000000000000").unwrap(),
        Amount::new("13370000000000000000").unwrap(),
        0xffff_ffff,
        AppDataHash::from_bytes([0u8; 32]),
        Amount::new("1000000000000000000").unwrap(),
        OrderKind::Sell,
        false,
        SellTokenSource::Erc20,
        BuyTokenDestination::Erc20,
    );

    let digest = hash_order(&domain, &order).expect("hashing must succeed");
    assert_eq!(
        digest.to_hex_string(),
        "0x65e25f4dac20ef9e411ba2e6a5c6c2697ce004564ffeeb5fe8a3d9f6529974f5",
    );
}

#[test]
fn eip1271_placement_decodes_owner_uid_and_trailer() {
    let eth_flow = Address::new("0x40A50cf069e992AA4536211B23F286eF88752187").unwrap();
    let signature_data = Bytes::from(eth_flow.as_slice().to_vec());
    let log = placement_log(
        0,
        signature_data,
        0x11,
        eth_flow_trailer(1_234_567, 1_893_456_000),
    );

    let decoded = decode_order_placement(&log).expect("eth-flow placement must decode");
    assert_eq!(decoded.signing_scheme, OnchainSigningScheme::Eip1271);
    assert_eq!(decoded.sender, Address::from_bytes([0x11; 20]));
    assert_eq!(decoded.order.sell_token, Address::from_bytes([0xc0; 20]));
    assert_eq!(decoded.order.valid_to, 0xffff_ffff);
    assert_eq!(decoded.order.kind, OrderKind::Sell);
    assert_eq!(
        decoded.resolve_owner().unwrap(),
        eth_flow,
        "EIP-1271 owner is carried in the signature payload",
    );

    let settlement = Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap();
    let domain = TypedDataDomain::new("Gnosis Protocol".to_owned(), "v2".to_owned(), 1, settlement);
    let expected_uid = compute_order_uid(&domain, &decoded.order, &eth_flow).unwrap();
    assert_eq!(decoded.order_uid(&domain).unwrap(), expected_uid);

    let parsed = parse_eth_flow_onchain_data(decoded.data.as_ref()).unwrap();
    assert_eq!(parsed.quote_id, 1_234_567);
    assert_eq!(parsed.user_valid_to, 1_893_456_000);
}

#[test]
fn presign_placement_owner_is_sender() {
    let log = placement_log(1, Bytes::from(vec![0u8; 56]), 0x22, eth_flow_trailer(0, 0));
    let decoded = decode_order_placement(&log).expect("presign placement must decode");
    assert_eq!(decoded.signing_scheme, OnchainSigningScheme::PreSign);
    assert_eq!(
        decoded.resolve_owner().unwrap(),
        Address::from_bytes([0x22; 20])
    );
}

#[test]
fn invalid_signing_scheme_is_rejected() {
    let log = placement_log(2, Bytes::from(vec![0u8; 20]), 0x11, eth_flow_trailer(0, 0));
    let error = decode_order_placement(&log).expect_err("scheme 2 must be rejected");
    assert!(matches!(error, ContractsError::UnsupportedSigningScheme(2)));
}

#[test]
fn eip1271_owner_requires_twenty_byte_signature_payload() {
    let log = placement_log(0, Bytes::from(vec![0u8; 32]), 0x11, eth_flow_trailer(0, 0));
    let decoded = decode_order_placement(&log).expect("placement still decodes");
    let error = decoded
        .resolve_owner()
        .expect_err("non-20-byte payload must be rejected");
    assert!(matches!(
        error,
        ContractsError::InvalidDecodedLength {
            expected: 20,
            actual: 32,
            ..
        }
    ));
}

#[test]
fn wrong_topic_count_is_rejected() {
    let valid = placement_log(0, Bytes::from(vec![0u8; 20]), 0x11, eth_flow_trailer(0, 0));
    let bad = LogData::new(vec![valid.topics()[0]], valid.data.clone()).unwrap();
    let error = decode_order_placement(&bad).expect_err("missing indexed topic must be rejected");
    assert!(matches!(
        error,
        ContractsError::UnexpectedEventTopics { .. }
    ));
}

#[test]
fn wrong_topic0_is_rejected() {
    let valid = placement_log(0, Bytes::from(vec![0u8; 20]), 0x11, eth_flow_trailer(0, 0));
    let bad = LogData::new(
        vec![
            ICoWSwapOnchainOrders::OrderInvalidation::SIGNATURE_HASH,
            valid.topics()[1],
        ],
        valid.data.clone(),
    )
    .unwrap();
    let error = decode_order_placement(&bad).expect_err("topic0 mismatch must be rejected");
    assert!(matches!(
        error,
        ContractsError::UnexpectedEventTopics { .. }
    ));
}

#[test]
fn order_invalidation_decodes_fifty_six_byte_uid() {
    let log = ICoWSwapOnchainOrders::OrderInvalidation {
        orderUid: Bytes::from(vec![0x09u8; 56]),
    }
    .encode_log_data();
    let decoded = decode_order_invalidation(&log).expect("invalidation must decode");
    assert_eq!(decoded.order_uid, OrderUid::from_bytes([0x09u8; 56]));
}

#[test]
fn order_invalidation_rejects_wrong_uid_length() {
    let log = ICoWSwapOnchainOrders::OrderInvalidation {
        orderUid: Bytes::from(vec![0x09u8; 55]),
    }
    .encode_log_data();
    let error = decode_order_invalidation(&log).expect_err("55-byte UID must be rejected");
    assert!(matches!(
        error,
        ContractsError::InvalidOrderUidLength { actual: 55 }
    ));
}

#[test]
fn eth_flow_trailer_rejects_wrong_length() {
    let error =
        parse_eth_flow_onchain_data(&[0u8; 11]).expect_err("11-byte trailer must be rejected");
    assert!(matches!(
        error,
        ContractsError::InvalidDecodedLength {
            expected: 12,
            actual: 11,
            ..
        }
    ));
}

#[test]
fn eth_flow_trailer_handles_negative_quote_id() {
    let parsed = parse_eth_flow_onchain_data(eth_flow_trailer(-1, 7).as_ref()).unwrap();
    assert_eq!(parsed.quote_id, -1);
    assert_eq!(parsed.user_valid_to, 7);
}

#[test]
fn unknown_order_marker_is_rejected() {
    let mut order = sample_order_data();
    order.kind = B256::repeat_byte(0x01); // not keccak256("sell") / "buy"
    let log = ICoWSwapOnchainOrders::OrderPlacement {
        sender: evm(0x11),
        order,
        signature: ICoWSwapOnchainOrders::OnchainSignature {
            scheme: 1,
            data: Bytes::new(),
        },
        data: eth_flow_trailer(0, 0),
    }
    .encode_log_data();
    let error = decode_order_placement(&log).expect_err("unknown kind marker must be rejected");
    assert!(matches!(error, ContractsError::UnknownOrderMarker(_)));
}
