use alloy_primitives::{Address, B256, Bytes, Signature, U256};
use alloy_sol_types::SolCall;

use crate::Call;
use crate::bindings::factory::COWShedFactory;

/// Encodes factory `executeHooks` calldata with an EOA compact signature.
#[must_use]
pub fn encode_execute_hooks_calldata(
    calls: &[Call],
    nonce: B256,
    deadline: U256,
    r_compact: [u8; 32],
    vs: [u8; 32],
    who: Address,
) -> Bytes {
    let call = COWShedFactory::executeHooksCall {
        calls: calls.to_vec(),
        nonce,
        deadline,
        user: who,
        signature: Bytes::from(eoa_signature_from_compact(&r_compact, &vs).to_vec()),
    };
    Bytes::from(call.abi_encode())
}

/// Decodes an [ERC-2098] compact signature `r || vs` into the canonical
/// 65-byte `r || s || v` layout with `v ∈ {27, 28}`.
///
/// Delegates to [`alloy_primitives::Signature::from_erc2098`] over the
/// 64-byte concatenation of `r_compact` and `vs`; the alloy primitive
/// extracts the y-parity bit from the high bit of `vs[0]`, masks it out
/// of the recovered `s`, and stores the canonical `Signature`.
/// [`Signature::as_bytes`] then emits the 65-byte `r || s || v` form
/// with `v = 27 + y_parity`.
///
/// [ERC-2098]: https://eips.ethereum.org/EIPS/eip-2098
#[must_use]
pub fn eoa_signature_from_compact(r_compact: &[u8; 32], vs: &[u8; 32]) -> [u8; 65] {
    let mut compact = [0_u8; 64];
    compact[..32].copy_from_slice(r_compact);
    compact[32..].copy_from_slice(vs);
    Signature::from_erc2098(&compact).as_bytes()
}
