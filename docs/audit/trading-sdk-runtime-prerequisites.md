# Trading SDK Runtime Prerequisites Audit

Status: Current  
Last reviewed: 2026-04-15

## Scope

This audit covers:

- ready-state and partial `TradingSdk` construction
- method-specific prerequisites across quote, post, cancellation, allowance,
  approval, and pre-sign helper flows
- the boundary between trading attribution requirements and chain-bound helper
  requirements

It does not cover browser-wallet session behavior, orderbook transport policy,
or unrelated credential-hygiene questions.

## Findings Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Ready-state SDK construction | `TradingSdk::build` and `TradingSdk::new` require `appCode` plus chain authority before exposing the ready quote/post surface | Conforms |
| Partial helper construction | Explicit partial constructors keep helper-only setup available without weakening the ready-state contract | Conforms |
| Chain-bound helper prerequisites | Allowance, approval, pre-sign, and on-chain cancellation no longer require `appCode` when only chain and protocol context are needed | Conforms |

## Findings

### Ready-state construction

`TradingSdk::build` and `TradingSdk::new` now encode the prerequisites of the
surface they advertise. A ready-state SDK must supply `appCode` and either an
explicit `chainId` or an injected orderbook client that fixes chain authority.
Construction therefore fails locally when those prerequisites are absent
instead of returning an instance that will only fail later during quote or post
execution.

### Explicit partial construction

`TradingSdk::build_partial` and `TradingSdk::new_partial` keep the narrower
helper-only contract explicit. They are intended for workflows such as
allowance reads, approval submission, pre-sign transaction construction, and
on-chain cancellation, where chain and protocol context matter but quote or
submission attribution does not.

### Helper-specific prerequisites

Allowance, approval, pre-sign, and on-chain cancellation now resolve only the
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
