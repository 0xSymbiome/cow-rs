# Audits

`docs/audit/` contains current-state review records for trust-significant
`cow-rs` surfaces.

> These are internal current-state engineering reviews by the maintainer, not
> independent third-party security audits. They record what was reviewed, the
> conclusion, and the repository-visible evidence that backs it.

## Audit Contract

Each record states:

- the named surface being reviewed
- the current reviewed conclusion
- the code, tests, or public docs that support that conclusion
- the explicit out-of-scope boundary
- the refresh trigger that would invalidate the record

This lane is not for exploratory notes, changelog fragments, ADR replacement, or
generic cleanup diaries.

## Artifact Types

| Type | Filename pattern | Use |
| --- | --- | --- |
| Standing audit | `<surface>-audit.md` | Current-state review record for a stable named surface |
| Fix review | `<surface>-fix-review.md` | Focused public follow-up when reviewed findings are remediated |
| Validation note | `<surface>-validation-note.md` | Narrow confirmation record smaller than a full audit |

The current set is 18 standing audits across the contract, transport, trading,
WASM, and cross-cutting-safety surfaces.

## Status Model

| Status | Meaning |
| --- | --- |
| `Current` | Reviewed against the present implementation and no invalidating change is known |
| `Refresh required` | The reviewed surface changed or a dependency/runtime shift invalidated the record |
| `Superseded` | A newer artifact replaced this one |
| `Archived` | Retained for history but not normative for the current surface |

## Refresh Rule

If a change materially touches an audited surface: identify the affected record,
confirm it is still `Current`, and refresh or supersede it in the same change set
if the reviewed truth moved. If the reviewed surface did not change, leave the
record alone.

## Contracts And On-Chain Surfaces

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Contract Bindings Parity Audit](contract-bindings-parity-audit.md) | Standing audit | `cow-sdk-contracts` `alloy::sol!` bindings | Byte-identity parity on the shipped settlement, EthFlow, on-chain-order event, wrapped-native, and ERC-20 bindings, the shared EIP-712 domain separator, and the wasm `k256` build path | Current | 2026-06-20 |
| [Event Log Decoding Audit](event-log-decoding-audit.md) | Standing audit | `cow-sdk-contracts` `CoWSwapOnchainOrders` and `GPv2Settlement` event decoders | Fail-closed log decoding, topic-0 byte-locks, owner resolution, UID derivation, and the shared topic-set guard | Current | 2026-06-20 |
| [ECDSA Signature Normalization Audit](ecdsa-signature-normalization-audit.md) | Standing audit | `cow_sdk_contracts::RecoverableSignature` and ECDSA recovery | 65-byte canonicalization, typed failure semantics, address recovery, ERC-2098 bridge, and the `27`/`28` recovery-byte contract | Current | 2026-06-20 |
| [EIP-1271 Verification Cache Audit](eip1271-verification-cache-audit.md) | Standing audit | `Eip1271Cache` trait and implementations, plus the WASM EIP-1271 payload parity | Signature-keyed positive-only caching, feature gating, and native/WASM/TS payload vector parity | Current | 2026-06-20 |
| [Deployment Registry Audit](deployment-registry-audit.md) | Standing audit | `cow-sdk-contracts::Registry` deployment authority | Typed key, const CREATE2 address table, per-chain provenance, live presence confirmation, and the Lens chain-taxonomy evidence | Current | 2026-06-20 |
| [COW Shed Contract Bindings Audit](cow-shed-contract-bindings-audit.md) | Standing audit | COW Shed bindings, proxy creation code, and app-data integration | Deployed-generation binding fidelity, selector/creation-code pinning, EIP-712 hashing, ERC-2098 round-trip, and hook app-data schema reuse | Current | 2026-06-20 |

## Transport And Adapters

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [HTTP Transport Contract Audit](http-transport-contract-audit.md) | Standing audit | `cow-sdk-core::HttpTransport` trait, its adapters, and the shared transport policy | Trait shape, `TransportResponse`, typed errors, `Retry-After` cooldown, URL redaction, and the retry/jitter/rate-limit/classification policy surface | Current | 2026-06-20 |
| [Bounded Response Reads Audit](bounded-response-reads-audit.md) | Standing audit | HTTP response reads across core, wasm, and the contracts signature-decode path | Per-client `max_response_bytes` bound on decoded bytes, non-retryable `ResponseTooLarge`, and signature hex pre-decode bounds | Current | 2026-06-20 |
| [Alloy Adapters Audit](alloy-adapters-audit.md) | Standing audit | The native Alloy adapter family, the transaction-lifecycle/receipt types, and the `LogProvider` capability | Read-only provider, local signer typed-data, umbrella composition, broadcast-only submission, receipt shape, single-call `get_logs`, redaction, cancellation, and dependency boundaries | Current | 2026-06-20 |

## Trading

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Trading Order Integrity Audit](trading-order-integrity-audit.md) | Standing audit | `cow-sdk-trading` order assembly, bounds validation, app-data merge, and EthFlow owner threading | Balance/same-token policy, builder-terminal parity, local signature validation, the post-sign owner-recovery gate, the `OrderBoundsValidator` client-rejection surface, the typed app-data merge, and EthFlow owner identity | Current | 2026-06-20 |

## TypeScript-Callable WASM

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [WASM Surface Audit](wasm-surface-audit.md) | Standing audit | `cow-sdk-wasm` TypeScript-callable crate, npm package, and runtime boundary | Surface/exports, capability coverage, type generation and schema versioning, the callback boundary, facade architecture and API stability, the performance budget, unsupported-target diagnostics, and the deterministic browser runner | Current | 2026-06-20 |

## Cross-Cutting Safety And Hygiene

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Credential Redaction Audit](credential-redaction-audit.md) | Standing audit | Cross-cutting credential redaction across config, transport, RPC, orderbook, subgraph, native Alloy, and wasm error surfaces | `Redacted<T>` storage, URL map/value wrappers, per-error-family redaction, subgraph `Display` non-tautology, and native Alloy opaque `Debug` | Current | 2026-06-21 |
| [Error Classification Audit](error-classification-audit.md) | Standing audit | The `class()` accessors on the error family, the shared `ErrorClass`, and signer-rejection routing | `ErrorClass` partition, per-type `class()`, composite granularity, typed-discriminant redaction, and the `UserRejection` signer-rejection routing | Current | 2026-06-21 |
| [Panic-Free Public Surface Audit](panic-free-public-surface-audit.md) | Standing audit | Every published-API-reachable `crates/*/src/**` file | The remaining static-invariant panic-site set with per-site rationale and allowlist coverage | Current | 2026-06-20 |
| [Fuzz Coverage Audit](fuzz-coverage-audit.md) | Standing audit | The `cow-sdk-fuzz` crate and its `cargo-fuzz` targets | Target inventory, per-target seed contract, stable-toolchain compile gate, property traceability, and the public-surface boundary | Current | 2026-06-20 |
| [Dependency Gate Audit](dependency-gate-audit.md) | Standing audit | The release-facing dependency-audit gate, including the CID dependency posture | Blocking advisory policy, CID encoding posture, WASM randomness alignment, advisory tolerance, source whitelist, adapter allow-lists, and wasm32 exclusions | Current | 2026-06-20 |
| [Workflow Security Audit](workflow-security-audit.md) | Standing audit | Every `.github/workflows/*.yml` file | SHA-pinned third-party actions, explicit permissions, reviewed action refs, and guarded `pull_request_target` use | Current | 2026-06-20 |
| [Source-Lock Provenance Audit](source-lock-provenance-audit.md) | Standing audit | Source-lock provenance and release preflight authority | Commit pins, per-file fixture provenance, upstream freshness disclosure, Alloy provenance, publication preflight, and refresh ownership | Current | 2026-06-20 |
