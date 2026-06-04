//! Contract: the `executeHooks` calldata encoders. `encode_execute_hooks_calldata_signed`
//! (typed 65-byte `RecoverableSignature`) reproduces the reference factory
//! calldata byte-for-byte and equals the compact-form
//! `encode_execute_hooks_calldata`. `encode_execute_hooks_calldata_with_signature`
//! (general) is a faithful wrapper for the EOA case and additionally encodes an
//! EIP-1271 contract-signature blob the typed path cannot represent.

use alloy_primitives::{Address, B256, Bytes, U256};
use cow_sdk_contracts::RecoverableSignature;
use cow_sdk_cow_shed::{
    Call, compact_signature, encode_execute_hooks_calldata, encode_execute_hooks_calldata_signed,
    encode_execute_hooks_calldata_with_signature,
};
use serde::Deserialize;

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
fn signed_calldata_matches_reference_and_compact_builder() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("calldata fixture parses");
    assert!(!fixture.rows.is_empty(), "calldata fixture must carry rows");

    for row in &fixture.rows {
        let calls = row.calls.iter().map(to_call).collect::<Vec<_>>();
        let signature = RecoverableSignature::parse_bytes(&bytes(&row.signature))
            .unwrap_or_else(|err| panic!("row {}: signature parses: {err:?}", row.name));
        let nonce = b256(&row.nonce);
        let deadline = decimal_u256(&row.deadline);
        let user = address(&row.user);

        let signed =
            encode_execute_hooks_calldata_signed(&calls, nonce, deadline, user, &signature);
        assert_eq!(
            signed,
            bytes(&row.factory_call_data),
            "row {}: signed calldata diverges from reference vector",
            row.name
        );

        let (r, vs) = compact_signature(&signature);
        let compact = encode_execute_hooks_calldata(&calls, nonce, deadline, r, vs, user);
        assert_eq!(
            signed, compact,
            "row {}: signed entry point must equal the compact-form builder",
            row.name
        );
    }
}

#[test]
fn with_signature_covers_eoa_and_eip1271() {
    use alloy_sol_types::SolCall;
    use cow_sdk_cow_shed::bindings::factory::COWShedFactory;

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
