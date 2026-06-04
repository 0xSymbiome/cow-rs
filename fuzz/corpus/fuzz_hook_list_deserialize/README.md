# `fuzz_hook_list_deserialize` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_hook_list_deserialize.rs`.
Each seed is consumed as raw bytes that the target parses as a JSON
value through `serde_json::from_slice::<Value>` before driving
`serde_json::from_value::<HookList>`. The structured-input width is
capped through `MAX_FUZZ_INPUT = 4096`.

Seed classes:

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-canonical-pre-post.json` | canonical | A representative pre/post envelope carrying one hook on each side, derived from the hooks schema validation contract pinned by `crates/app-data/tests/hooks_contract.rs`. |
| `seed-01-canonical-versioned.json` | canonical | A pre/post envelope with an explicit `version` string at the documented `0.2.0` release. |
| `seed-02-canonical-empty.json` | canonical | The empty `{}` shape; both `pre` and `post` default to empty vectors per the deserializer. |
| `seed-03-boundary-zero-gas.json` | boundary | A canonical envelope with `gasLimit = "0"`; exercises the lower end of the documented decimal-string `u64` range. |
| `seed-04-boundary-max-u64-gas.json` | boundary | A canonical envelope with `gasLimit = "18446744073709551615"` (`u64::MAX`); exercises the upper boundary. |
| `seed-05-adversarial-non-decimal-gas.json` | adversarial | A canonical envelope with a non-decimal `gasLimit` string (`"0xdeadbeef"`); the helper must reject it without panicking. |
| `seed-06-adversarial-unknown-field.json` | adversarial | A canonical envelope with an extra `__fuzz_extra_field` top-level key; the `deny_unknown_fields` attribute on `HookList` must reject the value. |
| `seed-07-adversarial-non-json.bin` | adversarial | Plain ASCII text that is not parseable as JSON; exercises the early-return `from_slice` path. |
