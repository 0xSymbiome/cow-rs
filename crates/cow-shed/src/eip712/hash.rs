use alloy_primitives::{B256, U256, keccak256};
use alloy_sol_types::SolStruct;

use crate::eip712::sol_types::ExecuteHooks;
use crate::types::Call;

/// Computes the COW Shed `ExecuteHooks` EIP-712 message hash.
///
/// Delegates to [`alloy_sol_types::SolStruct::eip712_hash_struct`] on
/// the macro-emitted [`ExecuteHooks`] struct declared in
/// [`crate::eip712::sol_types`]. The macro emits the canonical
/// `keccak256(type_hash || encoded_data)` per the EIP-712 specification.
/// The `parity/fixtures/cow_shed/execute_hooks_digest.json` rows lock
/// the per-row byte contract.
#[must_use]
pub fn execute_hooks_message_hash(calls: &[Call], nonce: B256, deadline: U256) -> B256 {
    ExecuteHooks {
        calls: calls.to_vec(),
        nonce,
        deadline,
    }
    .eip712_hash_struct()
}

/// Computes the EIP-712 digest to sign: `keccak256(0x19 || 0x01 ||
/// domain_separator || message_hash)`.
///
/// The fixed-size 66-byte buffer mirrors the canonical EIP-712 envelope
/// specification; the keccak invocation routes through
/// [`alloy_primitives::keccak256`]. The
/// `parity/fixtures/cow_shed/execute_hooks_digest.json` rows record the
/// expected per-row digest.
#[must_use]
pub fn hash_to_sign(domain_separator: B256, message_hash: B256) -> B256 {
    let mut payload = [0_u8; 66];
    payload[0] = 0x19;
    payload[1] = 0x01;
    payload[2..34].copy_from_slice(domain_separator.as_slice());
    payload[34..66].copy_from_slice(message_hash.as_slice());
    keccak256(payload)
}
