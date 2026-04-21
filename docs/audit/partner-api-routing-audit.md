# Partner API Routing Audit

Status: Current  
Last reviewed: 2026-04-21  
Owning surface: `cow-sdk-core` route selection and `cow-sdk-orderbook` partner header assembly  
Refresh trigger: Changes to `ApiContext` partner-route selection, API-key validation, `X-API-Key` header construction, or partner endpoint family activation  
Related docs:
- [ADR 0006](../adr/0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md)
- [Verification Guide](../verification-guide.md)
- [Verification Matrix](../verification-matrix.md)

## Scope

This audit covers:

- partner route selection in `ApiContext`
- local validation of partner API key input before orderbook request assembly
- `X-API-Key` header construction in `cow-sdk-orderbook`

It does not cover unrelated transport retry policy, subgraph gateway routing,
or broader credential-redaction questions already covered elsewhere.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Partner route selection | Partner endpoint families activate only when the configured API key is locally header-valid | Conforms |
| Header assembly | `X-API-Key` request headers are built from locally validated input instead of silently dropping invalid values | Conforms |
| Failure mode | Invalid partner API keys fail locally before route resolution or request transport proceeds | Conforms |

## Current Contract

### Partner Route Selection

`ApiContext` validates the configured partner API key before deciding whether
partner endpoint families are active. This prevents partner routing from being
selected on the basis of an unusable value.

### Header Assembly

`cow-sdk-orderbook` derives the `X-API-Key` header from the same validated
input used by route selection. The client no longer has a state where it
targets partner infrastructure while silently omitting the required header.

### Failure Mode

Invalid partner API keys fail as a local validation error before request
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
