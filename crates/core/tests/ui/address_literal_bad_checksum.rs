//! Compile-fail witness: a mixed-case literal whose EIP-55 checksum is
//! wrong can never become an `Address` constant. A checksum cannot be
//! verified during const evaluation, so the macro rejects every
//! mixed-case literal outright instead of accepting a checksummed-looking
//! one unchecked. This literal is the CoW vault relayer
//! (`0xC92E8bdf79f0507f65a392b0ab4667716BFE0110` in valid checksummed
//! form) with the leading `C` case-flipped, which breaks the checksum.
//! The accepted spelling is the lowercase wire form.

use cow_sdk_core::address;

fn main() {
    let _ = address!("0xc92E8bdf79f0507f65a392b0ab4667716BFE0110");
}
