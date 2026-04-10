# Review Guide

Last reviewed: 2026-04-10

This guide describes how to review the current Rust SDK boundaries and the evidence that keeps similar-looking code paths explainable.

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
| `HttpTransport` | Deferred adapter contract | Kept as an extension seam. The orderbook client currently owns typed request execution directly because retry, headers, status mapping, and rate-limit behavior are part of the orderbook transport contract. |
| `GraphTransport` | Deferred adapter contract | Kept as an extension seam. The subgraph client currently owns typed GraphQL query execution directly. |
| `PinningTransport` | Deferred adapter contract | Kept as an extension seam. App-data pinning currently uses app-data-specific request and credential semantics. |

The deferred transport traits should not be read as a claim that every transport crate routes through `cow-sdk-core` today. They are stable adapter contracts for consumers and future adapter work.

## Transport Policy Review

Review transport configuration in two passes:

1. Shared client settings:
   `cow_sdk_core::HttpClientPolicy` owns timeout and user-agent only.
2. Crate-local request behavior:
   orderbook, subgraph, and app-data each keep their own transport policy surface where semantics differ.

Use this split when evaluating changes:

| Surface | Shared input | Crate-local behavior that must stay explicit |
| --- | --- | --- |
| `cow-sdk-orderbook` | `HttpClientPolicy` | `OrderBookTransportPolicy` retry and rate-limit behavior; `ApiContext` chain/env/base URLs; explicit env override builders; optional `X-API-Key` header handling. |
| `cow-sdk-subgraph` | `HttpClientPolicy` | `SubgraphTransportPolicy` client wiring; `SubgraphConfig` chain and base URL selection; API-key-derived production endpoints. |
| `cow-sdk-app-data` | `IpfsFetchPolicy` read base URI | `IpfsConfig` write URI and pinning credentials; upload semantics remain separate from fetch. |

Default client policy is explicit and test-covered:

- native and wasm clients use a 10-second default timeout unless the caller disables it
- each transport crate sets its own crate-specific default user-agent
- base URL overrides remain separate from shared client settings

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

If a future field-similar type has no distinct wire, ABI, normalized, or user-domain boundary, it should be removed or merged.

## Typed Boundary Review

When reviewing public API changes, use this rule:

- User-domain Rust surfaces should accept and return `Address`, `Amount`, `SignedAmount`, `HexData`, `AppDataHash`, `OrderUid`, and `Hash32` aliases when those values carry protocol meaning.
- Raw `String` values are acceptable only for explicit orderbook wire DTOs, serialized compatibility models, or named legacy paths with a documented reason.
- Conversions from typed values into wire strings should happen as close as possible to the transport or ABI encoder boundary.

The public typed boundary is applied across these paths:

- `cow-sdk-core` runtime traits now use typed transaction values, gas limits, call data, and hashes.
- `cow-sdk-contracts` and `cow-sdk-signing` now terminate on typed order amounts and digests.
- `cow-sdk-trading` now exposes typed trade amounts, allowance values, approval hashes, on-chain cancellation hashes, and EthFlow existence checks.
- `cow-sdk-orderbook` intentionally remains string-heavy on HTTP request and response DTOs because that matches the upstream API contract.

## Package Boundaries

The `cow-sdk-*` package family is intentionally multi-crate:

- `cow-sdk-core` owns shared types, validation, config, and runtime contracts.
- `cow-sdk-contracts`, `cow-sdk-signing`, and `cow-sdk-app-data` own deterministic protocol transforms.
- `cow-sdk-orderbook` and `cow-sdk-subgraph` own API-specific transport surfaces.
- `cow-sdk-trading` owns quote-to-order orchestration and user-facing workflows.
- `cow-sdk-browser-wallet` owns WASM/browser wallet integration behind an additive feature.
- `cow-sdk` stays a thin facade.

This avoids a single crate becoming the owner of unrelated concerns while still giving consumers an ergonomic root package.

## Generated Or Schema-Derived Artifacts

No generated or schema-derived public API is introduced here. If schema mirrors are added later for drift evidence, they should remain non-public or test-only unless a later change explicitly promotes them into the public SDK API.

Orderbook OpenAPI and subgraph query evidence is tied to pinned entries in `parity/source-lock.yaml`; see [Parity Scope](parity-scope.md).

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
