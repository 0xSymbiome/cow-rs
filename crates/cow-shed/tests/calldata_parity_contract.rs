use alloy_primitives::{Address, B256, Bytes, U256};
use alloy_sol_types::SolCall;
use cow_sdk_cow_shed::bindings::{COWShed, COWShedFactory};
use cow_sdk_cow_shed::{
    Call, encode_execute_hooks_calldata, encode_execute_pre_signed_hooks_calldata,
};
use serde::Deserialize;

const FIXTURE: &str = include_str!("../../../parity/fixtures/cow_shed/execute_hooks_calldata.json");

#[derive(Debug, Deserialize)]
struct Fixture {
    calls: Vec<FixtureCall>,
    nonce: String,
    deadline: String,
    user: String,
    signature: String,
    #[serde(rename = "factory_call_data")]
    factory_call_data: String,
    #[serde(rename = "proxy_call_data")]
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
    let calls = fixture.calls.iter().map(to_call).collect::<Vec<_>>();
    let signature = bytes(&fixture.signature);
    let (r, vs) = compact_signature(&signature);

    let encoded = encode_execute_hooks_calldata(
        &calls,
        b256(&fixture.nonce),
        decimal_u256(&fixture.deadline),
        r,
        vs,
        address(&fixture.user),
    );
    assert_eq!(encoded, bytes(&fixture.factory_call_data));

    let decoded =
        COWShedFactory::executeHooksCall::abi_decode(&encoded).expect("factory calldata decodes");
    assert_eq!(decoded.abi_encode(), encoded.as_ref());
}

#[test]
fn proxy_execute_hooks_fixture_round_trips() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("calldata fixture parses");
    let encoded = bytes(&fixture.proxy_call_data);
    let decoded = COWShed::executeHooksCall::abi_decode(&encoded).expect("proxy calldata decodes");
    assert_eq!(decoded.abi_encode(), encoded.as_ref());
}

#[test]
fn execute_pre_signed_hooks_calldata_round_trips() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("calldata fixture parses");
    let calls = fixture.calls.iter().map(to_call).collect::<Vec<_>>();
    let encoded = encode_execute_pre_signed_hooks_calldata(
        &calls,
        b256(&fixture.nonce),
        decimal_u256(&fixture.deadline),
    );
    assert_eq!(&encoded[..4], COWShed::executePreSignedHooksCall::SELECTOR);
    let decoded = COWShed::executePreSignedHooksCall::abi_decode(&encoded)
        .expect("pre-signed calldata decodes");
    assert_eq!(decoded.abi_encode(), encoded.as_ref());
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
    Bytes::from(hex::decode(value.trim_start_matches("0x")).expect("fixture hex parses"))
}

fn address(value: &str) -> Address {
    value.parse().expect("fixture address parses")
}

fn b256(value: &str) -> B256 {
    let bytes = hex::decode(value.trim_start_matches("0x")).expect("fixture hash parses");
    let mut out = [0_u8; 32];
    out.copy_from_slice(&bytes);
    B256::from(out)
}
