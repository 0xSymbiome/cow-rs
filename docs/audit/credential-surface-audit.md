# Credential Surface Audit

Status: Current
Last reviewed: 2026-04-27
Owning surface: Credential-bearing builder storage, URL configuration, host-policy errors, wallet add-chain payloads, and Pinata upload-trait headers across orderbook, subgraph, browser-wallet, core, and app-data
Refresh trigger: Changes to orderbook or subgraph builder API-key storage, URL-bearing public configuration fields, external host-policy validation, browser wallet add-chain URL payload construction, `IpfsUploadTransport::post_json` header typing or Pinata header assembly, or any new credential-bearing surface that lands without a redacting storage type
Related docs:
- [ADR 0025](../adr/0025-workspace-url-redaction-convention.md)
- [URL Credential Redaction Audit](url-credential-redaction-audit.md)
- [Credential Surface Contract Hygiene Audit](credential-surface-contract-hygiene-audit.md)
- [Typestate Builder Contract Audit](typestate-builder-contract-audit.md)
- [Verification Matrix](../verification-matrix.md)

## Scope

This audit covers:

- `cow-sdk-orderbook::OrderBookApiBuilder` partner API-key storage
- `cow-sdk-subgraph::SubgraphApiBuilder` partner API-key storage
- credential-bearing URL fields in core, orderbook, subgraph, browser-wallet, and app-data
- sanitized host-policy failures for orderbook and subgraph endpoint overrides
- `cow-sdk-app-data::IpfsUploadTransport::post_json` header typing and the Pinata header assembly path

It does not cover unrelated transport error redaction or credential handling outside these named boundaries.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Orderbook builder | `OrderBookApiBuilder` stores the partner API key as `Redacted<String>` so builder debug output cannot print the raw key | Conforms |
| Subgraph builder | `SubgraphApiBuilder` stores the partner API key as `Redacted<String>` so builder debug output cannot print the raw key | Conforms |
| URL configuration | Credential-bearing URL values use redacting storage types and unwrap only at dispatch seams | Conforms |
| Host-policy errors | Orderbook and subgraph host-policy failures retain only a redacted host component and never serialize raw URL credentials, paths, queries, or fragments | Conforms |
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

### URL Configuration

`crates/core/src/redaction.rs` owns the shared URL-map redaction types.
`ApiContext`, `ApiContextOverride`, `SubgraphConfig`,
`SubgraphApiBuilder`, `WalletChainParameters`, and `IpfsConfig` store
credential-bearing URL values in redacting wrappers. Public debug and
serialized output emits `[redacted]` for configured URL values while routing,
wallet payload construction, and IPFS read/write policies use explicit raw
access at the dispatch boundary.

### Host-Policy Failures

`crates/core/src/config.rs` owns `ExternalHostPolicy` and
`HostPolicyError`. Orderbook and subgraph builders validate explicit service
endpoint overrides against canonical hosts by default. Rejections retain only
the host component wrapped in `Redacted<String>`, while parse failures collapse
to a `UrlParseFailureClass` and unsupported schemes use sanitized static
labels. Error debug, display, and serialized output therefore cannot echo URL
credentials, paths, query strings, or fragments.

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
- `crates/core/src/config.rs`
- `crates/core/src/redaction.rs`
- `crates/subgraph/src/builder.rs`
- `crates/subgraph/src/api.rs`
- `crates/browser-wallet/src/wallet.rs`
- `crates/app-data/src/types.rs`
- `crates/app-data/src/pinning.rs`

Primary regression coverage:

- `crates/orderbook/tests/builder_contract.rs::builder_debug_redacts_partner_api_key`
- `crates/core/tests/redaction_contract.rs`
- `crates/subgraph/tests/builder_contract.rs::builder_debug_redacts_partner_api_key`
- `crates/browser-wallet/tests/wallet_contract.rs::chain_parameters_public_debug_and_serialize_redact_url_credentials`
- `crates/app-data/tests/ipfs_config_redaction_contract.rs`
- `crates/app-data/tests/pinning_contract.rs::pinning_headers_debug_redacts_secret_bytes`
- `crates/core/tests/config_contract.rs::external_host_policy_accepts_canonical_and_explicit_hosts_only`
- `crates/orderbook/tests/host_policy_contract.rs`
- `crates/subgraph/tests/host_policy_contract.rs`

Validation surface:

```text
cargo test -p cow-sdk-core --test redaction_contract
cargo test -p cow-sdk-orderbook --test builder_contract
cargo test -p cow-sdk-orderbook --test api_contract
cargo test -p cow-sdk-subgraph --test builder_contract
cargo test -p cow-sdk-subgraph --test api_contract
cargo test -p cow-sdk-core --test config_contract
cargo test -p cow-sdk-orderbook --test host_policy_contract
cargo test -p cow-sdk-subgraph --test host_policy_contract
cargo test -p cow-sdk-browser-wallet --test wallet_contract
cargo test -p cow-sdk-app-data --test ipfs_config_redaction_contract
cargo test -p cow-sdk-app-data --test pinning_contract
cargo test --workspace --all-features
```
