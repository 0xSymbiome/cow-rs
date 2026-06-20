# ADR 0066: Trading Slippage and Fee Math Faithfully Implements the CoW SDK Convention

- Status: Accepted
- Date: 2026-06-04
- Last reviewed: 2026-06-04
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: trading, slippage, quote, fee, parity
- Anchors: Evidence-Backed Public Claims (primary)
- Related: [ADR 0058](0058-typed-quote-request-response-surface.md), [ADR 0021](0021-orderbook-total-fee-policy.md), [ADR 0015](0015-client-side-order-bounds-validator.md)

## Decision

cow-rs faithfully implements the established CoW Protocol SDK trade-construction
convention — the slippage transform, the fee folding (network, protocol, and
partner fees), and the slippage-suggestion heuristics. The signed-order amount
math is byte-for-byte identical to `@cowprotocol/cow-sdk` and a consistent
inverse of the `cowprotocol/services` quote-side fee accounting; every
constructed order satisfies the services market-price invariant.

cow-rs does not redefine this convention. The slippage layer is a client-side
convention shared across the CoW SDK ecosystem; cow-rs's responsibility is a
correct, deterministic implementation that interoperates with the protocol and
stays consistent with the reference SDK, not to alter a shared convention.

## Rationale

`cowprotocol/services` authoritatively defines everything the trade-construction
surface consumes:

- the `/quote` request/response DTOs and the directly-signable, fee-adjusted
  amounts — including the protocol/volume-fee adjustment in
  `crates/orderbook/src/quoter.rs`;
- the order-validity envelope a constructed order must satisfy — the
  market-price invariant in `crates/shared/src/order_validation.rs`.

What services does not define is the application of a user slippage tolerance to
the quoted amounts, or the slippage-suggestion heuristics (`50%` of fee, `0.5%`
of volume, and the bound clamping). Those are a client-side convention whose
canonical reference is the upstream TypeScript `@cowprotocol/cow-sdk`. cow-rs
implements the same convention so a caller's behaviour stays consistent with the
rest of the ecosystem; it is not cow-rs's place to diverge from it.

## Consequences

- The slippage convention's output is locked by the trading slippage
  contract tests, which derive the signable amounts the
  `cowprotocol/services` `quoter.rs` fee accounting and `order_validation.rs`
  market-price invariant must satisfy.
- `@cowprotocol/cow-sdk` is the convention's reference implementation (prior
  art). It is now a pinned parity source in `parity/source-lock.yaml` (app-data
  schemas plus the protocol-fee composition goldens); the wire, fee, and
  validity authority remains `cowprotocol/services`.
- The implementation lives in `crates/trading/src/slippage.rs`. The signed-order
  amount math (slippage transform, network/protocol/partner-fee folding, and the
  `Math.floor`/`Math.round` fixed-point truncation) is byte-for-byte identical to
  the reference SDK. The slippage-suggestion heuristics implement the same
  algorithm; their final percentage-to-basis-points conversion uses cow-rs's
  exact integer arithmetic — a step the reference SDK does not pin (its own tests
  mock that conversion) and which affects only the non-binding suggestion, never
  the signed order.
