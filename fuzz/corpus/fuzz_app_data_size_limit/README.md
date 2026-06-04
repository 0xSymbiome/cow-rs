# `fuzz_app_data_size_limit` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_app_data_size_limit.rs`. Each
seed is consumed as raw bytes; the first two bytes are decoded as a
big-endian 16-bit padding length that scales the rendered canonical
JSON byte size linearly. Inputs shorter than two bytes feed the
single-byte and zero-byte early-return branches.

Seed classes:

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-canonical-zero-pad.bin` | canonical | A short input feeding the zero-byte / single-byte early-return branches; produces the minimal app-data document well below the warning floor, derived from the size/`TooLarge` validation contract pinned by `crates/app-data/tests/validated_shape_contract.rs`. |
| `seed-01-canonical-sub-warning.bin` | canonical | A short text payload whose decoded padding length lands well below the documented 75%-of-max warning threshold. |
| `seed-02-boundary-at-warning.bin` | boundary | A text payload whose decoded padding length lands inside the band approaching the warning threshold; libFuzzer mutates from here across the exact boundary. |
| `seed-03-boundary-at-ceiling.bin` | boundary | A text payload whose decoded padding length lands near the hard ceiling without overshooting. |
| `seed-04-adversarial-overshoot.bin` | adversarial | A text payload whose decoded padding length is sized to push the rendered document past the documented hard ceiling; exercises the `AppDataError::TooLarge` rejection path. |
| `seed-05-adversarial-max-pad.bin` | adversarial | A text payload near the maximum-explored padding length; covers the upper end of the padding cap. |
