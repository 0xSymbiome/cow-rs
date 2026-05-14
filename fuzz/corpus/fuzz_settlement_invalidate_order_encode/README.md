# `fuzz_settlement_invalidate_order_encode` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_settlement_invalidate_order_encode.rs`.
The target consumes raw bytes directly as the `invalidateOrder(bytes)`
payload.

Seed classes:

- canonical: `seed-00-canonical-order-uid.bin` is the 56-byte order UID
  from `parity/fixtures/contracts.json::contracts-settlement-invalidate-order-calldata`.
- boundary: `seed-01-empty.bin`, `seed-02-short-55.bin`,
  `seed-03-exact-56.bin`, and `seed-04-long-57.bin` cover the empty,
  off-by-one, exact-width, and one-byte-over order UID widths.
- adversarial: `seed-05-all-ff.bin` stresses the dynamic-bytes encoder
  with an all-`0xff` order UID shaped payload.
- canonical (retained from prior smoke runs): `seed-happy.bin` is an
  early-corpus 56-byte order-UID-shaped payload kept from the original
  seeding pass so coverage discovered before the structured rename to
  `seed-NN-class-description.bin` is preserved.

