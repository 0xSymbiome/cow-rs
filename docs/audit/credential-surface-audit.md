# Credential Surface Audit

Status: Current
Last reviewed: 2026-05-12
Owning surface: Credential-bearing builder storage, URL configuration, host-policy errors, public error diagnostics, wallet add-chain payloads, Pinata upload-trait headers, wasm error envelopes, and the SDK facade
Refresh trigger: Changes to orderbook or subgraph builder API-key storage, URL-bearing public configuration fields, external host-policy validation, public error message/detail/body/data fields, browser wallet add-chain URL payload construction, `IpfsUploadTransport::post_json` header typing or Pinata header assembly, or any new credential-bearing surface that lands without a redacting storage type
Related docs:
- [ADR 0025](../adr/0025-workspace-url-redaction-convention.md)
- [URL Credential Redaction Audit](url-credential-redaction-audit.md)
- [Credential Surface Contract Hygiene Audit](credential-surface-contract-hygiene-audit.md)
- [Typestate Builder Contract Audit](typestate-builder-contract-audit.md)
- [Verification Matrix](../verification-matrix.md)
- [Alloy Umbrella Adapter Audit](alloy-umbrella-adapter-audit.md)
- [WASM Public API Stability Audit](wasm-public-api-stability-audit.md)

## Scope

This audit covers:

- `cow-sdk-orderbook::OrderBookApiBuilder` partner API-key storage
- `cow-sdk-subgraph::SubgraphApiBuilder` partner API-key storage
- credential-bearing URL fields in core, orderbook, subgraph, browser-wallet, and app-data
- sanitized host-policy failures for orderbook and subgraph endpoint overrides
- public error diagnostics that carry provider, signer, RPC, transport, response-body, orderbook-rejection, or caller-input message payloads
- `cow-sdk-app-data::IpfsUploadTransport::post_json` header typing and the Pinata header assembly path

It does not cover unrelated transport error redaction or credential handling outside these named boundaries.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Native Alloy adapters | Provider URLs, private-key material, signer internals, transport details, and pending-transaction details are redacted across the provider, signer, umbrella, and facade error tests | Conforms |
| Orderbook builder | `OrderBookApiBuilder` stores the partner API key as `Redacted<String>` so builder debug output cannot print the raw key | Conforms |
| Subgraph builder | `SubgraphApiBuilder` stores the partner API key as `Redacted<String>` so builder debug output cannot print the raw key | Conforms |
| URL configuration | Credential-bearing URL values use redacting storage types for debug, display, and serialization, and unwrap only at dispatch seams | Conforms |
| Host-policy errors | Orderbook and subgraph host-policy failures retain only a redacted host component and never serialize raw URL credentials, paths, queries, or fragments | Conforms |
| Public error diagnostics | Provider, signer, RPC, transport, response-body, subgraph context, orderbook API, orderbook rejection, and facade error payloads wrap secret-bearing messages in `Redacted<T>` or sanitize protocol identifiers before rendering, and redact credential-bearing diagnostics across `Debug`, `Display`, and existing `Serialize` surfaces | Conforms |
| Pinata upload trait | `IpfsUploadTransport::post_json` carries `Redacted<String>` header values and the Pinata header vector stays redacted under `Debug` | Conforms |
| WASM error envelope | `WasmError` maps transport, app-data, signing, orderbook, subgraph, and trading errors through display-safe messages and redacted response-body handling | Conforms |

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
access at the dispatch boundary. Orderbook and subgraph custom endpoint debug
output redacts userinfo-bearing URLs, and `IpfsConfig` display output follows
the same redaction rule.

### Host-Policy Failures

`crates/core/src/config/hosts.rs` owns `ExternalHostPolicy` and
`HostPolicyError`. Orderbook and subgraph builders validate explicit service
endpoint overrides against canonical hosts by default. Rejections retain only
the host component wrapped in `Redacted<String>`, while parse failures collapse
to a `UrlParseFailureClass` and unsupported schemes use sanitized static
labels. Error debug, display, and serialized output therefore cannot echo URL
credentials, paths, query strings, or fragments.

### Public Error Diagnostics

Public error variants that can carry provider, signer, RPC, transport,
response-body, orderbook-rejection, subgraph-context, browser-wallet, or
caller-input message payloads use `Redacted<String>`,
`Redacted<serde_json::Value>`, or `Redacted<ResponseBody>` for the
credential-bearing field. The wrapper keeps explicit inner access available
for callers that intentionally need the underlying diagnostic, while
`Debug`, `Display`, and existing `Serialize` implementations emit the shared
redaction marker. Typed diagnostics such as chain IDs, schema versions,
environment names, HTTP status codes, field names, validation classes, and
sanitized orderbook rejection tags remain visible so errors stay actionable.
The SDK facade regression test constructs every reviewed public error family
with URL, bearer-token, private-key-shaped, and PEM-shaped payloads and
verifies no secret substring appears in public renderings.

The wasm surface extends that contract to JavaScript. `WasmError` exposes
typed discriminants and low-cardinality fields while preserving redaction for
transport details, HTTP status response bodies, app-data transport detail,
wallet errors, and internal diagnostics. The mapping does not unwrap
`Redacted<T>` into a JS-visible field.

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
- `crates/core/src/config/hosts.rs`
- `crates/core/src/redaction.rs`
- `crates/core/src/errors.rs`
- `crates/core/src/transport/error.rs`
- `crates/subgraph/src/builder.rs`
- `crates/subgraph/src/api.rs`
- `crates/subgraph/src/error.rs`
- `crates/browser-wallet/src/wallet.rs`
- `crates/browser-wallet/src/error.rs`
- `crates/contracts/src/errors.rs`
- `crates/signing/src/errors.rs`
- `crates/trading/src/error.rs`
- `crates/orderbook/src/error.rs`
- `crates/orderbook/src/rejection.rs`
- `crates/orderbook/src/request.rs`
- `crates/app-data/src/types.rs`
- `crates/app-data/src/errors.rs`
- `crates/app-data/src/pinning.rs`
- `crates/sdk/src/lib.rs`
- `crates/wasm/src/exports/errors.rs`

Primary regression coverage:

- `crates/orderbook/tests/builder_contract.rs::builder_debug_redacts_partner_api_key`
- `crates/orderbook/tests/builder_contract.rs::builder_debug_redacts_userinfo_in_custom_base_url_overrides`
- `crates/core/tests/redaction_contract.rs`
- `crates/subgraph/tests/builder_contract.rs::builder_debug_redacts_partner_api_key`
- `crates/subgraph/tests/builder_contract.rs::builder_debug_redacts_userinfo_in_custom_endpoint_url`
- `crates/browser-wallet/tests/wallet_contract.rs::chain_parameters_public_debug_and_serialize_redact_url_credentials`
- `crates/app-data/tests/ipfs_config_redaction_contract.rs`
- `crates/app-data/tests/pinning_contract.rs::pinning_headers_debug_redacts_secret_bytes`
- `crates/app-data/tests/pinning_contract.rs::pinning_config_display_redacts_secret_bytes`
- `crates/sdk/tests/error_redaction_contract.rs`
- `crates/core/tests/config_contract.rs::external_host_policy_accepts_canonical_and_explicit_hosts_only`
- `crates/orderbook/tests/host_policy_contract.rs`
- `crates/subgraph/tests/host_policy_contract.rs`
- `crates/wasm/tests/wasm_redaction_contract.rs::transport_connect_error_uses_redacted_message`
- `crates/wasm/tests/wasm_redaction_contract.rs::http_status_error_redacts_headers_and_body`
- `crates/wasm/tests/wasm_redaction_contract.rs::errors_module_does_not_unwrap_redacted_values`

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
cargo test -p cow-sdk --test error_redaction_contract
cargo test -p cow-sdk --test error_redaction_contract --all-features
cargo test --workspace --all-features
```
