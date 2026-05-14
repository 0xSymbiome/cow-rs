# `fuzz_slippage_amounts` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_slippage_amounts.rs`. The
target maps arbitrary bytes through `Arbitrary` into a typed
`SlippageInput` (slippage bps, partner bps, protocol-fee tag, sell /
buy / fee amounts, validity, kind flag) and drives
`calculate_quote_amounts_and_costs`.

Seed sources:

- canonical: `seed-canonical-00-sell.bin` carries a sell-sided quote
  shape consistent with the documented slippage parity contract
  (`parity/fixtures/trading.json` id `trading-slippage-helper-bounds`).
- canonical: `seed-canonical-01-buy.bin` carries a buy-sided quote
  shape with the same parity bounds.
- boundary: `seed-boundary-02-zero-fee.bin` exercises the documented
  zero-network-cost branch of the calculator.
- boundary: `seed-boundary-03-max-slippage.bin` exercises the
  `MAX_SLIPPAGE_BPS` clamp boundary (10000 bps).
- adversarial: `seed-adversarial-04-large-partner-fee.bin` covers a
  partner-fee value near the documented monotone bound that exercises
  the truncating integer math.
- adversarial: `seed-adversarial-05-large-protocol-fee.bin` covers a
  protocol-fee f64 value near the documented sanitization cap and
  exercises the protocol-fee math overflow guard.
- adversarial: `seed-adversarial-06-extreme-values.bin` is a noisy
  byte pattern that pushes the sell, buy, and fee amounts toward the
  uint256 boundary.
