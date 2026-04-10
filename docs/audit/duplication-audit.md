# Duplication Audit

Last reviewed: 2026-04-10

This document tracks duplication risks in `cow-rs` and records how each category is handled. It distinguishes mechanical duplication, which should be removed, from repeated shapes that exist because the SDK models different protocol contracts.

## Audit Scope

Covered in this revision:

- orderbook request construction and execution,
- retry, status mapping, headers, rate-limit handling, and JSON/text/empty responses,
- order signing and cancellation signing payload preparation,
- trading posting wrapper paths.

Planned for a later revision:

- DTO boundary rationale across contract ABI, orderbook API, subgraph, and user-domain types,
- conversion evidence between semantically distinct but field-similar types,
- generated/schema-derived artifact review.

## Classification

| Category | Status | Decision |
| --- | --- | --- |
| Repeated HTTP request construction | Addressed | Use one shared orderbook request path. |
| Repeated retry/status/rate-limit loops | Addressed | Use one shared executor for JSON, text, and empty responses. |
| Repeated signing payload preparation | Addressed | Share payload construction between sync and async signing paths. |
| Trading posting wrapper pairs | Addressed | Keep ergonomic entry points thin and route workflow logic through the async implementation path. |
| Repeated order-like DTO fields | Open | Keep separate until each ABI, API, normalized, and user-domain boundary is documented and tested. |

## Addressed Items

### Orderbook Request Execution

`crates/orderbook/src/request.rs` uses shared internal helpers for request execution:

- `request_with` selects the response mode,
- `send_request` owns HTTP request construction,
- `request_headers` owns accept/content-type header construction,
- `execute_with` owns retry, status mapping, transport-error handling, and rate-limit acquisition.

Public request helpers remain API-compatible:

- `request_json`
- `request_text`
- `request_empty`
- `execute_json_with`
- `execute_text_with`
- `execute_empty_with`

Validation evidence:

- `crates/orderbook/tests/request_contract.rs::request_json_retries_429_and_preserves_headers_on_each_attempt`
- `crates/orderbook/tests/request_contract.rs::execute_json_with_retries_transient_statuses_until_success`
- `crates/orderbook/tests/request_contract.rs::execute_json_with_stops_on_non_retryable_api_error_and_preserves_body`
- `crates/orderbook/tests/request_contract.rs::request_text_and_empty_share_the_request_builder_and_success_path`
- `crates/orderbook/tests/request_contract.rs::rate_limiter_spaces_requests_after_token_budget_is_consumed`

### App-Data Upload Routing

`OrderBookApi::upload_app_data` routes through `fetch_json`, which uses the shared `request_json` path and the same request policy as other JSON orderbook endpoints.

### Signing Payload Preparation

Signing keeps separate sync and async entry points, while shared payload construction avoids repeated business logic:

- `crates/signing/src/order_signing.rs::order_signing_payload`
- `crates/signing/src/cancellation.rs::cancellation_signing_payload`

Validation evidence:

- `crates/signing/tests/order_signing_contract.rs::async_sign_order_paths_match_sync_signing_behavior`
- `crates/signing/tests/cancellation_contract.rs::async_cancellation_signing_paths_match_sync_variants`

### Trading Posting Wrappers

Trading posting keeps ergonomic public entry points while routing workflow logic through async implementation paths. Shared advanced-parameter extraction lives in:

- `crates/trading/src/post.rs::swap_additional_params`
- `crates/trading/src/post.rs::limit_additional_params`

Validation evidence:

- `crates/trading/tests/post_contract.rs::limit_posting_sync_signer_wrapper_matches_async_suffix_path`

## Open Items

Repeated order-like field sets exist across contract ABI types, orderbook DTOs, normalized order types, and user-domain order types. These should not be collapsed solely because the field names overlap.

The next audit revision should document the boundary for each repeated shape and add conversion evidence where useful. If a repeated type has no distinct boundary, behavior, or compatibility purpose, it should be removed or merged.

## Validation

Current validation commands:

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
