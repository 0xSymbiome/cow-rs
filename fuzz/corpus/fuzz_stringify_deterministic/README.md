# `fuzz_stringify_deterministic` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_stringify_deterministic.rs`.
Each seed is consumed as raw bytes that the target parses as a JSON
value through `serde_json::from_slice::<Value>` before driving
`stringify_deterministic`. The structured-input width is capped through
`MAX_FUZZ_INPUT = 4096`.

Seed classes:

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-canonical-app-data-doc.json` | canonical | A representative app-data document with multiple top-level keys derived from the canonical-info fixture pinned by `parity/fixtures/app-data.json::app-data-get-app-data-info-deterministic`. Exercises object key ordering. |
| `seed-01-canonical-nested.json` | canonical | A nested-object value with both keys and arrays; covers the recursive canonical renderer. |
| `seed-02-boundary-null.json` | boundary | The literal `null`; exercises the scalar path. |
| `seed-03-boundary-empty-object.json` | boundary | The empty `{}` value; exercises the zero-key object branch. |
| `seed-04-boundary-empty-array.json` | boundary | The empty `[]` value; exercises the zero-element array branch. |
| `seed-05-adversarial-unicode-escapes.json` | adversarial | A string carrying Unicode escape sequences and control characters; exercises the canonical string-escape path. |
| `seed-06-adversarial-deep-nesting.json` | adversarial | A deeply nested array structure; exercises the recursive renderer at depth. |
| `seed-07-adversarial-non-json.bin` | adversarial | Plain ASCII text that is not parseable as JSON; exercises the early-return `from_slice` path. |
