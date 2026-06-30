---
type: Decision Record
id: ADR-0073
title: "ADR 0073: Order Authorization As A Value With A Typed Placement Result And Bundled Pre-Sign Activation"
description: "Order placement selects its authorization mode through one Authorization value and returns a typed placement sum, and a smart-contract-wallet pre-sign order is activated by an SDK-built transaction bundle."
status: Accepted
date: 2026-06-30
authors: ["0xSymbiotic"]
tags: [trading, signing, eip1271, presign, account-abstraction, public-surface]
related: [ADR-0069, ADR-0070, ADR-0028, ADR-0048, ADR-0050, ADR-0051, ADR-0068]
timestamp: 2026-06-30T00:00:00Z
---

# ADR 0073: Order Authorization As A Value With A Typed Placement Result And Bundled Pre-Sign Activation

## Decision

Order placement takes its authorization mode as one `Authorization` value and returns a typed placement result, so a smart-contract-wallet order is the same call shape as an EOA order.

- `cow-sdk-trading` order placement (swap and limit) accepts `Authorization`: `Ecdsa` (a typed-data signer), `Eip1271` (a contract-signature producer), or `PreSign` (no signer). One entry per order type covers all three; there is no separate per-scheme placement function.
- Placement returns the sum `OrderPlacement::Live { order_uid }` for `Ecdsa` and `Eip1271` (the order is valid once posted) and `OrderPlacement::PendingActivation { order_uid, activation }` for `PreSign` (on-chain authorization is still owed). The scheme statically selects the arm.
- The `PreSign` `activation` is a `SafeActivation` carrying the ordered `approve` and `setPreSignature` calls as `UnsignedTransaction` values (ADR 0070) for one smart-account batch. The SDK does not return bare calldata for the caller to assemble, and the activation is transport-neutral (direct send or wallet-service proposal).
- The surface is uniform across the native crates, the wasm-bindgen npm lane (ADR 0039), and the WebAssembly Component lane (ADR 0071); the Component `authorization` is data-only because its signer is a host import.

## Why

A smart-contract wallet cannot produce an ECDSA signature, so it authorizes an order through an on-chain pre-sign transaction or an EIP-1271 contract signature. Modeling each scheme as a separate placement path fragments the consumer surface and pushes correctness — owner equals from, a zero fee amount, the empty pre-sign signature, app-data hashing, and order-UID derivation — onto the caller. Carrying the mode as a value keeps one call shape and lets the SDK own every invariant. The typed-sum result removes the sharp failure of posting a pre-sign order and never sending its on-chain activation, which leaves the order inert until it expires; the `PendingActivation` arm makes that obligation un-droppable because the order id is reachable only through it.

## Must Remain True

- Public surface: order placement accepts `Authorization` and returns the `OrderPlacement` sum; `Ecdsa` and `Eip1271` resolve to `Live`, `PreSign` to `PendingActivation`. The placement entries stay in the bound-method layer of ADR 0069, and the activation bundle composes the single-step builders of ADR 0070; no transport crate gains a signing dependency.
- Runtime and support: a pre-sign order posts an empty signature with `from == owner` and a zero fee amount; the activation is the ordered approve-then-set-pre-signature pair and is transport-neutral; an `Eip1271` order stays `Live`, with any on-chain pre-validation expressed through app-data pre-hooks rather than the activation.
- Validation and review: order-UID derivation, the signing scheme, and the empty pre-sign signature carry golden-vector parity against the orderbook contract; the placement sum is exercised on every target.
- Cost: one authorization type and one placement sum to document, plus one signer-plumbing divergence (the Component variant is data-only). The uniform call shape across the three targets is the offsetting benefit.

## Alternatives Rejected

- A scheme flag on the existing placement entries without a result change: reaches feature parity but leaves the on-chain pre-sign step as bare calldata the caller bundles, and the placement result cannot force the activation, so a pre-sign order can be posted and silently left inert.
- An async signer trait carrying every mode: clean in native Rust but not representable in the Component variant model and erased at the wasm-bindgen boundary, so it cannot present one surface across the three targets.
- An optional `activation` field on a single placement record: the caller can skip an absent-looking field and never activate the order; the typed sum removes that failure mode at the type level.

## Links

- [ADR 0069](0069-layered-trading-operation-surface-and-signing-free-transport.md)
- [ADR 0070](0070-onchain-transaction-helper-boundary.md)
- [ADR 0028](0028-account-abstraction-integration-plan.md)
- [ADR 0048](0048-composable-conditional-order-framework.md)
- [ADR 0050](0050-eip1271-signature-blob-encoding.md)
- [ADR 0051](0051-signing-owned-eip1271-signature-provider-trait.md)
- [ADR 0068](0068-payload-only-typed-data-signing.md)
