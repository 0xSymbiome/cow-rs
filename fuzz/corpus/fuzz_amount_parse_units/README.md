# `fuzz_amount_parse_units` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_amount_parse_units.rs`. The
target derives a structured `(value, decimals)` pair from arbitrary bytes
and runs it through `cow_sdk_core::Amount::parse_units`, asserting no
panic on any input, deterministic results on identical input, and that
every accepted value round-trips through `cow_sdk_core::Amount::format_units`
back to the originating typed amount.

Each seed file is the byte layout consumed by
`libfuzzer_sys::arbitrary::Arbitrary`: 1 byte of `decimals` (as `u8`),
then the remaining bytes as the candidate decimal string.

Seed classes:

- canonical: `seed-canonical-one.bin` (`"1"` at 0 decimals),
  `seed-canonical-one-eth.bin` (`"1"` at 18 decimals, one ether), and
  `seed-canonical-one-and-half.bin` (`"1.5"` at 18 decimals) cover the
  everyday integer, whole-token, and fractional ERC-20 cases.
- boundary: `seed-boundary-smallest-wei.bin` carries
  `"0.000000000000000001"` at 18 decimals, the smallest representable
  one-wei magnitude.
- adversarial: `seed-adversarial-empty.bin` and
  `seed-adversarial-negative.bin` carry inputs the constructor must
  reject without panicking: the empty payload and a leading-sign
  literal.

All seed files are intentionally tiny and platform-neutral.
