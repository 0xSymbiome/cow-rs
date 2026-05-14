# `fuzz_partner_fee_from_value` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_partner_fee_from_value.rs`.
Each seed is consumed as raw bytes that the target parses as a JSON
value through `serde_json::from_slice::<Value>` before driving
`PartnerFee::from_value`. The structured-input width is capped through
`MAX_FUZZ_INPUT = 4096`.

Seed classes:

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-canonical-volume.json` | canonical | A representative single-policy `{volumeBps, recipient}` value derived from the partner-fee fixture pinned by `parity/fixtures/app-data.json::app-data-validation-contract`. |
| `seed-01-canonical-multiple.json` | canonical | A representative array shape `[Volume, Surplus]` exercising the `Multiple` variant on the typed deserializer. |
| `seed-02-canonical-legacy-bps.json` | canonical | A legacy `{bps, recipient}` value the deserializer promotes into a `Volume` policy on success. |
| `seed-03-boundary-zero-volume.json` | boundary | Zero `volumeBps`; the deserializer accepts the u16 (documented as lenient on bounds), and `validate()` returns the expected `Err`. Exercises the parse-then-validate split. |
| `seed-04-boundary-max-volume.json` | boundary | Max-supported `volumeBps = 100`; at the upper end of the documented `[1..=100]` range. |
| `seed-05-adversarial-mixed-fields.json` | adversarial | Combination of `volumeBps` and `surplusBps` in one object; the deserializer must reject this as an unknown shape. |
| `seed-06-adversarial-non-json.bin` | adversarial | Plain ASCII text that is not parseable as JSON; exercises the early-return `from_slice` path. |
