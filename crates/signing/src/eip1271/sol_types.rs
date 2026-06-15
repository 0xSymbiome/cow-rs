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

// DO NOT SWAP: do not collapse Shape A and Shape B into a single blob
// encoder.
//
// Shape A (Safe muxer) is `selector(safeSignature(...)) ||
// abi.encode(handler, params, GPv2Order.Data, payload)`. The 4-byte
// selector prefix is load-bearing for Safe muxer dispatch; dropping
// it makes the muxer fail to route and the on-chain signature
// verification reverts at the Safe layer.
//
// Shape B (raw forwarder, this module) is `abi.encode(GPv2Order.Data,
// signature)` with no selector prefix. Adding a 4-byte prefix shifts
// every ABI field offset by 4 and the verifier `abi.decode` fails
// with `InvalidData`.
//
// The two shapes must remain distinct encoder entry points. Do not
// add a `fn encode_eip1271_blob<S: ShapeKind>(...)` helper that picks
// the shape from an enum argument; the failure mode is silent
// on-chain revert, not a compile error.
//
// ADR: docs/adr/0050-eip1271-signature-blob-encoding.md (the Shape A vs
// Shape B blob-encoding decision, its Must Remain True invariants, and
// the composable-deferral amendment).
// Doctrine: docs/alloy-doctrine.md, Bucket 2 rows for EIP-1271
// signature blob Shape A (Safe muxer) and Shape B (raw forwarder).
// Enforced by cargo check-source-fences (xtask/src/policy/fences.rs).
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
