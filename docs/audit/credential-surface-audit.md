# Credential Surface Audit

Status: Current
Last reviewed: 2026-04-23
Owning surface: Credential-bearing builder storage and Pinata upload-trait headers across orderbook, subgraph, and app-data
Refresh trigger: Changes to orderbook or subgraph builder API-key storage, changes to `IpfsUploadTransport::post_json` header typing or Pinata header assembly, or any new credential-bearing surface that lands without `Redacted<String>` wrapping
Related docs:
- [Credential Surface Contract Hygiene Audit](credential-surface-contract-hygiene-audit.md)
- [Typestate Builder Contract Audit](typestate-builder-contract-audit.md)
- [Verification Matrix](../verification-matrix.md)

## Scope

This audit covers:

- `cow-sdk-orderbook::OrderBookApiBuilder` partner API-key storage
- `cow-sdk-subgraph::SubgraphApiBuilder` partner API-key storage
- `cow-sdk-app-data::IpfsUploadTransport::post_json` header typing and the Pinata header assembly path

It does not cover unrelated config redaction, transport error redaction, or credential handling outside these three named boundaries.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Orderbook builder | `OrderBookApiBuilder` stores the partner API key as `Redacted<String>` so builder debug output cannot print the raw key | Conforms |
| Subgraph builder | `SubgraphApiBuilder` stores the partner API key as `Redacted<String>` so builder debug output cannot print the raw key | Conforms |
| Pinata upload trait | `IpfsUploadTransport::post_json` carries `Redacted<String>` header values and the Pinata header vector stays redacted under `Debug` | Conforms |

## Current Contract

### Orderbook Builder

`crates/orderbook/src/builder.rs` stores the optional partner API key as
`Option<Redacted<String>>`. The fluent `.api_key(...)` setter wraps the
incoming value before it is retained on the builder, so `Debug` on a partially
configured builder emits the workspace redaction marker instead of the secret.

### Subgraph Builder

`crates/subgraph/src/builder.rs` stores the required partner Graph API key as
`Option<Redacted<String>>`. The `.api_key(...)` setter wraps the input before
storage, so `Debug` on the typestate builder preserves the current redaction
contract while keeping the key available for deliberate downstream use.

### Pinata Upload Boundary

`crates/app-data/src/pinning.rs` widens
`IpfsUploadTransport::post_json` to
`headers: &[(String, Redacted<String>)]`. The Pinata upload helper constructs
the header vector with wrapped values, so any transport implementation that
needs the raw bytes must opt in to unwrap them and the boundary's default
`Debug` surface cannot print the secret bytes.

## Evidence

Primary implementation points:

- `crates/orderbook/src/builder.rs`
- `crates/subgraph/src/builder.rs`
- `crates/app-data/src/pinning.rs`

Primary regression coverage:

- `crates/orderbook/tests/builder_contract.rs::builder_debug_redacts_partner_api_key`
- `crates/subgraph/tests/builder_contract.rs::builder_debug_redacts_partner_api_key`
- `crates/app-data/tests/pinning_contract.rs::pinning_headers_debug_redacts_secret_bytes`

Validation surface:

```text
cargo test -p cow-sdk-orderbook --test builder_contract
cargo test -p cow-sdk-subgraph --test builder_contract
cargo test -p cow-sdk-app-data --test pinning_contract
cargo test --workspace --all-features
```
