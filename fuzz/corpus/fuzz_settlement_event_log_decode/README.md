# `fuzz_settlement_event_log_decode` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_settlement_event_log_decode.rs`.
The target maps raw bytes onto an event log — up to four 32-byte topics
drawn from the input head, the remainder as the data body — and feeds it
to `decode_settlement_log`. Five extra passes force each canonical
`GPv2Settlement` event topic-0 (`Trade`, `Interaction`, `Settlement`,
`OrderInvalidated`, `PreSignature`) so the ABI-decode and order-UID
length-check paths are also driven. The decoder must always return
`Ok`/`Err` and never panic.

## Named seeds

| File | Class | Derivation |
| --- | --- | --- |
| `empty_no_topics` | boundary | An empty input — exercises the no-topic rejection path and the forced-topic-0 passes under an empty body. |
| `one_extra_topic_word` | boundary | A single 32-byte word — exercises the topic-count rejection path. |
| `trade_topic0_zero_body` | boundary | A 288-byte zero body — drives the `Trade` ABI-decode and the order-UID length-check failure under the canonical topic-0. |

Per the workspace corpus convention only this README is tracked; binary
seeds are gitignored and regenerated locally before a fuzzing run.
