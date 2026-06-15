# ADR 0067: Idiomatic Accessor Naming Without A get_ Prefix

- Status: Accepted
- Date: 2026-06-06
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: api, naming, conventions, accessors
- Related: [ADR 0035](0035-alloy-provider-adapter.md), [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Decision

Public accessors and domain fetch methods do not use a `get_` prefix. A method that
reads or fetches a value is named by its bare domain noun (for example `quote`, `order`,
`orders`, `trades`, `app_data`, `native_price`, `totals`, `domain`, `order_to_sign`), and
a signer's address is `address()`.

The `get_` prefix is retained in exactly one place: the chain-RPC `Provider` and
`LogProvider` trait methods that mirror canonical Ethereum JSON-RPC names (`get_chain_id`,
`get_code`, `get_transaction_receipt`, `get_block`, `get_logs`). See ADR 0035.

## Why

Rust does not prefix getters with `get_`. The Rust API Guidelines (C-GETTER) state that,
with a few exceptions, the `get_` prefix is not used for getters, and RFC 344 defines the
same convention: a getter for a field `foo` is named `foo()`. The standard library
reserves `get` and `get_mut` for fallible or indexed container access that takes a key and
returns `Option<&T>` (for example `slice::get`, `HashMap::get`), and for the
single-obvious-value wrapper case (`Cell::get`). The upstream `cowprotocol/services` model
follows the bare-noun convention for its domain accessors.

The chain-RPC trait methods are the one documented exception because they are fallible
keyed lookups that mirror the `eth_get*` JSON-RPC names and the upstream `alloy` provider
surface the adapter crates implement. The prefix there denotes the keyed-lookup family,
not a field getter, and mirroring the wire names aids consumers who already know `alloy`.

No Clippy lint enforces the getter-prefix convention; `clippy::wrong_self_convention`
covers only the `as_`, `to_`, `into_`, `is_`, and `from_` receiver rules. This rule is
guideline-backed and review-enforced, with a repository name scan and the public-api and
TypeScript declaration snapshots as the mechanical completeness checks.

## Must Remain True

- Public surface: no public accessor or domain fetch method carries a `get_` prefix
  outside the chain-RPC `Provider` and `LogProvider` traits. New methods follow the
  bare-noun rule.
- Runtime and support: the change is naming only. Runtime behavior, serialized payloads,
  and the TypeScript and npm export names are unchanged, because the wasm bindings keep
  their existing JavaScript names.
- Validation and review: the public-api and `.d.ts` snapshots stay unchanged across the
  rename, and a name scan confirms no owned non-mirror `get_` accessor remains. The
  standing public audit records current-state conformance.
- Cost: a one-time breaking rename of public method names, taken before the first
  functional release when there are no published consumers to migrate.

## Alternatives Rejected

- Drop `get_` everywhere, including the chain-RPC traits: rejected because those methods
  are not field getters but keyed `eth_get*` lookups, so dropping the prefix loses the
  deliberate `alloy` and JSON-RPC mirror that aids discovery while gaining no idiom (the
  keyed-lookup `get` form is already sanctioned by the standard library).
- Keep the `get_` prefix inherited from the TypeScript client: rejected because it
  contradicts C-GETTER, RFC 344, the standard library, and the upstream `services` model,
  and reads as a non-idiomatic transliteration.

## Links

- [Rust API Guidelines, naming (C-GETTER, C-CONV)](https://rust-lang.github.io/api-guidelines/naming.html)
- [RFC 344, method-naming conventions](https://rust-lang.github.io/rfcs/0344-conventions-galore.html)
- [ADR 0035](0035-alloy-provider-adapter.md)
- [Public API naming convention audit](../audit/public-api-naming-convention-audit.md)
