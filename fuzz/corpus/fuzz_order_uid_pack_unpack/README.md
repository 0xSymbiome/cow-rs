# `fuzz_order_uid_pack_unpack` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_order_uid_pack_unpack.rs`.
The target accepts raw bytes at the documented `MIN_INPUT_LEN = 56` gate
and maps them onto the `OrderUid` layout: 32-byte digest, 20-byte
owner, and 4-byte big-endian `valid_to`. The triple round-trips
through `pack_order_uid_params` and `extract_order_uid_params`.

## Named seeds

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-canonical-uid.bin` | canonical | 56-byte triple anchored to the order-UID layout pinned by `parity/fixtures/contracts.json::contracts-order-uid-length` — an incrementing 32-byte digest, a constant `0xaa` 20-byte owner, and `valid_to = 0x00001000`. |
| `seed-01-boundary-zero.bin` | boundary | 56 zero bytes — zero digest, zero owner, `valid_to = 0`. |
| `seed-02-boundary-ff.bin` | boundary | 56 `0xff` bytes — maximum digest and owner, `valid_to = u32::MAX`. |
| `seed-03-boundary-max-valid-to.bin` | boundary | Zero digest and zero owner with `valid_to = u32::MAX` exercising the big-endian boundary at the trailing 4 bytes. |

## Discovered-corpus seeds

Six 64-character hex-named seeds retained from prior libFuzzer smoke
runs. Each is treated as adversarial-class coverage and kept so the
round-trip surface keeps any pack/extract invariants the prior fuzz
sessions exercised:

- `4238edca2bc499df012ce1a907b8f3fb238f5c487ae876e2059c8cf43c3e0b2d`
- `662837192abc97e047be4dabfa1698eb9ab20c8b6449c2695183deeeed0f2ae3`
- `7c8a56e76c416e334061ce708f49fd69a8f1aab1c24b64fbefcb1a2014d8caef`
- `938857f44668a9003c76df1d5b593aeae4e0d1c3f514d7994d2b09365258608b`
- `d2da088ad6d195773a1b9646904e7443adb053d1e1cc7e49037643dc8982d912`
- `e1704862dffaa564421bcb793f3d4dc1c0e9ac22e84211389b18e92b830ba284`

Inputs shorter than the 56-byte gate return early without panicking.
