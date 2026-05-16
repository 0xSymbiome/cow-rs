//! ABI calldata builders for COW Shed execution paths.

mod execute_hooks;
mod pre_sign;

pub use execute_hooks::encode_execute_hooks_calldata;
pub use pre_sign::encode_execute_pre_signed_hooks_calldata;

pub(crate) fn binding_calls(calls: &[crate::Call]) -> Vec<crate::bindings::factory::Call> {
    calls
        .iter()
        .map(|call| crate::bindings::factory::Call {
            target: call.target,
            value: call.value,
            callData: call.call_data.clone(),
            allowFailure: call.allow_failure,
            isDelegateCall: call.is_delegate_call,
        })
        .collect()
}

pub(crate) fn shed_binding_calls(calls: &[crate::Call]) -> Vec<crate::bindings::shed::Call> {
    calls
        .iter()
        .map(|call| crate::bindings::shed::Call {
            target: call.target,
            value: call.value,
            callData: call.call_data.clone(),
            allowFailure: call.allow_failure,
            isDelegateCall: call.is_delegate_call,
        })
        .collect()
}
