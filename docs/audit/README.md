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

## Contract Bindings And Deployment Authority

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Contract Bindings Parity Audit](contract-bindings-parity-audit.md) | Standing audit | `cow-sdk-contracts` `alloy::sol!`-generated binding surfaces | Byte-identity parity on the shipped settlement, vault-relayer, EthFlow, EIP-1967 proxy, ERC-20 / ERC-20 Permit bindings, and the shared EIP-712 domain-separator fixture | Current | 2026-04-27 |
| [Deployment Registry Audit](deployment-registry-audit.md) | Standing audit | `cow-sdk-contracts::Registry` typed deployment authority | Typed `(ContractId, SupportedChainId, CowEnv)` key, embedded TOML manifest, compile-time validation, and override composition | Current | 2026-04-21 |

## HTTP Transport And Construction

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [HTTP Transport Contract Audit](http-transport-contract-audit.md) | Standing audit | `cow-sdk-core::HttpTransport` trait and its native and browser default adapters | Trait shape, typed `TransportError`, additive response-header surface, orderbook `Retry-After` cooldown honor, URL-redaction contract, and cross-adapter classification parity | Current | 2026-04-24 |
| [Typestate Builder Contract Audit](typestate-builder-contract-audit.md) | Standing audit | `cow-sdk-orderbook::OrderBookApiBuilder` and `cow-sdk-subgraph::SubgraphApiBuilder` construction seams | Required-input typestate, marker sealing, host-policy validation, native default-transport convenience, wasm32 transport-required invariant, and retirement of legacy free-function constructors | Current | 2026-04-27 |

## Signature Verification Caching

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [EIP-1271 Verification Cache Audit](eip1271-verification-cache-audit.md) | Standing audit | `cow-sdk-contracts` `Eip1271VerificationCache` trait and its `Noop` and `InMemory` canonical impls | Trait contract, conservative caching semantics, verification telemetry, pre-interaction scope, thread-safety on the in-memory implementation, and its integration with `verify_eip1271_signature_async` | Current | 2026-04-27 |

## Signature Normalization

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [ECDSA Signature Normalization Audit](ecdsa-signature-normalization-audit.md) | Standing audit | `cow_sdk_contracts::normalized_ecdsa_signature` and `Signature::recover_ecdsa_address` | 65-byte recoverable-signature normalization, typed failure semantics, ECDSA address recovery, declared-address extraction, and parity plus fuzz evidence for the reviewed `27` / `28` contract | Current | 2026-04-27 |

## Browser Wallet

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Browser Wallet Chain Coherence Audit](browser-wallet-chain-coherence-audit.md) | Standing audit | `cow-sdk-browser-wallet` chain-bound signer and typed chain-management contract | Wallet-session chain coherence for browser-wallet-backed flows | Current | 2026-04-21 |
| [Browser Wallet Trust Posture Audit](browser-wallet-trust-posture-audit.md) | Standing audit | `cow-sdk-browser-wallet` EIP-1193 provider construction and wallet chain-management URL payloads | EIP-6963 provider metadata trust, explicit origin opt-in for anonymous providers, redacted trust failures, and wallet URL payload boundaries | Current | 2026-04-27 |
| [Browser-Wallet Alloy Dependency Audit](browser-wallet-alloy-dependency-audit.md) | Standing audit | `cow-sdk-browser-wallet` ABI helper family and reachable alloy advisories | Adoption of `alloy-primitives`, `alloy-dyn-abi`, and `alloy-json-abi` with revisit triggers for the reviewed advisories they transit | Current | 2026-04-27 |

## WASM Example Proof Posture

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [WASM Example Proof-Posture Audit](wasm-example-proof-posture-audit.md) | Standing audit | WASM verification consoles and their two-tier proof posture | Deterministic console proof, mock-versus-injected separation, staging-versus-proxy posture, and the shipped Playwright and wasm-bindgen-test evidence set | Current | 2026-04-27 |

## App-Data And Dependencies

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [CID Dependency Audit](cid-dependency-audit.md) | Standing audit | `cow-sdk-app-data` CID encoding and published dependency boundary | Supported CID paths, fail-closed encoding, and clean published-upstream posture | Current | 2026-04-27 |
| [Dependency Gate Audit](dependency-gate-audit.md) | Standing audit | Release-facing dependency-audit gate for current published surfaces | Blocking transport advisory policy, clean CID posture, canonical advisory tolerance, and source whitelist | Current | 2026-04-27 |

## Source Provenance

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Source-Lock Provenance Audit](source-lock-provenance-audit.md) | Standing audit | source-lock provenance and lifecycle preflight authority | Source-lock commit pins, upstream freshness disclosure, historical snapshot scope, and refresh ownership | Current | 2026-04-28 |

## Transport And Routing

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Partner API Routing Audit](partner-api-routing-audit.md) | Standing audit | `cow-sdk-core` route selection and `cow-sdk-orderbook` partner header assembly | Local partner-route activation and `X-API-Key` request construction | Current | 2026-04-21 |

## Trading Runtime Authority

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Trading Orderbook Context Audit](trading-orderbook-context-audit.md) | Standing audit | `cow-sdk-trading` runtime authority for orderbook-bound helpers | Canonical chain and environment authority when an orderbook client is injected | Current | 2026-04-21 |
| [Trading Quote Orderbook Binding Audit](trading-quote-orderbook-binding-audit.md) | Standing audit | `cow-sdk-trading` quote-origin binding for quote-derived posting | Runtime binding between quote creation and post-from-quote submission | Current | 2026-04-21 |
| [Trading SDK Runtime Prerequisites Audit](trading-sdk-runtime-prerequisites-audit.md) | Standing audit | `cow-sdk-trading` ready-state versus helper-only `TradingSdk` construction | Ready quote/post setup, helper-only setup, and helper prerequisites | Current | 2026-04-29 |

## Trading Order Integrity

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Trading Order Construction Integrity Audit](trading-order-construction-integrity-audit.md) | Standing audit | `cow-sdk-trading` order assembly and recoverable-signature posting contract | Balance semantics, builder terminal parity, and local signature validation | Current | 2026-04-29 |
| [Trading Order-Bounds Validator Audit](trading-order-bounds-validator-audit.md) | Standing audit | `cow-sdk-trading` `OrderBoundsValidator`, order validity bounds, and client rejection surface | Mandatory client-side submission validation, custom bounds plumbing, EthFlow skip rule, and fail-closed builder-level subset | Current | 2026-04-23 |
| [Trading App-Data Merge Audit](trading-app-data-merge-audit.md) | Standing audit | `cow-sdk-trading` quote-to-post app-data edit path | Typed app-data merge, hooks replacement semantics, typed signer derivation, and `merge_and_seal_app_data` / `params_from_doc` public helpers | Current | 2026-04-22 |
| [Trading EthFlow Owner Identity Audit](trading-ethflow-owner-identity-audit.md) | Standing audit | `cow-sdk-trading` EthFlow submission seam | `EthFlowTransaction.from` owner threading, preview identity selection, and EthFlow-aware validator invocation | Current | 2026-04-22 |

## Workspace-Wide Safety And Workflow Security

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Unsafe-Code Policy Audit](unsafe-code-policy-audit.md) | Standing audit | Workspace `unsafe_code = deny` lint declared in `Cargo.toml` workspace lint section | Workspace unsafe-code deny lint, crate lint adoption, public-path source posture, and clippy enforcement | Current | 2026-04-23 |
| [Panic-Free Public Surface Audit](panic-free-public-surface-audit.md) | Standing audit | Every `crates/*/src/**/*.rs` file accessible from the published public API | Public-runtime `expect`, `unwrap`, and `panic!` site set with rationale for every remaining static-invariant panic site | Current | 2026-04-23 |
| [Workflow Security Audit](workflow-security-audit.md) | Standing audit | Every `.github/workflows/*.yml` file | SHA-pinned third-party actions, explicit permissions, reviewed action source refs, and guarded `pull_request_target` use | Current | 2026-04-27 |

## Cross-Cutting Reviewability And Contract Hygiene

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Credential Surface Contract Hygiene Audit](credential-surface-contract-hygiene-audit.md) | Standing audit | Cross-cutting credential redaction and typed partner-fee public boundary | Secret-safe route identity, config diagnostics, URL-bearing config redaction, host-policy failures, `Redacted<T>` wrapper, transport error redaction, and typed user policy | Current | 2026-04-27 |
| [Credential Surface Audit](credential-surface-audit.md) | Standing audit | Credential-bearing builder storage, URL configuration, host-policy errors, wallet add-chain payloads, and Pinata upload-trait headers across orderbook, subgraph, browser-wallet, core, and app-data | Redacted credential storage plus sanitized host-policy failures and typed-redacted header values at the Pinata upload boundary | Current | 2026-04-27 |
| [URL Credential Redaction Audit](url-credential-redaction-audit.md) | Standing audit | URL-bearing public configuration across core, orderbook, subgraph, browser-wallet, and app-data | Redacting URL map and URL value wrappers for public diagnostics with explicit raw dispatch access at HTTP and wallet payload seams | Current | 2026-04-27 |
| [Shared Logic Reviewability Audit](shared-logic-reviewability-audit.md) | Standing audit | Orderbook, signing, and trading shared-logic reviewability boundary | Shared request execution, signing payload preparation, thin posting wrappers, and justified DTO separation | Current | 2026-04-21 |
| [Cooperative Cancellation Contract Audit](cooperative-cancellation-contract-audit.md) | Standing audit | Cross-cutting cooperative cancellation across core, orderbook, subgraph, and trading | Shared `CancellationToken` re-export, the `Cancellable` extension-trait combinator composed at the call site on every long-running public operation of `OrderBookApi`, `SubgraphApi`, and `TradingSdk`, typed `Cancelled` variants with `From<Cancelled>` bridges on every affected error aggregate, and biased-poll drop semantics | Current | 2026-04-21 |
