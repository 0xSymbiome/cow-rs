use alloy_primitives::{B256, U256};

use crate::address::{address_word, keccak256};
use crate::eip712::{CALL_TYPE_HASH, EXECUTE_HOOKS_TYPE_HASH};
use crate::types::Call;

/// Computes the COW Shed `ExecuteHooks` EIP-712 message hash.
#[must_use]
pub fn execute_hooks_message_hash(calls: &[Call], nonce: B256, deadline: U256) -> B256 {
    let calls_hash = calls_hash(calls);

    let mut encoded = Vec::with_capacity(32 * 4);
    encoded.extend_from_slice(EXECUTE_HOOKS_TYPE_HASH.as_slice());
    encoded.extend_from_slice(&calls_hash);
    encoded.extend_from_slice(nonce.as_slice());
    encoded.extend_from_slice(&deadline.to_be_bytes::<32>());
    B256::from(keccak256(encoded))
}

/// Computes the EIP-712 digest to sign.
#[must_use]
pub fn hash_to_sign(domain_separator: B256, message_hash: B256) -> B256 {
    let mut encoded = Vec::with_capacity(66);
    encoded.extend_from_slice(&[0x19, 0x01]);
    encoded.extend_from_slice(domain_separator.as_slice());
    encoded.extend_from_slice(message_hash.as_slice());
    B256::from(keccak256(encoded))
}

fn calls_hash(calls: &[Call]) -> [u8; 32] {
    let mut encoded = Vec::with_capacity(calls.len() * 32);
    for call in calls {
        encoded.extend_from_slice(&call_hash(call));
    }
    keccak256(encoded)
}

fn call_hash(call: &Call) -> [u8; 32] {
    let mut encoded = Vec::with_capacity(32 * 6);
    encoded.extend_from_slice(CALL_TYPE_HASH.as_slice());
    encoded.extend_from_slice(&address_word(call.target));
    encoded.extend_from_slice(&call.value.to_be_bytes::<32>());
    encoded.extend_from_slice(&keccak256(&call.call_data));
    encoded.extend_from_slice(&bool_word(call.allow_failure));
    encoded.extend_from_slice(&bool_word(call.is_delegate_call));
    keccak256(encoded)
}

const fn bool_word(value: bool) -> [u8; 32] {
    let mut out = [0_u8; 32];
    out[31] = value as u8;
    out
}
