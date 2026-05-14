# `fuzz_typed_data_digest` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_typed_data_digest.rs`. The
target consumes bytes through `arbitrary` into a typed `UnsignedOrder`
plus `TypedDataDomain`, runs them through `hash_order`, and also
exercises `hash_order_cancellations` against the packed UID. The
structured-input width is bounded by the `Arbitrary` derive on the
internal `FuzzInput` struct.

## Seeds

| File | Class | Derivation |
| --- | --- | --- |
| `p2_canonical_cancellation_uid` | canonical | EIP-712 order field shape derived from `parity/fixtures/signing.json::signing-typed-data-envelope` paired with a representative cancellation UID. |
| `p2_boundary_zero_domain` | boundary | All-zero domain name, version, chain id, and verifying contract bytes. |
| `p2_boundary_ff_values` | boundary | All-`0xff` byte fill across the typed-data input budget. |
| `p2_boundary_alternating_aa55` | boundary | Alternating `0xaa` / `0x55` byte pattern across the input budget. |
| `p2_boundary_incrementing` | boundary | Monotone incrementing byte pattern across the input budget. |
| `p2_adversarial_ascii_domain` | adversarial | Printable-ASCII domain name and version bytes exercising the `bounded_ascii` helper. |
| `p2_adversarial_mixed_case` | adversarial | Mixed-case ASCII plus high-entropy byte mix exercising the typed-data normalizer. |
| `2b63a55a4a9c90d0ce4d0e914fb07a8a943b7e90a8cef150d0106ce2f49d8c2c` | adversarial | Discovered-corpus entry retained from prior libFuzzer smoke runs. |
| `605ed279d0a1af786c79054f9424d196ed6a1f0331100a923d711885d42099bb` | adversarial | Discovered-corpus entry retained from prior libFuzzer smoke runs. |
| `6d9c54dee5660c46886f32d80e57e9dd0ffa57ee0cd2a762b036d9c8e0c3a33a` | adversarial | Discovered-corpus entry retained from prior libFuzzer smoke runs. |
| `8d0da01949ca937fe72102d511382e10828dd39eefdf8c2601cc5f909cbeb969` | adversarial | Discovered-corpus entry retained from prior libFuzzer smoke runs. |
| `d0f9b20e11b4dee02da0e8da52ebeda2c6f00792f241238819f2b280ad10ba33` | adversarial | Discovered-corpus entry retained from prior libFuzzer smoke runs. |

The canonical seed pins the typed-data domain shape against
`parity/fixtures/signing.json::signing-typed-data-envelope` and the
order field shape against
`parity/fixtures/signing.json::signing-eip712-order-fields`. The
boundary seeds cover the input-budget edges that exercise the
`bounded_ascii` length and seed parameters. The discovered-corpus
adversarial entries are kept so coverage observed during prior
libFuzzer smoke runs is preserved across audit refreshes.
