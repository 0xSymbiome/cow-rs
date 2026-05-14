# `fuzz_valid_to_relative` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_valid_to_relative.rs`. The
target derives a structured `(now, duration)` pair from arbitrary bytes
and runs it through `cow_sdk_core::ValidTo::relative`, asserting no
panic, deterministic results on identical input, the documented `u32`
ceiling on accepted timestamps, and the
`[VALID_TO_MIN_RELATIVE_SECONDS, VALID_TO_MAX_RELATIVE_SECONDS]`
acceptance window.

Each seed file is the little-endian byte layout consumed by
`libfuzzer_sys::arbitrary::Arbitrary`: 8 bytes of `now`, then 8 bytes of
`duration`.

Seed classes:

- canonical: `seed-canonical-happy.bin` is derived from the
  `core-environment-defaults` fixture id in
  `parity/fixtures/core.json` as the canonical
  one-hour window anchored at a representative mainnet timestamp.
- boundary: `seed-boundary-min-duration.bin` and
  `seed-boundary-max-duration.bin` cover the inclusive
  `VALID_TO_MIN_RELATIVE_SECONDS` (30 seconds) and
  `VALID_TO_MAX_RELATIVE_SECONDS` (90 days) endpoints.
  `seed-boundary-now-overflow.bin` covers the `now = u64::MAX`
  saturation path.
- adversarial: `seed-adversarial-zero-duration.bin` and
  `seed-adversarial-over-max.bin` exercise the documented
  out-of-range rejection path for durations below the minimum and beyond
  the maximum window.

All seed files are intentionally tiny and platform-neutral.
