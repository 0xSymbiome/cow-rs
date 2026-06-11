//! Compile-fail witness: the `address!` macro requires exactly one
//! string literal; the zero address is spelled `Address::ZERO`, not an
//! empty invocation.

use cow_sdk_core::address;

fn main() {
    let _ = address!();
}
