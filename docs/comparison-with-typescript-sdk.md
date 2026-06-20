# cow-rs and the TypeScript SDK

The canonical SDK for CoW Protocol is
[`@cowprotocol/cow-sdk`](https://github.com/cowprotocol/cow-sdk), a TypeScript
monorepo maintained by the protocol team. `cow-rs` is a Rust SDK for the same
protocol: it ports the canonical types from that project and from
[`cowprotocol/services`](https://github.com/cowprotocol/services), and locks its
protocol-critical paths byte-for-byte against them (see
[Parity and Provenance](parity.md)).

This page explains how the two relate — where `cow-rs` deliberately does less,
where Rust's type system lets it guarantee more, and where the TypeScript SDK
remains the better choice. It is not a scorecard. The two share a wire format and
a parity contract; they differ in the guarantees they place around it.

## Scope: the TypeScript SDK is broader

The TypeScript SDK ships programmable-order capability families that `cow-rs`
does not yet implement:

| Capability | TypeScript SDK | cow-rs |
| --- | --- | --- |
| Composable / conditional orders (TWAP and others) | `@cowprotocol/sdk-composable` | Deferred — not yet shipped; only the planned framework is recorded by [ADR 0048](adr/0048-composable-conditional-order-framework.md) |
| Cross-chain bridging | `@cowprotocol/sdk-bridging` | Not yet implemented |
| Flash-loan collateral swaps | `@cowprotocol/sdk-flash-loans` | Not yet implemented |
| Weiroll multi-step scripting | `@cowprotocol/sdk-weiroll` | Not yet implemented |
| JavaScript wallet adapters (viem, ethers v5/v6, wagmi) | dedicated adapter packages | Not applicable — Rust consumers implement the `Signer` / `Provider` traits or use the native Alloy adapters |

If you need those families today, use the TypeScript SDK.

## What cow-rs covers

Within the order lifecycle, `cow-rs` is complete: quote; sign under all four
schemes (EIP-712, EthSign, EIP-1271, pre-sign); post; look up (order, trades,
native price, solver competition, total surplus, competition status); cancel
both off-chain and on-chain; allowance and approval; native-asset wrap and unwrap; app-data generation and
upload; EthFlow native-currency sells; and COW Shed account-abstraction hooks
([ADR 0049](adr/0049-cow-shed-account-abstraction-proxy.md), behind the
`cow-shed` feature). Within that scope it adds the guarantees below.

## What Rust's type system adds

Each row is governed by an ADR and backed by executable evidence in
[`PROPERTIES.md`](../PROPERTIES.md). The point is not that Rust is faster; it is
that several classes of integration mistake become a compile error or are
unrepresentable, rather than a runtime bug.

| Guarantee | How Rust enforces it | Governed by |
| --- | --- | --- |
| **Typed values, not strings** | `Amount`, `Address`, and `OrderUid` are distinct newtypes; passing a hash where an address belongs does not compile | [ADR 0052](adr/0052-alloy-primitives-canonical-primitive-layer.md) |
| **Misconfiguration is unrepresentable** | Typestate builders make `build()` and `execute()` reachable only once the required inputs are set; a missing required input fails to compile, and named setters keep the sell and buy legs from being transposed the way positional arguments could be | [ADR 0011](adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md), [ADR 0013](adr/0013-http-transport-injection-and-typestate-builders.md) |
| **Capability isolation by crate boundary** | The local-key signer lives in a separate crate (`cow-sdk-alloy-signer`) held to its boundary by a CI allow-list; a consumer that does not opt in has no keystore in its dependency graph. A single crate cannot express that boundary | [ADR 0035](adr/0035-alloy-provider-adapter.md) |
| **Runtime-free protocol core** | Hashing, signing, and contract decoding need no async runtime; on `wasm32` the `reqwest` stack is removed from the build by target `cfg`, not switched off at runtime | [ADR 0010](adr/0010-runtime-neutral-async-and-transport-posture.md), [ADR 0041](adr/0041-transport-policy-l3-layering.md) |
| **Credential redaction by construction** | Secrets are stored in a `Redacted<T>` newtype with a private field; `Debug`, `Display`, and `Serialize` emit `[redacted]`, and the private field blocks any accessor-based bypass | [ADR 0025](adr/0025-workspace-url-redaction-convention.md) |
| **Audited panic surface** | A CI gate fails the build on any panic-capable call site that is not on an audited allow-list | [ADR 0033](adr/0033-minimum-viable-panic-surface.md) |
| **Typed failure taxonomy** | Every orderbook `errorType` decodes to a typed `OrderbookRejection` variant; `is_retryable()` and `backoff_hint()` turn retry logic into a `match` arm instead of string comparison | [ADR 0017](adr/0017-typed-orderbook-rejection-parser.md), [ADR 0060](adr/0060-uniform-error-classification.md) |
| **Forward-compatible surfaces** | `#[non_exhaustive]` enums and response DTOs make a new protocol variant or field a compile-time prompt for downstream `match` arms, enforced across crate versions | [ADR 0031](adr/0031-wire-dto-openapi-driven-with-order-auction-order-split.md) |

Two further safeguards are good engineering that `cow-rs` enforces rather than
Rust-exclusive language features: the quote-to-order binding fails closed when a
quote response drifts from its request
([ADR 0058](adr/0058-typed-quote-request-response-surface.md)), and every
protocol transform is compared against pinned upstream fixtures in CI
([ADR 0032](adr/0032-deployment-authority-machine-readable-provenance.md)). A
TypeScript SDK could implement either; `cow-rs` holds them as a typed `Result`
and a compile-time fixture comparison.

## Where the TypeScript SDK is the better choice

- **Breadth.** It ships the programmable-order families above; `cow-rs` does not yet.
- **Browser bundle size.** At equivalent feature subsets it is substantially
  smaller than a wasm build of `cow-rs`; for a standard browser dapp it is the
  right default.
- **Ecosystem fit.** It integrates directly with viem, ethers, and wagmi, and
  binds to React state through a global adapter.

The [when-to-use table](../README.md#when-to-use-cow-rs) in the workspace README
maps each runtime to the right choice.

## Same wire format, different guarantees

`cow-rs` and the TypeScript SDK produce byte-identical protocol output: the same
order UID, EIP-712 digest, signature encoding, and app-data hash. That equality
is asserted in CI against the TypeScript SDK and the contracts (see
[Parity and Provenance](parity.md)). The difference is not what goes on the wire;
it is the set of guarantees the type system places around it before the bytes are
produced.
