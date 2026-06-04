# `fuzz_amount_parse` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_amount_parse.rs`. The target
maps raw bytes through `String::from_utf8_lossy` into
`cow_sdk_core::Amount::new` and the serde `Deserialize` path, then
asserts no panic, canonical-string round-trip stability, the documented
`uint256` bit-width boundary, decimal/hex parsing equivalence, and
determinism on identical input.

Seed classes:

- canonical: `seed-canonical-zero.bin` and
  `seed-canonical-one-eth.bin` are derived from the `Amount`
  parse/round-trip contract pinned by
  `crates/core/tests/types_contract.rs` and
  `crates/sdk/tests/amount_roundtrip.rs` as canonical
  zero and `1e18` (one ether in atoms) representative amounts.
  `seed-canonical-hex.bin` carries the `0x`-hex literal form of the same
  one-ether amount.
- boundary: `seed-boundary-uint256-max.bin` carries the canonical
  `uint256` maximum literal, and
  `seed-boundary-uint256-overflow.bin` carries the next integer beyond
  the boundary so the rejection path is covered.
- adversarial: `seed-adversarial-negative.bin`,
  `seed-adversarial-whitespace.bin`, and `seed-adversarial-empty.bin`
  carry inputs that the parser must reject without panicking: a
  signed-decimal literal, a whitespace-padded decimal literal, and the
  empty payload.

All seed files are intentionally tiny and platform-neutral.
