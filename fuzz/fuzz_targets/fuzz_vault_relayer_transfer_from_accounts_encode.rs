#![no_main]

//! Fuzz target for `GPv2VaultRelayer.transferFromAccounts(Transfer[])`.
//!
//! **Property:** `PROP-CON-015`.
//! Drives arbitrary `Vec<Transfer>` slices (1 to [`MAX_TRANSFERS`]
//! entries) through the `alloy::sol!`-generated
//! `IGPv2VaultRelayer::transferFromAccountsCall` encoder and asserts:
//!
//! * The 4-byte selector prefix equals
//!   `keccak256("transferFromAccounts((address,address,uint256,uint8)[])")[0..4]`.
//! * The encoded call-data length equals
//!   `4 + 32 + 32 + n * 4 * 32` for `n` transfers (selector + dynamic
//!   offset + dynamic length + `n` static-tuple entries of four
//!   32-byte head words each).
//! * Encoding is panic-free on every arbitrary input.

use alloy_sol_types::{
    SolCall,
    private::{Address, U256},
    sol,
};
use libfuzzer_sys::{arbitrary::Arbitrary, fuzz_target};
use sha3::{Digest, Keccak256};

sol! {
    interface IGPv2VaultRelayer {
        struct Transfer {
            address account;
            address token;
            uint256 amount;
            uint8 balance;
        }

        function transferFromAccounts(Transfer[] transfers) external;
    }
}

const MAX_TRANSFERS: usize = 16;

#[derive(Debug, Arbitrary)]
struct FuzzTransfer {
    account: [u8; 20],
    token: [u8; 20],
    amount: u128,
    balance: u8,
}

#[derive(Debug, Arbitrary)]
struct FuzzInput {
    transfers: Vec<FuzzTransfer>,
}

fuzz_target!(|input: FuzzInput| {
    if input.transfers.is_empty() {
        return;
    }

    let transfers: Vec<IGPv2VaultRelayer::Transfer> = input
        .transfers
        .into_iter()
        .take(MAX_TRANSFERS)
        .map(|t| IGPv2VaultRelayer::Transfer {
            account: Address::from(t.account),
            token: Address::from(t.token),
            amount: U256::from(t.amount),
            balance: t.balance,
        })
        .collect();

    if transfers.is_empty() {
        return;
    }

    let n = transfers.len();

    let encoded = IGPv2VaultRelayer::transferFromAccountsCall { transfers }.abi_encode();

    let canonical_selector: [u8; 4] = {
        let signature = "transferFromAccounts((address,address,uint256,uint8)[])";
        let digest = Keccak256::digest(signature.as_bytes());
        [digest[0], digest[1], digest[2], digest[3]]
    };
    assert_eq!(
        &encoded[..4],
        &canonical_selector,
        "transferFromAccounts selector must match keccak256 of the canonical ABI signature",
    );

    let expected_len = 4 + 32 + 32 + n * 4 * 32;
    assert_eq!(
        encoded.len(),
        expected_len,
        "transferFromAccounts call-data must be selector + offset + length + n * 128 bytes",
    );
});
