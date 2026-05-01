# `fuzz_transport_error_classify` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_transport_error_classify.rs`.
The first byte selects a deterministic transport fixture class inside
the target; trailing bytes remain available to perturb arbitrary status,
body, and header values.

Seed classes:

- canonical: `seed-class-00-timeout.bin` through
  `seed-class-08-other.bin` cover every documented
  `TransportErrorClass` variant from `cow-sdk-core`.
- boundary: `seed-boundary-09-retry-after-negative.bin`,
  `seed-boundary-10-retry-after-nan.bin`,
  `seed-boundary-11-retry-after-http-date.bin`, and
  `seed-boundary-12-retry-after-extreme.bin` cover malformed or extreme
  `Retry-After` header shapes.
- adversarial: `seed-adversarial-13-url-credentials.bin` covers a body
  snippet containing URL userinfo and credential query material, while
  `seed-adversarial-14-json-rpc-error.bin` covers a JSON-RPC error body
  with credential-shaped message content.

