# `fuzz_onchain_order_log_decode` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_onchain_order_log_decode.rs`.
The target maps raw bytes onto an event log — up to four 32-byte topics
drawn from the input head, the remainder as the data body — and feeds it
to `decode_order_placement` and `decode_order_invalidation`. Two extra
passes force the canonical `OrderPlacement` / `OrderInvalidation` topic-0
so the body, marker-mapping, and owner-resolution paths are also driven.
The decoders must always return `Ok`/`Err` and never panic.

## Named seeds

| File | Class | Derivation |
| --- | --- | --- |
| `topic_count_one` | boundary | A single 32-byte topic and no body — exercises the topic-count rejection path. |
| `placement_topic0_empty_body` | boundary | The canonical `OrderPlacement` topic-0 plus a zero sender topic and an empty body — drives the ABI-decode failure under an otherwise valid topic set. |
| `invalidation_topic0_uid56` | boundary | The canonical `OrderInvalidation` topic-0 plus a 56-byte zero UID body — drives the invalidation decode path. |

Per the workspace corpus convention only this README is tracked; binary
seeds are gitignored and regenerated locally before a fuzzing run.
