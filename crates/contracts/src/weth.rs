//! Typed `IWrappedNativeToken` (WETH9-family) bindings and wrap / unwrap
//! interaction helpers.
//!
//! Every chain CoW Protocol supports exposes a wrapped-native ERC-20 (WETH on
//! Ethereum mainnet) whose `deposit()` / `withdraw(uint256)` entrypoints convert
//! between the native asset and its wrapped form. This module provides the
//! `alloy::sol!`-generated [`IWrappedNativeToken`] binding plus two helpers that
//! emit the canonical settlement [`Interaction`] for wrapping and unwrapping.
//!
//! The wrapped-native token address for a given chain is resolved by
//! [`cow_sdk_core::wrapped_native_token`]; callers pass that address into the
//! helpers below.
//!
//! The Solidity excerpt used to author these bindings is committed under
//! `crates/contracts/abi/weth/` for provenance.

use alloy_primitives::Bytes;
use alloy_sol_types::{SolCall, sol};

use cow_sdk_core::{Address, Amount};

use crate::interaction::Interaction;

sol! {
    // Canonical wrapped-native-token surface. The `deposit()` / `withdraw(uint256)`
    // signatures are reproduced verbatim from the cowprotocol/ethflowcontract
    // `IWrappedNativeToken` interface (upstream source at
    // https://github.com/cowprotocol/ethflowcontract); the 4-byte selectors they
    // generate are byte-identical to those every WETH9 deployment exposes. The
    // Solidity excerpt used to author these bindings is committed under
    // `crates/contracts/abi/weth/` for provenance.
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
