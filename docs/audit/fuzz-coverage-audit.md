# Fuzz Coverage Audit

Status: Current
Last reviewed: 2026-06-17
Owning surface: the standalone `cow-sdk-fuzz` crate (`fuzz/`) and every
`cargo-fuzz` target it ships against the published SDK crates
Refresh trigger: any new public untrusted-input surface, retired fuzz
target, changed seed contract, change to the fuzz dependency set,
change to the workspace quality-gate step that compiles the fuzz crate,
or refreshed empirical-run evidence after a fuzz sweep finds and fixes a
new panic class
Related docs:
- [PROPERTIES.md](../../PROPERTIES.md)
- [ADR 0011](../adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md)
- [ADR 0013](../adr/0013-http-transport-injection-and-typestate-builders.md)
- [ADR 0022](../adr/0022-ecdsa-signature-v-normalization.md)
- [ADR 0033](../adr/0033-minimum-viable-panic-surface.md)
- [ADR 0041](../adr/0041-transport-policy-l3-layering.md)
- [ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md)
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
- the documented seed-class taxonomy (canonical / boundary /
  adversarial) each target is seeded from in local working copies
- the workspace quality-gate step that type-checks the fuzz crate
  against the stable toolchain on every pull request
- the cross-link between each fuzz target and the `PROPERTIES.md`
  invariant it strengthens through its asserted contract

Beyond confirming the scheduled run lane exists and its public shape, it
does not cover fuzz EXECUTION internals. A report-only `fuzz` workflow
(`.github/workflows/fuzz.yml`) runs every target for a bounded time budget
on a weekly cron plus manual dispatch; it is non-gating and never a
pull-request check, and targets are also run locally on demand. The
libFuzzer-internal mutation strategy, per-run corpus growth, and the
Linux sanitizer-runtime requirements remain out of scope for this static
review.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Target inventory | 42 fuzz targets (authoritative list: `cargo +nightly fuzz list --fuzz-dir fuzz`) cover every reviewed public untrusted-input boundary across `cow-sdk-core`, `cow-sdk-contracts`, `cow-sdk-app-data`, `cow-sdk-orderbook`, `cow-sdk-subgraph`, `cow-sdk-signing`, and `cow-sdk-trading` | Conforms |
| Stable-toolchain compile | The fuzz crate compiles under `cargo +stable check --manifest-path fuzz/Cargo.toml` and is gated on every pull request through the shared workspace quality-gate workflow | Conforms |
| Scheduled run lane | A report-only `fuzz` workflow runs every target for a bounded time budget on a weekly cron and on demand, uploading any crash reproducer; it is non-gating and never a pull-request check | Conforms |
| Nightly-toolchain enumerate | `cargo +nightly fuzz list --fuzz-dir fuzz` enumerates the full target set by `[[bin]]` name | Conforms |
| Seed-class contract | Every target is seeded locally from the canonical / boundary / adversarial classes documented in this audit and the harness doc-comment header; the entire `fuzz/corpus/` tree (baseline seeds and the libFuzzer accumulator alike) is gitignored per the cargo-fuzz convention of keeping the working corpus out of version control, and is regenerated locally from the documented classes | Conforms |
| Property traceability | Every target carries a `**Property:**` doc-comment row citing one `PROP-*` invariant identifier from `PROPERTIES.md`; every cited identifier has its evidence column updated to reference the fuzz target source file | Conforms |
| Public-surface boundary | Every target imports only published SDK surface; crate-private helpers are exercised through the nearest public wrapper, with the routing documented in the target doc-comment header | Conforms |
| Invariant strength | Existing targets carry semantic assertions beyond bare panic-freedom: encoder targets check selector and decoder round-trip, classifier targets check determinism and class boundaries, redaction targets check credential-shape absence including URL userinfo, JWT prefixes, Bearer prefixes, and credential key/value forms | Conforms |

## Current Contract

### Target Inventory

The `cow-sdk-fuzz` crate ships its targets as `[[bin]]` entries in
`fuzz/Cargo.toml`, each with a matching `fuzz/fuzz_targets/<name>.rs`
source file. The authoritative inventory is
`cargo +nightly fuzz list --fuzz-dir fuzz` — this audit describes
coverage by boundary class rather than re-listing a count that would rot
on every add or cut.

| Boundary class | Surfaces exercised |
| --- | --- |
| Encoder | `CoWSwapEthFlow.createOrder` (`fuzz_ethflow_create_order_encode`) |
| Signing | EIP-712 typed-data digest, ECDSA `v` normalization, ECDSA address recovery, recoverable-signature hex parse, recoverable-signature differential, EIP-712 domain separator |
| Validator and bounds | Order bounds validator, `ValidTo::relative` window |
| Parser and decoder | Orderbook rejection envelope, orderbook rejection code allowlist, decoded body and canonical status text, subgraph GraphQL error decoder, transport-error classifier, retry-after header parser, retry policy delay, jitter strategy delay, partner-fee `from_value`, flashloan-hints deserializer, hook-list deserializer, on-chain order log decoder, settlement event log decoder, eth-flow event log decoder |
| Crypto envelope and hash | EIP-712 order-cancellations hash, EIP-1271 signature data decoder, EIP-1271 magic-value response decoder |
| Order UID and signature classifier | Order UID pack and unpack, signature classifier and signing-scheme discriminant |
| Core types and identities | `Amount` parser, hex identity validators (`Address`, `Hash32`, `AppDataHash`, `OrderUid`, `HexData`), `Amount::parse_units`, redaction body scanner |
| App-data | CID round-trip, CID-to-hex decoder, schema version `is_semver`, `stringify_deterministic`, app-data size limit, `params_from_doc` |
| Trading and slippage | App-data merge, slippage amounts, slippage policy helpers |
| Orderbook wire totals | `calculate_total_fee` |

### Stable-toolchain Compile Gate

The fuzz crate stands alone (excluded from the workspace root) because
`cargo-fuzz` requires a nightly toolchain with unstable `RUSTFLAGS`,
which the rest of the workspace does not consume. The shared quality
gate keeps the harness type-checked against the same stable toolchain
the rest of the repository uses by running
`cargo check --manifest-path fuzz/Cargo.toml` from the workspace root
on every pull request. The `ci` workflow path filter includes `fuzz/**`,
so a change confined to the fuzz crate triggers the same gate. This guards
against type drift between published crate surfaces and fuzz target
imports without forcing the workspace onto nightly.

### Nightly-toolchain Enumerate

Running `cargo +nightly fuzz list --fuzz-dir fuzz` enumerates every
target by its `[[bin]]` name. The same nightly toolchain runs the
targets locally on Linux and macOS, where the LLVM AddressSanitizer
runtime ships with the system clang/llvm package.

### Seed-class Contract

The entire `fuzz/corpus/` tree is gitignored — no corpus directory,
seed file, or README is committed. This is the standard cargo-fuzz
posture: the working corpus (baseline seeds plus the libFuzzer mutation
accumulator) churns constantly and can grow to hundreds of MB, so it
stays in maintainer-local working copies rather than version control.

Each target is seeded locally from three classes:

- **canonical** — at least one seed anchored to a
  `parity/fixtures/*.json` id or a pinned upstream test fixture, so the
  corpus starts from a real, parity-verified input shape.
- **boundary** — at least one input-domain edge: an empty payload,
  all-zero or all-`0xff` bytes, a single-element or capped-maximum list,
  or a numeric extreme.
- **adversarial** — at least one seed from a documented edge case,
  upstream regression, named audit risk, or known historical bug.

The recommended local-disk floor is five files per target. The
per-target class coverage and its parity-fixture provenance are recorded
in this audit and in each harness's doc-comment header, not in a
committed corpus README; the binary seeds themselves are regenerated
locally from those documented classes. A local run that finds a crash
writes the reproducer under the gitignored `fuzz/artifacts/<target>/`.

This posture keeps the public repository footprint to the harness source
under `fuzz/fuzz_targets/` plus this audit, while preserving the
parity-fixture cross-link and the documented seed-class taxonomy. Adding
a target requires only a `fuzz/Cargo.toml` `[[bin]]` entry, the harness
source, and a new coverage row in this audit; the global `.gitignore`
rule covers corpus exclusion without per-target edits.

### Property Traceability

Every target's doc-comment header carries a `**Property:**` row citing
exactly one `PROP-*` invariant identifier from `PROPERTIES.md`. Every
cited identifier has its evidence column updated to reference the
target source file (`fuzz/fuzz_targets/<name>.rs`). The cross-link is the
reviewer's path from a `PROPERTIES.md` row to the fuzz coverage that
strengthens it. The cited identifiers span the `PROP-CORE-*`,
`PROP-CON-*`, `PROP-SIG-*`, `PROP-AD-*`, `PROP-APP-*`, `PROP-OBK-*`,
`PROP-ORD-*`, `PROP-SBG-*`, `PROP-TPP-*`, and `PROP-TRD-*`
families. The set of `PROP-*` rows carrying fuzz-target evidence is
enumerated by `grep -n 'fuzz/fuzz_targets' PROPERTIES.md` rather than a
hardcoded tally, so the cross-link survives target additions and cuts
without a stale count.

### Public-surface Boundary

Every fuzz target imports only published SDK surface. Where the
underlying helper is `pub(crate)` or otherwise unreachable from outside
the owning crate, the target either:

- routes through the nearest public wrapper (with the routing recorded
  in the target's doc-comment header), or
- exercises an adjacent public surface that participates in the same
  data path (e.g. an orderbook rejection envelope through its public
  deserialization seam when the underlying coercion helper is
  crate-private).

The doc-comment header on every target that takes this routing
discloses the boundary and the rationale, so a reviewer can verify the
asserted invariant is meaningful for the public surface even when the
target cannot drive the private helper directly.

The fuzz crate declares `alloy-primitives` as a direct dependency in
`fuzz/Cargo.toml` so the `Amount` parser harness
and the slippage harnesses can construct `alloy_primitives::U256` and
`alloy_primitives::I256` boundary inputs without routing through a
published cow newtype constructor every iteration. The direct
dependency is fuzz-only; the published SDK crates continue to consume
`alloy-primitives` through the canonical primitive layer per
[ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md).

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
  fuzz runs surface them rather than silently accepting them.

### Empirical Run Evidence

A local sweep on a Linux x86-64 host (8-way parallel, 10-minute budget
per target, `timeout=10` per input) covered every target without
producing a panic. Earlier iterations of the same sweep surfaced three
real SDK defects on attacker-controlled surfaces and three over-strict
fuzz-target assertions, all of which were corrected before the clean
run:

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

The two fuzz-target invariants that were corrected are documented in
the target source headers: `fuzz_subgraph_graphql_error_decode` no
longer asserts round-trip equality on
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
- `fuzz/fuzz_targets/` (one source file per target; enumerate with
  `cargo +nightly fuzz list --fuzz-dir fuzz`; each header documents the
  target's seed-class coverage and parity-fixture provenance)
- `fuzz/README.md` (seed-class contract and harness conventions)
- `.gitignore` (global rule that excludes the entire `fuzz/corpus/`
  tree, so the working corpus stays in maintainer-local working copies)
- `.github/workflows/_quality-gate.yml` (stable-toolchain compile gate
  step `Check fuzz crate against the stable toolchain`)
- `.github/workflows/fuzz.yml` (scheduled report-only run lane)
- `PROPERTIES.md` (the `PROP-*` rows with fuzz-target evidence, enumerated
  by `grep -n 'fuzz/fuzz_targets' PROPERTIES.md`)

Primary regression coverage:

- Per-target invariant assertion inside each `fuzz_target!` body
- Workspace quality gate step running `cargo check --manifest-path fuzz/Cargo.toml`

Validation surface:

```text
cargo check --manifest-path fuzz/Cargo.toml
cargo +nightly fuzz list --fuzz-dir fuzz
cargo +nightly fuzz build --fuzz-dir fuzz
```

Local fuzz execution is supported on Linux and macOS targets where the
LLVM AddressSanitizer runtime ships with the system clang or LLVM
package. Local execution on Windows requires the
`clang_rt.asan_dynamic-x86_64.dll` runtime that ships with the MSVC
toolset rather than `rustup`; the build and enumerate steps work on
every nightly-supported host.
