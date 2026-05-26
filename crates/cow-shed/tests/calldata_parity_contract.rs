use alloy_primitives::{Address, B256, Bytes, U256};
use alloy_sol_types::SolCall;
use cow_sdk_cow_shed::bindings::{COWShed, COWShedFactory};
use cow_sdk_cow_shed::{
    Call, CallExt, encode_execute_hooks_calldata, encode_execute_pre_signed_hooks_calldata,
};
use serde::Deserialize;

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

fn decimal_u256(value: &str) -> U256 {
    U256::from(value.parse::<u64>().expect("fixture integer fits u64"))
}

fn bytes(value: &str) -> Bytes {
    Bytes::from(
        alloy_primitives::hex::decode(value.trim_start_matches("0x")).expect("fixture hex parses"),
    )
}

fn address(value: &str) -> Address {
    value.parse().expect("fixture address parses")
}

fn b256(value: &str) -> B256 {
    let bytes =
        alloy_primitives::hex::decode(value.trim_start_matches("0x")).expect("fixture hash parses");
    let mut out = [0_u8; 32];
    out.copy_from_slice(&bytes);
    B256::from(out)
}
