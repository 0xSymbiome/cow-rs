# `fuzz_subgraph_graphql_error_decode` Corpus

This corpus seeds
`fuzz/fuzz_targets/fuzz_subgraph_graphql_error_decode.rs`. The target
feeds arbitrary bytes (capped at `MAX_FUZZ_INPUT = 4096`) to the
serde-derived decoder for `SubgraphGraphQlError` twice — once as a
single-object candidate and once as a `Vec<SubgraphGraphQlError>`
candidate. Successful decodes must round-trip back to bytes through
`serde_json::to_vec` without losing the `message` and `locations`
fields, and the bundled `SubgraphError` enum surface is anchored via a
typed construction in the target.

## Named seeds

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-canonical-single-error.bin` | canonical | A single GraphQL error object with `message` and one `locations` entry; pins the decoder against the GraphQL error envelope shape exercised by `crates/subgraph/tests/query_contract.rs` and `crates/subgraph/tests/schema_evidence/schema.graphql`. |
| `seed-01-canonical-array.bin` | canonical | A two-element `errors` array shape; pins the sequence-decoder branch through the same contract surface. |
| `seed-02-boundary-empty.bin` | boundary | Zero-byte input; exercises the decoder's empty-input rejection without panicking. |
| `seed-03-boundary-deeply-nested.bin` | boundary | Ten levels of nested empty arrays (`[[[[[[[[[[]]]]]]]]]]`); exercises the serde nesting boundary without exceeding the default recursion limit. |
| `seed-04-adversarial-non-utf8.bin` | adversarial | Non-UTF-8 byte sequence (`0xff..0xfb`); exercises the decoder's non-UTF-8 rejection path. |

## Discovered-corpus seeds

49 forty-character hex-named seeds retained from prior libFuzzer smoke
runs. Each is treated as adversarial-class coverage and kept so any
decoder invariants the prior fuzz sessions exercised against malformed
JSON, unicode escapes, duplicate keys, or trailing-byte edge cases are
preserved. Filenames:

`099600a10a944114aac406d136b625fb416dd779`,
`0b000dbedeec6e500a9fa717e6aa37b37fd20d12`,
`11f4de6b8b45cf8051b1d17fa4cde9ad935cea41`,
`12c6fc06c99a462375eeb3f43dfd832b08ca9e17`,
`1a349dcc540a3978584510d982075f838b17cd6d`,
`1c6637a8f2e1f75e06ff9984894d6bd16a3a36a9`,
`1e5c2f367f02e47a8c160cda1cd9d91decbac441`,
`2ace62c1befa19e3ea37dd52be9f6d508c5163e6`,
`2e74d24e887678f0681d4c7c010477b8b9697f1a`,
`2e767d20b16145633273a2c80c51a8f61326701f`,
`3bc15c8aae3e4124dd409035f32ea2fd6835efc9`,
`3f3d2d8955322f325af6db2238355fa07007ebd9`,
`4889648e8a44b34f0c29210a988460e643285f84`,
`4e6653af100b296c3a83d504e7b97077d7e56c62`,
`5ac540e4239159c5ef98e733efe8043f3df1451b`,
`60ba4b2daa4ed4d070fec06687e249e0e6f9ee45`,
`6c14f405f2540628200e2076af5c1b475cacd24d`,
`71853c6197a6a7f222db0f1978c7cb232b87c5ee`,
`7448d8798a4380162d4b56f9b452e2f6f9e24e7a`,
`7c338ed2840d2bf55f9f5e4eed04f66c80840eb3`,
`80fea951e66a21ebe44f17e8527e552e73b88021`,
`877b38d07782d5c7a3cbca8cb6e0bc4a5edcb52b`,
`8c892606364f418c9fb4c2647c7f082ca86a5897`,
`8d883f1577ca8c334b7c6d75ccb71209d71ced13`,
`8d94f13a23f34b380f065c43f8a9bb990d09ab67`,
`9c6b057a2b9d96a4067a749ee3b3b0158d390cf1`,
`9ec564c64cd5a1f0743a9a11374db9cca7a98f7d`,
`a80c6f1fd402a2349bbacc8485980f02fb2f0fb6`,
`ac9231da4082430afe8f4d40127814c613648d8e`,
`b6589fc6ab0dc82cf12099d1c2d40ab994e8410c`,
`b858cb282617fb0956d960215c8e84d1ccf909c6`,
`bb44e18ada1402f32b39da7edc60c2ef0261473d`,
`be05559429dd20f253857065ec57f21458e02847`,
`cab9f3c712d04de874dafb0af0a0bf03e303e6e0`,
`ce8542f6a1abd83ac085cdfbfdba263402f73154`,
`cfebf8a85a51ef354942420e54ae6b429f817422`,
`d2b28da60ade64564c50208a4c82cca42d83afbe`,
`d67d4422dccb731ed3fcb61ffdb76a979af68dde`,
`da4b9237bacccdf19c0760cab7aec4a8359010b0`,
`e6a9fc04320a924f46c7c737432bb0389d9dd095`,
`e897ede2f51e9a02934d72bf73b33f7340908e4a`,
`f4b909b863f1ca1a5fb3e78333d1128800c6dac5`,
`f54e05483ee2719d21d713aa019737cbc8f3d17b`,
`f5e40675e30599fd0138da491f715bf8561b1dd1`,
`f834bde21859165b56c0aaa6547991a1ce7d94bd`,
`fa6955fc709954ff76e1000dc2991427d16f8a5e`,
`ff08a4fe47ace6477d2124b5f4c43c00f681dc45`,
`ffc739ddefe1953fa9bc2df5efa2fff8ee2b0b60`,
`fffed33fa6431437dbf79652f4a7ffceb3c5cce2`.
