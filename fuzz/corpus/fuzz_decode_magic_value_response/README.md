# `fuzz_decode_magic_value_response` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_decode_magic_value_response.rs`.
Each seed is a raw response-body candidate consumed by the stub
provider's `read_contract` return value, then routed through
`cow_sdk_contracts::verify_eip1271_signature`. The wrapper internally
calls the crate-private `decode_magic_value_response` decoder on the
response body, and the canonical magic value is pinned by
`parity/fixtures/contracts.json::contracts-eip1271-magic-value`
(`0x1626ba7e`).

## Seeds

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-canonical-magic-value.bin` | canonical | Raw hex string `0x1626ba7e` matching `contracts-eip1271-magic-value`; verifies the success path. |
| `seed-01-canonical-json-string.bin` | canonical | JSON-quoted form `"0x1626ba7e"`, exercising the JSON-string branch of the decoder. |
| `seed-02-canonical-uppercase.bin` | canonical | Mixed-case hex `0x1626BA7E`, exercising the case-insensitive accepted form. |
| `seed-03-boundary-json-number.bin` | boundary | JSON literal `42` (non-string), exercising the malformed-response branch for typed JSON values that are not strings. |
| `seed-04-boundary-empty.bin` | boundary | Empty body, exercising the empty-input malformed-response path. |
| `seed-05-boundary-flipped-byte.bin` | boundary | `0x1626ba7f` — single-bit mutation of the canonical magic value, exercising the `Eip1271MagicValueMismatch` branch. |
| `seed-06-adversarial-non-hex.bin` | adversarial | `0xZZZZZZZZ` — `0x` prefix with invalid hex characters, exercising the hex-decode failure mode of the malformed-response branch. |
