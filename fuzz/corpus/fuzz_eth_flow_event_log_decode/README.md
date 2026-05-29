# `fuzz_eth_flow_event_log_decode` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_eth_flow_event_log_decode.rs`.
The target maps raw bytes onto an event log — up to four 32-byte topics
drawn from the input head, the remainder as the data body — and feeds it
to `decode_eth_flow_log`. Three extra passes force each canonical eth-flow
lifecycle topic-0 (`OrderPlacement`, `OrderInvalidation`, `OrderRefund`)
so the ABI-decode, marker-mapping, owner-resolution, and order-UID
length-check paths are also driven. The decoder must always return
`Ok`/`Err` and never panic.

## Named seeds

| File | Class | Derivation |
| --- | --- | --- |
| `empty_no_topics` | boundary | An empty input — exercises the no-topic rejection path and the forced-topic-0 passes under an empty body. |
| `one_extra_topic_word` | boundary | A single 32-byte word — exercises the topic-count rejection path. |
| `refund_topic0_uid56` | boundary | A 128-byte zero body — drives the `OrderRefund` ABI-decode and the order-UID length-check under the canonical topic-0. |

Per the workspace corpus convention only this README is tracked; binary
seeds are gitignored and regenerated locally before a fuzzing run.
