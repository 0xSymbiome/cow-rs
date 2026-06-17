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
- [Credential Redaction Audit](credential-redaction-audit.md)
- [ECDSA Signature Normalization Audit](ecdsa-signature-normalization-audit.md)
- [Trading Order Integrity Audit](trading-order-integrity-audit.md)

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

Beyond confirming the scheduled run lane exists and its public shape, it does not
cover fuzz execution internals. A report-only `fuzz` workflow
(`.github/workflows/fuzz.yml`) runs every target for a bounded budget on a weekly
cron plus manual dispatch; it is non-gating and never a pull-request check. The
libFuzzer mutation strategy, per-run corpus growth, and sanitizer-runtime
requirements are out of scope for this static review.

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
`cargo +nightly fuzz list --fuzz-dir fuzz`; this audit describes coverage by
boundary class rather than a count that would rot on every add or cut.

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
`cargo-fuzz` requires a nightly toolchain with unstable `RUSTFLAGS` the rest of
the workspace does not consume. The shared quality gate keeps the harness
type-checked against the same stable toolchain by running
`cargo check --manifest-path fuzz/Cargo.toml` on every pull request; the `ci`
workflow path filter includes `fuzz/**`, so a fuzz-only change still triggers the
gate. This guards against type drift between published crate surfaces and fuzz
imports without forcing the workspace onto nightly.

### Nightly-toolchain Enumerate

`cargo +nightly fuzz list --fuzz-dir fuzz` enumerates every target by its
`[[bin]]` name. The same nightly toolchain runs the targets locally on Linux and
macOS, where the LLVM AddressSanitizer runtime ships with the system clang/llvm
package.

### Seed-class Contract

The entire `fuzz/corpus/` tree is gitignored per the standard cargo-fuzz posture:
the working corpus churns constantly and stays in maintainer-local working copies
rather than version control. Each target is seeded locally from three classes:

- **canonical** — at least one seed anchored to a `parity/fixtures/*.json` id or
  a pinned upstream test fixture, so the corpus starts from a parity-verified
  input shape.
- **boundary** — at least one input-domain edge (empty payload, all-zero or
  all-`0xff` bytes, single-element or capped-maximum list, or numeric extreme).
- **adversarial** — at least one seed from a documented edge case, upstream
  regression, named audit risk, or known historical bug.

The recommended local-disk floor is five files per target. Per-target class
coverage and parity-fixture provenance are recorded in this audit and in each
harness's doc-comment header; the binary seeds are regenerated locally from those
classes. A local run that finds a crash writes the reproducer under the gitignored
`fuzz/artifacts/<target>/`.

### Property Traceability

Every target's doc-comment header carries a `**Property:**` row citing exactly
one `PROP-*` invariant identifier from `PROPERTIES.md`, and every cited identifier
references the target source file in its evidence column — the reviewer's path
from a `PROPERTIES.md` row to the fuzz coverage that strengthens it. The cited
identifiers span the `PROP-CORE-*`, `PROP-CON-*`, `PROP-SIG-*`, `PROP-AD-*`,
`PROP-APP-*`, `PROP-OBK-*`, `PROP-ORD-*`, `PROP-SBG-*`, `PROP-TPP-*`, and
`PROP-TRD-*` families. The set carrying fuzz-target evidence is enumerated by
`grep -n 'fuzz/fuzz_targets' PROPERTIES.md` rather than a hardcoded tally.

### Public-surface Boundary

Every fuzz target imports only published SDK surface. Where the underlying helper
is `pub(crate)`, the target routes through the nearest public wrapper or an
adjacent public surface on the same data path, with the routing recorded in the
target's doc-comment header so a reviewer can verify the invariant is meaningful.
The fuzz crate declares `alloy-primitives` as a fuzz-only direct dependency so the
`Amount` and slippage harnesses can construct `U256`/`I256` boundary inputs
directly; the published SDK crates continue to consume `alloy-primitives` through
the canonical primitive layer per
[ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md).

### Invariant Strength

Every target asserts at least one semantic property beyond panic-freedom:

- **Encoder targets**: selector equality against `keccak256(signature)[0..4]`,
  decoder round-trip equality, encoded-length structural floor, and
  canonical-encoding determinism.
- **Crypto envelope targets**: independent keccak256 envelope equality, two-call
  determinism, mutation-resistance on every typed-data domain field.
- **Classifier and parser targets**: typed-error-partition coverage, two-call
  determinism, sanitization of rendered `Display`/`Debug` output (no raw newline
  or null byte, no URL userinfo, no JWT/`Bearer` prefix, no credential key=value).
- **Round-trip targets**: `from(to(x)) == x` byte-equivalent reconstruction plus
  encoder-idempotency where serialization is part of the contract.
- **Validator targets**: explicit enumeration of every documented rejection
  variant, with new variants triggering an explicit panic so fuzz runs surface
  them.

### Empirical Run Evidence

A local sweep on a Linux x86-64 host (8-way parallel, 10-minute budget per target,
`timeout=10` per input) covered every target without a panic. Earlier iterations
surfaced and fixed three real SDK defects on attacker-controlled surfaces —
`redact_response_body` (URL userinfo with mangled scheme prefixes, bare `Bearer`
tokens, embedded JWTs, partial credential keys), `parse_retry_after` (`i64`
civil-day arithmetic against IMF-fixdate year overflow), and
`calculate_quote_amounts_and_costs` (explicit `protocol_fee_bps >= 100%` guard
against a divide-by-zero) — and two over-strict fuzz assertions
(`fuzz_subgraph_graphql_error_decode` no longer asserts round-trip on `Redacted<T>`
fields; `fuzz_stringify_deterministic` no longer asserts byte-level JSON
idempotence on arbitrary `serde_json::Value`). The shipped canonical-form
stability is verified by the parity fixture and
`crates/app-data/tests/property_contract.rs`.

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

Local fuzz execution runs on Linux and macOS, where the LLVM AddressSanitizer
runtime ships with the system clang/LLVM package; the build and enumerate steps
work on every nightly-supported host.
