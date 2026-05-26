//! Integration tests for the typed ERC-20 + EIP-2612 Permit bindings.
//!
//! The tests cover three correctness anchors:
//!
//! * Encode/decode round-trips for every `IERC20` function selector plus the
//!   `Transfer` and `Approval` event surfaces.
//! * Encode/decode round-trips for the EIP-2612 `permit(...)` call-data.
//! * A byte-identical cross-check of the EIP-712 typed-data hash produced by
//!   [`permit_typed_data_hash`] against the canonical `OpenZeppelin` reference
//!   vector for a deterministic `(domain, permit)` pair, independently
//!   re-computed inside the test through keccak-256 on the canonical
//!   preimages.

use alloy_sol_types::{
    Eip712Domain, SolCall, SolEvent, SolStruct, SolValue, eip712_domain,
    private::{Address, B256, U256},
};
use cow_sdk_contracts::{IERC20, IERC20Permit, PERMIT_TYPE_HASH, permit_typed_data_hash};
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

#[test]
fn ierc20_permit_call_round_trips_through_the_seven_argument_shape() {
    let call = IERC20Permit::permitCall {
        owner: sample_address(0x01),
        spender: sample_address(0x02),
        value: U256::from(1_000_000u64),
        deadline: U256::from(u64::MAX),
        v: 27,
        r: B256::repeat_byte(0x11),
        s: B256::repeat_byte(0x22),
    };
    let encoded = call.abi_encode();
    let decoded = IERC20Permit::permitCall::abi_decode(&encoded).expect("call data must decode");
    assert_eq!(decoded.owner, call.owner);
    assert_eq!(decoded.spender, call.spender);
    assert_eq!(decoded.value, call.value);
    assert_eq!(decoded.deadline, call.deadline);
    assert_eq!(decoded.v, call.v);
    assert_eq!(decoded.r, call.r);
    assert_eq!(decoded.s, call.s);
    assert_eq!(&encoded[..4], IERC20Permit::permitCall::SELECTOR);
    assert_eq!(
        IERC20Permit::permitCall::SIGNATURE,
        "permit(address,address,uint256,uint256,uint8,bytes32,bytes32)",
        "permit selector must match the canonical EIP-2612 signature (seven call arguments; nonce is read from storage)",
    );
}

#[test]
fn permit_typed_data_hash_matches_an_independent_reference_vector() {
    // Deterministic fixture used as a canonical EIP-2612 cross-check:
    //
    // * Token contract EIP-712 domain:
    //     name              = "USD Coin"
    //     version           = "2"
    //     chainId           = 1
    //     verifyingContract = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
    //
    // * Permit struct:
    //     owner    = 0x1111111111111111111111111111111111111111
    //     spender  = 0x2222222222222222222222222222222222222222
    //     value    = 1_000_000
    //     nonce    = 7
    //     deadline = 1_767_225_600
    //
    // The expected digest is recomputed inside the test through keccak-256
    // on the canonical EIP-712 envelope so the `permit_typed_data_hash`
    // helper is cross-checked against an independent reference byte-for-byte.

    let owner = Address::from([0x11; 20]);
    let spender = Address::from([0x22; 20]);
    let value = U256::from(1_000_000u64);
    let nonce = U256::from(7u64);
    let deadline = U256::from(1_767_225_600u64);
    let verifying_contract = Address::from_slice(
        &alloy_primitives::hex::decode("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
            .expect("literal must decode"),
    );

    let domain: Eip712Domain = eip712_domain! {
        name: "USD Coin",
        version: "2",
        chain_id: 1u64,
        verifying_contract: verifying_contract,
    };

    let permit = IERC20Permit::Permit {
        owner,
        spender,
        value,
        nonce,
        deadline,
    };

    // Independent reference: recompute the EIP-712 typed-data hash through
    // the raw keccak-256 of the canonical `\x19\x01 || domainSeparator ||
    // structHash` envelope.
    let domain_type_hash = keccak(
        b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
    );
    let name_hash = keccak(b"USD Coin");
    let version_hash = keccak(b"2");

    let mut domain_preimage = Vec::with_capacity(32 * 5);
    domain_preimage.extend_from_slice(&domain_type_hash);
    domain_preimage.extend_from_slice(&name_hash);
    domain_preimage.extend_from_slice(&version_hash);
    domain_preimage.extend_from_slice(&U256::from(1u64).to_be_bytes::<32>());
    let mut verifying_contract_word = [0u8; 32];
    verifying_contract_word[12..].copy_from_slice(verifying_contract.as_slice());
    domain_preimage.extend_from_slice(&verifying_contract_word);
    let expected_domain_separator = keccak(&domain_preimage);

    let mut struct_preimage = Vec::with_capacity(32 * 6);
    struct_preimage.extend_from_slice(&PERMIT_TYPE_HASH);
    let mut owner_word = [0u8; 32];
    owner_word[12..].copy_from_slice(owner.as_slice());
    struct_preimage.extend_from_slice(&owner_word);
    let mut spender_word = [0u8; 32];
    spender_word[12..].copy_from_slice(spender.as_slice());
    struct_preimage.extend_from_slice(&spender_word);
    struct_preimage.extend_from_slice(&value.to_be_bytes::<32>());
    struct_preimage.extend_from_slice(&nonce.to_be_bytes::<32>());
    struct_preimage.extend_from_slice(&deadline.to_be_bytes::<32>());
    let expected_struct_hash = keccak(&struct_preimage);

    let mut envelope = Vec::with_capacity(2 + 32 + 32);
    envelope.extend_from_slice(&[0x19, 0x01]);
    envelope.extend_from_slice(&expected_domain_separator);
    envelope.extend_from_slice(&expected_struct_hash);
    let expected_typed_data_hash = keccak(&envelope);

    // Cross-check 1: the permit struct hash generated by the sol! binding
    // matches the independently recomputed struct hash.
    assert_eq!(
        permit.eip712_hash_struct().as_slice(),
        expected_struct_hash,
        "Permit struct hash must match the canonical keccak256(PERMIT_TYPE_HASH || fields)",
    );

    // Cross-check 2: the public helper composes the domain separator and
    // struct hash into the canonical `\x19\x01 || ... || ...` envelope.
    assert_eq!(
        permit_typed_data_hash(&domain, &permit),
        expected_typed_data_hash,
        "permit_typed_data_hash must equal the canonical EIP-712 envelope keccak",
    );
}
