//! Integration tests for the `IWrappedNativeToken` (WETH9-family) bindings and
//! the wrap / unwrap interaction helpers.
//!
//! The selector byte-locks cross-check the `alloy::sol!`-generated 4-byte
//! selectors against an independent keccak-256 of the canonical signature, the
//! same bytes every WETH9 deployment exposes.

use alloy_sol_types::SolCall;
use cow_sdk_contracts::{IWrappedNativeToken, unwrap_interaction, wrap_interaction};
use cow_sdk_core::{Address, Amount};
use sha3::{Digest, Keccak256};

fn keccak4(preimage: &[u8]) -> [u8; 4] {
    let digest: [u8; 32] = Keccak256::digest(preimage).into();
    [digest[0], digest[1], digest[2], digest[3]]
}

#[test]
fn deposit_selector_matches_canonical_keccak() {
    assert_eq!(
        IWrappedNativeToken::depositCall::SELECTOR,
        keccak4(b"deposit()"),
        "deposit() selector must equal keccak256(\"deposit()\")[..4]",
    );
    assert_eq!(
        IWrappedNativeToken::depositCall::SELECTOR,
        [0xd0, 0xe3, 0x0d, 0xb0]
    );
    assert_eq!(IWrappedNativeToken::depositCall::SIGNATURE, "deposit()");
}

#[test]
fn withdraw_selector_matches_canonical_keccak() {
    assert_eq!(
        IWrappedNativeToken::withdrawCall::SELECTOR,
        keccak4(b"withdraw(uint256)"),
        "withdraw(uint256) selector must equal keccak256(\"withdraw(uint256)\")[..4]",
    );
    assert_eq!(
        IWrappedNativeToken::withdrawCall::SELECTOR,
        [0x2e, 0x1a, 0x7d, 0x4d]
    );
    assert_eq!(
        IWrappedNativeToken::withdrawCall::SIGNATURE,
        "withdraw(uint256)"
    );
}

#[test]
fn wrap_interaction_calls_deposit_with_amount_as_value() {
    let weth = Address::from_bytes([0xc0; 20]);
    let amount = Amount::new("500000000000000000").unwrap();

    let interaction = wrap_interaction(weth, amount);

    assert_eq!(interaction.target, weth);
    assert_eq!(
        interaction.value, amount,
        "wrap value must equal the wrapped amount"
    );
    assert_eq!(
        &interaction.call_data[..],
        &[0xd0u8, 0xe3, 0x0d, 0xb0][..],
        "wrap calldata is the bare deposit() selector",
    );
}

#[test]
fn unwrap_interaction_calls_withdraw_with_zero_value() {
    let weth = Address::from_bytes([0xc0; 20]);
    let amount = Amount::new("500000000000000000").unwrap();

    let interaction = unwrap_interaction(weth, amount);

    assert_eq!(interaction.target, weth);
    assert_eq!(
        interaction.value,
        Amount::ZERO,
        "unwrap attaches zero native value"
    );
    assert_eq!(
        interaction.call_data.len(),
        4 + 32,
        "unwrap calldata is the withdraw selector plus one uint256 word",
    );
    assert_eq!(
        &interaction.call_data[..4],
        &[0x2e_u8, 0x1a, 0x7d, 0x4d][..]
    );
}
