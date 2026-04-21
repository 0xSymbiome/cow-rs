# Shared Logic Reviewability Audit

Status: Current  
Last reviewed: 2026-04-21  
Owning surface: Orderbook, signing, and trading shared-logic reviewability boundary  
Refresh trigger: Changes to shared orderbook request execution, signing payload construction, thin posting wrappers, or boundary-specific order DTO separation that materially affect correctness or reviewability  
Related docs:
- [ADR 0005](../adr/0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [Architecture](../architecture.md)
- [Verification Guide](../verification-guide.md)

## Scope

This audit covers:

- orderbook request construction and execution
- retry, status mapping, headers, rate-limit handling, and JSON, text, or
  empty responses
- order signing and cancellation signing payload preparation
- trading posting wrapper paths
- generated or schema-derived artifacts as a separate category

It does not cover style-only cleanup notes, generic refactor wishlists, or unrelated
internal refactors that do not affect correctness or reviewability.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Shared HTTP request construction | Use one shared orderbook request path | Conforms |
| Shared retry, status, and rate-limit execution | Use one shared executor for JSON, text, and empty responses | Conforms |
| Shared signing payload preparation | Share payload construction between sync and async signing paths | Conforms |
| Thin trading posting wrappers | Keep ergonomic entry points thin and route workflow logic through the async implementation path | Conforms |
| Boundary-specific order DTO separation | Retain distinct DTOs only where ABI, API, normalized, or user-domain boundaries differ materially | Conforms |

## Current Contract

### Orderbook Request Execution

Orderbook request execution is shared through internal helpers in
`crates/orderbook/src/request.rs`, including `request_with`, `send_request`,
`request_headers`, and `execute_with`.

### Signing Payload Preparation

Signing keeps separate sync and async entry points while sharing payload
construction through:

- `crates/signing/src/order_signing.rs::order_signing_payload`
- `crates/signing/src/cancellation.rs::cancellation_signing_payload`

### Thin Trading Posting Wrappers

Trading keeps ergonomic public entry points while routing workflow logic
through async implementation paths. Shared advanced-parameter extraction lives
in:

- `crates/trading/src/post.rs::swap_additional_params`
- `crates/trading/src/post.rs::limit_additional_params`

### Boundary-Specific Order DTO Separation

Order-like DTO separation is retained only where the boundary is materially
different:

- `cow_sdk_core::UnsignedOrder`
- `cow_sdk_contracts::Order`
- `cow_sdk_contracts::NormalizedOrder`
- `cow_sdk_orderbook::QuoteData`
- `cow_sdk_orderbook::OrderCreation`
- `cow_sdk_orderbook::Order`

Generated or schema-derived artifacts remain internal or test-only and are not
part of the public SDK API.

## Evidence

Primary regression coverage:

- `crates/orderbook/tests/request_contract.rs::request_json_retries_429_and_preserves_headers_on_each_attempt`
- `crates/orderbook/tests/request_contract.rs::request_text_and_empty_share_the_request_builder_and_success_path`
- `crates/orderbook/tests/request_contract.rs::rate_limiter_spaces_requests_after_token_budget_is_consumed`
- `crates/signing/tests/order_signing_contract.rs::async_sign_order_paths_match_sync_signing_behavior`
- `crates/signing/tests/cancellation_contract.rs::async_cancellation_signing_paths_match_sync_variants`
- `crates/trading/tests/post_contract.rs::limit_posting_sync_signer_wrapper_matches_async_suffix_path`
- `crates/contracts/tests/order_contract.rs::unsigned_order_conversion_makes_user_domain_and_contract_boundaries_explicit`
- `crates/orderbook/tests/types_contract.rs::order_creation_from_quote_keeps_quote_shape_and_quote_id`

Validation surface:

```text
cargo fmt --all --check
cargo test -p cow-sdk-orderbook
cargo test -p cow-sdk-signing
cargo test -p cow-sdk-trading
cargo test --workspace
cargo clippy -p cow-sdk-orderbook --all-targets --all-features -- -D warnings
cargo clippy -p cow-sdk-signing --all-targets --all-features -- -D warnings
cargo clippy -p cow-sdk-trading --all-targets --all-features -- -D warnings
```
