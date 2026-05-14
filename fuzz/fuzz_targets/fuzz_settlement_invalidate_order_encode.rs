#![no_main]

//! Fuzz target for `GPv2Settlement.invalidateOrder(bytes)` ABI encoding.
//!
//! **Property:** `PROP-CON-014`.
//! Feeds arbitrary bytes through the `alloy::sol!`-generated
//! `IGPv2Settlement::invalidateOrderCall` encoder and asserts:
//!
//! * The 4-byte selector prefix equals
//!   `keccak256("invalidateOrder(bytes)")[0..4]`.
//! * The encoded call-data length equals `4 + 32 + 32 + padded(input_len)`,
//!   where `padded(n) = ceil(n / 32) * 32` covers the dynamic-bytes
//!   offset, the length prefix, and the 32-byte-aligned payload.
//! * Encoding is panic-free for every arbitrary input.
//!
//! Inputs are capped at [`MAX_FUZZ_INPUT`] so each run stays bounded
//! even when libFuzzer explores long adversarial payloads.

use alloy_sol_types::{
    SolCall,
    private::Bytes,
    sol,
};
use libfuzzer_sys::fuzz_target;
use sha3::{Digest, Keccak256};

sol! {
    interface IGPv2Settlement {
        function invalidateOrder(bytes orderUid) external;
    }
}

/// Maximum input width accepted by the target. The on-chain
/// `orderUid` is a fixed 56-byte payload; 4096 bytes is more than
/// enough to stress the dynamic-bytes encoder on oversized and
/// misaligned shapes while keeping each run bounded.
const MAX_FUZZ_INPUT: usize = 4096;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_FUZZ_INPUT)];

    let encoded = IGPv2Settlement::invalidateOrderCall {
        orderUid: Bytes::from(data.to_vec()),
    }
    .abi_encode();

    let canonical_selector: [u8; 4] = {
        let digest = Keccak256::digest(b"invalidateOrder(bytes)");
        [digest[0], digest[1], digest[2], digest[3]]
    };
    assert_eq!(
        &encoded[..4],
        &canonical_selector,
        "invalidateOrder(bytes) selector must match keccak256 of the canonical signature",
    );

    let padded_len = data.len().div_ceil(32) * 32;
    let expected = 4 + 32 + 32 + padded_len;
    assert_eq!(
        encoded.len(),
        expected,
        "invalidateOrder call-data must be selector + offset + length + padded(input)",
    );
});
