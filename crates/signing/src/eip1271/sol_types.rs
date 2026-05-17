//! Canonical sol! type definitions for the COW EIP-1271 verifier
//! payload.
//!
//! The COW EIP-1271 verifier decodes `abi.encode(GPv2Order.Data,
//! bytes)` (a sequence of two values, not a wrapped tuple). The
//! on-chain `GPv2Order.Data` representation stores the `kind`,
//! `sellTokenBalance`, and `buyTokenBalance` fields as `bytes32` (the
//! keccak256 of the canonical label string), while the EIP-712
//! typed-data view of the same order treats those fields as `string`.
//! Both views describe the same protocol order; this module declares
//! the on-chain representation because that is what the EIP-1271
//! verifier expects on the wire.
//!
//! The payload type is the [`OrderAndSignature`] Rust tuple alias
//! `(OnchainOrder, alloy_primitives::Bytes)`. Encoding goes through
//! [`alloy_sol_types::SolValue::abi_encode_sequence`], which produces
//! the canonical `abi.encode(order, signature)` byte layout (no outer
//! tuple offset wrap).

alloy_sol_types::sol! {
    /// On-chain `GPv2Order.Data` representation as ABI-encoded into the
    /// EIP-1271 verifier payload. The `kind`, `sellTokenBalance`, and
    /// `buyTokenBalance` fields hold the keccak256 of the canonical
    /// label string rather than the string itself, matching the deployed
    /// settlement contract's storage layout.
    #[derive(Debug, Default, PartialEq, Eq)]
    struct OnchainOrder {
        address sellToken;
        address buyToken;
        address receiver;
        uint256 sellAmount;
        uint256 buyAmount;
        uint32 validTo;
        bytes32 appData;
        uint256 feeAmount;
        bytes32 kind;
        bool partiallyFillable;
        bytes32 sellTokenBalance;
        bytes32 buyTokenBalance;
    }
}

/// COW EIP-1271 verifier payload: an on-chain `GPv2Order.Data` plus the
/// raw ECDSA signature bytes the verifier consumes.
///
/// Encoded via
/// [`alloy_sol_types::SolValue::abi_encode_sequence`] on the underlying
/// `(OnchainOrder, alloy_primitives::Bytes)` tuple to produce the
/// canonical `abi.encode(order, signature)` byte layout.
pub type OrderAndSignature = (OnchainOrder, alloy_primitives::Bytes);
