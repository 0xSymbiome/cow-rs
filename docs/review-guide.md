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
- `cow_sdk_signing::order_typed_data_payload` and `order_cancellations_typed_data_payload` are the reviewed CoW payload builders. They are the preferred inputs for browser-wallet EIP-712 signing.
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

- `BrowserWallet::discover()` and `BrowserWallet::discover_with()` are the reviewed injected-wallet discovery entrypoints. They use a bounded async wait contract and return explicit discovery metadata plus discovered wallet candidates.
- `InjectedWalletDiscovery::single_wallet()` is valid only when discovery produced exactly one reviewed candidate. It fails with a typed error when explicit selection is required.
- `InjectedWalletDiscovery::wallet_at()` is the explicit selection path when more than one reviewed candidate is present.
- `BrowserWallet::detect()` is a compatibility helper for direct `window.ethereum` lookup. It is not the primary reviewed discovery contract.

Review these points on browser-wallet changes:

- modern injected discovery must be able to represent more than one candidate
- the wait contract must stay bounded and visible
- legacy direct-provider fallback must remain explicit
- example and product docs must not imply silent provider auto-selection when discovery is ambiguous

Browser-wallet session synchronization follows the same explicit contract:

- `WalletSession` is kept in sync from provider-emitted `accountsChanged`, `chainChanged`, `connect`, and `disconnect` signals for reviewed wallet transports.
- The public surface remains typed Rust state and events through `WalletSession` and `WalletEvent`; raw JS payloads stay local to `cow-sdk-browser-wallet`.
- Listener ownership follows cloned `BrowserWallet` and `Eip1193Provider` values. Cleanup happens when the last owning Rust value is dropped, without process-global event buses or singleton state.
- `refresh_session()` remains an explicit resynchronization helper, not the primary reviewed path for externally initiated account or chain changes.

Browser-wallet chain management is reviewed through the typed crate-local contract:

- `WalletChainParameters` and `WalletNativeCurrency` are the add-chain request surface. Browser-wallet callers provide explicit chain metadata and RPC URLs instead of assembling raw `wallet_addEthereumChain` payloads at call sites.
- `BrowserWallet::switch_chain()` remains the typed switch helper for reviewed supported chains.
- `BrowserWallet::add_chain()` and `BrowserWallet::switch_or_add_chain()` keep add-chain and switch-or-add behavior visible through `WalletChainChange` and `WalletChainChangeKind`.
- Invalid chain configuration fails locally with `BrowserWalletError::InvalidChainConfiguration`. Wallet-side rejection, unsupported methods, and chain-not-added outcomes remain distinct typed errors.
- Wallet-specific method growth stays leaf-owned and typed. The public contract does not include a generic raw wallet-RPC passthrough.
- Promotion beyond `cow-sdk-browser-wallet` requires another stable non-browser consumer and a materially larger typed wallet-method surface.

## Public Package Policy

Packaging posture is explicit in the manifests:

- public MSRV is Rust `1.94` through `workspace.package.rust-version`
- contributor execution is pinned to Rust `1.94.1` in `rust-toolchain.toml`
- key public crates opt into workspace lint policy through `[lints] workspace = true`
- docs.rs behavior is declared explicitly for the facade and the primary transport/core crates

For the facade specifically:

- `cow-sdk` docs.rs builds with the `browser-wallet` feature enabled so the optional browser-wallet surface is visible in rendered docs
- `cow-sdk-subgraph` is a separate package and is not folded into the facade for documentation convenience
- the facade is a re-export layer and does not gain implementation ownership through packaging polish

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

The workflow set is intentionally small: workspace validation, release-readiness
checks, WASM checks, and WASM example Pages deployment. Action references in
workflow files are pinned to immutable SHAs.

CID handling uses upstream crates for CID and multihash encoding. Legacy content-to-CID generation uses `ipfs-cid`; latest app-data CID conversion wraps an existing Keccak digest with `cid` and `multihash` because the SDK receives the digest as an app-data hash.

## Validation

Use the normal workspace checks:

```text
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo doc --workspace --no-deps
```
