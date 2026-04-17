# Trading SDK Runtime Prerequisites Audit

Status: Current  
Last reviewed: 2026-04-15  
Owning surface: `cow-sdk-trading` ready-state versus partial `TradingSdk` construction and helper-specific prerequisite contract  
Refresh trigger: Changes to ready-state `TradingSdk` constructors or builders, partial setup entry points, or method-specific prerequisite enforcement  
Related docs:
- [ADR 0002](../adr/0002-dedicated-trading-orchestration-crate.md)
- [ADR 0006](../adr/0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md)
- [ADR 0011](../adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md)
- [README](../../README.md)
- [Verification Guide](../verification-guide.md)
- [Verification Matrix](../verification-matrix.md)

## Scope

This audit covers:

- ready-state and partial `TradingSdk` construction
- method-specific prerequisites across quote, post, cancellation, allowance,
  approval, and pre-sign helper flows
- the boundary between trading attribution requirements and chain-bound helper
  requirements

It does not cover browser-wallet session behavior, orderbook transport policy,
or unrelated credential-hygiene questions.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Ready-state SDK construction | `TradingSdk::build` and `TradingSdk::new` require `appCode` plus chain authority before exposing the ready quote or post surface | Conforms |
| Partial helper construction | Explicit partial constructors keep helper-only setup available without weakening the ready-state contract | Conforms |
| Chain-bound helper prerequisites | Allowance, approval, pre-sign, and on-chain cancellation no longer require `appCode` when only chain and protocol context are needed | Conforms |

## Current Contract

### Ready-State Construction

`TradingSdk::build` and `TradingSdk::new` encode the prerequisites of the
surface they advertise. A ready-state SDK must supply `appCode` and either an
explicit `chainId` or an injected orderbook client that fixes chain authority.
Construction therefore fails locally when those prerequisites are absent
instead of returning an instance that will only fail later during quote or
post execution.

### Explicit Partial Construction

`TradingSdk::build_partial` and `TradingSdk::new_partial` keep the narrower
helper-only contract explicit. They are intended for workflows such as
allowance reads, approval submission, pre-sign transaction construction, and
on-chain cancellation, where chain and protocol context matter but quote or
submission attribution does not.

### Helper-Specific Prerequisites

Allowance, approval, pre-sign, and on-chain cancellation resolve only the
chain-bound protocol context they actually need. `appCode` remains required for
quote, post, and off-chain cancellation flows, where app-data attribution and
orderbook submission semantics depend on it, but it is no longer forced into
helpers that do not consume that contract.

## Evidence

Primary implementation points:

- `crates/trading/src/sdk.rs`
- `crates/trading/src/onchain.rs`
- `crates/sdk/src/lib.rs`
- `README.md`

Primary regression coverage:

- `crates/trading/tests/sdk_contract.rs`
- `crates/sdk/tests/public_api.rs`

Validation surface:

```text
cargo fmt --all --check
cargo test -p cow-sdk-trading
cargo test -p cow-sdk
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-features --target wasm32-unknown-unknown
```
