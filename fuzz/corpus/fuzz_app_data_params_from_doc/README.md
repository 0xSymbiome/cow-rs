# `fuzz_app_data_params_from_doc` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_app_data_params_from_doc.rs`.
The first byte selects a deterministic fixture class inside the target;
trailing bytes remain available to perturb the bounded arbitrary
generators that build the `(serde_json::Value, AppDataParams)` pair the
target merges through `merge_and_seal_app_data`.

Seed classes:

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-canonical-empty-base.bin` | canonical | Discriminant `0x00`; empty base document plus default override. Derived from the typed-merge fixture pinned by `parity/fixtures/trading.json::trading-quote-app-data-enrichment` for the empty-base canonical class. |
| `seed-01-canonical-populated-base.bin` | canonical | Discriminant `0x01`; populated base document with quote and order-class metadata plus a non-empty metadata override. |
| `seed-02-boundary-fully-typed-base.bin` | boundary | Discriminant `0x02`; populated base carrying signer, partnerFee, hooks, and flashloan plus a hooks-only override. Exercises the documented hooks-replacement rule. |
| `seed-03-adversarial-base-hooks-only.bin` | boundary | Discriminant `0x03`; populated base carrying only `metadata.hooks` plus a `metadata.hooks` override; exercises the metadata-hooks replacement branch. |
| `seed-04-adversarial-signer-only-override.bin` | adversarial | Discriminant `0x04`; empty base plus a signer-only override. Derived from the partner-fee-in-app-data fixture pinned by `parity/fixtures/trading.json::trading-partner-fee-in-app-data` for the override-merge surface. |
| `seed-05-adversarial-arbitrary.bin` | adversarial | Discriminant `0x09`; routes into the fully arbitrary base + override path so libFuzzer can explore the bounded-Arbitrary generators. |
