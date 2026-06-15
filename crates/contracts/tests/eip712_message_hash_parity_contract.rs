#![cfg(feature = "cow-shed")]

use alloy_primitives::{Address, B256, Bytes, U256};
use alloy_sol_types::SolStruct;
use cow_sdk_contracts::cow_shed::{
    Call, ExecuteHooks, SolCall, cow_shed_eip712_domain, execute_hooks_signing_hash,
};
use serde::Deserialize;

mod cow_shed_common;
use cow_shed_common::{address, b256, bytes, decimal_u256, parse_version};

const FIXTURE: &str = include_str!("../../../parity/fixtures/cow_shed/execute_hooks_digest.json");

#[derive(Debug, Deserialize)]
struct Fixture {
    call_type_hash: String,
    execute_hooks_type_hash: String,
    version: String,
    proxy: String,
    message: Message,
    rows: Vec<Row>,
}

#[derive(Debug, Deserialize)]
struct Row {
    chain_id: u64,
    domain_separator: String,
    digest: String,
}

#[derive(Debug, Deserialize)]
struct Message {
    calls: Vec<FixtureCall>,
    nonce: String,
    deadline: String,
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
fn execute_hooks_digest_matches_reference_vectors() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("digest fixture parses");

    // The EIP-712 type hashes and the `ExecuteHooks` message are chain-invariant,
    // so they live once at the fixture header: assert the type hashes once and
    // build the message once. Only the domain separator and the final digest vary
    // per chain.
    let call_sample = SolCall {
        target: Address::ZERO,
        value: U256::ZERO,
        callData: Bytes::default(),
        allowFailure: false,
        isDelegateCall: false,
    };
    let exec_sample = ExecuteHooks {
        calls: vec![],
        nonce: B256::ZERO,
        deadline: U256::ZERO,
    };
    assert_eq!(
        call_sample.eip712_type_hash(),
        b256(&fixture.call_type_hash)
    );
    assert_eq!(
        exec_sample.eip712_type_hash(),
        b256(&fixture.execute_hooks_type_hash)
    );

    let calls = fixture
        .message
        .calls
        .iter()
        .map(to_call)
        .collect::<Vec<_>>();
    let nonce = b256(&fixture.message.nonce);
    let deadline = decimal_u256(&fixture.message.deadline);

    for row in &fixture.rows {
        let domain = cow_shed_eip712_domain(
            row.chain_id,
            parse_version(&fixture.version),
            address(&fixture.proxy),
        );
        assert_eq!(domain.separator(), b256(&row.domain_separator));

        let actual = execute_hooks_signing_hash(&domain, &calls, nonce, deadline);
        assert_eq!(actual, b256(&row.digest));
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
