//! Public COW Shed helper types.

use alloy_primitives::{Address, Bytes, U256};

pub use crate::cow_shed::bindings::Call;
pub use cow_sdk_app_data::{Hook, HookList};

impl Call {
    /// Creates a hook call with failure tolerance and `delegatecall` dispatch
    /// disabled.
    ///
    /// The canonical [`Call`] is sol-generated and exposes the raw Solidity
    /// field names (`callData`, `allowFailure`, `isDelegateCall`); this builder
    /// takes snake-case inputs and leaves `allowFailure` and `isDelegateCall`
    /// `false`. Flip them with [`Call::allow_failure`] / [`Call::delegate_call`].
    #[must_use]
    pub const fn new(target: Address, value: U256, call_data: Bytes) -> Self {
        Self {
            target,
            value,
            callData: call_data,
            allowFailure: false,
            isDelegateCall: false,
        }
    }

    /// Returns a copy that tolerates a target revert without aborting the hook
    /// bundle.
    #[must_use]
    pub const fn allow_failure(mut self) -> Self {
        self.allowFailure = true;
        self
    }

    /// Returns a copy that executes the target via `delegatecall` rather than
    /// `call`.
    ///
    /// `delegatecall` runs the target in the proxy's own storage context, so
    /// per ADR 0049 each call site must justify the choice with a `// SAFETY:`
    /// comment.
    #[must_use]
    pub const fn delegate_call(mut self) -> Self {
        self.isDelegateCall = true;
        self
    }
}
