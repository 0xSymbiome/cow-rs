#![no_main]

//! Fuzz target for the `OrderUid` pack / extract round-trip.
//!
//! **Property:** `PROP-CON-007`.
//! The target takes arbitrary bytes, maps the first 56 bytes onto the
//! documented `OrderUid` layout (32-byte digest, 20-byte owner, 4-byte
//! big-endian `valid_to`), packs the triple through
//! [`pack_order_uid_params`], extracts the triple again through
//! [`extract_order_uid_params`], and asserts the extracted triple matches
//! the input triple. Shorter inputs return early instead of panicking so
//! the fuzzer itself stays alive.

use cow_sdk_contracts::order::{
    OrderUidParams, extract_order_uid_params, pack_order_uid_params,
};
use cow_sdk_core::{Address, OrderDigest};
use libfuzzer_sys::fuzz_target;

/// Minimum input length accepted by the target: 32-byte digest, 20-byte
/// owner, 4-byte big-endian `valid_to`.
const MIN_INPUT_LEN: usize = 32 + 20 + 4;

fuzz_target!(|data: &[u8]| {
    if data.len() < MIN_INPUT_LEN {
        return;
    }

    let mut digest_bytes = [0u8; 32];
    digest_bytes.copy_from_slice(&data[..32]);
    let mut owner_bytes = [0u8; 20];
    owner_bytes.copy_from_slice(&data[32..52]);
    let mut valid_to_bytes = [0u8; 4];
    valid_to_bytes.copy_from_slice(&data[52..56]);
    let valid_to = u32::from_be_bytes(valid_to_bytes);

    let order_digest = OrderDigest::from_bytes(digest_bytes);
    let owner = Address::from_bytes(owner_bytes);

    let params = OrderUidParams::new(order_digest.clone(), owner.clone(), valid_to);

    let uid = pack_order_uid_params(&params)
        .expect("pack_order_uid_params must accept hex-typed inputs from `from_bytes`");
    let extracted = extract_order_uid_params(&uid)
        .expect("extract_order_uid_params must round-trip a just-packed UID");

    assert_eq!(
        extracted.order_digest, order_digest,
        "extracted order_digest must equal the packed digest",
    );
    assert_eq!(
        extracted.owner, owner,
        "extracted owner must equal the packed owner",
    );
    assert_eq!(
        extracted.valid_to, valid_to,
        "extracted valid_to must equal the packed valid_to",
    );
});
