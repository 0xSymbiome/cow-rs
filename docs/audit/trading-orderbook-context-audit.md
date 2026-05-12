# Trading Orderbook Context Audit

Status: Current  
Last reviewed: 2026-05-12
Owning surface: `cow-sdk-trading` runtime authority for orderbook-bound helpers  
Refresh trigger: Changes to orderbook-bound quote, post, or off-chain cancellation helpers, or to chain and environment resolution when an orderbook client is injected  
Related docs:
- [ADR 0002](../adr/0002-dedicated-trading-orchestration-crate.md)
- [ADR 0006](../adr/0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md)
- [Architecture](../architecture.md)
- [Verification Guide](../verification-guide.md)
- [Verification Matrix](../verification-matrix.md)

## Scope

This audit covers:

- orderbook-bound free-function helpers in `cow-sdk-trading`
- chain and environment authority when callers supply both typed trading
  parameters and an injected orderbook client
- quote, post, and off-chain cancellation flows

It does not cover non-orderbook helpers that already take explicit chain
resolution from the caller, such as approval, allowance, or on-chain
transaction-building helpers.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Quote helpers | Reject conflicting explicit chain or environment inputs and use the injected orderbook context as the canonical runtime authority | Conforms |
| Posting helpers | Reject conflicting explicit chain or environment inputs and use the injected orderbook context as the canonical runtime authority | Conforms |
| Off-chain cancellation | Reject conflicting explicit chain or environment inputs and use the injected orderbook context as the canonical runtime authority | Conforms |
| Contract overrides | Preserve existing precedence for settlement and `EthFlow` override selection | Conforms |
| Non-orderbook helpers | Keep caller-resolved chain and environment behavior | Conforms |

## Current Contract

### Canonical Runtime Authority

When an orderbook client is injected into an orderbook-bound trading helper,
its `ApiContext` is the canonical source of `chain_id` and `env`.

### Validation Boundary

Any explicit chain or environment supplied through call-level parameters,
trader parameters, or quoter parameters must match the injected orderbook
client. Once validated, the helper uses `orderbook.context()` for typed-data
domain selection, contract resolution, `EthFlow` adjustments, cancellation
signing, and submission-time routing.

### Non-Orderbook Helper Boundary

This keeps the direct helper surface aligned with `TradingSdk` rather than
allowing two competing runtime authorities to coexist inside the same request.
Helpers that do not take an injected orderbook client keep their explicit
caller-resolved authority model.

## Evidence

Primary implementation points:

- `crates/trading/src/types/trader.rs`
- `crates/trading/src/types/context.rs`
- `crates/trading/src/types/options.rs`
- `crates/trading/src/sdk/{builder,helpers}.rs`
- `crates/trading/src/quote.rs`
- `crates/trading/src/post/generic.rs`
- `crates/trading/src/cancel.rs`

Primary regression coverage:

- `crates/trading/tests/sdk_contract.rs::sdk_builder_validates_injected_orderbook_context_and_client_context_can_supply_chain_and_env`
- `crates/trading/tests/sdk_contract.rs::sdk_orderbook_bound_calls_reject_env_conflicts_with_injected_client_context`
- `crates/trading/tests/quote_contract.rs::quote_helpers_reject_injected_orderbook_chain_conflicts`
- `crates/trading/tests/post_contract.rs::limit_posting_rejects_trader_env_conflicts_with_orderbook_context`
- `crates/trading/tests/cancel_contract.rs::offchain_cancellation_rejects_call_level_chain_conflicts_with_orderbook_context`

Validation surface:

```text
cargo test -p cow-sdk-trading
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
