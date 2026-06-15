# ADR 0011: Typed Amount Boundary And Typestate Ready-State Construction

- Status: Accepted
- Date: 2026-04-17
- Last reviewed: 2026-06-15
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: types, trading, builders, semver
- Related: [ADR 0002](0002-dedicated-trading-orchestration-crate.md), [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0013](0013-http-transport-injection-and-typestate-builders.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Decision

Two boundaries are typed so the most common integration mistakes surface as
compile errors instead of production failures: amount handling and `Trading`
construction.

**Amounts.** `Amount` is the single canonical atomic quantity across every
public crate — a `#[repr(transparent)]` newtype over `alloy_primitives::U256`
with a sealed inner field (per [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)).
It serializes to the orderbook's canonical base-10 decimal string, and its
`Deserialize` boundary is strict-decimal fail-closed. There is no second amount
type: human-decimal construction and display are methods on the atomic value —
`parse_units` (decimal string + token `decimals`), `from_units` (integer
whole-units + `decimals`), and `format_units` (atomic → scaled string) — each
scaling by `10^decimals` through checked integer arithmetic, with `decimals`
supplied per call and never carried on the wire. Arithmetic is fallible by
return only: `checked_add` / `checked_sub` / `checked_mul` (returning `Option`)
and explicit `saturating_*` clamps. `Amount` exposes no bare operators, no
`pow`, and no bit inspection, so a silent `uint256` wrap cannot occur at a call
site; a caller that genuinely wants raw wrapping reaches through `as_u256` /
`into_u256`, keeping the intent visible at the type boundary.

**Construction.** `Trading` constructs only through `TradingBuilder`, whose two
marker type parameters track whether `chain_id` and `app_code` have been
supplied. The terminal `build()` is implemented only on the fully-set
`TradingBuilder<ChainIdSet, AppCodeSet>`, and the `ready(params: TraderParams)
-> Trading` shortcut consumes a complete `TraderParams` and — with the default
per-chain orderbook — cannot fail. Inherent associated constructors
(`Trading::new` and equivalents) are forbidden in shipped crates, so a missing
prerequisite is a compile error rather than a first-quote runtime failure. On
`wasm32` the default orderbook factory builds a `FetchTransport`-backed client
([ADR 0013](0013-http-transport-injection-and-typestate-builders.md)); a custom
client is injected by value through `TradingBuilder::orderbook(...)` (or
`orderbook_shared(Arc<dyn OrderbookClient>)` for an already-shared handle).

**Trade parameters.** `TradeParams` is the pre-quote request shape (one amount
interpreted by `kind`); `LimitTradeParams` is the post-quote shape (both
`sell_amount` and `buy_amount` plus an optional `quote_id`).
`LimitTradeParamsFromQuote` is a real newtype that guarantees a non-`None`
`quote_id` by construction — produced only by
`swap_params_to_limit_order_params` and required by the EthFlow native-currency
submission seam — lifting the quote-id requirement from a runtime check to a
type-system guarantee. Neither `::new` takes token-decimal arguments. `owner` is
a per-call attribution on the parameter types; the SDK stores no default owner
and falls back to the signer address for signer-backed flows. One
`TradeAdvancedSettings` bundle is accepted by every public quote and post entry.

The common swap path also has a fluent entry, `Trading::swap()`, returning a
typestate `SwapBuilder` with named setters (`sell_token`, `buy_token`,
`sell_amount`, `buy_amount`) so two same-typed token addresses cannot be
transposed at the call boundary, and `execute` / `quote` terminals reachable
only once the required markers are set ([ADR 0013](0013-http-transport-injection-and-typestate-builders.md)
sealed markers).

## Why

A protocol SDK that accepts a raw integer everywhere makes the most common bot
bug — confusing a human-readable `1.5` with its atomic
`1_500_000_000_000_000_000` — a runtime failure at first submission instead of a
compile-time refusal. A `Trading` that builds without `chain_id` or `app_code`
and then fails on the first quote pushes the same discovery to hours after
startup. Moving both to the type system removes whole classes of latent defect
without widening the runtime surface. Checked-only arithmetic extends the same
discipline to money math: `U256` wraps silently in every build profile, so
`sell - fee` for `fee > sell` would otherwise become a value near `2^256`. This
aligns the typed-amount surface with [ADR 0033](0033-minimum-viable-panic-surface.md).

## Must Remain True

- `Amount` is the single atomic amount type on every public crate's
  amount-carrying surface, `#[repr(transparent)]` over `U256` with a private
  inner field; `as_u256` / `into_u256` are the typed accessors.
- The wire form of every amount is the canonical base-10 decimal string; the
  `Deserialize` boundary is strict-decimal fail-closed (rejects every radix
  prefix).
- `Amount` arithmetic is `checked_*` / `saturating_*` only — no bare operators,
  no `pow`, no bit inspection. A committed compile-fail witness pins the removal.
- `Trading` constructs only through `TradingBuilder`; `build()` is reachable
  only on `<ChainIdSet, AppCodeSet>`; inherent constructors are forbidden. A
  missing prerequisite is a compile error, not a runtime panic.
- `TradeParams::new` / `LimitTradeParams::new` take no token-decimal arguments;
  `LimitTradeParamsFromQuote` guarantees its `quote_id` by construction.
- The wire and ABI boundary stays byte-equal against the pinned upstream parity
  fixtures.

## Alternatives Rejected

- **Keep a raw integer as the only public amount surface.** Simpler, but
  preserves the silent human-vs-atomic failure the SDK is most blamed for.
- **A second `decimals`-carrying amount type.** A `DecimalAmount` pairing an
  atomic value with a scale was load-bearing for no shipped flow and has no
  analogue in the upstream `@cowprotocol/cow-sdk`; decimal I/O lives as methods
  on the one atomic `Amount` instead.
- **A runtime-only builder returning `Err` on missing prerequisites.** Matches
  many builder crates, but defers discovery to the first quote or post in
  production.
- **Make `Trading` generic over a mode type parameter.** Compile-time safe, but
  leaks the mode into every downstream signature.
- **Bare arithmetic operators delegating to the inner integer.** Reintroduces a
  silent wrap (release) and debug-only panic on financial amounts.

## Intentionally Out-of-Scope

The decision records the canonical atomic type; it does not mirror every
historical upstream surface, including the retired wire-string `Amount` wrapper.
The authoritative list of TypeScript-SDK surfaces `cow-rs` intentionally
declines to mirror lives in [Parity Scope](../parity.md); consult it before
filing any issue claiming a missing positive fixture implies a parity gap.

## Links

- [Parity Scope](../parity.md)
- [Architecture](../architecture.md)
- [Verification](../verification.md)
- [Properties](../../PROPERTIES.md)
- [ADR 0002](0002-dedicated-trading-orchestration-crate.md),
  [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md),
  [ADR 0013](0013-http-transport-injection-and-typestate-builders.md),
  [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

**Proven by:**

- [Trading SDK Runtime Prerequisites Audit](../audit/trading-sdk-runtime-prerequisites-audit.md)
- [Typestate Builder Contract Audit](../audit/typestate-builder-contract-audit.md)
- [Trade-Parameter Lifecycle Audit](../audit/trade-parameter-lifecycle-audit.md)

## Acknowledgements

The fluent typestate swap builder ergonomics were suggested by
[@mfw78](https://github.com/mfw78) in public design review
([comment](https://github.com/0xSymbiome/cow-rs/pull/5#issuecomment-4648911544)).
