use alloy_primitives::{Address, B256, Bytes, U256};
use alloy_sol_types::SolStruct;
use cow_sdk_cow_shed::{
    Call, ExecuteHooks, SolCall, cow_shed_eip712_domain, execute_hooks_signing_hash,
};
use serde::Deserialize;

mod common;
use common::{address, b256, bytes, decimal_u256, parse_version};

const FIXTURE: &str = include_str!("../../../parity/fixtures/cow_shed/execute_hooks_digest.json");

#[derive(Debug, Deserialize)]
struct Fixture {
    rows: Vec<Row>,
}

#[derive(Debug, Deserialize)]
struct Row {
    chain_id: u64,
    version: String,
    proxy: String,
    domain_separator: String,
    call_type_hash: String,
    execute_hooks_type_hash: String,
    message: Message,
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

    for row in fixture.rows {
        assert_eq!(call_sample.eip712_type_hash(), b256(&row.call_type_hash));
        assert_eq!(
            exec_sample.eip712_type_hash(),
            b256(&row.execute_hooks_type_hash)
        );

        let version = parse_version(&row.version);
        let domain = cow_shed_eip712_domain(row.chain_id, version, address(&row.proxy));
        assert_eq!(domain.separator(), b256(&row.domain_separator));

        let calls = row.message.calls.iter().map(to_call).collect::<Vec<_>>();
        let actual = execute_hooks_signing_hash(
            &domain,
            &calls,
            b256(&row.message.nonce),
            decimal_u256(&row.message.deadline),
        );
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
