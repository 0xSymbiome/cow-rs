//! Typed ABI bindings for the ERC-20 and wrapped-native token surfaces.
//!
//! - [`IERC20`] reproduces the minimal [EIP-20](https://eips.ethereum.org/EIPS/eip-20)
//!   surface every downstream consumer needs (`balanceOf`, `approve`,
//!   `allowance`, `transfer`, `transferFrom`) plus the standard `Transfer` and
//!   `Approval` events.
//! - [`IWrappedNativeToken`] reproduces the WETH9-family `deposit()` /
//!   `withdraw(uint256)` surface, with [`wrap_interaction`] /
//!   [`unwrap_interaction`] helpers that emit the canonical settlement
//!   [`Interaction`] for converting between the native asset and its wrapped
//!   form. The wrapped-native token address for a chain is resolved by
//!   [`cow_sdk_core::wrapped_native_token`].
//!
//! Both interfaces are authored inline as `alloy::sol!` against the published
//! EIP-20 standard and the upstream `cowprotocol/ethflowcontract`
//! `src/interfaces/IWrappedNativeToken.sol` surface (pinned by commit in
//! `parity/source-lock.yaml`); their wire shape is proven by the selector
//! fixtures under `parity/fixtures/` and the crate parity tests.

use alloy_primitives::Bytes;
use alloy_sol_types::{SolCall, sol};

use cow_sdk_core::{Address, Amount};

use crate::interaction::Interaction;

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

sol! {
    // Canonical wrapped-native-token surface. The `deposit()` / `withdraw(uint256)`
    // signatures mirror cowprotocol/ethflowcontract
    // `src/interfaces/IWrappedNativeToken.sol` (pinned by commit in
    // `parity/source-lock.yaml`); the 4-byte selectors they generate are
    // byte-identical to those every WETH9 deployment exposes and are proven by
    // the selector fixtures under `parity/fixtures/`.
    interface IWrappedNativeToken {
        function deposit() external payable;
        function withdraw(uint256 wad) external;
    }
}

/// Builds the interaction that wraps `amount` of the native asset into the
/// wrapped-native token.
///
/// The interaction calls `deposit()` on `wrapped_native_token` with `amount`
/// attached as the native value transferred with the call.
#[must_use]
pub fn wrap_interaction(wrapped_native_token: Address, amount: Amount) -> Interaction {
    Interaction::new(
        wrapped_native_token,
        amount,
        Bytes::from(IWrappedNativeToken::depositCall {}.abi_encode()),
    )
}

/// Builds the interaction that unwraps `amount` of the wrapped-native token back
/// into the native asset.
///
/// The interaction calls `withdraw(amount)` on `wrapped_native_token` with zero
/// native value attached.
#[must_use]
pub fn unwrap_interaction(wrapped_native_token: Address, amount: Amount) -> Interaction {
    Interaction::new(
        wrapped_native_token,
        Amount::ZERO,
        Bytes::from(
            IWrappedNativeToken::withdrawCall {
                wad: *amount.as_u256(),
            }
            .abi_encode(),
        ),
    )
}
