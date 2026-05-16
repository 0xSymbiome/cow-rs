use alloy_primitives::{Address, B256, Bytes, U256};
use alloy_sol_types::SolCall;

use crate::Call;
use crate::bindings::factory::COWShedFactory;
use crate::calls::binding_calls;

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
        calls: binding_calls(calls),
        nonce,
        deadline,
        user: who,
        signature: eoa_signature_from_compact(r_compact, vs).into(),
    };
    Bytes::from(call.abi_encode())
}

fn eoa_signature_from_compact(r: [u8; 32], mut vs: [u8; 32]) -> Vec<u8> {
    let v = 27 + (vs[0] >> 7);
    vs[0] &= 0x7f;

    let mut signature = Vec::with_capacity(65);
    signature.extend_from_slice(&r);
    signature.extend_from_slice(&vs);
    signature.push(v);
    signature
}
