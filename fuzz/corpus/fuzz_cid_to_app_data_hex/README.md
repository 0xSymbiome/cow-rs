# `fuzz_cid_to_app_data_hex` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_cid_to_app_data_hex.rs`. Each
seed is consumed as raw bytes; the target rejects non-UTF-8 candidates
through an early return and feeds the rest to `cid_to_app_data_hex` as a
candidate CID string. The structured-input width is capped through
`MAX_FUZZ_INPUT = 4096`.

Seed classes:

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-canonical-cidv1-keccak256.txt` | canonical | A representative CIDv1 string over the raw-codec / keccak-256 multihash pair derived from the digest layout pinned by `parity/fixtures/app-data.json::app-data-cid-v1-conversion`. Round-trips through `app_data_hex_to_cid` and back. |
| `seed-01-canonical-cidv1-base32.txt` | canonical | A second representative CIDv1 multibase-base32 string for the same digest layout, covering the alternative multibase prefix the inverse decoder accepts. |
| `seed-02-boundary-empty.bin` | boundary | Zero-byte payload; exercises the empty-string early-return path. |
| `seed-03-boundary-single-byte.bin` | boundary | One-byte payload (`0x66`); exercises the truncated-multibase rejection path. |
| `seed-04-adversarial-cidv0-sha2.txt` | adversarial | A CIDv0 (`Qm...`) string over the dag-pb / sha2-256 pair; must reject with `AppDataError::InvalidCid` per the documented codec gate. |
| `seed-05-adversarial-mismatched-codec.txt` | adversarial | A CIDv1 string whose codec is dag-pb instead of raw; must reject with `AppDataError::InvalidCid`. |
| `seed-06-adversarial-non-utf8.bin` | adversarial | A truncated CIDv1-shaped string (31-byte digest instead of 32); exercises the non-32-byte-digest rejection path documented for the inverse decoder. |
