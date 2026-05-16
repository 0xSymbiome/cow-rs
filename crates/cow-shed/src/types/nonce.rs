use alloy_primitives::B256;

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
