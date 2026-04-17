# ADR 0011: Typed Amount Boundary And Typestate Ready-State Construction

- Status: Accepted
- Date: 2026-04-17
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: types, trading, builders, semver
- Related: [ADR 0002](0002-dedicated-trading-orchestration-crate.md), [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)

## Decision

Public amount-carrying surfaces distinguish atomic and decimal-scaled
values through dedicated newtypes, and `TradingSdkBuilder` advertises its
prerequisites through typestate terminals. `AtomAmount` wraps an unsigned
256-bit integer in wire-native base-10 string form for ABI and transport
use; `DecimalAmount` pairs an atomic value with a `decimals` scale for
display and user-input flows. The builder carries two marker type
parameters that track whether `chain_id` and `app_code` have been
supplied, and the `build_ready` terminal is only reachable from the fully-
set state. `build_helper_only` is only reachable once the chain-id marker
is set. A permissive runtime-validated `build` terminal stays available on
every state for the migration window.

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

- Public surface: `AtomAmount` and `DecimalAmount` are the forward
  amount-carrying contract on the `cow-sdk-trading` request surface;
  existing `Amount`-typed signatures stay supported through the migration
  window with `From<BigUint>` and `Into<BigUint>` conversions.
  `TradingSdkBuilder` exposes `build_ready` (requires both markers set)
  and `build_helper_only` (requires only the chain-id marker). The
  permissive `build` and `build_partial` terminals remain on every state.
- Runtime and support: the wire form of every amount remains the
  canonical base-10 string already defined by the orderbook contract.
  `AtomAmount` serializes to that exact string; decimal scaling is a
  pure presentation concern. Helper-only `TradingSdk` instances fail
  quote, post, and off-chain cancellation flows with a typed
  `TradingError::HelperOnlyMode` while pre-sign, allowance, approval, and
  on-chain cancellation helpers stay fully usable.
- Validation and review: the wire and ABI boundary remains byte-equal
  with the previous `Amount`-only surface. Typestate failure modes (a
  missing prerequisite) are observable as a compile error, not a runtime
  panic. The new terminal names never regress to a single overloaded
  `build` that silently produces a helper-only instance.
- Cost: two public amount types and four builder terminals in place of
  one each. The migration window keeps the older surfaces available,
  which requires every amount-adjacent surface to carry both accessor
  shapes until the window closes.

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

## Links

- [Architecture](../architecture.md)
- [Verification Guide](../verification-guide.md)
- [Properties](../../PROPERTIES.md)
- [ADR 0002](0002-dedicated-trading-orchestration-crate.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)

**Proven by:**

- [Trading SDK Runtime Prerequisites Audit](../audit/trading-sdk-runtime-prerequisites-audit.md)
