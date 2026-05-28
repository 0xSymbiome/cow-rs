# `fuzz_eip1271_signature_data_codec` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_eip1271_signature_data_codec.rs`.
Each seed is interpreted as a 20-byte verifier prefix followed by up
to 256 bytes of signature payload, then round-tripped through
`cow_sdk_contracts::encode_eip1271_signature_data` and
`cow_sdk_contracts::decode_eip1271_signature_data`. The canonical seeds
mirror the EIP-1271 payload shapes documented alongside
`signing-eip1271-encode-payload` in `parity/fixtures/signing.json`.

## Seeds

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-empty-sig.bin` | boundary | 20-byte verifier and empty signature payload, exercising the minimum-length EIP-1271 round-trip. |
| `seed-01-typical-sig.bin` | canonical | 20-byte verifier and a 65-byte signature payload, exercising a populated EIP-1271 encode/decode round-trip. |

## Discovered-corpus seeds

41 forty-character hex-named seeds retained from prior libFuzzer smoke
runs, with the historical leading discriminant byte stripped so each
seed remains a valid arbitrary-byte input to the focused codec target.
Each is treated as adversarial-class coverage and kept so the codec
preserves any round-trip invariants the prior fuzz sessions exercised.
