# `fuzz_redact_response_body` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_redact_response_body.rs`. The
target maps raw bytes through `String::from_utf8_lossy` into
`cow_sdk_core::redact_response_body` and asserts the output is bounded in
length, deterministic on repeated calls, valid UTF-8, and free of
credential-shaped substrings (URL userinfo, JWT-shaped tokens, and
`key=secret` material).

Seed classes:

- canonical: `seed-canonical-empty.bin` (anchored to the
  `redact_response_body` contract pinned by
  `crates/core/tests/redaction_contract.rs` as the canonical
  empty-body baseline) and
  `seed-canonical-short-json.bin` cover the empty and short typed
  diagnostic bodies that the redactor is expected to pass through
  unchanged.
- boundary: `seed-boundary-256-bytes.bin` and
  `seed-boundary-512-bytes.bin` exercise the
  `REDACTED_RESPONSE_BODY_MAX_BYTES` length cap and the truncation
  marker append path.
- adversarial: `seed-adversarial-jwt.bin`,
  `seed-adversarial-userinfo-url.bin`, and
  `seed-adversarial-apikey-token.bin` carry JWT-shaped tokens, URL
  userinfo, and case-mixed credential `key=value` snippets that exercise
  every documented redaction class.

All seed files are intentionally tiny and platform-neutral.
