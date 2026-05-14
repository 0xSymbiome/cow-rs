# `fuzz_normalize_ecdsa_signature_str` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_normalize_ecdsa_signature_str.rs`.
Each seed begins with a discriminant byte: even discriminants drive
`cow_sdk_contracts::normalized_ecdsa_signature(&str)` with the trailing
bytes converted via `String::from_utf8_lossy`, and odd discriminants
build an `Eip1271SignatureData` from a 20-byte verifier and up to 256
bytes of signature, then round-trip it through
`encode_eip1271_signature_data` and `decode_eip1271_signature_data`.
The canonical seeds mirror the four accepted recovery bytes documented
in `parity/fixtures/signing.json::signing-ecdsa-v-normalization`.

## Seeds

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-canonical-v-27.bin` | canonical | Even discriminant + canonical 65-byte hex signature with `v = 0x1b`, mirroring the first positive case of `signing-ecdsa-v-normalization`. |
| `seed-01-canonical-v-28.bin` | canonical | Even discriminant + canonical 65-byte hex signature with `v = 0x1c`, mirroring the second positive case of `signing-ecdsa-v-normalization`. |
| `seed-02-canonical-v-0.bin` | canonical | Even discriminant + 65-byte hex signature with `v = 0x00`, exercising the EIP-2 to legacy normalization path. |
| `seed-03-canonical-v-1.bin` | canonical | Even discriminant + 65-byte hex signature with `v = 0x01`, exercising the second EIP-2 to legacy normalization path. |
| `seed-04-boundary-empty.bin` | boundary | Just the discriminant byte, exercising the empty-input rejection path. |
| `seed-05-boundary-short.bin` | boundary | Discriminant + short hex payload `0x1234`, exercising the length-mismatch rejection. |
| `seed-06-adversarial-mixed-case.bin` | adversarial | Mixed-case hex body with `0X` prefix and lowercase `v = 0x1b`, exercising the case-insensitive accepted path. |
| `seed-07-adversarial-invalid-v.bin` | adversarial | 65-byte hex with `v = 0x02`, the documented rejected recovery byte from `signing-ecdsa-v-normalization`. |
| `seed-08-eip1271-empty-sig.bin` | boundary | Odd discriminant + 20-byte verifier and empty signature payload, exercising the minimum-length EIP-1271 round-trip. |
| `seed-09-eip1271-typical-sig.bin` | canonical | Odd discriminant + 20-byte verifier and a 65-byte signature payload, exercising a populated EIP-1271 encode/decode round-trip. |
