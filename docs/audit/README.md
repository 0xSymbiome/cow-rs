# Audits

`docs/audit/` contains public current-state review records for
trust-significant `cow-rs` surfaces.

## Audit Contract

Each public audit should record:

- the named surface being reviewed
- the current reviewed conclusion
- the code, tests, or public docs that support that conclusion
- the explicit out-of-scope boundary
- the refresh trigger that would invalidate the current record

Public audits are self-sufficient current-state review records for named
surfaces once the reviewed contract and supporting evidence are stable enough
to publish clearly. This lane is not for exploratory notes, changelog
fragments, ADR replacement, or generic cleanup diaries.

## Artifact Types

| Type | Filename pattern | Use |
| --- | --- | --- |
| Standing audit | `<surface>-audit.md` | Canonical current-state review record for a stable named surface |
| Fix review | `<surface>-fix-review.md` | Focused public follow-up when previously reviewed findings are remediated |
| Validation note | `<surface>-validation-note.md` | Narrow public confirmation record for an assurance point smaller than a full audit |

The current public set is entirely standing audits.

## Status Model

| Status | Meaning |
| --- | --- |
| `Current` | Reviewed against the present implementation and no invalidating change is known |
| `Refresh required` | The reviewed surface changed or a dependency/runtime shift invalidated the record |
| `Superseded` | A newer artifact replaced this one |
| `Archived` | Retained for history but not normative for the current surface |

## Refresh Rule

If a change materially touches an audited surface:

1. identify the affected audit record
2. confirm it is still `Current`
3. refresh or supersede it in the same change set if the reviewed truth moved

If the reviewed surface did not change, leave the audit alone.

## Browser Wallet

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Browser Wallet Chain Coherence Audit](browser-wallet-chain-coherence-audit.md) | Standing audit | `cow-sdk-browser-wallet` chain-bound signer and typed chain-management contract | Wallet-session chain coherence for browser-wallet-backed flows | Current | 2026-04-15 |
| [Browser-Wallet Alloy Dependency Audit](browser-wallet-alloy-dependency-audit.md) | Standing audit | `cow-sdk-browser-wallet` ABI helper family and reachable alloy proc-macro advisories | Adoption of `alloy-primitives`, `alloy-dyn-abi`, and `alloy-json-abi` with revisit triggers for the two proc-macro advisories they transit | Current | 2026-04-16 |

## WASM Example Proof Posture

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [WASM Example Proof-Posture Audit](wasm-example-proof-posture-audit.md) | Standing audit | WASM verification consoles and their two-tier proof posture | Deterministic console proof, mock-versus-injected separation, staging-versus-proxy posture, and the shipped Playwright and wasm-bindgen-test evidence set | Current | 2026-04-17 |

## App-Data And Dependencies

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [CID Dependency Audit](cid-dependency-audit.md) | Standing audit | `cow-sdk-app-data` CID encoding and published dependency boundary | Supported CID paths, fail-closed encoding, and published-upstream posture | Current | 2026-04-15 |
| [Dependency Gate Audit](dependency-gate-audit.md) | Standing audit | Release-facing dependency-audit gate for current published surfaces | Blocking transport advisory policy and reviewed CID warning posture | Current | 2026-04-15 |

## Transport And Routing

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Partner API Routing Audit](partner-api-routing-audit.md) | Standing audit | `cow-sdk-core` route selection and `cow-sdk-orderbook` partner header assembly | Local partner-route activation and `X-API-Key` request construction | Current | 2026-04-15 |

## Trading Runtime Authority

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Trading Orderbook Context Audit](trading-orderbook-context-audit.md) | Standing audit | `cow-sdk-trading` runtime authority for orderbook-bound helpers | Canonical chain and environment authority when an orderbook client is injected | Current | 2026-04-15 |
| [Trading Quote Orderbook Binding Audit](trading-quote-orderbook-binding-audit.md) | Standing audit | `cow-sdk-trading` quote-origin binding for quote-derived posting | Runtime binding between quote creation and post-from-quote submission | Current | 2026-04-15 |
| [Trading SDK Runtime Prerequisites Audit](trading-sdk-runtime-prerequisites-audit.md) | Standing audit | `cow-sdk-trading` ready-state versus partial `TradingSdk` construction | Ready quote/post setup, partial helper-only setup, and helper prerequisites | Current | 2026-04-15 |

## Trading Order Integrity

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Trading Order Construction Integrity Audit](trading-order-construction-integrity-audit.md) | Standing audit | `cow-sdk-trading` order assembly and recoverable-signature posting contract | Balance semantics, constructor parity, and local signature validation | Current | 2026-04-16 |

## Cross-Cutting Reviewability And Contract Hygiene

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Credential Surface Contract Hygiene Audit](credential-surface-contract-hygiene-audit.md) | Standing audit | Cross-cutting credential redaction and typed partner-fee public boundary | Secret-safe route identity, config diagnostics, `Redacted<T>` wrapper, transport error redaction, and typed user policy | Current | 2026-04-17 |
| [Shared Logic Reviewability Audit](shared-logic-reviewability-audit.md) | Standing audit | Orderbook, signing, and trading shared-logic reviewability boundary | Shared request execution, signing payload preparation, thin posting wrappers, and justified DTO separation | Current | 2026-04-15 |
| [Cooperative Cancellation Contract Audit](cooperative-cancellation-contract-audit.md) | Standing audit | Cross-cutting cooperative cancellation across core, orderbook, subgraph, and trading | Shared `CancellationToken` re-export, `_with_cancellation` entry points, typed `Cancelled` variants, and biased `select!` propagation | Current | 2026-04-17 |
