# Principles

These principles define the public engineering posture of `cow-rs`.

## Deterministic Protocol Transforms

Hashing, signing, UID packing, app-data encoding, and CID handling must stay
deterministic for the same canonical input. Domain identities that share an
underlying byte width remain type-level distinct per
[ADR 0052](adr/0052-alloy-primitives-canonical-primitive-layer.md), so a
transform cannot silently consume the wrong domain's bytes.

**Anchored by**: [ADR 0012](adr/0012-alloy-sol-bindings-and-registry-authority.md) (primary). Supporting: [ADR 0011](adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md), [ADR 0022](adr/0022-ecdsa-signature-v-normalization.md), [ADR 0023](adr/0023-legacy-compatibility-shim-removal.md), [ADR 0052](adr/0052-alloy-primitives-canonical-primitive-layer.md).

## Explicit Runtime Boundaries

Pure transform crates do not perform hidden HTTP, RPC, GraphQL, or pinning I/O.
Runtime interaction belongs in explicit clients and adapters.

**Anchored by**: [ADR 0010](adr/0010-runtime-neutral-async-and-transport-posture.md) (primary). Supporting: [ADR 0019](adr/0019-http-transport-sole-dispatch.md).

## Thin Facade, Real Crate Boundaries

`cow-sdk` is the ergonomic entrypoint, not a second implementation layer.
Leaf crates own transport, orchestration, browser integration, and other
specialized behavior.

**Anchored by**: [ADR 0001](adr/0001-multi-crate-sdk-family-with-thin-facade.md) (primary). Supporting: [ADR 0002](adr/0002-dedicated-trading-orchestration-crate.md), [ADR 0003](adr/0003-separate-read-only-subgraph-crate.md), [ADR 0008](adr/0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md).

## Instance-Scoped Configuration

Policy-heavy behavior such as quote settings, transport tuning, caching, and
provider selection must be configured per instance through typed builders or
options. `cow-rs` does not hide process-global mutable state behind convenience
APIs.

**Anchored by**: [ADR 0006](adr/0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md) (primary). Supporting: none.

## Strong Typed Public Surfaces

Public Rust APIs prefer domain types for protocol meanings such as addresses,
hashes, identifiers, and amounts. String-heavy representations are reserved for
explicit wire contracts and compatibility boundaries.

Public protocol-driven and upstream-growing enums carry `#[non_exhaustive]` so
future additive variants do not break exhaustive matches. SDK-local
state-machine enums may be exhaustive when documented in the workspace enum
policy manifest. Public response DTOs preserve unknown fields under `serde`
defaults rather than `deny_unknown_fields`, so upstream-services additions
remain backward-compatible. Wire-DTO field shapes are derived from the
source-lock-pinned OpenAPI inventory, not from hand-written prior memory.

**Anchored by**: [ADR 0011](adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md) (primary). Supporting: [ADR 0005](adr/0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0015](adr/0015-client-side-order-bounds-validator.md), [ADR 0016](adr/0016-split-sell-and-buy-token-balance-enums.md), [ADR 0017](adr/0017-typed-orderbook-rejection-parser.md), [ADR 0018](adr/0018-typed-app-data-merge.md), [ADR 0021](adr/0021-orderbook-total-fee-policy.md), [ADR 0052](adr/0052-alloy-primitives-canonical-primitive-layer.md).

## Type The Lifecycle

Public transaction APIs name the lifecycle state they actually observe.
Signer-backed submission returns `TransactionBroadcast`, which carries only the
broadcast hash. Provider receipt lookup returns `TransactionReceipt`, which may
carry mined execution status, block, gas, sender, and recipient fields when the
runtime exposes them. Submission must not hide implicit receipt polling behind a
hash-shaped return value.

**Anchored by**: [ADR 0038](adr/0038-transaction-lifecycle-types.md) (primary). Supporting: [ADR 0010](adr/0010-runtime-neutral-async-and-transport-posture.md), [ADR 0024](adr/0024-asyncprovider-asyncsigningprovider-capability-split.md), [ADR 0029](adr/0029-trait-evolution-extension-traits.md).

## Additive Optional Ecosystems

Optional capabilities grow through leaf crates and feature-gated additions.
Browser-runtime behavior, provider-specific behavior, and future capability
families do not silently widen the default facade contract.

**Anchored by**: [ADR 0008](adr/0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md) (primary). Supporting: [ADR 0004](adr/0004-feature-gated-browser-wallet-sidecar.md), [ADR 0007](adr/0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md), [ADR 0009](adr/0009-wasm-verification-consoles-hybrid-extensibility-and-two-tier-proof.md).

## Sole Construction Seam

`OrderBookApi`, `SubgraphApi`, and `TradingSdk` construct exclusively through
their typestate builders. The required inputs (chain, environment or API key,
transport, appCode) are encoded as compile-time markers so a misconstructed
client is a build error rather than a first-quote runtime surprise. **No
inherent associated constructors remain on any of the three except
`builder()`**; ergonomic shortcuts ship as builder-terminal methods that
consume *total* typed inputs and never `Partial*` shapes. Builder typestate
marker types use private tuple fields so external crates cannot construct them.

**Anchored by**: [ADR 0013](adr/0013-http-transport-injection-and-typestate-builders.md) (primary). Supporting: [ADR 0011](adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md).

## Chain-RPC Runtime Neutrality

The default SDK path stays provider-neutral. Consumers own their chain-RPC
runtime through the `Provider` seam in `cow-sdk-core`, while native Alloy
support is available only through explicit adapter crates and facade features.
The `alloy-provider` and `alloy-signer-local` allow-list checks are
release-gating invariants rather than aspirations.

The trait abstraction is the mechanism that keeps a single trading path working
across runtimes. `cow-sdk-trading` depends on `cow-sdk-core` traits rather than
on a concrete provider library, so native Alloy, the browser-wallet leaf, and a
custom simulator or fork-test adapter can all satisfy the same helper calls
without widening the default facade.

**Anchored by**: [ADR 0024](adr/0024-asyncprovider-asyncsigningprovider-capability-split.md) (primary). Supporting: [ADR 0010](adr/0010-runtime-neutral-async-and-transport-posture.md), [ADR 0014](adr/0014-eip1271-verification-cache.md), [ADR 0028](adr/0028-account-abstraction-integration-plan.md).

**Operational doctrine**: [Alloy Doctrine](alloy-doctrine.md) — the
three-bucket classification (ALWAYS-ALLOY, COW-OWNED, BOUNDARY-ADAPTER)
and the decision tree for when a new primitive joins each bucket.

## Canonical Contract Bindings

Every ABI binding the SDK emits call-data against is generated through
`alloy::sol!` from byte-identical Solidity mirrors committed under
`crates/contracts/abi/` and gated by `cargo parity-verify-sol-provenance`
against SHA-256 rows in `parity/source-lock.yaml`. Hand-rolled encoders
are not allowed in shipped crates, and every chain-scoped address lookup
routes through the typed `Registry` authority in `cow-sdk-contracts`.
The canonical EVM primitive layer is `alloy_primitives` and the canonical
EIP-712 / Solidity-binding layer is `alloy_sol_types` per
[ADR 0052](adr/0052-alloy-primitives-canonical-primitive-layer.md).

**Anchored by**: [ADR 0012](adr/0012-alloy-sol-bindings-and-registry-authority.md) (primary). Supporting: [ADR 0020](adr/0020-ethflow-owner-threading.md), [ADR 0022](adr/0022-ecdsa-signature-v-normalization.md), [ADR 0023](adr/0023-legacy-compatibility-shim-removal.md), [ADR 0052](adr/0052-alloy-primitives-canonical-primitive-layer.md), [ADR 0054](adr/0054-onchain-order-event-decoding-is-fail-closed.md).

**Operational doctrine**: [Alloy Doctrine](alloy-doctrine.md) — the
three-bucket classification (ALWAYS-ALLOY, COW-OWNED, BOUNDARY-ADAPTER)
and the decision tree for when a new primitive joins each bucket.

## Evidence-Backed Public Claims

Compatibility, support posture, parity, and release claims must be justified by
repository-visible tests, examples, fixtures, and reproducible validation
documentation.

Source-lock provenance is reproducible from the upstream commit hash. Local
upstream snapshots are reference-only and never substitute for git checkouts at
the pinned commits during release validation.

Wire-DTO coverage for upstream-services-controlled types is driven by the
source-lock-pinned OpenAPI schema inventory at `parity/openapi/`. Each public
response DTO has its own coverage target. `Order` and `AuctionOrder` are
separate schemas and require separate Rust types and separate inventory files.
Regression fixtures recorded from live or replayed services responses prove
every modeled field round-trips.

Deployment authority claims are backed by structured provenance entries in
`crates/contracts/deployment-provenance.yaml` plus release-mode live
confirmation that records `code_hash` and (where ABI permits) selector probes.
Skipped live checks are never silently allowed in release mode.

**Anchored by**: [ADR 0026](adr/0026-alloy-major-release-absorption-plan.md) (primary). Supporting: [ADR 0025](adr/0025-workspace-url-redaction-convention.md), [ADR 0030](adr/0030-workspace-locked-versioning-tag-baseline.md), [ADR 0032](adr/0032-deployment-authority-machine-readable-provenance.md), [ADR 0052](adr/0052-alloy-primitives-canonical-primitive-layer.md).

## Forward-Compatible Public Surfaces

Public protocol-driven and upstream-growing enums use `#[non_exhaustive]`. The
workspace `enum-policy.yaml` manifest classifies every public enum as
`protocol-fixed-exhaustive`, `upstream-growing`, `sdk-local-state`, or
`private-leak`; CI enforces the manifest. Public response DTOs preserve unknown
fields under `serde` defaults so upstream additions remain backward-compatible.
Public traits evolve through extension traits (the `*Ext` pattern) rather than
silently adding methods. Adding `#[must_use]` and `# Errors` doc sections to
fallible public APIs is a release-gating lint.

**Anchored by**: [ADR 0031](adr/0031-wire-dto-openapi-driven-with-order-auction-order-split.md) (primary). Supporting: [ADR 0027](adr/0027-post-quantum-signing-absorption-plan.md), [ADR 0029](adr/0029-trait-evolution-extension-traits.md).

## Credential Redaction by Construction

Credential-bearing types use the workspace `Redacted<T>` wrapper. Their
`Debug`, `Display`, `Serialize`, and panic-path renderings emit only sanitised
identity information. Transport errors strip credential-bearing query strings
before wrapping URL strings; orderbook and subgraph diagnostics expose only
redacted route identity. Public error diagnostics for provider, signer, RPC,
transport, response-body, orderbook rejection, subgraph context, browser-wallet,
and facade errors keep credential-bearing message payloads behind explicit
`Redacted<T>` access or typed sanitizers, while safe diagnostics such as chain
IDs, schema versions, environment labels, and sanitized orderbook rejection tags
remain visible. `crates/sdk/tests/error_redaction_contract.rs` proves the
contract by rendering the reviewed error families with URL, bearer-token,
private-key-shaped, and PEM-shaped payloads across `Debug`, `Display`, and
existing `Serialize` surfaces. No code path bypasses redaction through `Deref`
or transparent re-exports of the inner string.

Credential-bearing types in the native Alloy adapter family also avoid derived
`Debug`. Hand-written implementations print opaque placeholders for configured
RPC URLs, private keys, and inner signers so wrapper diagnostics cannot leak
credentials through `Display`, `Debug`, or `Error::source()`.

**Anchored by**: [ADR 0025](adr/0025-workspace-url-redaction-convention.md) (primary). Supporting: [ADR 0005](adr/0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0010](adr/0010-runtime-neutral-async-and-transport-posture.md).

## Cooperative Cancellation Coverage

Every long-running async public method on `OrderBookApi`, `SubgraphApi`,
`TradingSdk`, or any future client is composable with
`cow_sdk_core::Cancellable::cancel_with(&token)`. The error aggregate of every
public API lifts `Cancelled` through `From`. Cancellation is cooperative —
callers own the token, and the SDK never installs hidden global cancellation
state.

**Anchored by**: [ADR 0010](adr/0010-runtime-neutral-async-and-transport-posture.md) (primary). Supporting: [ADR 0006](adr/0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md).

## Minimum-Viable Panic Surface

Production code in shipped crates does not contain
`unwrap`/`expect`/`panic!`/`unreachable!`/`todo!`/`unimplemented!` outside of
statically-invariant compile-time guarantees. Each statically-invariant panic
site carries a `# Panics` rustdoc section on its public function and an inline
`// SAFETY:` comment naming the build-time invariant. The file
`.github/config/panic-allowlist.yaml` keys allowed sites by item path; the
regression contract fails on uncommented additions.

**Anchored by**: [ADR 0033](adr/0033-minimum-viable-panic-surface.md) (primary). Supporting: none.

## Off-Chain Orchestration Boundary

Composable order helpers expose deterministic encoders, decoders, selectors,
event payloads, and single-call provider operations. They do not embed
production watcher loops, persistence adapters, notification integrations,
automatic order posting, or hidden retry schedulers. Long-running orchestration
belongs to applications and services built on top of the SDK primitives.

**Required**: yes.

**Anchored by**: [ADR 0048](adr/0048-composable-conditional-order-framework.md) (primary). Supporting: [ADR 0010](adr/0010-runtime-neutral-async-and-transport-posture.md), [ADR 0024](adr/0024-asyncprovider-asyncsigningprovider-capability-split.md), [ADR 0050](adr/0050-eip1271-signature-blob-encoding.md).
