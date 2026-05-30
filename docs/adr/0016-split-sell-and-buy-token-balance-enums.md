# ADR 0016: Split `SellTokenSource` And `BuyTokenDestination` Into Distinct Side-Specific Enums

- Status: Accepted
- Date: 2026-04-21
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: core, contracts, types, balance-sources, error-typing
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0011](0011-typed-amount-boundary-and-typestate-ready-state-construction.md)

## Decision

`cow-sdk-core` exposes the sell-side allowance path and the
buy-side payout path as two distinct typed enums. `SellTokenSource`
carries `Erc20`, `External`, and `Internal`. `BuyTokenDestination`
carries `Erc20` and `Internal`. Both enums are `#[non_exhaustive]`
with `Default = Erc20`, both round-trip byte-identically with the
reviewed services authority through snake-case serde, and the type
system rejects any cross-side coercion at compile time. The prior
shared `OrderBalance` enum and its `normalize_for_buy` helper that
silently rewrote `External -> Erc20` on the buy side are retired.

## Why

A shared `OrderBalance` enum collapses two semantically distinct
contracts into one shape and forces every consumer to remember
which variants are valid on which side. The reviewed services
authority models the two sides as separate enums for exactly this
reason: the buy-side payout path has no notion of an external
balance source, and silently downgrading `External` to `Erc20` on
the buy side is a silent rewriting of a reviewed wire contract.
Splitting the enums in `cow-sdk-core` lifts the side discipline
into the type system, makes cross-side coercion a compile error
across the workspace, and lets the `OrderCreation`,
`OrderData`, `QuoteData`, and `Order` DTOs carry the
side-specific type on the right field.

## Must Remain True

- Public surface: `cow_sdk_core::SellTokenSource { Erc20, External,
  Internal }` and `cow_sdk_core::BuyTokenDestination { Erc20,
  Internal }` are the canonical types. Both enums carry
  `#[non_exhaustive]`, `Default = Erc20`, and snake-case serde
  encoding. The shared `OrderBalance` enum and the
  `OrderBalance::normalize_for_buy` helper do not exist on the
  shipped surface.
- Runtime and support: every DTO and helper that previously
  carried `OrderBalance` for the sell-side now carries
  `SellTokenSource`, and every DTO and helper that previously
  carried `OrderBalance` for the buy-side now carries
  `BuyTokenDestination`. The contract-encoding helpers in
  `cow-sdk-contracts` split into `sell_balance_name`,
  `buy_balance_name`, `sell_balance_id`, and `buy_balance_id`, and
  the settlement-encoding flags route the sell- and buy-side
  values through their respective enums. No internal shim collapses
  the two enums back into one shape.
- Validation and review: a fixture round-trip test pins both enums
  to the reviewed snake-case wire strings (`"erc20"`, `"external"`,
  `"internal"`) and asserts the closed `BuyTokenDestination` domain
  rejects the sell-only `"external"` value on deserialization. A
  pinned compile-fail witness under
  `crates/core/tests/ui/token_balance_split_cross_side.rs` proves
  that assigning a `SellTokenSource` value into a
  `BuyTokenDestination`-typed field on `OrderData` fails to
  compile.
- Cost: every shipped crate (`cow-sdk-core`, `cow-sdk-contracts`,
  `cow-sdk-orderbook`, `cow-sdk-signing`, `cow-sdk-trading`,
  `cow-sdk`) carries the side-specific type on its sell- and
  buy-side fields. The `#[non_exhaustive]` discipline keeps the
  contract open to future minor wire additions without breaking
  downstream exhaustive matches.

## Alternatives Rejected

- Keep the shared `OrderBalance` enum and add a side parameter to
  every consumer: shorter type list, but every consumer must then
  remember which variants are valid on which side and the
  compile-time check evaporates.
- Keep the shared enum and harden the
  `OrderBalance::normalize_for_buy` helper with `Result`-returning
  semantics: explicit, but still allows `External` to flow into
  buy-side construction code paths that the type system should
  reject before the call.
- Use a single closed enum with the sell-only variants gated
  behind `#[cfg]` flags: clever, but inverts the runtime contract
  and forces every reviewer to read the `cfg` to know whether a
  variant is reachable.

## Links

- [Architecture](../architecture.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0011](0011-typed-amount-boundary-and-typestate-ready-state-construction.md)
