# Trading SDK Runtime Prerequisites Audit

Status: Current
Last reviewed: 2026-06-02
Owning surface: `cow-sdk-trading` ready-state `Trading` construction, the chain-bound helper free functions, helper-specific prerequisite contract, and per-trade owner attribution
Refresh trigger: Changes to ready-state `Trading` builder terminals, the chain-bound helper free functions, method-specific prerequisite enforcement, the per-trade owner-attribution placement, or any change that weakens the wasm32 orderbook-client requirement inside `build()`
Related docs:
- [ADR 0002](../adr/0002-dedicated-trading-orchestration-crate.md)
- [ADR 0006](../adr/0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md)
- [ADR 0011](../adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md)
- [README](../../README.md)
- [Verification Guide](../verification.md)
- [Verification Matrix](../verification.md)
- [Trade-Parameter Lifecycle Audit](trade-parameter-lifecycle-audit.md)

## Scope

This audit covers:

- ready-state `Trading` construction and the chain-bound helper free functions
- method-specific prerequisites across quote, post, cancellation, allowance,
  approval, and pre-sign helper flows
- the boundary between trading attribution requirements and chain-bound helper
  requirements

It does not cover browser-wallet session behavior, orderbook transport policy,
or unrelated credential-hygiene questions.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| AppCode attribution | Trading attribution uses the `AppCode` newtype, rejecting empty strings, NUL bytes, and ASCII control characters before ready-state construction | Conforms |
| Typestate ready construction | `TradingBuilder::build` and `TradingBuilder::ready` require total chain id plus validated `appCode` inputs before ready-state construction | Conforms |
| wasm32 build() requires injected orderbook client | `build()` returns `TradingError::MissingInjectedOrderbookClient` when `options.orderbook_client().is_none()` on `wasm32` | Conforms |
| Chain-bound helper free functions | `cow_protocol_allowance`, `approval_transaction`, `pre_sign_transaction`, and `cancel_order_onchain` need chain authority but no `appCode`, and run without a trading client | Conforms |
| Chain-bound helper prerequisites | Allowance, approval, pre-sign, and on-chain cancellation no longer require `appCode` when only chain and protocol context are needed | Conforms |
| Per-trade owner attribution | `TradeParameters.owner`, `LimitTradeParameters.owner`, and `OrderTraderParameters` carry the per-trade owner. The SDK does not store a default owner; for signer-backed flows the signer address resolved through `Signer::address` is the implicit fallback, and for quote-only flows the owner must come from `TradeParameters.owner` or `advanced_settings.quote_request.from`. | Conforms |

## Current Contract

### Ready-State Construction

`TradingBuilder::build` is available only after the builder has both
chain id and `AppCode` typestate markers set, so missing ready-state
prerequisites are rejected at compile time for fluent builder callers and
invalid attribution strings are rejected before the SDK handle is returned.
`TradingBuilder::ready` is the one-call ready-state shortcut for callers
that already hold total `TraderParameters`; it does not accept partial defaults.

### wasm32 Typestate Ready Terminal

`TradingBuilder::build()` is the stronger typestate terminal. On
native targets it remains compatible with the default orderbook factory. On
`wasm32`, the terminal additionally requires an injected orderbook client
because the browser runtime does not ship a default HTTP transport; the
terminal now returns `TradingError::MissingInjectedOrderbookClient` instead of
returning a misleading ready-state handle whose first quote or post call would
fail in orderbook binding resolution.

The root `cow-sdk` facade re-exports `TradingOptions` so consumers can
inject the browser orderbook client from the same first-touch import surface
used by native ready-state construction.

### Chain-Bound Helper Free Functions

Allowance reads, approval submission, pre-sign transaction construction, and
on-chain cancellation are the crate's free functions —
`cow_protocol_allowance`, `approval_transaction`, `pre_sign_transaction`,
and `cancel_order_onchain`. They take chain and protocol context directly,
need no `appCode`, and require no trading client, so an appCode-less integration
(an allowance/approval screen, a pre-sign tool) calls them without constructing
`Trading`. The full `Trading` client also exposes these as conveniences for
callers that already hold one.

### Helper-Specific Prerequisites

Allowance, approval, pre-sign, and on-chain cancellation resolve only the
chain-bound protocol context they actually need. `AppCode` remains required for
quote, post, and off-chain cancellation flows, where app-data attribution and
orderbook submission semantics depend on it, but it is no longer forced into
helpers that do not consume that contract.

### Per-Trade Owner Attribution

The trading SDK does not store a default owner. The `owner` field
lives on the per-trade types (`TradeParameters`, `LimitTradeParameters`)
and on `OrderTraderParameters` for order-context flows. The
`TradingBuilder` does not expose `with_owner`, and
`PartialTraderParameters` does not carry an `owner` field.

Resolved owner precedence is:

- Quote-only flows (`quote_only`):
  `advanced_settings.quote_request.from` → `TradeParameters.owner` →
  `TradingError::MissingOwner`.
- Signer-backed flows (`post_swap_order`,
  `post_swap_order_from_quote`, `post_limit_order`,
  `quote_results`): `TradeParameters.owner` → signer address
  resolved through `Signer::address`.

Documented owner precedence is the only owner contract observed by the
SDK; no SDK-level fallback fires.

## Evidence

Primary implementation points:

- `crates/trading/src/sdk/builder.rs`
- `crates/trading/src/types/trader.rs`
- `crates/trading/src/types/options.rs`
- `crates/trading/src/onchain.rs`
- `crates/trading/src/sdk/helpers.rs`
- `crates/trading/src/quote.rs`
- `crates/sdk/src/prelude.rs`
- `crates/sdk/src/lib.rs`
- `README.md`

Primary regression coverage:

- `crates/trading/tests/sdk_contract.rs::build_rejects_missing_injected_orderbook_client_on_wasm32`
- `crates/trading/tests/sdk_contract.rs::build_succeeds_on_wasm32_with_injected_orderbook_client`
- `crates/trading/tests/sdk_contract.rs::build_succeeds_on_native_without_injected_orderbook_client`
- `crates/trading/tests/sdk_contract.rs::sdk_ready_shortcut_accepts_total_trader_parameters`
- `crates/trading/tests/app_code_contract.rs`
- `crates/trading/tests/types_contract.rs`
- `crates/sdk/tests/public_api.rs`

Validation surface:

```text
cargo fmt --all --check
cargo test -p cow-sdk-trading
cargo test -p cow-sdk
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-features --target wasm32-unknown-unknown
cd crates/trading && wasm-pack test --headless --chrome
```
