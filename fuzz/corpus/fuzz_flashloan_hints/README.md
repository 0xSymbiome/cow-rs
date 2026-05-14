# `fuzz_flashloan_hints` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_flashloan_hints.rs`. Each seed
is consumed as raw bytes that the target parses as a JSON value through
`serde_json::from_slice::<Value>` before driving
`serde_json::from_value::<FlashloanHints>`. The structured-input width
is capped through `MAX_FUZZ_INPUT = 4096`.

Seed classes:

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-canonical-full.json` | canonical | A representative five-field flash-loan hint with non-zero addresses and a positive amount, derived from the schema validation case pinned by `parity/fixtures/app-data.json::app-data-validation-contract`. |
| `seed-01-canonical-large-amount.json` | canonical | A second representative value covering a large decimal amount string (`10**18`) that still parses through the typed `Amount` constructor. |
| `seed-02-boundary-zero-amount.json` | boundary | A payload with `amount = "0"`; the derived shape-only deserializer accepts the value, and `validate()` returns the expected zero-amount `Err`. Exercises the parse-then-validate split. |
| `seed-03-boundary-zero-address.json` | boundary | A payload with `liquidityProvider = 0x00..00`; the shape-only deserializer accepts the value, and `validate()` returns the expected zero-address `Err`. Exercises the parse-then-validate split. |
| `seed-04-adversarial-unknown-field.json` | adversarial | A canonical payload with an extra `__fuzz_extra_field` top-level key; the `deny_unknown_fields` attribute must reject the value. |
| `seed-05-adversarial-missing-field.json` | adversarial | A payload missing the required `token` field; serde must reject the value at the missing-field guard. |
| `seed-06-adversarial-non-json.bin` | adversarial | Plain ASCII text that is not parseable as JSON; exercises the early-return `from_slice` path. |
