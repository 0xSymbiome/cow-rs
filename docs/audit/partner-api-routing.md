# Partner API Routing Audit

Status: Current  
Last reviewed: 2026-04-15

## Scope

This audit covers:

- partner route selection in `ApiContext`
- local validation of partner API key input before orderbook request assembly
- `X-API-Key` header construction in `cow-sdk-orderbook`

It does not cover unrelated transport retry policy, subgraph gateway routing,
or broader credential-redaction questions already covered elsewhere.

## Findings Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Partner route selection | Partner endpoint families activate only when the configured API key is locally header-valid | Conforms |
| Header assembly | `X-API-Key` request headers are built from locally validated input instead of silently dropping invalid values | Conforms |
| Failure mode | Invalid partner API keys fail locally before route resolution or request transport proceeds | Conforms |

## Findings

### Partner route selection

`ApiContext` now validates the configured partner API key before deciding
whether partner endpoint families are active. This prevents partner routing
from being selected on the basis of an unusable value.

### Header assembly

`cow-sdk-orderbook` now derives the `X-API-Key` header from the same validated
input used by route selection. The client therefore no longer has a state where
it targets partner infrastructure while silently omitting the required header.

### Failure mode

Invalid partner API keys now fail as a local validation error before request
transport begins. This keeps the problem at the configuration boundary instead
of converting it into a remote authorization failure with ambiguous cause.

## Evidence

Primary implementation points:

- `crates/core/src/config.rs`
- `crates/orderbook/src/api.rs`

Primary regression coverage:

- `crates/core/tests/config_contract.rs`
- `crates/orderbook/tests/api_contract.rs`

Validation surface:

```text
cargo fmt --all --check
cargo test -p cow-sdk-core
cargo test -p cow-sdk-orderbook
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-features --target wasm32-unknown-unknown
```
