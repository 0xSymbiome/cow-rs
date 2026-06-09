//! Integration tests for the typed ERC-20 bindings.
//!
//! Encode/decode round-trips for every `IERC20` function selector plus the
//! `Transfer` and `Approval` event surfaces.

use alloy_sol_types::{
    SolCall, SolEvent, SolValue,
    private::{Address, B256, U256},
};
use cow_sdk_contracts::IERC20;
use sha3::{Digest, Keccak256};

fn sample_address(byte: u8) -> Address {
    Address::from([byte; 20])
}

fn keccak(value: &[u8]) -> [u8; 32] {
    Keccak256::digest(value).into()
}

#[test]
fn ierc20_balance_of_round_trips_call_data() {
    let call = IERC20::balanceOfCall {
        account: sample_address(0x11),
    };
    let encoded = call.abi_encode();
    let decoded = IERC20::balanceOfCall::abi_decode(&encoded).expect("call data must decode");
    assert_eq!(decoded.account, call.account);
    assert_eq!(&encoded[..4], IERC20::balanceOfCall::SELECTOR);
    assert_eq!(
        IERC20::balanceOfCall::SIGNATURE,
        "balanceOf(address)",
        "balanceOf selector must match the canonical ERC-20 signature",
    );
}

#[test]
fn ierc20_approve_round_trips_call_data() {
    let call = IERC20::approveCall {
        spender: sample_address(0x22),
        value: U256::from(1_000_000u64),
    };
    let encoded = call.abi_encode();
    let decoded = IERC20::approveCall::abi_decode(&encoded).expect("call data must decode");
    assert_eq!(decoded.spender, call.spender);
    assert_eq!(decoded.value, call.value);
    assert_eq!(&encoded[..4], IERC20::approveCall::SELECTOR);
    assert_eq!(IERC20::approveCall::SIGNATURE, "approve(address,uint256)");
}

#[test]
fn ierc20_allowance_round_trips_call_data() {
    let call = IERC20::allowanceCall {
        owner: sample_address(0x33),
        spender: sample_address(0x44),
    };
    let encoded = call.abi_encode();
    let decoded = IERC20::allowanceCall::abi_decode(&encoded).expect("call data must decode");
    assert_eq!(decoded.owner, call.owner);
    assert_eq!(decoded.spender, call.spender);
    assert_eq!(&encoded[..4], IERC20::allowanceCall::SELECTOR);
    assert_eq!(
        IERC20::allowanceCall::SIGNATURE,
        "allowance(address,address)"
    );
}

#[test]
fn ierc20_transfer_round_trips_call_data() {
    let call = IERC20::transferCall {
        to: sample_address(0x55),
        value: U256::from(42u64),
    };
    let encoded = call.abi_encode();
    let decoded = IERC20::transferCall::abi_decode(&encoded).expect("call data must decode");
    assert_eq!(decoded.to, call.to);
    assert_eq!(decoded.value, call.value);
    assert_eq!(&encoded[..4], IERC20::transferCall::SELECTOR);
    assert_eq!(IERC20::transferCall::SIGNATURE, "transfer(address,uint256)");
}

#[test]
fn ierc20_transfer_from_round_trips_call_data() {
    let call = IERC20::transferFromCall {
        from: sample_address(0x66),
        to: sample_address(0x77),
        value: U256::from(u128::MAX),
    };
    let encoded = call.abi_encode();
    let decoded = IERC20::transferFromCall::abi_decode(&encoded).expect("call data must decode");
    assert_eq!(decoded.from, call.from);
    assert_eq!(decoded.to, call.to);
    assert_eq!(decoded.value, call.value);
    assert_eq!(&encoded[..4], IERC20::transferFromCall::SELECTOR);
    assert_eq!(
        IERC20::transferFromCall::SIGNATURE,
        "transferFrom(address,address,uint256)"
    );
}

#[test]
fn ierc20_transfer_event_round_trips_through_the_generated_decoder() {
    let from = sample_address(0xaa);
    let to = sample_address(0xbb);
    let value = U256::from(7_777u64);

    let topics: [B256; 3] = [
        IERC20::Transfer::SIGNATURE_HASH,
        B256::left_padding_from(from.as_slice()),
        B256::left_padding_from(to.as_slice()),
    ];
    let data = value.abi_encode();

    let decoded =
        IERC20::Transfer::decode_raw_log(topics, &data).expect("Transfer event must decode");
    assert_eq!(decoded.from, from);
    assert_eq!(decoded.to, to);
    assert_eq!(decoded.value, value);

    let expected_topic_hash = keccak(b"Transfer(address,address,uint256)");
    assert_eq!(
        IERC20::Transfer::SIGNATURE_HASH.as_slice(),
        expected_topic_hash,
        "Transfer event topic0 must equal the canonical ERC-20 hash",
    );
}

#[test]
fn ierc20_approval_event_round_trips_through_the_generated_decoder() {
    let owner = sample_address(0xcc);
    let spender = sample_address(0xdd);
    let value = U256::from(1u64);

    let topics: [B256; 3] = [
        IERC20::Approval::SIGNATURE_HASH,
        B256::left_padding_from(owner.as_slice()),
        B256::left_padding_from(spender.as_slice()),
    ];
    let data = value.abi_encode();

    let decoded =
        IERC20::Approval::decode_raw_log(topics, &data).expect("Approval event must decode");
    assert_eq!(decoded.owner, owner);
    assert_eq!(decoded.spender, spender);
    assert_eq!(decoded.value, value);

    let expected_topic_hash = keccak(b"Approval(address,address,uint256)");
    assert_eq!(
        IERC20::Approval::SIGNATURE_HASH.as_slice(),
        expected_topic_hash,
        "Approval event topic0 must equal the canonical ERC-20 hash",
    );
}
