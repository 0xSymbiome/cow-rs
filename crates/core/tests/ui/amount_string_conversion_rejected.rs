//! Compile-fail witness: the typed amount surface exposes no infallible
//! `From<String>` or `From<&str>` conversion, so a raw string can never
//! become an `Amount` through `.into()`. Construction must go through the
//! fallible `Amount::new` / `Amount::parse_units` boundary, which fails
//! closed on malformed input rather than accepting it and deferring
//! validation. The captured diagnostic snapshot alongside this file pins
//! the contract; adding a `From<String>` or `From<&str>` impl would change
//! the diagnostic and fail the harness.

use cow_sdk_core::Amount;

fn main() {
    let _: Amount = String::from("1").into();
    let _: Amount = "1".into();
}
