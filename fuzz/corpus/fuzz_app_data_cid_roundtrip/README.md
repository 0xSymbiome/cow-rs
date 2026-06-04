# `fuzz_app_data_cid_roundtrip` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_app_data_cid_roundtrip.rs`.
The first byte selects which helper the target exercises: even
discriminants route into `app_data_hex_to_cid` (then `cid_to_app_data_hex`
on success), odd discriminants route into `cid_to_app_data_hex` with
arbitrary candidate UTF-8 bytes. The structured-input width is capped
through `MAX_FUZZ_INPUT = 4096`.

## Named seeds

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-canonical-hex-digest.bin` | canonical | Discriminant `0x00` plus a 32-byte incrementing digest; exercises the hex-to-CID encoder and its inverse against the digest layout pinned by `crates/app-data/tests/cid_contract.rs`. |
| `seed-01-canonical-cid-string.bin` | canonical | Discriminant `0x01` plus a representative base32-multibase CIDv1 string; exercises the inverse `cid_to_app_data_hex` decoder against the codec and multihash gates pinned by `crates/app-data/tests/cid_contract.rs`. |
| `seed-02-boundary-empty-payload.bin` | boundary | Discriminant only (`0x00`), zero-byte payload; exercises the empty-input early-return path. |
| `seed-03-boundary-truncated-hex.bin` | boundary | Discriminant `0x00` plus 1 trailing byte; below the 32-byte digest length and exercises the malformed-hex candidate path. |
| `seed-04-adversarial-non-utf8-cid.bin` | adversarial | Discriminant `0x01` plus non-UTF-8 bytes (`0xff..0xf8`); exercises the `cid_to_app_data_hex` decoder against bytes that fail the `from_utf8` gate. |

## Discovered-corpus seeds

25 forty-character hex-named seeds retained from prior libFuzzer smoke
runs. Each is treated as adversarial-class coverage and kept so the
roundtrip surface keeps any encoder or decoder invariants the prior
fuzz sessions exercised. Filenames:

`0f28dfede025e7786ec18a62943a80f17bde47fc`,
`1077c47027f284182a6e503762dbe125593726f9`,
`10a42d494b0908c631b4acd80ea2207c30d9bc95`,
`127e79c9d2026c8d36553550e6c5c9ddbb575cb1`,
`1489f923c4dca729178b3e3233458550d8dddf29`,
`3f3d2d8955322f325af6db2238355fa07007ebd9`,
`4367d332b258f911e21326fd86633d92b7060fe7`,
`43b2f08bccee18f87ca24e0b2848bd864ab55a9d`,
`44853b124def120466c7ccd262bceef0f3a45a82`,
`4afaa522f19431ded5ab3b2753adedba4333f7fb`,
`592750664e5ef1d7a123142f78dd35aca0b77b75`,
`5ba93c9db0cff93f52b521d7420e43f6eda2784f`,
`71853c6197a6a7f222db0f1978c7cb232b87c5ee`,
`77fc5b8a80dac27b46ce1b582135759bcc616474`,
`8087de582c622752e40f96ed13eb199d3e606a81`,
`80910d62fd88e09ef2abcc8d5fab6a60ce314cf9`,
`929dce2cb63ffa46c9b7600bc49dd16c74ef2f64`,
`9d5f0b8a0ee0806d07ca92b4cdf20f2ed3bb4c21`,
`bc85c9fa1b17f3b8e24eac3432fff626f75665f0`,
`cefdbe28817279ab45120d31b7394d630e461bdc`,
`e9dbcd4721adbe8796d970f0f9163a5786d3f9aa`,
`edcb8fc28ea6abd7c4ad333aee46df36376f00cf`,
`eee7af2fa6910a21edba8081b4956b806c86dd2c`,
`f9124324970c227bc83eecc116ed91a436edbd12`,
`ff24c2609d9d3ef97b8eacd7e67f7638c75546e6`.
