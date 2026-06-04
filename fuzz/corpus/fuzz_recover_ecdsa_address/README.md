# `fuzz_recover_ecdsa_address` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_recover_ecdsa_address.rs`.
Each binary seed follows the target's `arbitrary` layout:
32 digest bytes, 65 signature bytes, then a signing-scheme selector byte.

Seed classes:

- canonical: `seed-00-canonical-v27.bin` is derived from
  `parity/fixtures/ecdsa/v_normalization.json` (exercised by
  `crates/contracts/tests/v_normalization_contract.rs`) by taking
  the canonical `v = 27` signature shape and pairing it with a fixed
  digest.
- boundary: `seed-01-all-zero.bin` and `seed-02-all-ff.bin` exercise
  all-zero and all-`0xff` digest/signature boundaries.
- adversarial: `seed-03-rejected-v02.bin` and
  `seed-04-rejected-vff.bin` are derived from the fixture's rejected
  recovery-byte examples for `v = 2` and `v = 255`.
- canonical (retained from prior smoke runs): `seed` is an
  early-corpus 30-byte payload kept from the original seeding pass.
  Inputs shorter than the 98-byte `Arbitrary` budget cause the target to
  return early before the recover call, so this seed exercises only the
  short-input early-return path.

