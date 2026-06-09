//! Typed ABI binding for the minimal ERC-20 surface.
//!
//! [`IERC20`] reproduces the minimal [EIP-20](https://eips.ethereum.org/EIPS/eip-20)
//! surface every downstream consumer in this workspace needs (`balanceOf`,
//! `approve`, `allowance`, `transfer`, `transferFrom`) plus the standard
//! `Transfer` and `Approval` events.
//!
//! The interface is authored inline as `alloy::sol!` against the published
//! EIP-20 standard; its wire shape is proven by the selector fixtures under
//! `parity/fixtures/` and the crate parity tests.

use alloy_sol_types::sol;

sol! {
    /// Minimal ERC-20 interface.
    ///
    /// Reproduces the canonical surface defined by
    /// <https://eips.ethereum.org/EIPS/eip-20>: `balanceOf`, `approve`,
    /// `allowance`, `transfer`, `transferFrom` plus the `Transfer` and
    /// `Approval` events. Every method selector and event topic matches the
    /// standard wire shape byte-identically; the 4-byte selectors generated
    /// through this binding are the same bytes emitted by every EIP-20
    /// implementation.
    #[sol(rename_all = "camelcase")]
    interface IERC20 {
        event Transfer(address indexed from, address indexed to, uint256 value);
        event Approval(address indexed owner, address indexed spender, uint256 value);

        function balanceOf(address account) external view returns (uint256);

        function transfer(address to, uint256 value) external returns (bool);

        function allowance(address owner, address spender) external view returns (uint256);

        function approve(address spender, uint256 value) external returns (bool);

        function transferFrom(address from, address to, uint256 value) external returns (bool);
    }
}
