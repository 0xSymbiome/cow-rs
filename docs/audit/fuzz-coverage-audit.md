# Fuzz Coverage Audit

Status: Current
Last reviewed: 2026-05-14
Owning surface: the standalone `cow-sdk-fuzz` crate (`fuzz/`) and every
`cargo-fuzz` target it ships against the published SDK crates
Refresh trigger: any new public untrusted-input surface, retired fuzz
target, changed seed contract, change to the fuzz dependency set,
change to the workspace quality-gate step that compiles the fuzz crate,
or refreshed empirical-run evidence after a scheduled sweep finds and
fixes a new panic class
Related docs:
- [PROPERTIES.md](../../PROPERTIES.md)
- [ADR 0011](../adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md)
- [ADR 0013](../adr/0013-http-transport-injection-and-typestate-builders.md)
- [ADR 0022](../adr/0022-ecdsa-signature-v-normalization.md)
- [ADR 0033](../adr/0033-minimum-viable-panic-surface.md)
- [ADR 0041](../adr/0041-transport-policy-l3-layering.md)
- [Credential Surface Audit](credential-surface-audit.md)
- [URL Credential Redaction Audit](url-credential-redaction-audit.md)
- [ECDSA Signature Normalization Audit](ecdsa-signature-normalization-audit.md)
- [Trading App-Data Merge Audit](trading-app-data-merge-audit.md)

## Scope

This audit covers:

- the standalone `cow-sdk-fuzz` crate manifest, its target inventory,
  and its dependency surface against the published SDK crates
- every `fuzz/fuzz_targets/*.rs` source file and the public-API surface
  each target exercises
- every `fuzz/corpus/<target>/` directory, its README, its seed class
  coverage, and its compliance with the per-target seed contract
- the workspace quality-gate step that type-checks the fuzz crate
  against the stable toolchain on every pull request
- the cross-link between each fuzz target and the `PROPERTIES.md`
  invariant it strengthens through its asserted contract

It does not cover scheduled fuzz EXECUTION (the workflow that runs
`cargo +nightly fuzz run`), Linux-only sanitizer runtime requirements,
or libFuzzer-internal mutation strategy; those concerns are owned by
the scheduled-fuzz workflow rather than by this static review.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Target inventory | 46 fuzz targets cover every reviewed public untrusted-input boundary across `cow-sdk-core`, `cow-sdk-contracts`, `cow-sdk-app-data`, `cow-sdk-orderbook`, `cow-sdk-subgraph`, `cow-sdk-signing`, `cow-sdk-trading`, `cow-sdk-transport-policy`, and `cow-sdk-browser-wallet` | Conforms |
| Stable-toolchain compile | The fuzz crate compiles under `cargo +stable check --manifest-path fuzz/Cargo.toml` and is gated on every pull request through the shared workspace quality-gate workflow | Conforms |
| Nightly-toolchain enumerate | `cargo +nightly fuzz list --fuzz-dir fuzz` enumerates all 46 targets | Conforms |
| Per-target seed contract | Every target ships a corpus directory with a `README.md`, at least 5 tracked seed files, and explicit class coverage (canonical / boundary / adversarial) anchored to a named parity fixture or upstream test fixture | Conforms |
| Property traceability | Every target carries a `**Property:**` doc-comment row citing one `PROP-*` invariant identifier from `PROPERTIES.md`; every cited identifier has its evidence column updated to reference the fuzz target source and corpus directory | Conforms |
| Public-surface boundary | Every target imports only published SDK surface; crate-private helpers are exercised through the nearest public wrapper, with the routing documented in the target doc-comment header | Conforms |
| Invariant strength | Existing targets carry semantic assertions beyond bare panic-freedom: encoder targets check selector and decoder round-trip, classifier targets check determinism and class boundaries, redaction targets check credential-shape absence including URL userinfo, JWT prefixes, Bearer prefixes, and credential key/value forms | Conforms |
| Boundary on `pub(crate)` surfaces | Three browser-wallet helpers (`hex_quantity`, `parse_quantity_to_decimal`, `json_to_dyn_value`, `parse_u256`, `parse_i256`, `bytes_from_json`, `decode_hex`, `transaction_to_rpc`) are crate-private and reachable only through `async fn` wrappers. The fuzz crate carries no async executor, so the targets exercise adjacent public deserialization surfaces (`RpcErrorPayload`, `ContractCall`, `TransactionRequest`) and document the routing | Conforms (documented gap) |

## Current Contract

### Target Inventory

The `cow-sdk-fuzz` crate ships 46 `cargo-fuzz` targets. Each target is
declared as a `[[bin]]` entry in `fuzz/Cargo.toml`, has a matching
`fuzz/fuzz_targets/<name>.rs` source file, and has a populated
`fuzz/corpus/<name>/` seed directory.

| Domain | Target count | Surfaces exercised |
| --- | --- | --- |
| Encoder | 5 | `GPv2Settlement.settle`, `GPv2Settlement.invalidateOrder`, `CoWSwapEthFlow.createOrder`, `GPv2VaultRelayer.transferFromAccounts`, EIP-2612 permit envelope |
| Signing | 5 | EIP-712 typed-data digest, ECDSA `v` normalization (byte array), ECDSA `v` normalization (string), ECDSA address recovery, EIP-712 domain separator |
| Validator and bounds | 2 | Order bounds validator, `ValidTo::relative` window |
| Parser and decoder | 12 | Orderbook rejection envelope, orderbook rejection code allowlist, decoded body and canonical status text, append query string, subgraph GraphQL error decoder, transport-error classifier, retry-after header parser, retry policy delay, jitter strategy delay, partner-fee `from_value`, flashloan-hints deserializer, hook-list deserializer |
| Crypto envelope and hash | 4 | EIP-712 order hash, EIP-712 order-cancellations hash, EIP-1271 signature data decoder, EIP-1271 magic-value response decoder |
| Order UID and signature classifier | 2 | Order UID pack and unpack, signature classifier and signing-scheme discriminant |
| Core types and identities | 5 | `Amount` parser, `SignedAmount` parser, hex identity validators (`Address`, `Hash32`, `AppDataHash`, `OrderUid`, `HexData`), `DecimalAmount::from_whole_approx`, redaction body scanner |
| App-data | 6 | CID round-trip, CID-to-hex decoder, schema version `is_semver`, `stringify_deterministic`, app-data size limit, `params_from_doc` |
| Trading and slippage | 3 | App-data merge, slippage amounts, slippage policy helpers |
| Orderbook wire totals | 1 | `calculate_total_fee` |
| Browser-wallet adjacent | 3 | Hex-quantity helpers (adjacent), JSON-to-dyn-value (adjacent), transaction-to-RPC (adjacent) |

### Stable-toolchain Compile Gate

The fuzz crate stands alone (excluded from the workspace root) because
`cargo-fuzz` requires a nightly toolchain with unstable `RUSTFLAGS`,
which the rest of the workspace does not consume. The shared quality
gate keeps the harness type-checked against the same stable toolchain
the rest of the repository uses by running
`cargo check --manifest-path fuzz/Cargo.toml` from the workspace root
on every pull request. This guards against type drift between published
crate surfaces and fuzz target imports without forcing the workspace
onto nightly.

### Nightly-toolchain Enumerate

Running `cargo +nightly fuzz list --fuzz-dir fuzz` enumerates all 46
targets by their `[[bin]]` names. The same nightly toolchain is the
one the scheduled fuzz workflow runs on `ubuntu-latest`, where the
LLVM AddressSanitizer runtime ships with the system clang/llvm package.

### Per-target Seed Contract

Every target ships a corpus directory under `fuzz/corpus/<target>/`
that satisfies the contract documented in `fuzz/README.md`:

- at least 5 tracked seed files (excluding the directory `README.md`)
- explicit canonical / boundary / adversarial class coverage
- the per-target `README.md` names the parity fixture id (or pinned
  upstream test fixture) the canonical class is anchored to, and lists
  every seed by class and derivation

Seed files are tracked in version control through allow-list entries
in the workspace `.gitignore`. New corpus directories require both a
`fuzz/Cargo.toml` `[[bin]]` entry and a matching `.gitignore`
allow-list so seed files are not silently dropped by the default
`fuzz/corpus/*` exclusion.

### Property Traceability

Every target's doc-comment header carries a `**Property:**` row citing
exactly one `PROP-*` invariant identifier from `PROPERTIES.md`. Every
cited identifier has its evidence column updated to reference the
target source file (`fuzz/fuzz_targets/<name>.rs`) and the matching
corpus directory (`fuzz/corpus/<name>/`). The cross-link is the
reviewer's path from a `PROPERTIES.md` row to the fuzz coverage that
strengthens it. The 22 `PROP-*` identifiers cited across the 46 targets
span `PROP-CORE-*`, `PROP-CON-*`, `PROP-SIG-*`, `PROP-AD-*`,
`PROP-APP-*`, `PROP-OBK-*`, `PROP-ORD-*`, `PROP-SBG-*`, `PROP-TPP-*`,
`PROP-TRD-*`, and `PROP-BWL-*` families.

### Public-surface Boundary

Every fuzz target imports only published SDK surface. Where the
underlying helper is `pub(crate)` or otherwise unreachable from outside
the owning crate, the target either:

- routes through the nearest public wrapper (with the routing recorded
  in the target's doc-comment header), or
- exercises an adjacent public surface that participates in the same
  data path (e.g. wallet RPC normalization through
  `RpcErrorPayload` deserialization when the underlying helper is
  reachable only through `async fn`).

The doc-comment header on every target that takes this routing
discloses the boundary and the rationale, so a reviewer can verify the
asserted invariant is meaningful for the public surface even when the
target cannot drive the private helper directly.

### Invariant Strength

Every target asserts at least one semantic property beyond
panic-freedom. The asserted properties include:

- **Encoder targets**: 4-byte selector equality against
  `keccak256(signature)[0..4]`, decoder round-trip equality on the
  encoded bytes, dynamic-argument-offset structural floor on encoded
  length, and canonical-encoding determinism on identical input.
- **Crypto envelope targets**: independent keccak256 envelope equality
  against the helper output, two-call determinism, mutation-resistance
  on every typed-data domain field.
- **Classifier and parser targets**: typed-error-partition coverage,
  two-call determinism, sanitization assertions on rendered `Display`
  and `Debug` output (no raw newline, no raw null byte, no URL
  userinfo, no JWT prefix shape, no `Bearer` prefix, no credential
  key=value pattern).
- **Round-trip targets**: `from(to(x)) == x` byte-equivalent
  reconstruction for every successfully constructed typed value,
  plus encoder-idempotency assertions where serialization is part of
  the contract.
- **Validator targets**: explicit enumeration of every documented
  rejection variant, with new variants triggering an explicit panic so
  scheduled fuzz runs surface them rather than silently accepting them.

### Boundary On `pub(crate)` Browser-wallet Helpers

The browser-wallet adjacent targets (`fuzz_hex_quantity_helpers`,
`fuzz_json_to_dyn_value`, `fuzz_transaction_to_rpc`) name a documented
gap: the helpers are `pub(crate)` and reachable only through
`async fn` methods on `AsyncProvider` and `AsyncSigningProvider`. The
fuzz crate does not link an async executor, so the targets exercise
the adjacent public deserialization surfaces (`RpcErrorPayload`,
`ContractCall`, `TransactionRequest`) that participate in the same
normalization pipeline. The boundary is recorded in the target
doc-comment headers and in this audit, so a future refresh that adds
an async runtime or expands the public surface can close the gap with
no further design change.

### Empirical Run Evidence

A scheduled-equivalent sweep run on a Linux x86-64 host (8-way parallel,
10-minute budget per target, `timeout=10` per input) covered every one
of the 46 targets without producing a panic. Earlier iterations of the
same sweep surfaced three real SDK defects on attacker-controlled
surfaces and three over-strict fuzz-target assertions, all of which
were corrected before the clean run:

- `redact_response_body` was strengthened against URL userinfo with
  mangled or non-ASCII scheme prefixes, bare `Bearer <token>` strings
  outside the `Authorization` key context, JWT payloads embedded inside
  credential-key positions, and partial or corrupted credential key
  names. The detector pipeline is now layered as JWT, Bearer, strict
  URL, bare userinfo, and credential-keyed value with recursive key
  prefix scanning.
- `parse_retry_after` was promoted to `i64` civil-day arithmetic so an
  attacker-controlled IMF-fixdate year value cannot panic the retry
  loop with an `i32` multiplication overflow; the caller's
  `checked_mul`/`checked_add` chain provides the final downstream guard.
- `calculate_quote_amounts_and_costs` was given an explicit
  `protocol_fee_bps >= 100%` guard so an out-of-range partner protocol
  fee returns a typed `InvalidInput` error rather than panicking on the
  sell path through a `BigInt` divide-by-zero.

The three fuzz-target invariants that were corrected are documented in
the target source headers: `fuzz_subgraph_graphql_error_decode` and
`fuzz_hex_quantity_helpers` no longer assert round-trip equality on
`Redacted<T>` fields (the wrapper's `Serialize` impl deliberately
writes the sanitized placeholder rather than the inner value), and
`fuzz_stringify_deterministic` no longer asserts byte-level canonical
JSON idempotence on arbitrary `serde_json::Value` inputs (a literal
like `3e+23` cannot round-trip bit-identically through `f64` because
the shortest-representation rendering can vary by one ULP across
parse-then-render cycles). The shipped canonical-form stability is
verified by the parity fixture and `crates/app-data/tests/property_contract.rs`
unit tests on realistic inputs.

## Evidence

Primary implementation points:

- `fuzz/Cargo.toml`
- `fuzz/fuzz_targets/` (46 fuzz target source files)
- `fuzz/corpus/` (46 corpus directories with READMEs and seed files)
- `fuzz/README.md` (per-target seed contract and harness conventions)
- `.gitignore` (corpus directory allow-list)
- `.github/workflows/_quality-gate.yml` (stable-toolchain compile gate
  step `Check fuzz crate against the stable toolchain`)
- `PROPERTIES.md` (22 `PROP-*` rows with fuzz target evidence)

Primary regression coverage:

- Per-target invariant assertion inside each `fuzz_target!` body
- Workspace quality gate step running `cargo check --manifest-path fuzz/Cargo.toml`
- Scheduled `cargo +nightly fuzz` workflow under `.github/workflows/`

Validation surface:

```text
cargo check --manifest-path fuzz/Cargo.toml
cargo +nightly fuzz list --fuzz-dir fuzz
cargo +nightly fuzz build --fuzz-dir fuzz
```

Local scheduled-fuzz execution is supported on Linux and macOS targets
where the LLVM AddressSanitizer runtime ships with the system clang or
LLVM package. Local execution on Windows requires the
`clang_rt.asan_dynamic-x86_64.dll` runtime that ships with the MSVC
toolset rather than `rustup`; the build and enumerate steps work on
every nightly-supported host.
