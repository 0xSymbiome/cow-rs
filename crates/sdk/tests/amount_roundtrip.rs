//! Atomic-unit `Amount` round-trip contract.
//!
//! The cow `Amount` newtype is `#[repr(transparent)]` over
//! `alloy_primitives::U256` (ADR 0052) with cow-owned serde that preserves the
//! decimal-string wire format. This pins the core invariant directly: a
//! canonical decimal atomic-unit string parses through `Amount::new` and renders
//! back byte-identically across representative magnitudes — including zero and
//! the full uint256 ceiling — and the parse is deterministic.

use cow_sdk_core::Amount;

#[test]
fn amount_strings_round_trip_byte_identically() {
    for raw in [
        "0",
        "1",
        "1000000",
        "1000000000000000000",
        "30000000000000000000",
        "98646335338956442",
        // uint256 ceiling
        "115792089237316195423570985008687907853269984665640564039457584007913129639935",
    ] {
        let amount = Amount::new(raw).unwrap_or_else(|err| panic!("{raw}: {err}"));
        assert_eq!(
            amount.to_string(),
            raw,
            "{raw}: amount string did not round-trip byte-identically",
        );
    }

    // The parse is deterministic: the same literal always decodes to the same
    // typed `Amount`, which compares its inner U256 bit-for-bit.
    assert_eq!(
        Amount::new("1000000000000000000").unwrap(),
        Amount::new("1000000000000000000").unwrap(),
    );
}
