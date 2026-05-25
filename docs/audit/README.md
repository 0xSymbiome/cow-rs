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

The current public set is 53 standing audits and 1 validation note.

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
| [Contract Bindings Parity Audit](contract-bindings-parity-audit.md) | Standing audit | `cow-sdk-contracts` `alloy::sol!`-generated binding surfaces | Byte-identity parity on the shipped settlement, vault-relayer, EthFlow, EIP-1967 proxy, ERC-20 / ERC-20 Permit bindings, and the shared EIP-712 domain-separator fixture | Current | 2026-05-22 |
| [Deployment Registry Audit](deployment-registry-audit.md) | Standing audit | `cow-sdk-contracts::Registry` typed deployment authority | Typed `(ContractId, SupportedChainId, CowEnv)` key, embedded TOML manifest, per-chain provenance, compile-time validation, and override composition | Current | 2026-05-12 |
| [Composable Contract Bindings Audit](composable-contract-bindings-audit.md) | Standing audit | Byte-identical composable-cow Solidity mirrors and deployment registry rows | Conditional-order binding mirrors, schema v2 deployment rows, provenance evidence, and coverage-only Ink posture | Current | 2026-05-15 |
| [Composable Watch-Tower Boundary Audit](composable-watch-tower-boundary-audit.md) | Standing audit | Composable helper crate boundary | Selector, decoder, event, and single-call provider helper scope with production watcher behavior explicitly out of scope | Current | 2026-05-15 |
| [COW Shed Contract Bindings Audit](cow-shed-contract-bindings-audit.md) | Standing audit | Byte-identical COW Shed Solidity mirrors, proxy creation code, and deployment registry rows | Version-keyed proxy creation-code artifacts, digest validation, deployment rows, and factory ABI evidence | Current | 2026-05-22 |
| [COW Shed App-Data Integration Audit](cow-shed-app-data-integration-audit.md) | Standing audit | COW Shed hook metadata and app-data schema integration | Hook metadata shape, app-data schema reuse, and EIP-1271 signing-boundary evidence | Current | 2026-05-15 |
| [Lens Chain Evidence Audit](lens-chain-evidence-audit.md) | Standing audit | Deployment registry chain taxonomy | Lens deployment evidence, runtime support exclusion, provenance lockstep, public route probes, and refresh triggers | Current | 2026-05-15 |

## HTTP Transport And Construction

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [HTTP Transport Contract Audit](http-transport-contract-audit.md) | Standing audit | `cow-sdk-core::HttpTransport` trait and its native and browser default adapters | Trait shape, typed `TransportError`, additive response-header surface, orderbook `Retry-After` cooldown honor, URL-redaction contract, and cross-adapter classification parity | Current | 2026-05-14 |
| [Transport Policy Coverage Audit](transport-policy-coverage-audit.md) | Standing audit | `cow-sdk-transport-policy` public retry, jitter, rate-limit, classification, and `Retry-After` parser surfaces | `parse_retry_after` accept/reject contract across delta-seconds and IMF-fixdate forms, civil-day arithmetic panic-free posture, `JitterStrategy` delay-window invariants, `RetryPolicy` decision points and backoff clamps, `RequestRateLimiter` scope and cancellation, `NetworkErrorKind::from_transport_error_class` total mapping, and the optional `reqwest-classifier` dispatch | Current | 2026-05-17 |
| [Typestate Builder Contract Audit](typestate-builder-contract-audit.md) | Standing audit | `cow-sdk-orderbook::OrderBookApiBuilder`, `cow-sdk-subgraph::SubgraphApiBuilder`, `cow-sdk-trading::TradingSdkBuilder`, and native Alloy adapter builders | Required-input typestate, marker sealing, host-policy validation, native default-transport convenience, wasm32 transport-required and injected-orderbook invariants, validated `AppCode`, distinct `TradingSdk`/`HelperOnlySdk` terminals, native Alloy adapter terminals, and retirement of legacy free-function constructors | Current | 2026-05-12 |

## Native Alloy Adapters

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Alloy Provider Adapter Audit](alloy-provider-adapter-audit.md) | Standing audit | `cow-sdk-alloy-provider` `RpcAlloyProvider`, its builder, and its `AsyncProvider` implementation | Read-only provider methods, HTTP typestate construction, redacted error classification, `read_contract` ABI handling, rich receipt population, doc-hidden helper seam, and dependency boundary | Current | 2026-05-14 |
| [Alloy Signer Adapter Audit](alloy-signer-adapter-audit.md) | Standing audit | `cow-sdk-alloy-signer` `LocalAlloyKeystoreSigner`, its builder, and its `AsyncSigner` implementation | Local-keystore message and typed-data signing, primary-type preservation, signature normalization, provider-required transaction methods, redacted errors, cancellation bridge, and dependency boundary | Current | 2026-05-07 |
| [Alloy Umbrella Adapter Audit](alloy-umbrella-adapter-audit.md) | Standing audit | `cow-sdk-alloy` `AlloyClient`, its builder, provider implementation, and owned signer handle | Wallet-filler composition, provider delegation, typed-data signing, broadcast-only transaction submission, no-broadcast raw signing deferral, redaction, cancellation, and dependency boundary | Current | 2026-05-14 |
| [Transaction Receipt Shape Audit](transaction-receipt-shape-audit.md) | Standing audit | `cow-sdk-core` transaction lifecycle types and adapter receipt conversions | Broadcast acknowledgement type, mined receipt shape, Alloy status mapping, browser-wallet strict parsing, and cross-adapter timing | Current | 2026-05-13 |
| [WASM Unsupported Target Audit](wasm-unsupported-target-audit.md) | Standing audit | Native Alloy adapter crates and facade Alloy features on `wasm32` | Compile-time unsupported-target diagnostics and browser-runtime guidance for Alloy-adapter feature selection | Current | 2026-05-06 |

## Signature Verification Caching

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [EIP-1271 Verification Cache Audit](eip1271-verification-cache-audit.md) | Standing audit | `cow-sdk-contracts` `Eip1271VerificationCache` trait and its `Noop` and `InMemory` canonical impls | Trait contract, conservative caching semantics, verification telemetry, pre-interaction scope, thread-safety on the in-memory implementation, and its integration with `verify_eip1271_signature_async` | Current | 2026-05-01 |

## Signature Normalization

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [ECDSA Signature Normalization Audit](ecdsa-signature-normalization-audit.md) | Standing audit | `cow_sdk_contracts::normalized_ecdsa_signature` and `Signature::recover_ecdsa_address` | 65-byte recoverable-signature normalization, typed failure semantics, ECDSA address recovery, declared-address extraction, and parity plus fuzz evidence for the reviewed `27` / `28` contract | Current | 2026-05-17 |

## Browser Wallet

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Browser Wallet Chain Coherence Audit](browser-wallet-chain-coherence-audit.md) | Standing audit | `cow-sdk-browser-wallet` chain-bound signer and typed chain-management contract | Wallet-session chain coherence for browser-wallet-backed flows | Current | 2026-05-13 |
| [Browser Wallet Trust Posture Audit](browser-wallet-trust-posture-audit.md) | Standing audit | `cow-sdk-browser-wallet` EIP-1193 provider construction and wallet chain-management URL payloads | EIP-6963 provider metadata trust, explicit origin opt-in for anonymous providers, redacted trust failures, and wallet URL payload boundaries | Current | 2026-05-22 |
| [Browser-Wallet Alloy Dependency Audit](browser-wallet-alloy-dependency-audit.md) | Standing audit | `cow-sdk-browser-wallet` ABI helper family and reachable alloy advisories | Adoption of `alloy-primitives`, `alloy-dyn-abi`, and `alloy-json-abi` with revisit triggers for the reviewed advisories they transit, plus explicit separation from native Alloy adapter dependencies | Current | 2026-05-13 |

## WASM Example Proof Posture

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [WASM Example Proof-Posture Audit](wasm-example-proof-posture-audit.md) | Standing audit | WASM verification consoles and their two-tier proof posture | Deterministic console proof, mock-versus-injected separation, staging-versus-proxy posture, and the shipped Playwright and wasm-bindgen-test evidence set | Current | 2026-04-30 |
| [WASM Browser Runner Determinism Audit](wasm-browser-runner-determinism-audit.md) | Standing audit | Pinned Chrome-for-Testing runner used by browser-targeted WASM validation lanes | Committed browser-runner pin, setup command, freshness gate, and workflow use of pinned Chrome/chromedriver for wasm-pack tests | Current | 2026-05-01 |

## TypeScript-Callable WASM SDK

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [WASM Surface Audit](wasm-surface-audit.md) | Standing audit | `cow-sdk-wasm` TypeScript-callable crate, npm package layout, and callback runtime boundary | Pure-helper layering, runtime support matrix, JavaScript callback HTTP transport, facade package exports, and error posture | Current | 2026-05-11 |
| [WASM Type Generation Audit](wasm-type-generation-audit.md) | Standing audit | `cow-sdk-wasm` DTO exports and TypeScript declarations | tsify policy, host-gating rule, raw and facade declaration snapshots, map-typed DTO field alignment, and package export verification | Current | 2026-05-22 |
| [WASM EIP-1271 Parity Audit](wasm-eip1271-parity-audit.md) | Standing audit | `cow-sdk-wasm` EIP-1271 payload helpers and smart-account signing callbacks | Native Rust and upstream TypeScript SDK vector parity, facade-resolves-callback contract, callback capability split, and UID/digest string reuse | Current | 2026-05-11 |
| [WASM Component Model Future Prep Audit](wasm-component-model-future-prep-audit.md) | Standing audit | `cow-sdk-pure-helpers` host-safe helper crate and deterministic helper boundary | Pure-helper crate extraction, JavaScript FFI exclusion, host parity tests, and future adapter readiness without claiming component packaging | Current | 2026-05-13 |
| [WASM Callback Shape Design Audit](wasm-callback-shape-design-audit.md) | Standing audit | `cow-sdk-wasm` typed JavaScript callback boundary | Named wallet, signer, cancellation, EIP-1271, and HTTP callback shapes; internal registry ownership; timeout/abort mapping; and typed failure behavior | Current | 2026-05-22 |
| [WASM Performance Budget Audit](wasm-performance-budget-audit.md) | Standing audit | `cow-sdk-wasm` release profile, flavor build outputs, and size-budget gate | Feature-scoped wasm flavors, optimization pass, measured raw/brotli/gzip budgets, and Cloudflare-specific package budget | Current | 2026-05-22 |
| [WASM Public API Stability Audit](wasm-public-api-stability-audit.md) | Standing audit | `cow-sdk-wasm` facade declarations, package exports, config shapes, and error envelopes | Facade declaration snapshots, raw export denylist, single-object constructors, transport policy config, and schema-versioned errors | Current | 2026-05-22 |
| [WASM Schema Versioning Policy Audit](wasm-schema-versioning-policy-audit.md) | Standing audit | `cow-sdk-wasm` JavaScript-visible success and error envelopes | `schemaVersion` output fields, unknown-variant sentinel behavior, facade error normalization, and versioned declaration evidence | Current | 2026-05-11 |
| [WASM Facade Architecture Audit](wasm-facade-architecture-audit.md) | Standing audit | TypeScript facade modules under `crates/wasm/npm/src/**` | Public facade ownership, raw binding boundary, resource cleanup, runtime flavor declarations, and package resolution tests | Current | 2026-05-22 |
| [cow-sdk-wasm Comparative Benchmark Validation Note](cow-sdk-wasm-comparative-benchmark-validation-note.md) | Validation note | `cow-sdk-wasm` crate and npm package | Bundle-size, correctness-parity, latency, primitive-performance, boundary-cost, and workflow-decomposition comparison against upstream TypeScript SDK references | Current | 2026-05-22 |

## App-Data And Dependencies

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [CID Dependency Audit](cid-dependency-audit.md) | Standing audit | `cow-sdk-app-data` CID encoding and published dependency boundary | Supported CID paths, fail-closed encoding, and clean published-upstream posture | Current | 2026-05-22 |
| [Dependency Gate Audit](dependency-gate-audit.md) | Standing audit | Release-facing dependency-audit gate for current published surfaces | Blocking transport advisory policy, clean CID posture, direct WASM randomness alignment, canonical advisory tolerance, source whitelist, native Alloy provider/signer dependency allow-lists, `cow-sdk-wasm` wasm32 dependency exclusions, and pure-helper FFI exclusion | Current | 2026-05-22 |

## Source Provenance

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Source-Lock Provenance Audit](source-lock-provenance-audit.md) | Standing audit | source-lock provenance and release preflight authority | Source-lock commit pins, upstream freshness disclosure, Alloy runtime/core dependency provenance, publication preflight metadata, historical snapshot scope, and refresh ownership | Current | 2026-05-12 |

## Orderbook Wire DTO Coverage

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Wire DTO Coverage Audit](wire-dto-coverage-audit.md) | Standing audit | `cow-sdk-orderbook` DTO coverage | OpenAPI-vendored orderbook response DTO inventories, request-payload fixtures, field-level round trips, and response forward compatibility | Current | 2026-05-22 |

## Transport And Routing

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Partner API Routing Audit](partner-api-routing-audit.md) | Standing audit | `cow-sdk-core` route selection and `cow-sdk-orderbook` partner header assembly | Local partner-route activation and `X-API-Key` request construction | Current | 2026-05-12 |

## Trading Runtime Authority

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Trading Orderbook Context Audit](trading-orderbook-context-audit.md) | Standing audit | `cow-sdk-trading` runtime authority for orderbook-bound helpers | Canonical chain and environment authority when an orderbook client is injected | Current | 2026-05-12 |
| [Trading Quote Orderbook Binding Audit](trading-quote-orderbook-binding-audit.md) | Standing audit | `cow-sdk-trading` quote-origin binding for quote-derived posting | Runtime binding between quote creation and post-from-quote submission | Current | 2026-05-12 |
| [Trading SDK Runtime Prerequisites Audit](trading-sdk-runtime-prerequisites-audit.md) | Standing audit | `cow-sdk-trading` ready-state `TradingSdk` and helper-only `HelperOnlySdk` construction | Ready quote/post setup, validated `AppCode`, helper-only setup, and helper prerequisites | Current | 2026-05-14 |

## Trading Order Integrity

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Trading Order Construction Integrity Audit](trading-order-construction-integrity-audit.md) | Standing audit | `cow-sdk-trading` order assembly and recoverable-signature posting contract | Balance semantics, same-token builder policy, builder terminal parity, and local signature validation | Current | 2026-05-22 |
| [Trading Order-Bounds Validator Audit](trading-order-bounds-validator-audit.md) | Standing audit | `cow-sdk-trading` `OrderBoundsValidator`, order validity bounds, and client rejection surface | Mandatory client-side submission validation, same-token parity, custom bounds plumbing, EthFlow skip rule, and builder-level subset | Current | 2026-05-12 |
| [Trading App-Data Merge Audit](trading-app-data-merge-audit.md) | Standing audit | `cow-sdk-trading` quote-to-post app-data edit path | Typed app-data merge, hooks replacement semantics, typed signer derivation, and `merge_and_seal_app_data` / `params_from_doc` public helpers | Current | 2026-05-13 |
| [Trading EthFlow Owner Identity Audit](trading-ethflow-owner-identity-audit.md) | Standing audit | `cow-sdk-trading` EthFlow submission seam | `EthFlowTransaction.from` owner threading, preview identity selection, and EthFlow-aware validator invocation | Current | 2026-05-22 |

## Workspace-Wide Safety And Workflow Security

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Unsafe-Code Policy Audit](unsafe-code-policy-audit.md) | Standing audit | Workspace `unsafe_code = deny` lint declared in `Cargo.toml` workspace lint section | Workspace unsafe-code deny lint, crate lint adoption, public-path source posture, and clippy enforcement | Current | 2026-04-23 |
| [Panic-Free Public Surface Audit](panic-free-public-surface-audit.md) | Standing audit | Every `crates/*/src/**/*.rs` file accessible from the published public API | Public-runtime `expect`, `unwrap`, and `panic!` site set with rationale for every remaining static-invariant panic site, including native Alloy adapter conversion, trading receipt-wait helpers, wasm exports, pure helpers, and allowlist coverage | Current | 2026-05-22 |
| [Workflow Security Audit](workflow-security-audit.md) | Standing audit | Every `.github/workflows/*.yml` file | SHA-pinned third-party actions, explicit permissions, reviewed action source refs, and guarded `pull_request_target` use | Current | 2026-05-08 |
| [Fuzz Coverage Audit](fuzz-coverage-audit.md) | Standing audit | The standalone `cow-sdk-fuzz` crate and every `cargo-fuzz` target it ships against the published SDK crates | Target inventory across encoder, signing, validator, parser, classifier, crypto envelope, app-data, transport, browser-wallet adjacent, and trading surfaces; per-target seed contract; stable-toolchain compile gate; nightly enumerate; property traceability; public-surface boundary on `pub(crate)` helpers | Current | 2026-05-22 |

## Cross-Cutting Reviewability And Contract Hygiene

| Artifact | Type | Owning surface | Scope | Status | Last reviewed |
| --- | --- | --- | --- | --- | --- |
| [Credential Surface Contract Hygiene Audit](credential-surface-contract-hygiene-audit.md) | Standing audit | Cross-cutting credential redaction and typed partner-fee public boundary | Secret-safe route identity, config diagnostics, URL-bearing config redaction, native Alloy key/URL redaction, host-policy failures, `Redacted<T>` wrapper, transport error redaction, and typed user policy | Current | 2026-05-14 |
| [Credential Surface Audit](credential-surface-audit.md) | Standing audit | Credential-bearing builder storage, URL configuration, host-policy errors, public error diagnostics, wallet add-chain payloads, Pinata upload-trait headers, wasm error envelopes, and the SDK facade | Redacted credential storage plus sanitized host-policy failures, typed-redacted public error diagnostics, typed-redacted Pinata headers, and redacted JS-visible `WasmError` diagnostics | Current | 2026-05-14 |
| [URL Credential Redaction Audit](url-credential-redaction-audit.md) | Standing audit | URL-bearing public configuration across core, orderbook, subgraph, browser-wallet, app-data, native Alloy adapters, and wasm error conversion | Redacting URL map and URL value wrappers for public diagnostics with explicit raw dispatch access at HTTP, wallet, Alloy RPC, and wasm error seams | Current | 2026-05-14 |
| [Shared Logic Reviewability Audit](shared-logic-reviewability-audit.md) | Standing audit | Orderbook, signing, and trading shared-logic reviewability boundary | Shared request execution, signing payload preparation, thin posting wrappers, and justified DTO separation | Current | 2026-05-22 |
| [Cooperative Cancellation Contract Audit](cooperative-cancellation-contract-audit.md) | Standing audit | Cross-cutting cooperative cancellation across core, orderbook, subgraph, trading, native Alloy adapters, and wasm callback transport | Shared `CancellationToken` re-export, the `Cancellable` extension-trait combinator, typed `Cancelled` variants, biased-poll drop semantics, trading receipt-wait helpers, native Alloy adapter calls, and wasm callback timeout abort handling | Current | 2026-05-12 |
| [Signer Error Classification Audit](signer-error-classification-audit.md) | Standing audit | Cross-crate `cow_sdk_core::SignerError` trait and the typed rejection routing in `cow-sdk-signing`, `cow-sdk-browser-wallet`, `cow-sdk-alloy-signer`, and `cow-sdk-alloy` | Trait surface and per-variant implementations, the `signer_error` routing helper, the `SigningError::SignerRejection` variant, the `S::Error: fmt::Display + SignerError` bound on every public signing helper, and the workspace-level end-to-end propagation invariant | Current | 2026-05-19 |
