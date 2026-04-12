# Review Guide

This guide describes the Rust SDK boundaries and the evidence that keeps similar-looking code paths explainable.

## Review Order

Start with:

- [Security And Test Matrix](security-matrix.md)
- [Architecture](architecture.md)
- [Parity Matrix](parity-matrix.md)
- [Parity Sources](parity-sources.md)
- [Parity Scope](parity-scope.md)
- [Audits](audit/README.md)

Then inspect the crate tests that cover the surface under review. The most useful entry points are the `*_contract.rs` integration tests in each crate.

## Runtime Traits

`cow-sdk-core` defines runtime-neutral traits so higher crates can share signer and provider contracts without depending on a single async runtime, wallet implementation, or HTTP client.

| Trait | Status | Concrete use |
| --- | --- | --- |
| `Signer` | Active | Used by signing and trading flows for native/test signers. |
| `AsyncSigner` | Active | Used by async signing/trading paths and implemented directly by browser-wallet adapters. Sync signers receive a blanket implementation. |
| `Provider` | Active | Used by contracts and trading helpers for storage reads, contract calls, allowance checks, approvals, and transactions. |
| `AsyncProvider` | Active | Used by async/browser-wallet paths. Sync providers receive a blanket implementation when their signer supports the async signer contract. |
| `HttpTransport` | Deferred adapter contract | Kept as an extension seam. The orderbook client owns typed request execution directly because retry, headers, status mapping, and rate-limit behavior are part of the orderbook transport contract. |
| `GraphTransport` | Deferred adapter contract | Kept as an extension seam. The subgraph client owns typed GraphQL query execution directly. |
| `PinningTransport` | Deferred adapter contract | Kept as an extension seam. App-data pinning uses app-data-specific request and credential semantics. |

The deferred transport traits are stable extension contracts. Orderbook, subgraph, and app-data keep request execution local because each transport surface has distinct retry, header, credential, and decoding semantics.

Typed-data review has two layers on purpose:

- `cow_sdk_core::TypedDataPayload` is the signer-facing EIP-712 contract. It carries the explicit primary type, the full type map, and canonical message JSON for runtime adapters such as `cow-sdk-browser-wallet`.
- `cow_sdk_signing::OrderTypedData` is the order-facing convenience envelope returned by signing and trading helpers. That keeps typed order UX for consumers without forcing signer implementations to recover structure from field-name heuristics.
- `cow_sdk_signing::order_typed_data_payload` and `order_cancellations_typed_data_payload` are the supported CoW payload builders. They are the preferred inputs for browser-wallet EIP-712 signing.
- `cow_sdk_browser_wallet::Eip1193Signer::sign_typed_data` remains a compatibility seam for the narrow CoW order and order-cancellation field sets only. Other primary types must use `sign_typed_data_payload`.

Smart-account verification follows the same explicit-seam rule:

- `cow_sdk_contracts::verify_eip1271_signature` and `verify_eip1271_signature_async` are the only low-level EIP-1271 contract-call helpers. They require explicit provider inputs and fail with typed errors for missing code, malformed responses, provider failures, and wrong magic values.
- `cow_sdk_trading::post::verify_eip1271_order_signature` and `verify_eip1271_order_signature_async` are order-level helpers. They compute the CoW order digest explicitly and then call the contracts helper.
- Submission paths keep EIP-1271 verification opt-in so custom-signature generation and pure payload construction do not inherit hidden provider requirements.

## Transport Policy Review

Review transport configuration in two passes:

1. Shared client settings:
   `cow_sdk_core::HttpClientPolicy` owns timeout and user-agent only.
2. Crate-local request behavior:
   orderbook, subgraph, and app-data each keep their own transport policy surface where semantics differ.

Use this split when evaluating changes:

| Surface | Shared input | Crate-local behavior that must be explicit |
| --- | --- | --- |
| `cow-sdk-orderbook` | `HttpClientPolicy` | `OrderBookTransportPolicy` retry and rate-limit behavior; `ApiContext` chain/env/base URLs; explicit env override builders; optional `X-API-Key` header handling; instance-scoped async-safe limiter sharing across clones of the same client. |
| `cow-sdk-subgraph` | `HttpClientPolicy` | `SubgraphTransportPolicy` client wiring; `SubgraphConfig` chain and base URL selection; API-key-derived production endpoints; `SubgraphQueryRequest` for explicit document, variables, and operation-name input on generic queries; `SubgraphError` for typed transport, HTTP, GraphQL, serialization, missing-data, unsupported-network, and empty-totals failures. |
| `cow-sdk-app-data` | `IpfsFetchPolicy` read base URI | `IpfsConfig` write URI and pinning credentials; upload semantics are separate from fetch. |

Default client policy is explicit and test-covered:

- native and wasm clients use a 10-second default timeout unless the caller disables it
- each transport crate sets its own crate-specific default user-agent
- base URL overrides are separate from shared client settings
- orderbook base URL precedence is: explicit `with_env_base_url()` override, then `ApiContext` explicit base URLs or partner resolution, then the default env-and-chain map
- `with_context_override()` updates chain, env, explicit base URLs, and API key on the cloned orderbook client without mutating unrelated instances
- `with_transport_policy()` rebuilds the reqwest client and creates a fresh instance-scoped limiter for that clone lineage
- orderbook rate-limit waits happen before each attempt, retry backoff happens after retryable failures, and cancelling a waiting request does not poison the shared limiter state

## Trading SDK Precedence Review

Review `TradingSdk` with this order in mind:

1. injected orderbook client context for orderbook-bound chain and env
2. advanced quote or post settings for overlapping trade fields
3. call-level parameters
4. SDK trader defaults
5. signer address as the final owner fallback

Key review points:

- `TradingSdk::builder()` and `TradingSdkOptions` keep policy instance-scoped rather than mutation-driven.
- Injected orderbook clients do not act as a silent suggestion. They define the active orderbook context, and conflicts surface as typed errors.
- Advanced quote and post settings must align with the effective trade parameters used for request construction, app-data generation, signing payloads, and submission payloads.
- Orderbook-bound flows and non-orderbook flows must apply the same call-level-over-default precedence for env and protocol address overrides.
- Quote-only flows resolve owner from the effective trade parameters first and otherwise use the explicit quoter account. Signer-backed quote and post flows use the signer address only as the final owner fallback.
- Limit-order posting uses `0` basis points when slippage is omitted, so app-data, signing payloads, and submission payloads stay aligned.
- Slippage and fee helpers use integer math on the wire values. Public docs should keep rounding, truncation, and clamping behavior explicit when those values are surfaced to callers.

## Native Trading Example Coverage

The native example gallery exposes the full mandatory trading workflow surface
through focused deterministic scenarios.

- `examples/native/scenarios/quote_only_simulation.rs` covers quote construction without order submission.
- `examples/native/scenarios/limit_order_simulation.rs` covers signed limit-order submission.
- `examples/native/scenarios/order_lifecycle_simulation.rs` covers order lookup and off-chain cancellation.
- `examples/native/scenarios/ethflow_transaction_simulation.rs` covers native-sell / EthFlow transaction construction and simulated submission.
- `examples/native/scenarios/onchain_order_actions_simulation.rs` covers pre-sign transaction generation plus regular-order and EthFlow on-chain cancellation routing.

These scenarios are transport-mocked and runnable without browser runtimes,
wallet extensions, or live order placement.

## On-Chain Helper Encoding

The on-chain transaction builders in `cow-sdk-trading` treat tuple amount, quote-id,
and validity fields as ABI `uint256` values.

Review these points:

- EthFlow transaction generation and EthFlow on-chain cancellation encode `uint256` fields as unsigned 32-byte ABI words across the full `uint256` range.
- Negative numeric inputs are rejected before call data is produced.
- High-range coverage is exercised in the public trading integration tests so ABI behavior is verified at the transaction-builder boundary, not only in a private helper.

## DTO Boundaries

Repeated order-like field names are intentional only when they model distinct protocol contracts:

| Type | Boundary | Evidence |
| --- | --- | --- |
| `cow_sdk_core::UnsignedOrder` | User-domain order prepared for signing and trading workflows. | Converts into `cow_sdk_contracts::Order` before contract hashing. |
| `cow_sdk_core::Order` | Optional user-domain envelope with owner or uid context. | Kept separate from orderbook responses. |
| `cow_sdk_contracts::Order` | Contract ABI and EIP-712 payload with optional receiver and balance fields before normalization. | `crates/contracts/tests/order_contract.rs` covers conversion from `UnsignedOrder`. |
| `cow_sdk_contracts::NormalizedOrder` | Canonical contract hashing payload after defaults and receiver validation. | `crates/contracts/tests/order_contract.rs` covers normalization rules and hashing helpers. |
| `cow_sdk_orderbook::QuoteData` | Quote response wire DTO from the orderbook API. | `crates/orderbook/tests/types_contract.rs` covers full app-data echo handling. |
| `cow_sdk_orderbook::OrderCreation` | Order submission wire DTO for `/api/v1/orders`. | `crates/orderbook/tests/types_contract.rs` covers quote-to-submission conversion, signature, signer, and quote-id additions. |
| `cow_sdk_orderbook::Order` | Orderbook order response DTO with status, owner, uid, execution totals, and EthFlow metadata. | Kept separate from signing and contract hashing types because it models persisted API state. |

A field-similar type without a distinct wire, ABI, normalized, or user-domain boundary should be removed or merged.

## Typed Boundary Review

Use this rule when evaluating the public API:

- User-domain Rust surfaces should accept and return `Address`, `Amount`, `SignedAmount`, `HexData`, `AppDataHash`, `OrderUid`, and `Hash32` aliases when those values carry protocol meaning.
- Raw `String` values are acceptable only for explicit orderbook wire DTOs, serialized compatibility models, or named legacy paths with a documented reason.
- Conversions from typed values into wire strings should happen as close as possible to the transport or ABI encoder boundary.

The public typed boundary is applied across these paths:

- `cow-sdk-core` runtime traits use typed transaction values, gas limits, call data, and hashes.
- `cow-sdk-contracts` and `cow-sdk-signing` terminate on typed order amounts and digests.
- `cow-sdk-trading` exposes typed trade amounts, allowance values, approval hashes, on-chain cancellation hashes, and EthFlow existence checks.
- `cow-sdk-orderbook` intentionally uses string-heavy HTTP request and response DTOs because that matches the upstream API contract.

## Package Boundaries

The `cow-sdk-*` package family is intentionally multi-crate:

- `cow-sdk-core` owns shared types, validation, config, and runtime contracts.
- `cow-sdk-contracts`, `cow-sdk-signing`, and `cow-sdk-app-data` own deterministic protocol transforms.
- `cow-sdk-orderbook` and `cow-sdk-subgraph` own API-specific transport surfaces.
- `cow-sdk-trading` owns quote-to-order orchestration and user-facing workflows.
- `cow-sdk-browser-wallet` owns WASM/browser wallet integration behind an additive feature.
- `cow-sdk` is a thin facade.

This avoids a single crate becoming the owner of unrelated concerns while giving consumers an ergonomic root package.

## Browser Wallet Discovery Review

Use the browser-wallet discovery surface in this order:

- `BrowserWallet::discover()` and `BrowserWallet::discover_with()` are the injected-wallet discovery entrypoints. They use a bounded async wait contract and return explicit discovery metadata plus discovered wallet candidates.
- `InjectedWalletDiscovery::single_wallet()` is valid only when discovery produced exactly one candidate. It fails with a typed error when explicit selection is required.
- `InjectedWalletDiscovery::wallet_at()` is the explicit selection path when more than one candidate is present.
- `BrowserWallet::detect()` is a compatibility helper for direct `window.ethereum` lookup. It is not the primary discovery contract.

Review these points on browser-wallet changes:

- modern injected discovery must be able to represent more than one candidate
- the wait contract must stay bounded and visible
- legacy direct-provider fallback must remain explicit
- example and product docs must not imply silent provider auto-selection when discovery is ambiguous

Browser-wallet session synchronization follows the same explicit contract:

- `WalletSession` is kept in sync from provider-emitted `accountsChanged`, `chainChanged`, `connect`, and `disconnect` signals for supported wallet transports.
- The public surface remains typed Rust state and events through `WalletSession` and `WalletEvent`; raw JS payloads stay local to `cow-sdk-browser-wallet`.
- Listener ownership follows cloned `BrowserWallet` and `Eip1193Provider` values. Cleanup happens when the last owning Rust value is dropped, without process-global event buses or singleton state.
- `refresh_session()` remains an explicit resynchronization helper, not the primary path for externally initiated account or chain changes.

Browser-wallet chain management uses the typed crate-local contract:

- `WalletChainParameters` and `WalletNativeCurrency` are the add-chain request surface. Browser-wallet callers provide explicit chain metadata and RPC URLs instead of assembling raw `wallet_addEthereumChain` payloads at call sites.
- `BrowserWallet::switch_chain()` remains the typed switch helper for supported chains.
- `BrowserWallet::add_chain()` and `BrowserWallet::switch_or_add_chain()` keep add-chain and switch-or-add behavior visible through `WalletChainChange` and `WalletChainChangeKind`.
- Invalid chain configuration fails locally with `BrowserWalletError::InvalidChainConfiguration`. Wallet-side rejection, unsupported methods, and chain-not-added outcomes remain distinct typed errors.
- Wallet-specific method growth stays leaf-owned and typed. The public contract does not include a generic raw wallet-RPC passthrough.
- Promotion beyond `cow-sdk-browser-wallet` requires another stable non-browser consumer and a materially larger typed wallet-method surface.

Browser-wallet support posture stays explicit across the public surface:

- The default `cow-sdk` facade does not assume browser-wallet access. Browser-wallet support is exposed only through the `browser-wallet` feature and the `cow-sdk-browser-wallet` crate.
- Deterministic proof comes from mock-wallet contract tests and the mock mode in the browser-wallet console.
- `MockEip1193Transport` is the deterministic proof seam. It is part of the public leaf-crate contract for tests and review surfaces, not a hidden helper.
- In the browser-wallet console, `Reset Session` clears console session state without dropping the selected wallet handle or confirmed provider choice, while `Forget Wallet` clears both explicitly.
- In the browser-wallet console, `Detect` caches discovered wallet candidates, `Confirm Wallet` records the provider choice when more than one candidate is present, `Connect / Reconnect` uses the confirmed provider or retained selected wallet handle, and `Rescan` refreshes the candidate set while revalidating or clearing the confirmed choice.
- Injected-provider support covers the typed EIP-1193 flows exercised by `cow-sdk-browser-wallet` on supported chains with explicit user authorization.
- Off-WASM discovery is intentionally a typed no-op. `discover()` and `discover_with()` return an empty result set, and `detect()` returns `None`, instead of implying browser-provider availability outside a browser runtime.
- Public Result-returning wallet APIs should keep failure modes explicit: user rejection, disconnected provider, wrong chain, chain-not-added, malformed response, unsupported method, invalid typed chain configuration, and environment-sensitive unavailability are distinct error classes.
- Broader extension variability remains outside the SDK contract. Extension-specific prompts, authorization persistence, chain inventory, and non-standard vendor behavior are not normalized SDK guarantees.
- Public docs and examples should keep the root facade narrower than the leaf crate and avoid language that implies universal browser-wallet compatibility.

## Public Package Policy

Packaging posture is explicit in the manifests:

- public MSRV is Rust `1.94.0` through `workspace.package.rust-version`
- contributor execution is pinned to Rust `1.94.1` in `rust-toolchain.toml`
- every published crate opts into workspace lint policy through `[lints] workspace = true`
- workspace Clippy policy explicitly covers `missing_errors_doc`, `missing_panics_doc`, `must_use_candidate`, and `unreadable_literal`
- docs.rs behavior is declared explicitly across the published crate family
- the compatibility floor is exercised directly in CI with `cargo check --workspace --all-features` and `cargo test --workspace` on Rust `1.94.0`
- a separate Windows stable lane runs `cargo check --workspace --all-features` and `cargo test --workspace --lib --tests` on `windows-latest`
- browser-target validation stays in dedicated WASM workflows instead of redefining the native compatibility floor for unrelated crates
- repo-local publication validation uses `parity/source-lock.yaml` validation plus the full published package-family dry-run from the current workspace
- provenance-sensitive parity validation uses pinned independent upstream checkouts and does not treat same-checkout copies as proof of upstream source state

For the facade specifically:

- `cow-sdk` docs.rs builds with the `browser-wallet` feature enabled so the optional browser-wallet surface is visible in rendered docs
- `cow-sdk-subgraph` is a separate package and is not folded into the facade for documentation convenience
- the facade is a re-export layer and does not gain implementation ownership through packaging polish
- feature-gated browser-wallet re-export does not widen the default-facade support contract
- `cow-sdk::prelude` is a curated convenience import. Behavioral contracts still belong to the leaf crates that define the underlying types and workflows.

## Generated Or Schema-Derived Artifacts

Generated or schema-derived artifacts are not part of the public SDK API. Schema mirrors, if present, belong in non-public or test-only locations rather than the supported public surface.

Orderbook OpenAPI and subgraph query evidence is tied to pinned entries in `parity/source-lock.yaml`; see [Parity Scope](parity-scope.md).

For subgraph specifically, saved query documents live under `crates/subgraph/src/query_documents/`, while test-only schema and generated evidence belongs under `crates/subgraph/tests/schema_evidence/`.

For subgraph custom queries specifically, review the explicit request contract before transport details:

- `SubgraphQueryRequest` carries `document`, optional `variables`, and optional `operation_name`.
- Anonymous single-operation documents are allowed without `operation_name`.
- Multi-operation documents require caller-supplied `operation_name`; the SDK does not infer it from the query string.
- `SubgraphError` keeps failure classes separate: transport, HTTP status, GraphQL payload, serialization, missing data, unsupported network, and the helper-specific empty-totals case.

Subgraph example review follows the same package boundary:

- native subgraph scenarios import `cow-sdk-subgraph` directly rather than relying on the root facade
- custom-query examples use `SubgraphQueryRequest` explicitly
- live examples require explicit environment configuration and remain opt-in

## CI Configuration

The repository ships three validation layers:

- `ci.yml` runs formatting, baseline Clippy, workspace library and integration tests, a dedicated workspace doctest lane, `nextest`, docs builds with rustdoc warnings denied, typo checks, dependency-policy checks for bans, licenses, and sources, feature-matrix validation, published-crate public API rustc lint enforcement, advisory reporting, repo-local parity/source-lock validation, and published package-family dry-runs on the pinned `1.94.1` contributor toolchain for every PR.
- `ci.yml` also runs a separate compatibility-floor job on Rust `1.94.0` with `cargo check --workspace --all-features` and `cargo test --workspace`.
- `ci.yml` also runs a light Windows stable job with `cargo check --workspace --all-features` and `cargo test --workspace --lib --tests` on `windows-latest`.
- `ci.yml` publishes a final `ci-success` aggregate status for branch protection after the required routine native jobs complete.
- `codeql.yml` runs dedicated CodeQL analysis for Rust and GitHub Actions on pull requests, pushes to `main` and `develop`, and a weekly schedule.
- `docs-quality.yml` keeps workspace doctests explicit, adds `cargo test --all-features --workspace --doc`, and runs a nightly docs.rs-style rustdoc build with `DOCS_RS=1` plus nightly rustdoc presentation flags.
- `crate-checks.yml` is a scheduled and manual maintenance lane that runs `cargo hack check --workspace --each-feature --no-dev-deps` to catch crate-isolation regressions that routine workspace-wide checks can miss.
- `release-readiness.yml` reruns the pinned library checks, the dedicated workspace doctest lane, the compatibility-floor job, and the light Windows stable job, then executes the repo-local publication contract and a separate pinned-upstream provenance lane that provisions independent checkouts from `parity/source-lock.yaml` before explicit-root validation.
- `wasm.yml` and `wasm-pages.yml` cover the WASM compatibility and example deployment surfaces.

Action references in workflow files are pinned to immutable SHAs.
Routine native validation workflows use explicit `timeout-minutes` budgets and disable credential persistence on checkout.
The crate-isolation lane is maintenance-depth evidence only; it is intentionally separate from the routine PR-blocking contract and does not mutate repository state.

Public-library rustc lints checked in CI include:

- `missing_docs`
- `missing_debug_implementations`
- `unreachable_pub`
- `unnameable_types`

The blocking public rustc lint gate applies to the published crate family: `cow-sdk-core`, `cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-app-data`, `cow-sdk-orderbook`, `cow-sdk-subgraph`, `cow-sdk-trading`, `cow-sdk-browser-wallet`, and the `cow-sdk` facade.

Workspace Clippy policy checked in CI includes:

- `missing_errors_doc`
- `missing_panics_doc`
- `must_use_candidate`
- `unreadable_literal`

Maintenance-depth Clippy review uses:

- `cargo clippy --workspace --all-targets --all-features --message-format short -- -W clippy::pedantic -W clippy::cargo -A clippy::multiple_crate_versions`
- duplicate-version review is kept authoritative in `.github/config/deny.toml` together with `cargo tree -d --workspace`, rather than in the coarse global `clippy::multiple_crate_versions` signal

Dependency policy includes:

- `cargo-deny` bans, licenses, and source policy checks
- approved duplicate-version tolerances for the `ethabi` browser-wallet path, the test-only `graphql_client` schema/codegen path, and the platform-specific verifier subtree under `rustls-platform-verifier`
- RustSec advisory enforcement through `cargo-audit`
- CodeQL semantic security analysis for Rust and GitHub Actions in a dedicated workflow
- a temporary `RUSTSEC-2026-0097` exception while `cow-sdk-browser-wallet` still depends on `ethabi`
- a separate read-only dependency freshness report built from `cargo update --dry-run` plus `cargo tree -d --workspace`
- weekly freshness automation lives on the scheduled `release-readiness.yml` path, while `ci.yml` exposes the same report through manual dispatch only
- the freshness report is assembled directly in workflow steps instead of adding a repo-side maintenance script language for one narrow CI concern

CID handling uses upstream crates for CID and multihash encoding. Legacy content-to-CID generation uses `ipfs-cid`; latest app-data CID conversion wraps an existing Keccak digest with `cid` and `multihash` because the SDK receives the digest as an app-data hash.

## Validation

Use the normal workspace checks:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --workspace --all-targets --all-features --message-format short -- -W clippy::pedantic -W clippy::cargo -A clippy::multiple_crate_versions
cargo test --workspace
cargo test --workspace --doc
cargo test --all-features --workspace --doc
cargo +1.94.0 check --workspace --all-features
cargo +1.94.0 test --workspace
cargo nextest run --workspace --all-features --config-file .github/config/nextest.toml
cargo doc --workspace --all-features --no-deps
cargo hack check --workspace --feature-powerset --depth 1
typos --config .github/config/typos.toml
cargo deny check bans licenses sources --config .github/config/deny.toml
cargo audit --deny warnings --ignore RUSTSEC-2026-0097
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate --source-lock parity/source-lock.yaml
```

Use this command when checking the docs.rs-style nightly docs lane:

```text
DOCS_RS=1 RUSTDOCFLAGS="--cfg docsrs -D warnings -Zunstable-options --generate-link-to-definition --show-type-layout --enable-index-page" cargo +nightly doc --workspace --all-features --no-deps
```

Use these commands when checking the provenance-sensitive parity lane:

```text
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- provision-upstreams --source-lock parity/source-lock.yaml --output-root <path>
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate --source-lock parity/source-lock.yaml --cow-sdk-root <path>/cow-sdk --contracts-root <path>/contracts --services-root <path>/services
```

Use this command when reviewing public-surface documentation and export hygiene:

```text
RUSTFLAGS="-Wmissing-docs -Wmissing-debug-implementations -Wunreachable-pub -Wunnameable-types" cargo check --workspace --all-features
```

Use this command when reviewing dependency freshness without mutating the lockfile:

```text
cargo update --dry-run --color never
cargo tree -d --workspace
```
