# `fuzz_slippage_policy_helpers` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_slippage_policy_helpers.rs`.
The target maps arbitrary bytes through `Arbitrary` into a typed
`PolicyInput` carrying a candidate protocol-fee bps string, fee amount,
multiplier tag, volume amounts, slippage tag, and sell-flag, then walks
`sanitize_protocol_fee_bps`, `suggest_slippage_from_fee`, and
`suggest_slippage_from_volume`.

Seed sources:

- canonical: `seed-canonical-00-default-sell.bin` carries the
  documented sell-sided slippage shape from the parity contract
  (`parity/fixtures/trading.json` id `trading-slippage-helper-bounds`).
- canonical: `seed-canonical-01-default-buy.bin` carries the buy-sided
  shape from the same parity contract.
- boundary: `seed-boundary-02-zero-fee.bin` exercises the zero-fee
  branch of `suggest_slippage_from_fee`.
- boundary: `seed-boundary-03-equal-volume.bin` exercises the boundary
  where the pre- and post-network-cost volumes match.
- adversarial: `seed-adversarial-04-nan-multiplier.bin` exercises the
  NaN multiplier rejection branch.
- adversarial: `seed-adversarial-05-negative-fee.bin` exercises the
  negative-fee rejection branch.
- adversarial: `seed-adversarial-06-malformed-numeric.bin` is a
  malformed numeric quantity that must be rejected without panicking.
