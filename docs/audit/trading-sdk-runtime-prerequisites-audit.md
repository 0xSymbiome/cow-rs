# Trading SDK Runtime Prerequisites Audit

Status: Current
Last reviewed: 2026-05-12
Owning surface: `cow-sdk-trading` ready-state `TradingSdk` construction, helper-only `HelperOnlySdk` construction, and helper-specific prerequisite contract
Refresh trigger: Changes to ready-state `TradingSdk` builder terminals, helper-only setup entry points, method-specific prerequisite enforcement, or any change that weakens the wasm32 orderbook-client requirement inside `build_ready()`
Related docs:
- [ADR 0002](../adr/0002-dedicated-trading-orchestration-crate.md)
- [ADR 0006](../adr/0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md)
- [ADR 0011](../adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md)
- [README](../../README.md)
- [Verification Guide](../verification-guide.md)
- [Verification Matrix](../verification-matrix.md)

## Scope

This audit covers:

- ready-state `TradingSdk` and helper-only `HelperOnlySdk` construction
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
| Typestate ready construction | `TradingSdkBuilder::build_ready` and `TradingSdkBuilder::ready` require total chain id plus validated `appCode` inputs before ready-state construction | Conforms |
| wasm32 build_ready() requires injected orderbook client | `build_ready()` returns `TradingError::MissingInjectedOrderbookClient` when `options.orderbook_client().is_none()` on `wasm32` | Conforms |
| Helper-only construction | `TradingSdkBuilder::build_helper_only` and `TradingSdkBuilder::helper_only` return the distinct `HelperOnlySdk` type on native and wasm32 without weakening the ready-state contract | Conforms |
| Chain-bound helper prerequisites | Allowance, approval, pre-sign, and on-chain cancellation no longer require `appCode` when only chain and protocol context are needed | Conforms |

## Current Contract

### Ready-State Construction

`TradingSdkBuilder::build_ready` is available only after the builder has both
chain id and `AppCode` typestate markers set, so missing ready-state
prerequisites are rejected at compile time for fluent builder callers and
invalid attribution strings are rejected before the SDK handle is returned.
`TradingSdkBuilder::ready` is the one-call ready-state shortcut for callers
that already hold total `TraderParameters`; it does not accept partial defaults.

### wasm32 Typestate Ready Terminal

`TradingSdkBuilder::build_ready()` is the stronger typestate terminal. On
native targets it remains compatible with the default orderbook factory. On
`wasm32`, the terminal additionally requires an injected orderbook client
because the browser runtime does not ship a default HTTP transport; the
terminal now returns `TradingError::MissingInjectedOrderbookClient` instead of
returning a misleading ready-state handle whose first quote or post call would
fail in orderbook binding resolution.

The root `cow-sdk` facade re-exports `TradingSdkOptions` so consumers can
inject the browser orderbook client from the same first-touch import surface
used by native ready-state construction.

### Helper-Only Construction

`TradingSdkBuilder::build_helper_only` and `TradingSdkBuilder::helper_only`
keep the narrower helper-only contract explicit. They are intended for
workflows such as allowance reads, approval submission, pre-sign transaction
construction, and on-chain cancellation, where chain and protocol context
matter but quote or submission attribution does not. Both construction paths
require a chain id and produce `HelperOnlySdk`. On `wasm32`, helper-only
construction does not require an injected orderbook client because the
resulting type does not expose quote, post, or off-chain cancellation methods.

### Helper-Specific Prerequisites

Allowance, approval, pre-sign, and on-chain cancellation resolve only the
chain-bound protocol context they actually need. `AppCode` remains required for
quote, post, and off-chain cancellation flows, where app-data attribution and
orderbook submission semantics depend on it, but it is no longer forced into
helpers that do not consume that contract.

## Evidence

Primary implementation points:

- `crates/trading/src/sdk/builder.rs`
- `crates/trading/src/types/trader.rs`
- `crates/trading/src/types/options.rs`
- `crates/trading/src/onchain.rs`
- `crates/sdk/src/prelude.rs`
- `crates/sdk/src/lib.rs`
- `README.md`

Primary regression coverage:

- `crates/trading/tests/sdk_contract.rs::build_ready_rejects_missing_injected_orderbook_client_on_wasm32`
- `crates/trading/tests/sdk_contract.rs::build_ready_succeeds_on_wasm32_with_injected_orderbook_client`
- `crates/trading/tests/sdk_contract.rs::build_helper_only_succeeds_on_wasm32_without_injected_orderbook_client`
- `crates/trading/tests/sdk_contract.rs::build_ready_succeeds_on_native_without_injected_orderbook_client`
- `crates/trading/tests/sdk_contract.rs::sdk_ready_shortcut_accepts_total_trader_parameters`
- `crates/trading/tests/sdk_contract.rs::sdk_helper_only_shortcut_builds_helper_only_type`
- `crates/trading/tests/app_code_contract.rs`
- `crates/trading/tests/ui/helper_only_sdk_no_quote_methods.rs`
- `crates/trading/tests/ui/helper_only_sdk_no_offchain_cancel.rs`
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
