# `fuzz_amount_from_units` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_amount_from_units.rs`. The target
derives a structured `(whole, decimals)` pair from arbitrary bytes and runs
it through `cow_sdk_core::Amount::from_units`, asserting no panic on any
input, deterministic results on identical input, agreement with
`cow_sdk_core::Amount::parse_units` applied to the same whole number, and
that every accepted value round-trips through
`cow_sdk_core::Amount::format_units` back to the originating typed amount.

Each seed file is the byte layout the target consumes: 1 byte of `decimals`
(as `u8`), then up to sixteen bytes decoded little-endian into the `u128`
whole-unit count.

Seed classes:

- canonical: `seed-canonical-one-eth.bin` (whole `1` at 18 decimals, one
  ether) and `seed-canonical-thousand-usdc.bin` (whole `1000` at 6 decimals)
  cover the everyday whole-token construction cases.
- boundary: `seed-boundary-zero.bin` carries whole `0` at 18 decimals (the
  zero amount); `seed-boundary-max-decimals.bin` carries whole `1` at 77
  decimals (`alloy_primitives::utils::Unit::MAX`).
- adversarial: `seed-adversarial-decimals-oob.bin` (whole `1` at 78
  decimals) and `seed-adversarial-overflow.bin` (`u128::MAX` whole units at
  77 decimals) carry inputs the constructor must reject without panicking:
  the out-of-range scale and the over-`uint256` overflow.

All seed files are intentionally tiny and platform-neutral.
