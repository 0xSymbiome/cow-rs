# `fuzz_recoverable_signature_parse_hex` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_recoverable_signature_parse_hex.rs`.
Each seed is converted to a UTF-8 string via `String::from_utf8_lossy`
and handed to `cow_sdk_contracts::RecoverableSignature::parse_hex`. The
canonical seeds mirror the four accepted recovery bytes documented in
`parity/fixtures/ecdsa/v_normalization.json` and
`parity/fixtures/signing.json::signing-ecdsa-v-normalization`.

## Seeds

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-canonical-v-27.bin` | canonical | Canonical 65-byte hex signature with `v = 0x1b`, mirroring the first positive case of `signing-ecdsa-v-normalization`. |
| `seed-01-canonical-v-28.bin` | canonical | Canonical 65-byte hex signature with `v = 0x1c`, mirroring the second positive case. |
| `seed-02-canonical-v-0.bin` | canonical | 65-byte hex signature with `v = 0x00`, exercising the EIP-2 to legacy canonicalisation path. |
| `seed-03-canonical-v-1.bin` | canonical | 65-byte hex signature with `v = 0x01`, exercising the second EIP-2 to legacy canonicalisation path. |
| `seed-04-boundary-empty.bin` | boundary | Empty input, exercising the empty-string rejection path. |
| `seed-05-boundary-short.bin` | boundary | Short hex payload `0x1234`, exercising the length-mismatch rejection. |
| `seed-06-adversarial-mixed-case.bin` | adversarial | Mixed-case hex body with `0X` prefix and lowercase `v = 0x1b`, exercising the case-insensitive accepted path. |
| `seed-07-adversarial-invalid-v.bin` | adversarial | 65-byte hex with `v = 0x02`, the documented rejected recovery byte from `signing-ecdsa-v-normalization`. |

## Discovered-corpus seeds

70 forty-character hex-named seeds retained from prior libFuzzer smoke
runs, with the historical leading discriminant byte stripped so each
seed remains a valid arbitrary-byte input to the focused parse-hex
target. Each is treated as adversarial-class coverage and kept so the
typestate parser preserves any input invariants the prior fuzz sessions
exercised.
