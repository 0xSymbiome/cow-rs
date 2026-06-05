use alloy_primitives::Bytes;
use alloy_sol_types::SolCall;
use cow_sdk_cow_shed::bindings::{COWShed, COWShedFactory};
use cow_sdk_cow_shed::{
    Call, encode_execute_hooks_calldata, encode_execute_pre_signed_hooks_calldata,
};
use serde::Deserialize;

mod common;
use common::{address, b256, bytes, decimal_u256};

const FIXTURE: &str = include_str!("../../../parity/fixtures/cow_shed/execute_hooks_calldata.json");

#[derive(Debug, Deserialize)]
struct Fixture {
    rows: Vec<Row>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FixtureCall {
    target: String,
    value: String,
    call_data: String,
    allow_failure: bool,
    is_delegate_call: bool,
}

#[test]
fn execute_hooks_calldata_matches_and_round_trips() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("calldata fixture parses");
    assert!(
        !fixture.rows.is_empty(),
        "execute_hooks_calldata fixture must carry at least one row"
    );

    for row in &fixture.rows {
        let calls = row.calls.iter().map(to_call).collect::<Vec<_>>();
        let signature = bytes(&row.signature);
        let (r, vs) = compact_signature(&signature);

        let encoded = encode_execute_hooks_calldata(
            &calls,
            b256(&row.nonce),
            decimal_u256(&row.deadline),
            r,
            vs,
            address(&row.user),
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
fn execute_pre_signed_hooks_calldata_round_trips() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("calldata fixture parses");
    for row in &fixture.rows {
        let calls = row.calls.iter().map(to_call).collect::<Vec<_>>();
        let encoded = encode_execute_pre_signed_hooks_calldata(
            &calls,
            b256(&row.nonce),
            decimal_u256(&row.deadline),
        );
        assert_eq!(
            &encoded[..4],
            COWShed::executePreSignedHooksCall::SELECTOR,
            "row {}: pre-signed selector",
            row.name
        );
        let decoded = COWShed::executePreSignedHooksCall::abi_decode(&encoded)
            .unwrap_or_else(|err| panic!("row {}: pre-signed calldata decodes: {err}", row.name));
        assert_eq!(
            decoded.abi_encode(),
            encoded.as_ref(),
            "row {}: pre-signed calldata round-trip",
            row.name
        );
    }
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

fn compact_signature(signature: &Bytes) -> ([u8; 32], [u8; 32]) {
    assert_eq!(signature.len(), 65, "fixture signature is r || s || v");
    let mut r = [0_u8; 32];
    r.copy_from_slice(&signature[..32]);
    let mut vs = [0_u8; 32];
    vs.copy_from_slice(&signature[32..64]);
    if signature[64] == 28 {
        vs[0] |= 0x80;
    }
    (r, vs)
}
