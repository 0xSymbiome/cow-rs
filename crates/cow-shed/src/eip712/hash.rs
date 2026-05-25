use alloy_primitives::{B256, U256};
use alloy_sol_types::{Eip712Domain, SolStruct};

use crate::eip712::sol_types::ExecuteHooks;
use crate::types::Call;

/// Computes the COW Shed `ExecuteHooks` EIP-712 signing hash.
///
/// Delegates to [`alloy_sol_types::SolStruct::eip712_signing_hash`] on
/// the macro-emitted [`ExecuteHooks`] struct declared in
/// [`crate::eip712::sol_types`]. The macro emits the canonical EIP-712
/// envelope per the specification (the `0x19` prefix followed by the
/// `0x01` typed-data version, then the domain separator and the struct
/// hash, all routed through [`alloy_primitives::keccak256`]). The
/// `parity/fixtures/cow_shed/execute_hooks_digest.json` rows lock the
/// per-row byte contract.
///
/// The `domain` argument is the value returned by
/// [`cow_shed_eip712_domain`](super::cow_shed_eip712_domain) for the
/// target chain, version, and proxy address.
#[must_use]
pub fn execute_hooks_signing_hash(
    domain: &Eip712Domain,
    calls: &[Call],
    nonce: B256,
    deadline: U256,
) -> B256 {
    ExecuteHooks {
        calls: calls.to_vec(),
        nonce,
        deadline,
    }
    .eip712_signing_hash(domain)
}
