# `fuzz_erc20_permit_typed_data_hash` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_erc20_permit_typed_data_hash.rs`.
The target interprets seed bytes through `arbitrary` into an EIP-712
domain and EIP-2612 permit payload.

Seed classes:

- canonical: `seed-00-canonical-usdc-permit.bin` is derived from
  `parity/fixtures/contracts.json::contracts-erc20-permit-typed-data-hash`
  using the USDC domain and permit fields.
- boundary: `seed-01-empty-domain.bin`, `seed-02-all-zero.bin`, and
  `seed-03-all-ff.bin` exercise empty optional domain fields and byte
  extremes across the permit tuple.
- adversarial: `seed-04-max-amounts.bin` stresses maximum `uint256`
  amount, nonce, and deadline byte patterns.
- canonical (retained from prior smoke runs): `seed-happy.bin` is a
  bounded arbitrary-derived permit payload kept from the original corpus
  seeding pass so coverage discovered before the structured rename to
  `seed-NN-class-description.bin` is preserved.

