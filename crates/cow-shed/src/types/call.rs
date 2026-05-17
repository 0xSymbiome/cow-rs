//! Re-export of the canonical COW Shed `Call` struct.
//!
//! The crate-level [`cow_sdk_cow_shed::Call`](crate::Call) alias resolves
//! through this module to the macro-emitted `Call` declared in the
//! canonical [`crate::eip712::sol_types`] sol! block. The
//! [`CallExt`] extension trait carries the ergonomic builder helpers
//! (`new`, `allow_failure`, `delegate_call`) on top of the sol-generated
//! type.

use alloy_primitives::{Address, Bytes, U256};

pub use crate::eip712::sol_types::Call;

/// Ergonomic builder helpers for [`Call`].
///
/// The canonical [`Call`] is sol-generated and therefore exposes the raw
/// Solidity field names (`callData`, `allowFailure`, `isDelegateCall`).
/// `CallExt` provides a snake-case builder API: `Call::new(target,
/// value, call_data)` constructs a `Call` with failure tolerance and
/// `delegatecall` dispatch disabled, and `allow_failure` and
/// `delegate_call` flip the matching boolean fields.
pub trait CallExt: Sized {
    /// Creates a hook call with failure tolerance and `delegatecall`
    /// dispatch disabled.
    #[must_use]
    fn new(target: Address, value: U256, call_data: Bytes) -> Self;

    /// Returns a copy that tolerates a target revert without aborting the
    /// hook bundle.
    #[must_use]
    fn allow_failure(self) -> Self;

    /// Returns a copy that executes the target via `delegatecall` rather
    /// than `call`.
    #[must_use]
    fn delegate_call(self) -> Self;
}

impl CallExt for Call {
    fn new(target: Address, value: U256, call_data: Bytes) -> Self {
        Self {
            target,
            value,
            callData: call_data,
            allowFailure: false,
            isDelegateCall: false,
        }
    }

    fn allow_failure(mut self) -> Self {
        self.allowFailure = true;
        self
    }

    fn delegate_call(mut self) -> Self {
        self.isDelegateCall = true;
        self
    }
}
