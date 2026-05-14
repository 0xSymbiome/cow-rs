# `fuzz_append_query_string` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_append_query_string.rs`. The
target uses an `Arbitrary` impl whose first input byte selects a seed
class. Seed classes `0..=4` route to the fixed orderbook URL shapes
baked into the target; any other first byte exercises a freely
generated `(base, method, pairs)` triple. The canonical-class fixture
is anchored to the orderbook list-endpoint shape pinned by
`parity/fixtures/orderbook.json::orderbook-get-orders-pagination`.

The internal `append_query_string` helper that joins a base URL with
query pairs is crate-private and only reachable through the async
transport dispatch path. The target therefore exercises the closest
public sync surface — the [`FetchParams`] descriptor that captures the
same `(path, method, query, body)` material the private helper would
consume — and asserts the public assembly path stays panic-free and
deterministic for any caller-controlled input.

## Named seeds

| File | Class | Derivation |
| --- | --- | --- |
| `seed-00-canonical-orders-pagination.bin` | canonical | Seed-class byte `'0'`; routes to the GET `/orders` URL plus the `owner`, `offset`, `limit` pagination triple anchored to `parity/fixtures/orderbook.json::orderbook-get-orders-pagination`. |
| `seed-01-boundary-empty-pairs.bin` | boundary | Seed-class byte `'1'`; routes to the GET `/version` URL with zero query pairs; exercises the no-op path the documented helper returns early on. |
| `seed-02-boundary-empty-base.bin` | boundary | Seed-class byte `'2'`; routes to a zero-length base string with a single arbitrary `(key, value)` pair; exercises the empty-base boundary. |
| `seed-03-adversarial-ipv6-control-bytes.bin` | adversarial | Seed-class byte `'3'`; routes to an IPv6-bracket-form base URL plus a query value carrying percent-encoded and ASCII control bytes (`%00`, `0x01`, `0x7f`). |
| `seed-04-adversarial-malformed-base.bin` | adversarial | Seed-class byte `'4'`; routes to the literal string `"not a url"` plus newline-bearing query material (`"\n"`, `"\r"`). |
| `seed-05-adversarial-random-bytes.bin` | adversarial | Alternating `0x55 0xaa` pattern; first byte short-circuits into the generic-input path so the `Arbitrary` reader generates random `(base, method, pairs)` material. |
