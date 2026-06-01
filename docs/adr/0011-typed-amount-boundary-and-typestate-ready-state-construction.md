# ADR 0011: Typed Amount Boundary And Typestate Ready-State Construction

- Status: Accepted (amended)
- Date: 2026-04-17
- Last reviewed: 2026-06-01
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: types, trading, builders, semver
- Related: [ADR 0002](0002-dedicated-trading-orchestration-crate.md), [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Decision

Public amount-carrying surfaces distinguish atomic and decimal-scaled
values through dedicated newtypes, and `TradingBuilder` advertises its
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
instead of a compile-time refusal. A `Trading` that builds successfully
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
  `TradingBuilder` exposes exactly two terminals: `build_ready`
  (requires both markers set) and `build_helper_only` (requires only the
  chain-id marker).
- `Trading` and `TradingBuilder` expose ready-state and helper-only
  construction exclusively through typestate-builder terminal methods.
  **Inherent associated constructors** (`Trading::new`,
  `Trading::new_partial`, or any future equivalent) are forbidden in
  shipped crates. One-call ergonomic shortcuts (e.g.,
  `TradingBuilder::ready(...)`) are typestate terminals consuming
  *total* typed inputs and never `Partial*` shapes.
- On `wasm32` targets, `build_ready()` additionally requires an injected
  orderbook client through `TradingOptions::with_orderbook_client(...)`.
  The default orderbook factory does not run on `wasm32` because the
  browser runtime does not ship a default `HttpTransport` (see ADR 0013).
- Runtime and support: the wire form of every amount remains the
  canonical base-10 string already defined by the orderbook contract.
  `Amount` serializes to that exact string via a custom serializer;
  decimal scaling is a pure presentation concern. Helper-only flows use
  the distinct `TradingHelpers` type, which exposes pre-sign, allowance,
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
- Trade-parameter surface: `TradeParameters` and `LimitTradeParameters`
  carry the protocol-level fields (kind, tokens, amounts, and the
  documented optional overrides) and do not accept token-decimal
  arguments at their `::new` constructors. The wasm input DTOs
  `SwapParametersInput` and `LimitTradeParametersInput` follow the
  same scope. `DecimalAmount` remains the canonical
  typed-amount-boundary home for token decimals across every
  display and user-input flow.
- Trade-parameter lifecycle: `TradeParameters` is the pre-quote
  request shape carrying a single amount interpreted by `kind`;
  `LimitTradeParameters` is the post-quote / canonical-submission
  shape carrying both `sell_amount` and `buy_amount` plus an
  optional `quote_id`. The lifecycle distinction is enforced
  through nominal typing on every submission and on-chain helper
  that needs both amounts. `LimitTradeParametersFromQuote` is a
  real newtype around `LimitTradeParameters` that guarantees a
  non-`None` `quote_id` by construction; it is produced exclusively
  by `swap_params_to_limit_order_params` and accepted by the
  `EthFlow` native-currency submission seam and the `EthFlow`
  transaction helper so the quote-identifier requirement is
  enforced at the type system rather than as a runtime check on
  the submission path. The `with_*` setter bodies shared by
  `TradeParameters` and `LimitTradeParameters` are factored
  through one internal definition that emits inherent methods on
  each public type without altering the public surface.
- Advanced-settings surface: one `TradeAdvancedSettings` bundle is
  accepted by every public quote and post entry. Limit-order
  callers leave `slippage_suggester` as `None` because the limit
  submission path does not apply slippage in the same shape as
  swaps; the field is documented but unused on that flow. The
  wasm export surface mirrors the same single-type shape.

## Alternatives Rejected

- Keep `Amount = BigUint` as the only public surface: simpler, but
  preserves the silent human-vs-atomic failure mode the SDK is most
  frequently blamed for.
- Make the builder runtime-only and return `Err` on missing
  prerequisites: matches many builder-pattern crates, but defers the
  discovery to the first quote or post call when the consumer is already
  in production.
- Make `Trading` generic over a mode type parameter: compile-time
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
- [Trade-Parameter Lifecycle Audit](../audit/trade-parameter-lifecycle-audit.md)

## Amendment 2026-05-22: canonical primitive layer (per ADR 0052)

`Amount` and `SignedAmount` ship as cow-owned `#[repr(transparent)]`
newtypes around `alloy_primitives::U256` and `alloy_primitives::I256`
respectively per
[ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md). The
newtypes carry cow-owned `Display`, `Serialize`, and `Deserialize`
impls plus a fallible-by-return arithmetic surface (`checked_*`
returning `Option`, and explicit `saturating_*` clamps). The
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

## Amendment 2026-05-26: trade-parameter decimals scope

`TradeParameters` and `LimitTradeParameters` carry the protocol-level
fields only. The `sell_token_decimals` and `buy_token_decimals`
fields, the matching positional `u8` arguments on `::new`, and the
equivalents on the wasm `SwapParametersInput` and
`LimitTradeParametersInput` DTOs are removed from the public surface;
the generated TypeScript declaration snapshots are refreshed in the
same change set. `DecimalAmount` remains the canonical
typed-amount-boundary home for token decimals; the typed-amount
invariants recorded above are preserved verbatim.

## Amendment 2026-05-27: trade-parameter consolidation and `LimitTradeParametersFromQuote` newtype

The trade-parameter surface consolidates around the lifecycle
distinction recorded above. `LimitTradeParametersFromQuote` ships
as a real newtype around `LimitTradeParameters` that guarantees a
non-`None` `quote_id` by construction; the prior transparent type
alias is removed. The `EthFlow` native-currency submission seam
(`post_sell_native_currency_order`) and the `EthFlow` transaction
helper (`get_eth_flow_transaction`) accept only
`LimitTradeParametersFromQuote` on their public entries, lifting
the prior `MissingQuoteId` runtime check on the `EthFlow` path to
a compile-time guarantee at the public boundary while preserving
the public diagnostic shape for callers that explicitly attempt
construction with a missing quote id.

The two prior advanced-settings types `SwapAdvancedSettings` and
`LimitOrderAdvancedSettings` collapse into one
`TradeAdvancedSettings` type accepted by every public post and
quote entry. Limit-order callers leave `slippage_suggester` as
`None` because the limit submission path does not apply slippage
in the same shape as swaps; the field is documented but unused on
that flow.

The shared `with_*` setter bodies on `TradeParameters` and
`LimitTradeParameters` continue to exist as inherent methods on
both public types, with the implementation factored through one
internal definition that is invoked once per target struct.

## Amendment 2026-05-28: owner placement

The `owner` field is a per-trade attribution that lives on
`TradeParameters`, `LimitTradeParameters`, and `OrderTraderParameters`.
It does not live on `PartialTraderParameters`, on `TraderParameters`,
or on the `TradingBuilder`. The SDK does not store a default
owner; the call-level owner is the only owner the SDK observes.

For signer-backed flows (`post_swap_order`,
`post_swap_order_from_quote`, `post_limit_order`,
`get_quote_results`) the signer address resolved through
`Signer::get_address` is the implicit fallback when
`TradeParameters.owner` is `None`. For quote-only flows
(`get_quote_only`) the owner must be supplied through
`TradeParameters.owner` or through
`advanced_settings.quote_request.from`; missing owner surfaces as
`TradingError::MissingOwner` at the call boundary.

The retired SDK-default-owner surface
(`TradingBuilder::with_owner`, `PartialTraderParameters::owner`,
`PartialTraderParameters::with_owner`) was load-bearing for no shipped
flow because per-call `TradeParameters.owner` won precedence in every
observing helper. The removal narrows the public surface without
changing observable behaviour.

The Trade-Parameter Lifecycle Audit and the Trading SDK Runtime
Prerequisites Audit are the standing current-state proofs for the
post-amendment invariant.

## Amendment 2026-05-28: checked-only typed-amount arithmetic

`Amount` and `SignedAmount` expose no bare arithmetic operators. The
`Add` / `Sub` / `Mul` (and `*Assign`) impls and the `pow` method are
removed from both newtypes. The supported arithmetic surface is
`checked_add` / `checked_sub` / `checked_mul` / `checked_pow` (each
returning `Option`), the explicit `saturating_*` clamps, and — on
`SignedAmount` — `checked_neg` / `checked_abs` / `checked_unsigned_abs`.
A caller that needs raw wrapping reaches through `as_u256` /
`into_u256` (respectively `as_i256` / `into_i256`), keeping the wrapping
intent visible at the type boundary.

The bare operators delegated to the inner alloy primitives, whose
overflow behaviour is unsafe for financial amounts: `U256` wraps
silently in every build profile (so `sell - fee` for `fee > sell`
silently became a value near `2^256`), while `I256` panicked only in
debug builds and wrapped silently in release. Removing the operators
makes the two newtypes symmetric and total/fallible — an overflow or
underflow is always either an `Option::None` the caller must handle or
an explicit clamp, never a silent corruption and never a
runtime-input-dependent panic, which also aligns the typed-amount
surface with [ADR 0033](0033-minimum-viable-panic-surface.md). The
arithmetic-operator clause in the 2026-05-22 amendment above is
superseded accordingly; the typed atomic-vs-decimal boundary and the
typestate-builder terminals recorded in the Decision are preserved
verbatim. A committed compile-fail witness pins the removal so a
wrapping (or debug-only panicking) operator cannot silently return to
the typed amount surface.

## Amendment 2026-06-01: decimal I/O on the atomic `Amount`; `DecimalAmount` removed

`DecimalAmount` is removed from the public surface. The
atomic-vs-decimal split recorded in the original Decision named a
second amount-carrying newtype that paired an atomic value with a
`decimals` scale; that type was load-bearing for no shipped flow —
zero type-position uses across every public crate, and no analogue in
the upstream `@cowprotocol/cow-sdk` it ports. The removal is exactly
parallel to the 2026-05-28 `with_owner` removal above: dead public
surface that no observing helper depended on, narrowed without
changing observable behaviour.

Exact human-decimal construction and display now live directly on the
atomic `Amount`. `Amount::parse_units(value, decimals) -> Result<Amount,
CoreError>` builds an atomic amount from a human-readable decimal
string and the token's `decimals` scale, and `Amount::format_units(&self,
decimals) -> String` renders an atomic amount back to its scaled decimal
string. These are the typed analogues of the viem/ethers
`parseUnits`/`formatUnits` helpers, scaling by `10^decimals` through
integer arithmetic so the result is exact. `parse_units` fails closed on
empty/whitespace input, on a leading `+`/`-` sign (`Amount` is
unsigned), and on `decimals` above `77`, surfacing
`ValidationError::DecimalsOutOfRange` for the last case per
[ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md).

`Amount::from_units(whole, decimals) -> Result<Amount, CoreError>` is the
numeric companion to `parse_units` for the common case where the amount is a
whole number already held as an integer (for example
`Amount::from_units(1000, 6)` for 1000 USDC): it scales a `u128` whole-unit
count by `10^decimals` with the same checked integer arithmetic and the same
`decimals <= 77` bound, so a caller never has to render a number as a string
or hand-count zeros. A decimal string is therefore required only for
genuinely fractional or untrusted-text input, never for a whole-token
literal.

`Amount` stays atomic: none of these methods store a scale on the value, so
the single canonical `Amount(U256)` newtype remains the one atomic
amount type across every public crate. `decimals` is supplied per call
at the decimal-I/O boundary and is never carried on the wire. The wire
form is unchanged — every amount still serialises to the canonical
base-10 string defined by the orderbook contract, and the strict-decimal
fail-closed `Deserialize` boundary recorded in the 2026-05-22 amendment
is preserved. `TradeParameters::new` and `LimitTradeParameters::new`
still take no token-decimal arguments; the 2026-05-26 trade-parameter
decimals-scope amendment is preserved verbatim, with the decimal-I/O
home now the atomic `Amount` rather than the retired `DecimalAmount`.
The typed atomic-vs-decimal boundary, the typestate-builder terminals,
and every prior amendment recorded above are otherwise preserved
verbatim.
