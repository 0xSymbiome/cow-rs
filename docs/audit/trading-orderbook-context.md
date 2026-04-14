# Trading Orderbook Context Audit

Status: Current  
Last reviewed: 2026-04-14

## Scope

This audit covers:

- orderbook-bound free-function helpers in `cow-sdk-trading`
- chain and environment authority when callers supply both typed trading
  parameters and an injected orderbook client
- quote, post, and off-chain cancellation flows

It does not cover non-orderbook helpers that already take explicit chain
resolution from the caller, such as approval, allowance, or on-chain
transaction-building helpers.

## Decision Summary

| Area | Decision |
| --- | --- |
| Quote helpers | Reject conflicting explicit chain or environment inputs and use the injected orderbook context as the canonical runtime authority |
| Posting helpers | Reject conflicting explicit chain or environment inputs and use the injected orderbook context as the canonical runtime authority |
| Off-chain cancellation | Reject conflicting explicit chain or environment inputs and use the injected orderbook context as the canonical runtime authority |
| Contract overrides | Keep existing precedence for settlement and `EthFlow` override selection |
| Non-orderbook helpers | Keep caller-resolved chain and environment behavior |

## Current Contract

When an orderbook client is injected into an orderbook-bound trading helper,
its `ApiContext` is the canonical source of `chain_id` and `env`.

The public helper surface therefore follows two rules:

- any explicit chain or environment supplied through call-level parameters,
  trader parameters, or quoter parameters must match the injected orderbook
  client
- once validated, the helper uses `orderbook.context()` for typed-data domain
  selection, contract resolution, `EthFlow` adjustments, cancellation signing,
  and submission-time routing

This keeps the direct helper surface aligned with `TradingSdk` rather than
allowing two competing runtime authorities to coexist inside the same request.

## Evidence

Relevant source files:

- `crates/trading/src/types.rs`
- `crates/trading/src/sdk.rs`
- `crates/trading/src/quote.rs`
- `crates/trading/src/post.rs`
- `crates/trading/src/cancel.rs`

Relevant contract coverage:

- `crates/trading/tests/sdk_contract.rs::sdk_builder_validates_injected_orderbook_context_and_client_context_can_supply_chain_and_env`
- `crates/trading/tests/sdk_contract.rs::sdk_orderbook_bound_calls_reject_env_conflicts_with_injected_client_context`
- `crates/trading/tests/quote_contract.rs::quote_helpers_reject_injected_orderbook_chain_conflicts`
- `crates/trading/tests/post_contract.rs::limit_posting_rejects_trader_env_conflicts_with_orderbook_context`
- `crates/trading/tests/cancel_contract.rs::offchain_cancellation_rejects_call_level_chain_conflicts_with_orderbook_context`

Validation commands:

```text
cargo test -p cow-sdk-trading
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
