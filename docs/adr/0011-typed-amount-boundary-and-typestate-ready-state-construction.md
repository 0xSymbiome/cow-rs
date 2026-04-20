# ADR 0011: Typed Amount Boundary And Typestate Ready-State Construction

- Status: Accepted
- Date: 2026-04-17
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: types, trading, builders, semver
- Related: [ADR 0002](0002-dedicated-trading-orchestration-crate.md), [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)

## Decision

Public amount-carrying surfaces distinguish atomic and decimal-scaled
values through dedicated newtypes, and `TradingSdkBuilder` advertises its
prerequisites through typestate terminals. `Amount` wraps an unsigned
256-bit integer as a typed `BigUint` with wire-native base-10 string
serialization for ABI and transport use; `DecimalAmount` pairs an atomic
value with a `decimals` scale for display and user-input flows. The
`Amount(BigUint)` newtype is the single canonical atomic type across
`cow-sdk-core`, `cow-sdk-orderbook`, `cow-sdk-trading`, `cow-sdk-signing`,
`cow-sdk-app-data`, and `cow-sdk-contracts`; the retired wire-string
wrapper no longer exists. The builder carries two marker type
parameters that track whether `chain_id` and `app_code` have been
supplied, and the `build_ready` terminal is only reachable from the
fully-set state. `build_helper_only` is only reachable once the chain-id
marker is set. Permissive runtime-validated `build` and `build_partial`
terminals stay available on every state for runtime-driven construction.

## Why

A protocol SDK that accepts raw `BigUint` everywhere makes the most common
class of bot bug — confusing a human-readable `1.5` with its atomic
`1_500_000_000_000_000_000` — a runtime failure at first submission
instead of a compile-time refusal. A `TradingSdk` that builds successfully
without `chain_id` or `app_code` and then fails on the first quote
pushes the same discovery to hours after startup. Moving both discoveries
to the type system removes entire classes of latent defect without
widening the runtime surface.

## Must Remain True

- Public surface: `Amount` (typed `BigUint`) and `DecimalAmount` are the
  amount-carrying contract on the `cow-sdk-trading` request surface and
  every other public crate; `From<BigUint>`, `Into<BigUint>`, and
  `TryFrom<&str>` conversions keep atomic interop ergonomic, and
  `Amount::as_biguint` / `Amount::into_biguint` expose the inner value
  for typed arithmetic without reparsing a decimal string.
  `TradingSdkBuilder` exposes `build_ready` (requires both markers set)
  and `build_helper_only` (requires only the chain-id marker). The
  permissive `build` and `build_partial` terminals remain on every state.
- Runtime and support: the wire form of every amount remains the
  canonical base-10 string already defined by the orderbook contract.
  `Amount` serializes to that exact string via a custom serializer;
  decimal scaling is a pure presentation concern. Helper-only
  `TradingSdk` instances fail quote, post, and off-chain cancellation
  flows with a typed `TradingError::HelperOnlyMode` while pre-sign,
  allowance, approval, and on-chain cancellation helpers stay fully
  usable.
- Validation and review: the wire and ABI boundary remains byte-equal
  against the pinned upstream fixtures; every per-crate parity contract
  suite continues to pass against the same vectors that validated the
  prior surface. Typestate failure modes (a missing prerequisite) are
  observable as a compile error, not a runtime panic. The new terminal
  names never regress to a single overloaded `build` that silently
  produces a helper-only instance.
- Cost: two public amount types and four builder terminals in place of
  one each. The single canonical `Amount(BigUint)` newtype replaces the
  retired wire-string wrapper, so every amount-adjacent surface carries
  one accessor shape instead of two.

## Alternatives Rejected

- Keep `Amount = BigUint` as the only public surface: simpler, but
  preserves the silent human-vs-atomic failure mode the SDK is most
  frequently blamed for.
- Make the builder runtime-only and return `Err` on missing
  prerequisites: matches many builder-pattern crates, but defers the
  discovery to the first quote or post call when the consumer is already
  in production.
- Make `TradingSdk` generic over a mode type parameter: compile-time
  safe, but forces every downstream signature to leak the mode and
  collapses the ergonomic path for consumers who do not care about the
  helper-only lane.

## Intentionally Out-of-Scope

The typed-amount decision records what the canonical atomic type is; it
does not attempt to mirror every historical upstream surface. The
authoritative list of TypeScript-SDK surfaces that `cow-rs` intentionally
declines to mirror — including the retired wire-string `Amount` wrapper
and the related parity-scope exclusions from the same release cycle —
lives in [Parity Scope](../parity-scope.md). Reviewers and contributors
should consult that document before filing any issue claiming a missing
positive fixture implies a parity gap; the parity-scope discipline is
governed by ADR-Reset-030 in the architecture decision record.

## Links

- [Parity Scope](../parity-scope.md)
- [Architecture](../architecture.md)
- [Verification Guide](../verification-guide.md)
- [Properties](../../PROPERTIES.md)
- [ADR 0002](0002-dedicated-trading-orchestration-crate.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)

**Proven by:**

- [Trading SDK Runtime Prerequisites Audit](../audit/trading-sdk-runtime-prerequisites-audit.md)
