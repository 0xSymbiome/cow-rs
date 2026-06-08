# Credential Surface Audit

Status: Current
Last reviewed: 2026-06-08
Owning surface: Credential-bearing builder storage, URL configuration, host-policy errors, public error diagnostics, wallet add-chain payloads, wasm error envelopes, and the SDK facade
Refresh trigger: Changes to orderbook or subgraph builder API-key storage, URL-bearing public configuration fields, external host-policy validation, public error message/detail/body/data fields, browser wallet add-chain URL payload construction, the `redact_response_body` token-detection layers, the `cow_sdk_app_data` typed metadata validation and the matching `ValidationResult::errors` rendering, the `cow_sdk_orderbook::OrderbookError::Serialization`, `cow_sdk_app_data::AppDataError::Json`, or `cow_sdk_contracts::ContractsError::Serialization` structural-diagnostic shape or their `From<serde_json::Error>` construction, the `cow_sdk_app_data::AppDataError::Calculation` render, the `cow_sdk_app_data::AppDataParams` sub-metadata deserializer, or any new credential-bearing surface that lands without a redacting storage type or an equivalent safe-by-construction render
Related docs:
- [ADR 0025](../adr/0025-workspace-url-redaction-convention.md)
- [URL Credential Redaction Audit](url-credential-redaction-audit.md)
- [Credential Surface Contract Hygiene Audit](credential-surface-contract-hygiene-audit.md)
- [Typestate Builder Contract Audit](typestate-builder-contract-audit.md)
- [Verification Matrix](../verification.md)
- [Alloy Umbrella Adapter Audit](alloy-umbrella-adapter-audit.md)
- [WASM Public API Stability Audit](wasm-public-api-stability-audit.md)

## Scope

This audit covers:

- `cow-sdk-orderbook::OrderbookApiBuilder` partner API-key storage
- `cow-sdk-subgraph::SubgraphApiBuilder` partner API-key storage
- credential-bearing URL fields in core, orderbook, subgraph, browser-wallet, and app-data
- sanitized host-policy failures for orderbook and subgraph endpoint overrides
- public error diagnostics that carry provider, signer, RPC, transport, response-body, orderbook-rejection, or caller-input message payloads

It does not cover unrelated transport error redaction or credential handling outside these named boundaries.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Native Alloy adapters | Provider URLs, private-key material, signer internals, transport details, and pending-transaction details are redacted across the provider, signer, umbrella, and facade error tests | Conforms |
| Orderbook builder | `OrderbookApiBuilder` stores the partner API key as `Redacted<String>` so builder debug output cannot print the raw key | Conforms |
| Subgraph builder | `SubgraphApiBuilder` stores the partner API key as `Redacted<String>` so builder debug output cannot print the raw key | Conforms |
| URL configuration | Credential-bearing URL values use redacting storage types for debug, display, and serialization, and unwrap only at dispatch seams | Conforms |
| Host-policy errors | Orderbook and subgraph host-policy failures retain only a redacted host component and never serialize raw URL credentials, paths, queries, or fragments | Conforms |
| Public error diagnostics | Provider, signer, RPC, transport, response-body, subgraph context, orderbook API, orderbook rejection, and facade error payloads wrap secret-bearing messages in `Redacted<T>`, render through a safe-by-construction sanitization pipeline, or sanitize protocol identifiers before rendering (the orderbook `Api` fallback surfaces the HTTP status, and browser-wallet errors surface the EIP-1193 RPC method name, while their free-form bodies and wallet messages stay redacted), and redact credential-bearing diagnostics across `Debug`, `Display`, and existing `Serialize` surfaces | Conforms |
| App-data validation output | App-data validation surfaces failures as typed `AppDataError` values whose `Display` and the matching `ValidationResult::errors` field name only the offending public field and the canonical `ValidationReason`, never the caller-supplied value, so they are safe to interpolate into `Display`, `Debug`, and `Serialize` without a `Redacted<T>` wrapper | Conforms |
| JSON decode-failure and digest-calculation diagnostics | The orderbook, app-data, and contracts JSON decode failures each surface only the serde failure category and the 1-based line/column position, the app-data document sub-metadata deserializer maps malformed caller values to fixed field-tagged messages, and `AppDataError::Calculation` surfaces only a stable label, so none of these paths renders the raw serde error or boxed source that could echo decoded or caller-supplied bytes | Conforms |
| WASM error envelope | `WasmError` maps transport, app-data, signing, orderbook, subgraph, and trading errors through display-safe messages and redacted response-body handling | Conforms |
| Response-body credential scanner | `redact_response_body` enforces a defense-in-depth detector pipeline (JWT, Bearer, strict URL, bare userinfo, credential-keyed value with recursive key-prefix scanning) and the credential-key matcher recognizes `apikey`, `token`, `secret`, `password`, `authorization`, and `bearer` substrings so a partial or mangled credential key does not bypass redaction | Conforms |

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

`crates/core/src/redaction/wrappers.rs` owns the shared URL-map redaction types.
`ApiContext`, `ApiContextOverride`, `SubgraphConfig`,
`SubgraphApiBuilder`, `WalletChainParameters`, and `IpfsConfig` store
credential-bearing URL values in redacting wrappers. Public debug and
serialized output emits `[redacted]` for configured URL values while routing,
wallet payload construction, and IPFS read policies use explicit raw access at
the dispatch boundary. Orderbook and subgraph custom endpoint debug output
redacts userinfo-bearing URLs, and `IpfsConfig` display output follows the same
redaction rule.

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

### JSON Decode-Failure And Digest-Calculation Diagnostics

`cow_sdk_orderbook::OrderbookError::Serialization`,
`cow_sdk_app_data::AppDataError::Json`, and
`cow_sdk_contracts::ContractsError::Serialization` each carry a structured
`{ category, line, column }` triple rather than the raw `serde_json::Error`.
Their `From<serde_json::Error>` conversions capture only the serde failure
category (`syntax`, `data`, `eof`, or `io`) and the 1-based line and column
position, so a malformed or unexpected decoded body — an unknown field under
`deny_unknown_fields`, or a type-mismatched value — can never reach the error's
`Display` or `Debug` surface, nor the app-data variant's `Serialize` output.
These decode paths therefore stay free of upstream-authored content while
keeping an actionable structural diagnostic.

`cow_sdk_app_data::AppDataParams` lifts caller-supplied `metadata.signer`,
`metadata.flashloan`, and `metadata.hooks` values out of an app-data document
through scoped deserializers. A malformed value maps to a fixed, field-tagged
message that names only the public wire key, so the offending caller key or
value is never folded into the surfaced error.

`cow_sdk_app_data::AppDataError::Calculation` renders only the stable
`appDataHex calculation failed` label through `Display` and `Serialize`; the
boxed typed source stays behind the `#[source]` chain for callers that
deliberately cross the redaction boundary. The label-only render keeps the
digest-calculation surface safe even if a future hashing or CID backend embeds
caller-derived bytes in its own message.

### App-Data Validation Output

`cow_sdk_app_data` validates documents through typed construction
([ADR 0064](../adr/0064-app-data-typed-validation.md)), not a JSON-Schema
validator. A failed validation surfaces a typed `AppDataError` —
`InvalidAppDataProvided`, `InvalidPartnerFee`, `InvalidFlashloanHints`,
`InvalidSchemaVersion`, or `MissingSchemaVersion` — whose `Display` names only
the offending public wire field and the canonical `ValidationReason`, never the
caller-supplied value. `InvalidSchemaVersion` wraps the rejected version string
in `Redacted<String>` so even the version token stays masked. A
present-but-malformed `metadata.flashloan` or `metadata.partnerFee` is rejected
with a fixed, family-named message rather than the raw serde error, matching the
sub-metadata deserializer's redaction pattern. `ValidationResult::errors` is an
`Option<String>` carrying that same typed rendering. A regression test at
`crates/app-data/tests/schema_contract.rs::non_semver_version_is_rejected_without_leaking_the_value`
pins that a non-semver version value is rejected without echoing the value.

The wasm surface extends that contract to JavaScript. `WasmError` exposes
typed discriminants and low-cardinality fields while preserving redaction for
transport details, HTTP status response bodies, app-data transport detail,
wallet errors, and internal diagnostics. The mapping does not unwrap
`Redacted<T>` into a JS-visible field.

## Evidence

Primary implementation points:

- `crates/orderbook/src/builder.rs`
- `crates/core/src/config/hosts.rs`
- `crates/core/src/redaction/wrappers.rs`
- `crates/core/src/errors.rs`
- `crates/core/src/transport/error.rs`
- `crates/subgraph/src/builder.rs`
- `crates/subgraph/src/api.rs`
- `crates/subgraph/src/error.rs`
- `crates/browser-wallet/src/wallet/chain.rs`
- `crates/browser-wallet/src/error.rs`
- `crates/contracts/src/errors.rs`
- `crates/signing/src/errors.rs`
- `crates/trading/src/error.rs`
- `crates/orderbook/src/error.rs`
- `crates/orderbook/src/rejection.rs`
- `crates/orderbook/src/request.rs`
- `crates/app-data/src/types/ipfs.rs`
- `crates/app-data/src/errors.rs`
- `crates/app-data/src/types/params.rs`
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
- `crates/sdk/tests/error_redaction_contract.rs`
- `crates/sdk/tests/error_redaction_contract.rs::orderbook_serialization_error_drops_decoded_response_bytes`
- `crates/sdk/tests/error_redaction_contract.rs::app_data_and_contracts_serialization_errors_drop_decoded_bytes`
- `crates/sdk/tests/error_redaction_contract.rs::app_data_metadata_parse_failures_do_not_echo_caller_input`
- `crates/sdk/tests/error_redaction_contract.rs::app_data_calculation_error_does_not_render_boxed_source`
- `crates/app-data/tests/schema_contract.rs::non_semver_version_is_rejected_without_leaking_the_value`
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
cargo test -p cow-sdk --test error_redaction_contract
cargo test -p cow-sdk --test error_redaction_contract --all-features
cargo test --workspace --all-features
```
