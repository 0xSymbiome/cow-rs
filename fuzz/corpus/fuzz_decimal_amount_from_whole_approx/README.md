# `fuzz_decimal_amount_from_whole_approx` Corpus

This corpus seeds
`fuzz/fuzz_targets/fuzz_decimal_amount_from_whole_approx.rs`. The target
derives a structured `(whole_units, decimals)` pair from arbitrary bytes
and runs it through `cow_sdk_core::DecimalAmount::from_whole_approx`,
asserting no panic, deterministic results on identical input, the
documented decimals-scale preservation, and the documented
zero-atoms-on-clamp behavior for NaN, infinite, and negative inputs.

Each seed file is the little-endian byte layout consumed by
`libfuzzer_sys::arbitrary::Arbitrary`: 8 bytes of `whole_units` (as
`f64`), then 1 byte of `decimals` (as `u8`).

Seed classes:

- canonical: `seed-canonical-zero.bin` and
  `seed-canonical-one-eth-18.bin` are derived from the
  `core-shared-order-and-quote-surfaces` fixture id in
  `parity/fixtures/core.json` as the canonical
  zero magnitude and one-ether-at-18-decimals representative case.
- boundary: `seed-boundary-large-magnitude.bin` and
  `seed-boundary-decimals-zero.bin` cover the documented
  `u128::MAX`-as-`f64` clamp ceiling and the zero-decimals path.
- adversarial: `seed-adversarial-nan.bin`,
  `seed-adversarial-negative.bin`, and `seed-adversarial-infinity.bin`
  exercise the documented zero-atoms-on-clamp path for NaN, negative,
  and infinite magnitudes.

All seed files are intentionally tiny and platform-neutral.
