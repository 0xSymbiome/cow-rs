use alloy_primitives::B256;
use cow_sdk_cow_shed::{CALL_TYPE_HASH, EIP712_DOMAIN_TYPE_HASH, EXECUTE_HOOKS_TYPE_HASH};
use sha3::{Digest, Keccak256};

#[test]
fn type_hashes_match_canonical_type_strings() {
    assert_eq!(
        CALL_TYPE_HASH,
        keccak(
            "Call(address target,uint256 value,bytes callData,bool allowFailure,bool isDelegateCall)"
        )
    );
    assert_eq!(
        EXECUTE_HOOKS_TYPE_HASH,
        keccak(
            "ExecuteHooks(Call[] calls,bytes32 nonce,uint256 deadline)Call(address target,uint256 value,bytes callData,bool allowFailure,bool isDelegateCall)"
        )
    );
    assert_eq!(
        EIP712_DOMAIN_TYPE_HASH,
        keccak(
            "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
        )
    );
}

fn keccak(value: &str) -> B256 {
    let digest = Keccak256::digest(value.as_bytes());
    let mut out = [0_u8; 32];
    out.copy_from_slice(&digest);
    B256::from(out)
}
