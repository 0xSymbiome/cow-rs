# Credential Surface Contract Hygiene Audit

Status: Current  
Last reviewed: 2026-04-15

## Scope

This audit covers:

- stable subgraph route identity and typed request-failure context
- default diagnostic and serialized behavior for credential-bearing config
  structs in `cow-sdk-core`, `cow-sdk-orderbook`, and `cow-sdk-app-data`
- user-facing partner-fee policy on the `cow-sdk-trading` request surface

It does not cover browser-wallet session management, unrelated transport-policy
questions, or future capability crates that are still outside the active SDK
surface.

## Findings Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Subgraph route identity | Keep Graph API credentials out of stable metadata and typed failure context | Conforms |
| Credential-bearing config diagnostics | Redact secret material in default `Debug` and serialized forms while preserving explicit inputs | Conforms |
| Trading partner-fee policy | Keep user-facing partner-fee inputs typed until explicit app-data translation | Conforms |

## Findings

### Subgraph route identity

`cow-sdk-subgraph` keeps the Graph API key private to request routing. The
stable production route map is now redacted, and typed request failures expose
only non-secret route identity. Custom override routes remain explicit, but the
public failure context is sanitized to a public origin or a generic override
marker instead of echoing a credential-bearing URL.

### Credential-bearing config diagnostics

`ApiContext`, `ApiContextOverride`, and `IpfsConfig` continue to accept explicit
credential input, but their default `Debug` and serialized forms now redact
secret material. This keeps routine diagnostics and generic serialization from
turning partner API keys or Pinata credentials into ordinary log output.

### Typed partner-fee boundary

`cow-sdk-trading` now accepts typed partner-fee policy values on its public
request types. Raw JSON remains confined to explicit app-data metadata
translation boundaries, and invalid raw metadata is rejected before quote or
posting transport proceeds.

## Evidence

Primary implementation points:

- `crates/core/src/config.rs`
- `crates/app-data/src/types.rs`
- `crates/orderbook/src/types.rs`
- `crates/subgraph/src/api.rs`
- `crates/subgraph/src/error.rs`
- `crates/trading/src/types.rs`
- `crates/trading/src/quote.rs`
- `crates/trading/src/post.rs`
- `crates/trading/src/slippage.rs`

Primary regression coverage:

- `crates/core/tests/config_contract.rs`
- `crates/orderbook/tests/types_contract.rs`
- `crates/subgraph/tests/api_contract.rs`
- `crates/trading/tests/quote_contract.rs`
- `crates/trading/tests/post_contract.rs`
- `crates/trading/tests/property_contract.rs`
- `crates/trading/tests/slippage_contract.rs`
- `crates/sdk/tests/public_api.rs`

Validation surface:

```text
cargo fmt --all --check
cargo test -p cow-sdk-core
cargo test -p cow-sdk-app-data
cargo test -p cow-sdk-orderbook
cargo test -p cow-sdk-subgraph
cargo test -p cow-sdk-trading
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --target wasm32-unknown-unknown -p cow-sdk
cargo build --target wasm32-unknown-unknown -p cow-sdk-app-data
python tools/validate_trace_links.py --root .
python tools/validate_task_packs.py --root .
python tools/generate_drift_report.py --root . --output ./.spec-graph/drift-report.md
```
