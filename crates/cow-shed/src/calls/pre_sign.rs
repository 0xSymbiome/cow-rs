use alloy_primitives::{B256, Bytes, U256};
use alloy_sol_types::SolCall;

use crate::Call;
use crate::bindings::shed::COWShed;
use crate::calls::shed_binding_calls;

/// Encodes proxy `executePreSignedHooks` calldata.
#[must_use]
pub fn encode_execute_pre_signed_hooks_calldata(
    calls: &[Call],
    nonce: B256,
    deadline: U256,
) -> Bytes {
    let call = COWShed::executePreSignedHooksCall {
        calls: shed_binding_calls(calls),
        nonce,
        deadline,
    };
    Bytes::from(call.abi_encode())
}
