# `fuzz_orderbook_rejection_decode` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_orderbook_rejection_decode.rs`.
Each JSON seed is a services-style rejection envelope consumed directly
as raw response-body bytes.

Seed classes:

- canonical: `seed-tag-*.json` files are derived from
  `crates/orderbook/tests/rejection_contract.rs::every_known_services_tag_parses_to_its_typed_variant`
  by serializing each known `errorType` with the same
  `services-authoritative description` string.
- boundary: `seed-boundary-empty.bin`, `seed-boundary-malformed-json.bin`,
  `seed-boundary-missing-error-type.json`,
  `seed-boundary-nested-data-wrong-type.json`, and
  `seed-boundary-non-utf8.bin` cover empty, malformed, missing-field,
  wrong-data-shape, and non-UTF-8 response bodies.
- adversarial: `seed-adversarial-embedded-quote.json`,
  `seed-adversarial-duplicate-order-typo.json`, and
  `seed-adversarial-sell-amount-bad-data.json` are derived from the
  regression cases that preserve embedded descriptions and unknown-tag
  fallback behavior.

