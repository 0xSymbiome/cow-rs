//! Harness guarding the pinned compile-fail witness for the `wasm32`
//! missing-transport typestate on `SubgraphApiBuilder`.
//!
//! The witness source lives at
//! `tests/ui/builder_wasm32_missing_transport.rs` and its captured
//! diagnostic snapshot lives alongside as
//! `tests/ui/builder_wasm32_missing_transport.stderr`. Cargo's default
//! discovery only picks up `tests/*.rs`, so the witness source under
//! `tests/ui/` is NOT compiled by `cargo test --workspace`. This
//! harness runs on every platform and asserts both witness artifacts
//! remain in the tree with the expected invariants so drift is
//! caught at test time rather than in reviewer diffs.
//!
//! The compile failure the witness records is: on `wasm32`, the
//! convenience default-transport `SubgraphApiBuilder::build` impl is
//! gated on `#[cfg(not(target_arch = "wasm32"))]`, so reaching
//! `.build()` against `SubgraphApiBuilder<ChainIdSet, ApiKeySet,
//! TransportUnset>` is a type error — the browser consumer must
//! supply a `FetchTransport` (from `cow-sdk-transport-wasm`) via
//! [`SubgraphApiBuilder::transport`] before `.build()` becomes
//! reachable.

use std::path::Path;

const WITNESS_SOURCE: &str = "tests/ui/builder_wasm32_missing_transport.rs";
const WITNESS_STDERR: &str = "tests/ui/builder_wasm32_missing_transport.stderr";

#[test]
fn wasm32_missing_transport_witness_source_is_present() {
    assert!(
        Path::new(WITNESS_SOURCE).exists(),
        "wasm32 missing-transport compile-fail witness source must \
         remain pinned at {WITNESS_SOURCE}",
    );
    let body = std::fs::read_to_string(WITNESS_SOURCE)
        .expect("reading the pinned witness source must succeed");
    assert!(
        body.contains("#![cfg(target_arch = \"wasm32\")]"),
        "witness source must carry the wasm32-only cfg gate so the \
         compile failure is scoped to the browser target",
    );
    assert!(
        body.contains(".build()"),
        "witness source must still attempt to reach `.build()` \
         without `.transport(...)` so the intended compile error is \
         preserved",
    );
    assert!(
        !body.contains(".transport("),
        "witness source must NOT call `.transport(...)` — the whole \
         point is to prove the transportless build path does not \
         compile on wasm32",
    );
}

#[test]
fn wasm32_missing_transport_witness_stderr_captures_the_expected_error() {
    assert!(
        Path::new(WITNESS_STDERR).exists(),
        "wasm32 missing-transport compile-fail stderr snapshot must \
         remain pinned at {WITNESS_STDERR}",
    );
    let stderr = std::fs::read_to_string(WITNESS_STDERR)
        .expect("reading the pinned witness stderr must succeed");
    assert!(
        stderr.contains("no method named `build`"),
        "pinned stderr must record the `build` method-missing \
         diagnostic so reviewers can verify the intended shape",
    );
    assert!(
        stderr.contains("TransportUnset"),
        "pinned stderr must reference the `TransportUnset` typestate \
         marker to prove the compile error is scoped to the correct \
         builder state",
    );
}
