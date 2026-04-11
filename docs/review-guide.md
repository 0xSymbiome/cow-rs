# Review Guide

Last reviewed: 2026-04-11

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
| `cow-sdk-orderbook` | `HttpClientPolicy` | `OrderBookTransportPolicy` retry and rate-limit behavior; `ApiContext` chain/env/base URLs; explicit env override builders; optional `X-API-Key` header handling. |
| `cow-sdk-subgraph` | `HttpClientPolicy` | `SubgraphTransportPolicy` client wiring; `SubgraphConfig` chain and base URL selection; API-key-derived production endpoints. |
| `cow-sdk-app-data` | `IpfsFetchPolicy` read base URI | `IpfsConfig` write URI and pinning credentials; upload semantics are separate from fetch. |

Default client policy is explicit and test-covered:

- native and wasm clients use a 10-second default timeout unless the caller disables it
- each transport crate sets its own crate-specific default user-agent
- base URL overrides are separate from shared client settings

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

Generated or schema-derived artifacts are not part of the public SDK API. Schema mirrors, if present, belong in internal or test-only locations rather than the supported public surface.

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
