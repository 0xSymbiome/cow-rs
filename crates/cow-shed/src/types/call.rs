use alloy_primitives::{Address, Bytes, U256};

/// COW Shed hook call matching the Solidity `Call` struct.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Call {
    /// Target contract invoked by the hook.
    pub target: Address,
    /// Native asset value sent with the call.
    pub value: U256,
    /// ABI calldata supplied to the target.
    pub call_data: Bytes,
    /// Whether a revert should be tolerated.
    pub allow_failure: bool,
    /// Whether to execute via `delegatecall`.
    pub is_delegate_call: bool,
}

impl Call {
    /// Creates a hook call with failure and delegatecall disabled.
    #[must_use]
    pub const fn new(target: Address, value: U256, call_data: Bytes) -> Self {
        Self {
            target,
            value,
            call_data,
            allow_failure: false,
            is_delegate_call: false,
        }
    }

    /// Returns a copy that tolerates target reverts.
    #[must_use]
    pub const fn allow_failure(mut self) -> Self {
        self.allow_failure = true;
        self
    }

    /// Returns a copy that executes the target via `delegatecall`.
    #[must_use]
    pub const fn delegate_call(mut self) -> Self {
        self.is_delegate_call = true;
        self
    }
}

impl Default for Call {
    fn default() -> Self {
        Self::new(Address::ZERO, U256::ZERO, Bytes::default())
    }
}
