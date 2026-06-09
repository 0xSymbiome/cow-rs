//! Shared fixture helpers for the cow-shed parity-contract test suite.
//!
//! The reference vectors are JSON fixtures whose scalar fields arrive as
//! strings; these helpers decode them into the typed primitives the contract
//! assertions compare against. Each parity test binary imports the subset of
//! helpers its own vectors need.

#![allow(
    dead_code,
    reason = "each parity-contract test binary imports this shared module but uses only the subset of fixture helpers its own vectors need"
)]

use alloy_primitives::{Address, B256, Bytes, U256};
use cow_sdk_contracts::cow_shed::CowShedVersion;

/// Parses a fixture address string (`0x`-hex) through `Address`'s `FromStr`.
pub fn address(value: &str) -> Address {
    value.parse().expect("fixture address parses")
}

/// Decodes a fixture 32-byte hash string (`0x`-hex) into a `B256`.
pub fn b256(value: &str) -> B256 {
    let bytes =
        alloy_primitives::hex::decode(value.trim_start_matches("0x")).expect("fixture hash parses");
    let mut out = [0_u8; 32];
    out.copy_from_slice(&bytes);
    B256::from(out)
}

/// Decodes a fixture byte string (`0x`-hex) into `Bytes`.
pub fn bytes(value: &str) -> Bytes {
    Bytes::from(
        alloy_primitives::hex::decode(value.trim_start_matches("0x")).expect("fixture hex parses"),
    )
}

/// Parses a decimal fixture integer (fits in `u64`) into a `U256`.
pub fn decimal_u256(value: &str) -> U256 {
    U256::from(value.parse::<u64>().expect("fixture integer fits u64"))
}

/// Maps a fixture version string to the typed [`CowShedVersion`].
pub fn parse_version(value: &str) -> CowShedVersion {
    match value {
        "1.0.0" => CowShedVersion::V1_0_0,
        "1.0.1" => CowShedVersion::V1_0_1,
        other => panic!("unsupported fixture version {other}"),
    }
}
