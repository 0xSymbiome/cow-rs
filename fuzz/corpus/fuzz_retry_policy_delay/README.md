# `fuzz_retry_policy_delay` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_retry_policy_delay.rs`. The
target maps arbitrary bytes through `Arbitrary` into a typed
`RetryInput` containing an attempt index, base/max delay, jitter tag,
seed, status, and a header-selector byte; each seed file is a raw
byte string that drives one or more documented retry shapes.

Seed sources:

- canonical: `seed-canonical-00-default-backoff.bin` carries the
  default-backoff shape (50 ms base, 3200 ms cap) referenced by the
  orderbook retry policy parity contract (`parity/fixtures/orderbook.json`
  id `orderbook-request-helper-policy`).
- canonical: `seed-canonical-01-retry-after-delta.bin` exercises the
  `Retry-After: 120` delta-seconds branch through `delay_for_status`.
- boundary: `seed-boundary-02-attempt-zero.bin` is the zero attempt
  index that must trigger the documented saturating-shift fast path.
- boundary: `seed-boundary-03-large-attempt.bin` is a high attempt
  index that must remain bounded by `max_delay`.
- adversarial: `seed-adversarial-04-malformed-retry-after.bin` is a
  malformed `Retry-After: not-a-number` header that must fall back to
  the backoff delay without panicking.
- adversarial: `seed-adversarial-05-imf-fixdate.bin` carries the
  IMF-fixdate `Retry-After` branch that exercises the date parser.
- adversarial: `seed-adversarial-06-mixed-noise.bin` is a noisy
  arbitrary byte pattern that perturbs every typed field.
