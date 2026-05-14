# `fuzz_vault_relayer_transfer_from_accounts_encode` Corpus

This corpus seeds
`fuzz/fuzz_targets/fuzz_vault_relayer_transfer_from_accounts_encode.rs`.
The target interprets seed bytes through `arbitrary` into a bounded list
of vault relayer transfer structs.

Seed classes:

- canonical: `seed-00-canonical-transfer.bin` is derived from
  `parity/fixtures/contracts.json::contracts-vault-relayer-transfer-from-accounts-calldata`
  by reusing the canonical account, token, amount, and balance bytes.
- boundary: `seed-01-empty-list.bin`, `seed-02-single-transfer.bin`, and
  `seed-03-capped-transfers.bin` exercise empty, single-transfer, and
  cap-near list shapes.
- adversarial: `seed-04-all-zero-transfer.bin` and
  `seed-05-all-ff-transfer.bin` stress all-zero and all-`0xff` field
  boundaries from the transfer tuple.
- canonical (retained from prior smoke runs): `seed-happy.bin` is a
  bounded arbitrary-derived transfer list payload kept from the original
  corpus seeding pass so coverage discovered before the structured rename
  to `seed-NN-class-description.bin` is preserved.

