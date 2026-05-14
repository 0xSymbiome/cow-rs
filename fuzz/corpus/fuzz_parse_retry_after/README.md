# `fuzz_parse_retry_after` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_parse_retry_after.rs`. The
target feeds the raw bytes through `String::from_utf8_lossy` directly
into `parse_retry_after`, so every seed is interpreted as a candidate
`Retry-After` header value.

Seed sources:

- canonical: `seed-canonical-00-delta-seconds.bin` carries the
  `120` delta-seconds value documented for transient-status backoff
  under the parity contract for the orderbook retry policy
  (`parity/fixtures/orderbook.json` id
  `orderbook-request-helper-policy`).
- canonical: `seed-canonical-01-imf-fixdate.bin` carries an
  RFC 7231 IMF-fixdate value (`Thu, 01 Jan 1970 00:00:10 GMT`) that
  exercises the date branch of the parser.
- boundary: `seed-boundary-02-empty.bin` is an empty header value
  that must be rejected without panic.
- boundary: `seed-boundary-03-zero.bin` is the literal `0` value
  that exercises the documented zero-delay branch.
- adversarial: `seed-adversarial-04-negative.bin` is `-1`, an
  invalid signed integer that must be rejected as `None`.
- adversarial: `seed-adversarial-05-nan.bin` is the literal `NaN`
  string, which must not slip through the digit-only fast path.
- adversarial: `seed-adversarial-06-oversized.bin` is a
  `999999999999999999999` value that exceeds `u64::MAX` and must be
  rejected without overflow.
- adversarial: `seed-adversarial-07-malformed-date.bin` is a
  partially formed IMF-fixdate (`Thu, 01 Jan` only) that must be
  rejected as `None`.
