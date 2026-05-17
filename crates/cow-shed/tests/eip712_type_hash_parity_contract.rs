use alloy_primitives::{Address, B256, Bytes, U256};
use alloy_sol_types::{Eip712Domain, SolStruct};
use cow_sdk_cow_shed::{ExecuteHooks, SolCall};
use sha3::{Digest, Keccak256};

/// Asserts the macro-emitted EIP-712 type hashes for the COW Shed
/// `Call` and `ExecuteHooks` structs and the `EIP712Domain` struct equal
/// the keccak of the canonical EIP-712 type strings. The keccak helper
/// below runs `sha3::Keccak256` directly so the comparison uses an
/// independent keccak implementation.
#[test]
fn type_hashes_match_canonical_type_strings() {
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
    let domain_sample = Eip712Domain {
        name: Some("COWShed".into()),
        version: Some("1.0.1".into()),
        chain_id: Some(U256::from(1_u64)),
        verifying_contract: Some(Address::ZERO),
        salt: None,
    };

    assert_eq!(
        call_sample.eip712_type_hash(),
        keccak(
            "Call(address target,uint256 value,bytes callData,bool allowFailure,bool isDelegateCall)"
        )
    );
    assert_eq!(
        exec_sample.eip712_type_hash(),
        keccak(
            "ExecuteHooks(Call[] calls,bytes32 nonce,uint256 deadline)Call(address target,uint256 value,bytes callData,bool allowFailure,bool isDelegateCall)"
        )
    );
    assert_eq!(
        domain_sample.type_hash(),
        keccak(
            "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
        )
    );
}

// Hand-rolled `sha3::Keccak256` helper used by the assertions above.
// Crate code reaches the type hashes through
// `alloy_sol_types::SolStruct::eip712_type_hash` and
// `alloy_sol_types::Eip712Domain::type_hash`; this helper deliberately
// runs `sha3::Keccak256` directly so the parity check compares the
// macro-emitted accessors against an independent keccak implementation.
fn keccak(value: &str) -> B256 {
    let digest = Keccak256::digest(value.as_bytes());
    let mut out = [0_u8; 32];
    out.copy_from_slice(&digest);
    B256::from(out)
}
