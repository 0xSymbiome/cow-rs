//! Compile-fail witness: the typed amount surface exposes no bare
//! arithmetic operators (`+` `-` `*` and the `*Assign` variants) and no
//! `pow`. Typed arithmetic is fallible-by-return (`checked_*` ->
//! `Option`) or an explicit `saturating_*` clamp, so a silent overflow
//! wrap — or a debug-only overflow panic — can never re-enter the
//! `Amount` / `SignedAmount` boundary. The captured diagnostic snapshot
//! alongside this file pins the contract; re-introducing any operator
//! would change the diagnostic and fail the harness.

use cow_sdk_core::{Amount, SignedAmount};

fn main() {
    let a = Amount::ZERO;
    let b = Amount::ZERO;
    let _ = a - b;
    let _ = a + b;
    let _ = a * b;
    let _ = a.pow(&b);

    let s = SignedAmount::ZERO;
    let t = SignedAmount::ZERO;
    let _ = s - t;
    let _ = s.pow(&Amount::ZERO);
}
