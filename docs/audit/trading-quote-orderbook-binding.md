# Trading Quote Orderbook Binding Audit

Status: Current  
Last reviewed: 2026-04-14

## Scope

This audit covers:

- swap quote results produced by `cow-sdk-trading`
- the `post_swap_order_from_quote*` helper surface
- runtime binding between quote creation and order submission

It does not cover direct limit-order posting, approval, allowance, or on-chain
transaction-building helpers that do not reuse quote-derived posting state.

## Decision Summary

| Area | Decision |
| --- | --- |
| Quote results | Capture the originating orderbook runtime binding |
| Post-from-quote helpers | Require submission to use the same orderbook runtime binding |
| Quote identifiers | Treat quote-derived identifiers as bound to the quote origin, not as portable across orderbook clients |
| Detached serialization | Keep quote results serializable, but reject reuse against a mismatched orderbook binding |

## Current Contract

Quote-derived posting now follows one runtime contract from quote creation
through submission.

`get_quote_results*` stores the originating orderbook runtime binding inside
the returned `QuoteResults`. That binding captures:

- `chain_id`
- `env`
- the resolved orderbook base URL when the client exposes it

`post_swap_order_from_quote*` validates the submission-time orderbook client
against the captured binding before it merges app-data overrides, signs, or
submits the order.

This keeps quote identifiers, typed-data domains, and submission routing bound
to the same orderbook runtime rather than allowing a quote produced by one
orderbook client to be reused through another.

## Evidence

Relevant source files:

- `crates/orderbook/src/api.rs`
- `crates/trading/src/types.rs`
- `crates/trading/src/quote.rs`
- `crates/trading/src/post.rs`

Relevant contract coverage:

- `crates/trading/tests/quote_contract.rs::quote_results_capture_originating_orderbook_runtime_binding`
- `crates/trading/tests/post_contract.rs::post_from_quote_reuses_matching_orderbook_binding_and_submits_order`
- `crates/trading/tests/post_contract.rs::post_from_quote_rejects_orderbook_binding_mismatch_before_signing_or_submission`

Validation commands:

```text
cargo fmt --all --check
cargo test -p cow-sdk-trading
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
