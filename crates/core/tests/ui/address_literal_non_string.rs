//! Compile-fail witness: the `address!` macro accepts only a string
//! literal, so a numeric literal cannot masquerade as an address.

use cow_sdk_core::address;

fn main() {
    let _ = address!(42);
}
