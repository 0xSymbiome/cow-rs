# `fuzz_settlement_settle_encode` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_settlement_settle_encode.rs`.
The target interprets the seed bytes through `arbitrary` into bounded
settlement token, price, trade, and interaction vectors.

Seed classes:

- canonical: `seed-00-canonical-settlement.bin` is derived from
  `parity/fixtures/contracts.json::contracts-settlement-free-filled-amount-storage-calldata`
  by using the fixture's packed order UID and dynamic-array shape bytes
  as the structured-input prefix.
- boundary: `seed-01-empty-lists.bin`, `seed-02-single-trade.bin`, and
  `seed-03-capped-lists.bin` exercise empty, single-element, and
  cap-near vector shapes for the bounded encoder input.
- adversarial: `seed-04-mixed-interactions.bin` is derived from the
  settlement encoder upstream interaction tests and perturbs all three
  interaction stages in one seed.
- canonical (retained from prior smoke runs): `seed-happy.bin` is a
  bounded arbitrary-derived settlement payload kept from the original
  corpus seeding pass so coverage discovered before the structured
  rename to `seed-NN-class-description.bin` is preserved.

