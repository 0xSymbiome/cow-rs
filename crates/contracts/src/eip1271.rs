//! Typed ABI binding for the EIP-1271 standard signature-validation interface.
//!
//! The module exposes one `alloy::sol!`-generated interface:
//!
//! * [`IERC1271`] — the
//!   [EIP-1271](https://eips.ethereum.org/EIPS/eip-1271) `isValidSignature`
//!   surface every smart-account verifier in the CoW Protocol family
//!   implements.
//!
//! The macro emits the canonical
//! `<IERC1271::isValidSignatureCall as alloy_sol_types::SolCall>::SELECTOR`
//! constant as the byte-identical 4-byte function selector
//! `[0x16, 0x26, 0xba, 0x7e]`. The same four bytes are the on-the-wire
//! magic value EIP-1271 verifiers return from a successful
//! `isValidSignature(bytes32,bytes)` call, so the cow signature path uses
//! the macro-emitted constant for both the dispatch selector and the
//! response magic-value comparison.
//!
//! This interface mirrors cowdao-grants/cow-shed `src/interfaces/IERC1271.sol`,
//! pinned by commit in `parity/source-lock.yaml`, and its selector is proven by
//! the crate parity tests. It is the canonical EIP-1271 interface for the
//! cow-rs workspace.

use alloy_sol_types::sol;

sol! {
    /// EIP-1271 smart-account signature-validation interface.
    ///
    /// Reproduces the canonical surface defined by
    /// [EIP-1271](https://eips.ethereum.org/EIPS/eip-1271). Verifier
    /// contracts return the 4-byte function selector
    /// `keccak256("isValidSignature(bytes32,bytes)")[..4]` on a successful
    /// validation; the cow signature path compares the decoded response
    /// against [`isValidSignatureCall::SELECTOR`].
    interface IERC1271 {
        function isValidSignature(bytes32 hash, bytes calldata signature) external view returns (bytes4);
    }
}
