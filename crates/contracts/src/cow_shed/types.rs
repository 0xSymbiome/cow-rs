//! Public COW Shed helper types.

use std::time::Duration;

use alloy_primitives::{Address, B256, Bytes, U256};

pub use crate::cow_shed::bindings::Call;
pub use cow_sdk_app_data::{Hook, HookList};

/// COW Shed proxy address type.
pub type ProxyAddress = alloy_primitives::Address;

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

/// Deadline strategy for COW Shed hook authorization.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Deadline {
    /// No practical expiry.
    Never,
    /// Absolute UNIX timestamp in seconds.
    Absolute(u64),
    /// Relative duration from the supplied `now` timestamp.
    Relative(Duration),
}

impl Deadline {
    /// Resolves this deadline to the `uint256` value encoded into calldata.
    #[must_use]
    pub fn resolve(self, now: u64) -> U256 {
        match self {
            Self::Never => U256::MAX,
            Self::Absolute(value) => U256::from(value),
            Self::Relative(duration) => U256::from(now.saturating_add(duration.as_secs())),
        }
    }
}

/// Nonce strategy for COW Shed hook authorization.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nonce {
    /// Caller supplies entropy when signing.
    Random,
    /// Monotonic caller-managed numeric nonce.
    Sequential(u64),
    /// Exact nonce value supplied by the caller.
    Explicit(B256),
}
