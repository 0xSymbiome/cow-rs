# `fuzz_app_data_merge` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_app_data_merge.rs`. The first
byte selects a deterministic fixture class inside the target; trailing
bytes remain available to perturb the bounded arbitrary generators.

Seed classes:

- canonical: `seed-00-empty-base.bin`,
  `seed-01-deeply-nested-base.bin`, and
  `seed-02-populated-metadata.bin` are derived from
  `crates/trading/tests/app_data_merge_contract.rs` fixtures for empty,
  nested, and fully populated app-data merge inputs.
- boundary: `seed-08-key-collision.bin` covers base-plus-override key
  collision and object replacement boundaries.
- adversarial: `seed-03-signer-conflict.bin`,
  `seed-04-partner-fee-single.bin`, `seed-05-partner-fee-multiple.bin`,
  `seed-06-hooks-replacement.bin`, and `seed-07-flashloan.bin` are
  derived from the signer, partner-fee, hooks, and flash-loan regression
  cases in the typed merge contract.

