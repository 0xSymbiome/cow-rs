# URL Credential Redaction Audit

Status: Current
Last reviewed: 2026-05-06
Owning surface: Credential-bearing URL storage and dispatch boundaries across core, orderbook, subgraph, browser-wallet, and app-data
Refresh trigger: Changes to URL-bearing public configuration fields, browser wallet add-chain URL payload construction, IPFS URI dispatch, or the `RedactedUrlMap` and `RedactedOptionalUrlMap` contracts
Related docs:
- [ADR 0025](../adr/0025-workspace-url-redaction-convention.md)
- [Credential Surface Audit](credential-surface-audit.md)
- [Verification Matrix](../verification-matrix.md)
- [Alloy Umbrella Adapter Audit](alloy-umbrella-adapter-audit.md)

## Scope

This audit covers:

- orderbook and core API base-URL maps
- subgraph custom and production base-URL maps
- browser wallet `wallet_addEthereumChain` URL parameters
- app-data IPFS read and write URI configuration

It does not cover non-URL credentials such as API-key header values, except
where they share the same `Redacted<T>` storage contract.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Core and orderbook base URLs | Configured base-URL map values, including userinfo-bearing custom overrides, redact in public diagnostics and serialization while routing keeps raw URLs | Conforms |
| Subgraph base URLs | Optional base-URL map values, including userinfo-bearing custom endpoints, redact and unsupported-chain `None` markers remain visible | Conforms |
| Browser wallet add-chain URLs | Public chain parameters redact URL vectors while the EIP-1193 payload uses raw URL bytes outside SDK service-host policy | Conforms |
| Native Alloy URLs | Provider and umbrella builders store configured RPC URLs behind redacting state and debug output never prints credentials or query secrets | Conforms |
| App-data IPFS URIs | IPFS URI config fields redact in public debug, display, and serialization while fetch and upload policies use raw URI bytes | Conforms |

## Current Contract

### Core And Orderbook Base URLs

`cow-sdk-core::ApiBaseUrls` is a `RedactedUrlMap<u64>`.
`ApiContext`, `ApiContextOverride`, and `OrderBookApiBuilder` store explicit
base-URL maps in that type. Public formatting and JSON output retain the chain
id keys and emit `[redacted]` for every URL value. Base-URL resolution reads the
raw map through `as_inner()` immediately before routing.
`OrderBookApiBuilder` debug output follows the same rule for custom
userinfo-bearing base-URL overrides.

### Subgraph Base URLs

`cow-sdk-subgraph::SubgraphApiBaseUrls` is a
`RedactedOptionalUrlMap<SupportedChainId>`. A configured URL serializes as
`[redacted]`; an unsupported chain serializes as `null`. Query dispatch and
public error-context sanitization read the raw map through `as_inner()` at the
subgraph routing boundary.
`SubgraphApiBuilder` debug output follows the same rule for custom
userinfo-bearing endpoint overrides.

### Browser Wallet Add-Chain URLs

`WalletChainParameters` stores `rpc_urls`, `block_explorer_urls`, and
`icon_urls` as `Vec<Redacted<String>>`. Public `Debug` and JSON serialization
therefore redact configured URL values. The wallet request path builds a
crate-local payload from explicit raw borrows so `wallet_addEthereumChain`
still receives the exact URL strings required by EIP-3085. These URLs are
wallet payload data and are not routed through `ExternalHostPolicy`, which
only governs SDK service endpoints.

### App-Data IPFS URIs

`IpfsConfig` stores `uri`, `write_uri`, and `read_uri` as
`Option<Redacted<String>>`. Public diagnostics and JSON serialization redact
the configured URI values. `IpfsFetchPolicy::from_config` and
`pin_json_in_pinata_ipfs` unwrap only at the read/write dispatch seams.
`Display` follows the same redaction contract as `Debug`.

## Evidence

Primary implementation points:

- `crates/core/src/redaction.rs`
- `crates/core/src/config.rs`
- `crates/orderbook/src/builder.rs`
- `crates/orderbook/src/types.rs`
- `crates/subgraph/src/api.rs`
- `crates/subgraph/src/builder.rs`
- `crates/browser-wallet/src/wallet.rs`
- `crates/app-data/src/types.rs`
- `crates/app-data/src/fetch.rs`
- `crates/app-data/src/pinning.rs`

Primary regression coverage:

- `crates/core/tests/redaction_contract.rs`
- `crates/orderbook/tests/builder_contract.rs::builder_debug_redacts_base_url_credentials`
- `crates/orderbook/tests/builder_contract.rs::builder_debug_redacts_userinfo_in_custom_base_url_overrides`
- `crates/orderbook/tests/api_contract.rs::api_debug_redacts_context_base_url_credentials`
- `crates/orderbook/tests/types_contract.rs::quote_request_supports_buy_side_and_context_overrides`
- `crates/subgraph/tests/builder_contract.rs::builder_debug_redacts_base_url_credentials`
- `crates/subgraph/tests/builder_contract.rs::builder_debug_redacts_userinfo_in_custom_endpoint_url`
- `crates/subgraph/tests/api_contract.rs::config_debug_and_serialize_redact_custom_base_url_credentials`
- `crates/browser-wallet/tests/provider_contract.rs::wallet_add_chain_payload_urls_are_not_subject_to_external_host_policy`
- `crates/browser-wallet/tests/wallet_contract.rs::chain_parameters_public_debug_and_serialize_redact_url_credentials`
- `crates/app-data/tests/ipfs_config_redaction_contract.rs`
- `crates/app-data/tests/pinning_contract.rs::pinning_config_display_redacts_secret_bytes`

Validation surface:

```text
cargo fmt --all --check
cargo test -p cow-sdk-core --test redaction_contract
cargo test -p cow-sdk-orderbook --test builder_contract
cargo test -p cow-sdk-orderbook --test api_contract
cargo test -p cow-sdk-subgraph --test builder_contract
cargo test -p cow-sdk-browser-wallet --test wallet_contract
cargo test -p cow-sdk-app-data --test ipfs_config_redaction_contract
cargo test --workspace --all-features
```
