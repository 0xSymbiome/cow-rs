# `fuzz_decoded_body_canonical_status_text` Corpus

This corpus seeds
`fuzz/fuzz_targets/fuzz_decoded_body_canonical_status_text.rs`. The
target uses an `Arbitrary` impl whose first input byte selects a seed
class. Seed classes `0..=4` route to the fixed orderbook response
shapes baked into the target; any other first byte exercises a freely
generated `(status, content_type, body)` triple. The canonical-class
fixture is anchored to the services rejection envelope pinned by
`parity/fixtures/orderbook.json::orderbook-duplicate-order-error`.

## Named seeds

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-canonical-rejection-envelope.bin` | canonical | Seed-class byte `'0'`; routes to a `400 Bad Request` envelope with `application/json` content-type carrying the duplicate-order rejection body anchored to `parity/fixtures/orderbook.json::orderbook-duplicate-order-error`. |
| `seed-01-boundary-204-no-content.bin` | boundary | Seed-class byte `'1'`; routes to the `204 No Content` envelope shape that must always decode to `ResponseBody::Empty`. |
| `seed-02-boundary-empty-body.bin` | boundary | Seed-class byte `'2'`; routes to a `200` envelope with an empty body that must also decode to `ResponseBody::Empty` per the documented decision rule. |
| `seed-03-adversarial-text-content-type.bin` | adversarial | Seed-class byte `'3'`; routes to a `500` envelope with `text/plain` content-type carrying a non-JSON body that must decode to `ResponseBody::Text`. |
| `seed-04-adversarial-malformed-json.bin` | adversarial | Seed-class byte `'4'`; routes to a `400` envelope with `application/json; charset=utf-8` content-type carrying a malformed JSON body; exercises the fallback to `ResponseBody::Text` after the JSON parse fails. |
| `seed-05-adversarial-random-bytes.bin` | adversarial | Twelve `0xff` bytes; the first byte short-circuits into the generic-input path so the `Arbitrary` reader exercises freely generated `(status, content_type, body)` triples. |
