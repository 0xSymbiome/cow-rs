# URL Credential Redaction Audit

Status: Current
Last reviewed: 2026-05-31
Owning surface: Credential-bearing URL storage and dispatch boundaries across core, orderbook, subgraph, browser-wallet, app-data, and wasm error conversion
Refresh trigger: Changes to URL-bearing public configuration fields, browser wallet add-chain URL payload construction, IPFS URI dispatch, wasm transport-error mapping, the `RedactedUrlMap` and `RedactedOptionalUrlMap` contracts, or the `redact_response_body` token-detection layers
Related docs:
- [ADR 0025](../adr/0025-workspace-url-redaction-convention.md)
- [Credential Surface Audit](credential-surface-audit.md)
- [Verification Matrix](../verification-matrix.md)
- [Alloy Umbrella Adapter Audit](alloy-umbrella-adapter-audit.md)
- [WASM Public API Stability Audit](wasm-public-api-stability-audit.md)

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
| WASM transport errors | `From<TransportError> for WasmError` uses display-safe transport messages and redacted response bodies before crossing the JavaScript ABI | Conforms |
| Response-body scanner detection | JWT and Bearer scheme detectors run before the URL detector; bare userinfo, strict URL, and credential-keyed value detectors each cover a distinct evasion shape; the credential-key matcher uses substring matching for `apikey`, `token`, `secret`, `password`, `authorization`, and `bearer` and recursively scans key prefixes for embedded credentials | Conforms |

## Current Contract

### Core And Orderbook Base URLs

`cow-sdk-core::ApiBaseUrls` is a `RedactedUrlMap<u64>`.
`ApiContext`, `ApiContextOverride`, and `OrderbookApiBuilder` store explicit
base-URL maps in that type. Public formatting and JSON output retain the chain
id keys and emit `[redacted]` for every URL value. Base-URL resolution reads the
raw map through `as_inner()` immediately before routing.
`OrderbookApiBuilder` debug output follows the same rule for custom
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

### WASM Error Envelope

`crates/wasm/src/exports/errors.rs` maps `TransportError` into the
JavaScript-visible `WasmError` envelope through `Display` and
`cow_sdk_core::redact_response_body`. It does not call `Redacted::into_inner`
for JS-visible detail, so URL credentials and secret-shaped response snippets
remain redacted across `Debug`, `Display`, and serialized error output.

### Response-Body Scanner Detection Layers

`cow_sdk_core::redact_response_body` runs a single-pass byte-offset scanner
over an arbitrary response-body string and replaces every credential-shaped
span with the sanitized placeholder. The detection layers run in the
documented order so a more specific pattern never gets reclassified as a
more general one:

1. JWT-shaped tokens (`eyJ` prefix followed by at least 23 credential-value
   characters) are matched first so an opaque JSON Web Token surrounded by
   URL syntax cannot get re-interpreted as a URL scheme prefix and ship
   verbatim ahead of userinfo redaction.
2. `Bearer <token>` schemes are matched anywhere in the input, with no
   word-boundary constraint, so a partner response that echoes
   `someBearer secret-...` or repeats the keyword inside a freeform key
   still has its trailing token redacted.
3. Strict URLs (`scheme://userinfo@host`) are matched when the scheme is a
   contiguous IANA-shaped identifier (alphanumeric plus `+`, `-`, or `.`).
   The userinfo span gets the sanitized placeholder while the scheme and
   host bytes are retained for diagnostics.
4. Bare userinfo (`://user:pass@host` with no preceding scheme word) is
   matched as a separate pass so a mangled or non-ASCII scheme prefix that
   defeats the strict URL detector still ships with the userinfo stripped.
5. Credential-keyed values (`key=value` and `key:value`) match when the
   normalized key name contains `apikey`, `token`, `secret`, `password`,
   `authorization`, or `bearer`, or matches the canonical name exactly.
   The key prefix is recursively redacted before the value is replaced,
   so a credential key carrying an embedded JWT or URL userinfo also
   sheds its inner credential material rather than copying through.

## Evidence

Primary implementation points:

- `crates/core/src/redaction/wrappers.rs`
- `crates/core/src/redaction/body.rs`
- `crates/core/src/config/hosts.rs`
- `crates/orderbook/src/builder.rs`
- `crates/orderbook/src/types/mod.rs`
- `crates/subgraph/src/api.rs`
- `crates/subgraph/src/builder.rs`
- `crates/browser-wallet/src/wallet/chain.rs`
- `crates/app-data/src/types/ipfs.rs`
- `crates/app-data/src/fetch.rs`
- `crates/app-data/src/pinning.rs`
- `crates/wasm/src/exports/errors.rs`

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
- `crates/wasm/tests/wasm_redaction_contract.rs::http_status_error_redacts_headers_and_body`
- `crates/wasm/tests/wasm_redaction_contract.rs::display_format_of_redacted_transport_error_does_not_expose_secret`
- `crates/wasm/tests/wasm_redaction_contract.rs::errors_module_does_not_unwrap_redacted_values`

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
