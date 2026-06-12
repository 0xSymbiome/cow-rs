#![cfg(feature = "cow-shed")]

//! Contract: the `executeHooks` calldata encoders.
//! `encode_execute_hooks_calldata_signed` (typed 65-byte `RecoverableSignature`)
//! reproduces the reference factory calldata byte-for-byte and is a faithful
//! wrapper of the general `encode_execute_hooks_calldata_with_signature`, which
//! additionally encodes an EIP-1271 contract-signature blob the typed path
//! cannot represent — keeping the proxy's length-based on-chain dispatch
//! (ECDSA recovery for 65 bytes, `isValidSignature` otherwise) reachable for
//! both owner kinds. The reference vectors also decode and re-encode through
//! the `sol!` call types for both the factory and proxy entry points.

use alloy_primitives::Bytes;
use alloy_sol_types::SolCall;
use cow_sdk_contracts::RecoverableSignature;
use cow_sdk_contracts::cow_shed::bindings::{COWShed, COWShedFactory};
use cow_sdk_contracts::cow_shed::{
    Call, encode_execute_hooks_calldata_signed, encode_execute_hooks_calldata_with_signature,
};
use serde::Deserialize;

mod cow_shed_common;
use cow_shed_common::{address, b256, bytes, decimal_u256};

const FIXTURE: &str = include_str!("../../../parity/fixtures/cow_shed/execute_hooks_calldata.json");

#[derive(Deserialize)]
struct Fixture {
    rows: Vec<Row>,
}

#[derive(Deserialize)]
struct Row {
    name: String,
    calls: Vec<FixtureCall>,
    nonce: String,
    deadline: String,
    user: String,
    signature: String,
    factory_call_data: String,
    proxy_call_data: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FixtureCall {
    target: String,
    value: String,
    call_data: String,
    allow_failure: bool,
    is_delegate_call: bool,
}

#[test]
fn signed_calldata_matches_reference_vectors_and_round_trips() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("calldata fixture parses");
    assert!(!fixture.rows.is_empty(), "calldata fixture must carry rows");

    for row in &fixture.rows {
        let calls = row.calls.iter().map(to_call).collect::<Vec<_>>();
        let signature = RecoverableSignature::parse_bytes(&bytes(&row.signature))
            .unwrap_or_else(|err| panic!("row {}: signature parses: {err:?}", row.name));

        let encoded = encode_execute_hooks_calldata_signed(
            &calls,
            b256(&row.nonce),
            decimal_u256(&row.deadline),
            address(&row.user),
            &signature,
        );
        assert_eq!(
            encoded,
            bytes(&row.factory_call_data),
            "row {}: factory executeHooks calldata diverges from reference vector",
            row.name
        );

        let decoded = COWShedFactory::executeHooksCall::abi_decode(&encoded)
            .unwrap_or_else(|err| panic!("row {}: factory calldata decodes: {err}", row.name));
        assert_eq!(
            decoded.abi_encode(),
            encoded.as_ref(),
            "row {}: factory calldata round-trip",
            row.name
        );
    }
}

#[test]
fn proxy_execute_hooks_fixture_round_trips() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("calldata fixture parses");
    for row in &fixture.rows {
        let encoded = bytes(&row.proxy_call_data);
        let decoded = COWShed::executeHooksCall::abi_decode(&encoded)
            .unwrap_or_else(|err| panic!("row {}: proxy calldata decodes: {err}", row.name));
        assert_eq!(
            decoded.abi_encode(),
            encoded.as_ref(),
            "row {}: proxy calldata round-trip",
            row.name
        );
    }
}

#[test]
fn with_signature_covers_eoa_and_eip1271() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("calldata fixture parses");
    let row = fixture.rows.first().expect("fixture has at least one row");
    let calls = row.calls.iter().map(to_call).collect::<Vec<_>>();
    let nonce = b256(&row.nonce);
    let deadline = decimal_u256(&row.deadline);
    let user = address(&row.user);

    // EOA: the typed `_signed` path is exactly the general encoder fed the same
    // 65-byte signature, so the typed entry point is a faithful wrapper.
    let signature = RecoverableSignature::parse_bytes(&bytes(&row.signature)).expect("65-byte sig");
    let via_signed =
        encode_execute_hooks_calldata_signed(&calls, nonce, deadline, user, &signature);
    let via_general = encode_execute_hooks_calldata_with_signature(
        &calls,
        nonce,
        deadline,
        user,
        signature.to_bytes().to_vec(),
    );
    assert_eq!(
        via_signed, via_general,
        "EOA path: general encoder must equal the typed wrapper"
    );

    // EIP-1271: a non-65-byte contract-signature blob — which the EOA typestate
    // cannot represent — encodes and round-trips through the factory call intact.
    let blob = Bytes::from(vec![0xAB_u8; 96]);
    assert!(
        RecoverableSignature::parse_bytes(&blob).is_err(),
        "the EOA typestate cannot represent a 1271 blob"
    );
    let encoded =
        encode_execute_hooks_calldata_with_signature(&calls, nonce, deadline, user, blob.clone());
    let decoded =
        COWShedFactory::executeHooksCall::abi_decode(&encoded).expect("1271 calldata decodes");
    assert_eq!(
        decoded.signature, blob,
        "the 1271 blob survives encode/decode"
    );
    assert_eq!(decoded.user, user);
    assert_eq!(decoded.nonce, nonce);
    assert_eq!(decoded.deadline, deadline);
}

fn to_call(call: &FixtureCall) -> Call {
    let mut out = Call::new(
        address(&call.target),
        decimal_u256(&call.value),
        bytes(&call.call_data),
    );
    if call.allow_failure {
        out = out.allow_failure();
    }
    if call.is_delegate_call {
        out = out.delegate_call();
    }
    out
}
