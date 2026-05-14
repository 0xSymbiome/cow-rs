# `fuzz_jitter_delay_for_attempt` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_jitter_delay_for_attempt.rs`.
The target maps arbitrary bytes through `Arbitrary` into a typed
`JitterInput` containing a seed, attempt index, base/max delay, and a
strategy tag; each seed file is a raw byte string that drives one of
the documented jitter shapes.

Seed sources:

- canonical: `seed-canonical-00-decorrelated-default.bin` carries the
  decorrelated jitter shape exercised by the `RetryPolicy` default and
  the parity contract for orderbook retry behavior
  (`parity/fixtures/orderbook.json` id `orderbook-request-helper-policy`).
- canonical: `seed-canonical-01-full-jitter.bin` exercises the
  full-jitter variant with a documented 50 ms base and 3200 ms cap.
- boundary: `seed-boundary-02-zero-base.bin` is a zero base delay that
  must collapse to a zero offset without panicking.
- boundary: `seed-boundary-03-base-equals-max.bin` is the boundary
  shape where the base delay equals the cap, so the offset window
  collapses.
- adversarial: `seed-adversarial-04-large-attempt.bin` is a high
  attempt index that exercises the saturating-shift path through
  `splitmix64`.
- adversarial: `seed-adversarial-05-equal-jitter-noise.bin` perturbs
  the equal-jitter variant with a noisy seed.
- adversarial: `seed-adversarial-06-extreme-window.bin` covers a base
  delay near the documented soft cap and a small cap, asserting the
  documented clamp-to-max branch.
