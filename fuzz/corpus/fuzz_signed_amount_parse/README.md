# `fuzz_signed_amount_parse` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_signed_amount_parse.rs`. The
target maps raw bytes through `String::from_utf8_lossy` into
`cow_sdk_core::SignedAmount::new`, asserts no panic on any input, and
verifies that every accepted value round-trips through its canonical
decimal-string form deterministically.

Seed classes:

- canonical: `seed-canonical-zero.bin` and
  `seed-canonical-positive.bin` are derived from the `SignedAmount`
  parse/round-trip contract pinned by
  `crates/core/tests/types_contract.rs` as the canonical
  zero and a representative positive signed magnitude.
  `seed-canonical-negative.bin` covers the negative-decimal path.
- boundary: `seed-boundary-large-magnitude.bin` carries a magnitude well
  past `uint256` so the parser exercises arbitrary-precision storage.
- adversarial: `seed-adversarial-hex.bin` carries a `0x`-prefixed literal
  the parser must reject (signed amount is decimal-only),
  `seed-adversarial-whitespace.bin` carries leading whitespace, and
  `seed-adversarial-empty.bin` carries the empty payload.

All seed files are intentionally tiny and platform-neutral.
