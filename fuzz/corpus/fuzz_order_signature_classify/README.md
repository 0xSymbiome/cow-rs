# `fuzz_order_signature_classify` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_order_signature_classify.rs`.
The first byte selects a `SigningScheme` discriminant routed through
both `SigningScheme::try_from(u8)` and `decode_signing_scheme(u8)`.
The remaining bytes feed `decode_eip1271_signature_data` as a hex
candidate (and as a raw UTF-8 candidate) and
`serde_json::from_slice::<Signature>` as a candidate JSON envelope.
The structured-input width is capped through `MAX_FUZZ_INPUT = 256`.

## Named seeds

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-canonical-eip712.bin` | canonical | Scheme byte `0x00` (`SigningScheme::Eip712`) plus a minimal `0x00` hex tail; pins the discriminant set anchored by `parity/fixtures/contracts.json::contracts-signing-scheme-discriminants`. |
| `seed-01-canonical-ethsign.bin` | canonical | Scheme byte `0x01` (`SigningScheme::EthSign`) plus a minimal `0x00` hex tail; pins the second discriminant on the same fixture surface. |
| `seed-02-canonical-eip1271-verifier.bin` | canonical | Scheme byte `0x03` (`SigningScheme::Eip1271`) plus a `0x`-prefixed valid 24-byte hex sequence; exercises the verifier and signature split in `decode_eip1271_signature_data`. |
| `seed-03-boundary-scheme-only.bin` | boundary | Single byte `0xff`; exercises a scheme discriminant past the documented `SigningScheme` range and triggers the `Err` branch of both decoders. |
| `seed-04-adversarial-json-signature.bin` | adversarial | Scheme byte `0xfe` plus the JSON literal `{"signingScheme":"presign","data":"0x"}`; exercises the serde-derived `Signature` decoder rather than the hex paths. |

## Discovered-corpus seeds

29 forty-character hex-named seeds retained from prior libFuzzer smoke
runs. Each is treated as adversarial-class coverage and kept so any
classifier or decoder invariants the prior fuzz sessions exercised are
preserved. Filenames:

`0ef269320ab765ed4fd1dc5c9493b959db76852b`,
`0f04a071bb577d5edc0b3afcc07e6613a0250261`,
`222b2b4ab2fdd2dd8ef261d8953351ac6bc457fa`,
`2952b70c1bb382b8dd1d9fff87f4e94460d6c963`,
`2c7c1a727ad282aaae5b102dd7e4e463fae22dd5`,
`2c8bb56bf5af6978da3a6606a13a2b2b0150ef53`,
`5beaa2dc9ffeb9d413afbcc1e81605e33a00282e`,
`7643b6b8efadb1df634675276f7cbe3493e623bd`,
`7b96d607cc6804348889c92b781ffb4ad14ac5fb`,
`8c7b5173f6006484091633994681c4c93d24817a`,
`924006902419a4724a901672481926330c878094`,
`9c1b0492be9d12bd16fead7f9a669430363aeceb`,
`9d6eb80418571a852e35ab007cdc2d1bbea54713`,
`a02fa2ef2a336b8d95df62108e19073f7e13ee78`,
`b3138eb871419a1b90f7f275251f38b8ee2ae5bd`,
`b57bd2e6cfd45bc8ac130b0cdf6e692d6a8ddd27`,
`b6713c0f48fef18cd02f2e2684ec44f2e1179b54`,
`bf8b4530d8d246dd74ac53a13471bba17941dff7`,
`c422e4f289950d4771f2eba3892c16284a37c63b`,
`c55583a5701cf02edda4c85e3fa71c69621a80de`,
`c78295f5626ccd6578dba481e956719ec1a57e3a`,
`cd5a8b2d93940c8daddd2344364a0d48e5b81883`,
`d0c63deb8ae1a3612de1cd16a638f944a6f8c8ac`,
`d9be6524a5f5047db5866813acf3277892a7a30a`,
`dc0238fbaa4b4cdd46df1f7c19a144a9270e03da`,
`f0b1f605d57f7d9c33cccda3b2d1fd1ab839de97`,
`f272c55db79c3e8e5c21e027e80d35eb42719740`,
`f6460d7d9d2b32d0dbd200d75a696a0a3e3a09e1`,
`f83590323fcf766214f3a40caead03c45b92a3c6`.
