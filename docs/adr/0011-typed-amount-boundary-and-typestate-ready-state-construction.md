# ADR 0011: Typed Amount Boundary And Typestate Ready-State Construction

- Status: Accepted (amended)
- Date: 2026-04-17
- Last reviewed: 2026-05-22
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: types, trading, builders, semver
- Related: [ADR 0002](0002-dedicated-trading-orchestration-crate.md), [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

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
marker is set. The permissive runtime-validated builder terminals have been
removed so construction flows through those two typestate-gated terminals.

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
  `TradingSdkBuilder` exposes exactly two terminals: `build_ready`
  (requires both markers set) and `build_helper_only` (requires only the
  chain-id marker).
- `TradingSdk` and `TradingSdkBuilder` expose ready-state and helper-only
  construction exclusively through typestate-builder terminal methods.
  **Inherent associated constructors** (`TradingSdk::new`,
  `TradingSdk::new_partial`, or any future equivalent) are forbidden in
  shipped crates. One-call ergonomic shortcuts (e.g.,
  `TradingSdkBuilder::ready(...)`) are typestate terminals consuming
  *total* typed inputs and never `Partial*` shapes.
- On `wasm32` targets, `build_ready()` additionally requires an injected
  orderbook client through `TradingSdkOptions::with_orderbook_client(...)`.
  The default orderbook factory does not run on `wasm32` because the
  browser runtime does not ship a default `HttpTransport` (see ADR 0013).
- Runtime and support: the wire form of every amount remains the
  canonical base-10 string already defined by the orderbook contract.
  `Amount` serializes to that exact string via a custom serializer;
  decimal scaling is a pure presentation concern. Helper-only flows use
  the distinct `HelperOnlySdk` type, which exposes pre-sign, allowance,
  approval, and on-chain cancellation helpers and does not expose quote,
  post, or off-chain cancellation methods.
- Validation and review: the wire and ABI boundary remains byte-equal
  against the pinned upstream fixtures; every per-crate parity contract
  suite continues to pass against the same vectors that validated the
  prior surface. Typestate failure modes (a missing prerequisite) are
  observable as a compile error, not a runtime panic. The new terminal
  names never regress to a single overloaded `build` that silently
  produces a helper-only instance.
- Cost: two public amount types and two builder terminals in place of
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
recorded alongside the typed-amount decision in the shipped
architecture record.

## Links

- [Parity Scope](../parity-scope.md)
- [Architecture](../architecture.md)
- [Verification Guide](../verification-guide.md)
- [Properties](../../PROPERTIES.md)
- [ADR 0002](0002-dedicated-trading-orchestration-crate.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)

**Proven by:**

- [Trading SDK Runtime Prerequisites Audit](../audit/trading-sdk-runtime-prerequisites-audit.md)
- [Typestate Builder Contract Audit](../audit/typestate-builder-contract-audit.md)

## Amendment 2026-05-22: canonical primitive layer (per ADR 0052)

`Amount` and `SignedAmount` ship as cow-owned `#[repr(transparent)]`
newtypes around `alloy_primitives::U256` and `alloy_primitives::I256`
respectively per
[ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md). The
newtypes carry cow-owned `Display`, `Serialize`, `Deserialize`, and
arithmetic operator (`Add`, `Sub`, `Mul`, `AddAssign`, etc.) impls,
plus checked, saturating, and `pow` arithmetic surfaces. The
decimal-string wire format is locked by the cow-owned
`Serialize`/`Deserialize` impls; the strict-decimal-only fail-closed
contract on the `Deserialize` boundary rejects `0x`/`0X`/`0o`/`0O`/`0b`/`0B`-prefixed
input that alloy's default `ruint::Uint::FromStr` impl would otherwise
accept silently. The Decision body's references to `Amount` as a "typed
`BigUint`" and to the `as_biguint` / `into_biguint` accessor names
predate the canonical primitive layer; the recorded decision (typed
atomic-vs-decimal amount boundary and the typestate-builder terminals)
is preserved verbatim while the inner type and accessor surface follow
ADR 0052. The owned accessor surface on `Amount` is `as_u256` /
`into_u256` (and equivalent `as_i256` / `into_i256` on `SignedAmount`),
named to match the canonical alloy primitive that backs each newtype.
