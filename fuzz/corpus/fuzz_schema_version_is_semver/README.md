# `fuzz_schema_version_is_semver` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_schema_version_is_semver.rs`.
Each seed is consumed as raw bytes that the target coerces to a
candidate version string through `String::from_utf8_lossy` before
calling `SchemaVersion::new`. The structured-input width is capped
through `MAX_FUZZ_INPUT = 4096`.

Seed classes:

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-canonical-1.14.0.txt` | canonical | The latest bundled app-data schema version derived from the deterministic-info contract pinned by `crates/app-data/tests/app_data_info_contract.rs`. |
| `seed-01-canonical-0.1.0.txt` | canonical | The minimal three-part decimal-semver value. |
| `seed-02-canonical-large.txt` | canonical | A three-part value with multi-digit segments (`999.0.42`); covers the `is_non_empty_digits` branch. |
| `seed-03-boundary-empty.bin` | boundary | A near-empty payload; must reject through the documented `^\d+\.\d+\.\d+$` regex. |
| `seed-04-boundary-two-part.txt` | boundary | `1.0`; must reject because the third segment is missing. |
| `seed-05-boundary-four-part.txt` | boundary | `1.0.0.1`; must reject because a fourth segment exists. |
| `seed-06-adversarial-alpha-middle.txt` | adversarial | `1.two.3`; must reject because the middle segment carries non-digit characters. |
| `seed-07-adversarial-leading-v.txt` | adversarial | `v1.0.0`; must reject because the leading-`v` prefix is not an ASCII digit. |
