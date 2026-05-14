# `fuzz_ethflow_create_order_encode` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_ethflow_create_order_encode.rs`.
The target interprets seed bytes through `arbitrary` into an
`EthFlowOrderData` value.

Seed classes:

- canonical: `seed-00-canonical-ethflow.bin` is derived from
  `parity/fixtures/contracts.json::contracts-ethflow-create-order-calldata`
  by using the fixture's buy token, receiver, amounts, app-data, valid-to,
  fillability, and quote-id bytes.
- boundary: `seed-01-zero-amounts.bin`, `seed-02-max-valid-to.bin`, and
  `seed-03-all-zero.bin` exercise zero amount fields, `u32::MAX`
  validity, and all-zero field bytes.
- adversarial: `seed-04-all-ff.bin` stresses maximum byte patterns across
  address, amount, app-data, and quote-id fields.
- canonical (retained from prior smoke runs): `seed-happy.bin` is a
  bounded arbitrary-derived EthFlow payload kept from the original corpus
  seeding pass so coverage discovered before the structured rename to
  `seed-NN-class-description.bin` is preserved.

