# Credential Redaction Audit

Status: Current
Last reviewed: 2026-06-21
Owning surface: Cross-cutting credential redaction across config/builder storage, URL-bearing config, transport/RPC/orderbook/subgraph error diagnostics, native Alloy adapters, and wasm error envelopes
Refresh trigger: Changes to orderbook or subgraph builder API-key storage, URL-bearing public configuration fields, external host-policy validation, the `Redacted<T>` newtype or the `RedactedUrlMap`/`RedactedOptionalUrlMap` contracts, subgraph production routing or its `Authorization` header, public error message/detail/body/data fields, any `SubgraphError` variant or its `#[error(...)]` template, the transport `From<reqwest::Error>` classifiers, the `redact_response_body` token-detection layers, native Alloy adapter `Debug`/redaction state, the wasm transport-error mapping, the JSON decode-failure or digest-calculation diagnostic shapes, the app-data typed validation render, or any new credential-bearing surface that lands without a redacting storage type or an equivalent safe-by-construction render
Related docs:
- [ADR 0025](../adr/0025-workspace-url-redaction-convention.md)
- [Verification Matrix](../verification.md)

## Scope

This audit covers:

- `OrderbookApiBuilder` and `SubgraphApiBuilder` partner API-key storage
- credential-bearing URL fields in core, orderbook, subgraph, and app-data, plus their dispatch seams
- subgraph production routing and its `Authorization: Bearer` header
- sanitized host-policy failures for orderbook and subgraph endpoint overrides
- the `Redacted<T>` newtype contract and the shared URL-map wrappers
- public error diagnostics carrying provider, signer, RPC, transport, response-body, orderbook-rejection, subgraph-context, or caller-input payloads
- the `Display` rendering of every diagnostic `SubgraphError` variant and its redacted-route/plaintext-diagnostic pairing rule
- native Alloy provider, signer, umbrella, and facade adapter error output
- the `redact_response_body` token-detection pipeline
- app-data typed validation output and the JSON decode-failure and digest-calculation diagnostics
- the `WasmError` JavaScript error envelope

It does not cover unrelated transport-policy questions, non-URL credentials outside the named `Redacted<T>` storage contract, the typed parsing of the GraphQL response envelope (wire-DTO coverage audit), or future capability crates outside the active SDK surface.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| `Redacted<T>` secret wrapper | Generic newtype whose `Debug`, `Display`, and `Serialize` emit `[redacted]`, with an explicit `into_inner`/`as_inner` escape; secret-bearing config fields carry it at the type level | Conforms |
| Orderbook builder | `OrderbookApiBuilder` stores the partner API key as `Redacted<String>` so builder debug output cannot print the raw key | Conforms |
| Subgraph builder | `SubgraphApiBuilder` stores the partner Graph API key as `Redacted<String>` so builder debug output cannot print the raw key | Conforms |
| Subgraph route identity | Graph API credentials stay out of stable metadata and typed failure context; failure context is sanitized to a public origin or generic override marker | Conforms |
| Core and orderbook base URLs | `ApiBaseUrls` (`RedactedUrlMap<u64>`) values, including userinfo-bearing custom overrides, redact in diagnostics and serialization while routing reads raw URLs via `as_inner()` | Conforms |
| Subgraph base URLs | `SubgraphApiBaseUrls` (`RedactedOptionalUrlMap`) configured URLs serialize as `[redacted]`, unsupported chains as `null`; production routing carries the key in the `Authorization` header so the route map and request path are key-free | Conforms |
| App-data IPFS URIs | `IpfsConfig` `uri`/`read_uri` redact in debug and serialization; fetch/upload policies unwrap only at the dispatch seam | Conforms |
| Host-policy failures | Orderbook and subgraph overrides fail closed against canonical hosts; `HostPolicyError` retains only a redacted host or sanitized failure class, never raw URL credentials, paths, queries, or fragments | Conforms |
| Native Alloy adapters | Provider URLs, private-key material, signer internals, transport details, and pending-transaction details are redacted across provider, signer, umbrella, and facade error tests | Conforms |
| Transport error redaction | `From<reqwest::Error>` on the orderbook surface classifies via the canonical core reqwest classifier (which strips the URL) before wrapping; the subgraph surface receives an already-redacted `TransportError` from the transport seam | Conforms |
| Public error diagnostics | Provider, signer, RPC, transport, response-body, subgraph-context, orderbook-API, and orderbook-rejection payloads wrap secrets in `Redacted<T>` or render through a safe-by-construction pipeline; typed diagnostics (chain ids, status codes, field names, rejection tags) stay visible | Conforms |
| COW Shed helper errors | `CowShedError::OwnerResolution`/`Signing` capture the signer-authored message behind `Redacted<String>`, so a custom `Signer` cannot leak credential-bearing text through `Display` or `Debug` | Conforms |
| Subgraph `Display` pairing | Every chain-scoped `SubgraphError` variant pairs redacted `context.api` with plaintext `chain_id` plus a typed structural token; `TransportConfiguration` pairs the typed `class` label and carries no chain id | Conforms |
| Subgraph `Display` non-tautology | Every chain-scoped diagnostic variant carries at least one ASCII-digit token, and `TransportConfiguration` carries its typed `class` label, so renderings never collapse to a placeholder-only string | Conforms |
| Subgraph `Redacted<T>` posture | No `Display` template interpolates `.as_inner()`, including the free-form `errors[].message` payload on `GraphQl` | Conforms |
| App-data validation output | Typed `AppDataError` `Display` names only the offending public field and the canonical `ValidationReason`, never the caller value; `InvalidSchemaVersion` wraps the rejected string in `Redacted<String>` | Conforms |
| JSON decode and digest-calculation diagnostics | Orderbook, app-data, and contracts JSON failures surface only the serde category and 1-based line/column; the sub-metadata deserializer maps malformed values to field-tagged messages; `AppDataError::Calculation` surfaces only a stable label | Conforms |
| Trading partner-fee boundary | Partner-fee inputs stay typed until explicit app-data translation; invalid raw metadata is rejected before transport | Conforms |
| WASM error envelope | `WasmError` maps transport, app-data, signing, orderbook, subgraph, and trading errors through display-safe messages and redacted response bodies without unwrapping `Redacted<T>` | Conforms |
| Response-body credential scanner | `redact_response_body` runs an ordered defense-in-depth detector pipeline (JWT, Bearer, strict URL, bare userinfo, credential-keyed value with recursive key-prefix scanning) | Conforms |

## Current Contract

### `Redacted<T>` Wrapper And Builder/Config Storage

`crates/core/src/redaction/wrappers.rs` owns `Redacted<T>`, a generic newtype
whose `Debug`, `Display`, and `Serialize` implementations emit the literal
`[redacted]` placeholder; consumers reach the secret through an explicit
`into_inner`/`as_inner` escape. `OrderbookApiBuilder`
(`crates/orderbook/src/builder.rs`) stores the partner API key as
`Option<Redacted<String>>`, wrapping in the `.api_key(...)` setter so `Debug`
on a partially configured builder emits the marker instead of the secret.
`SubgraphApiBuilder` (`crates/subgraph/src/builder.rs`) is typestate-checked:
an unconfigured builder holds the `ApiKeyUnset` marker (no secret stored), and
the `.api_key(...)` setter advances it to `ApiKeySet(Redacted<String>)`, so a
configured builder's `Debug` emits the marker instead of the secret.
`ApiContext`, `ApiContextOverride`, `IpfsConfig`, and the internal subgraph API
key slot carry `Redacted<String>` at the type level, so accidental logging,
default serialization, and ad-hoc diagnostics cannot print the secret while
explicit input is preserved.

### URL Redaction Map And Value Wrappers

`cow-sdk-core::ApiBaseUrls` is a `RedactedUrlMap<u64>`; `ApiContext`,
`ApiContextOverride`, and `OrderbookApiBuilder` store base-URL maps in it.
Public formatting and JSON output retain the chain-id keys and emit
`[redacted]` for every URL value; resolution reads the raw map through
`as_inner()` immediately before routing, and builder debug follows the same
rule for userinfo-bearing custom overrides.

`cow-sdk-subgraph::SubgraphApiBaseUrls` is a
`RedactedOptionalUrlMap<SupportedChainId>`: a configured URL serializes as
`[redacted]`, an unsupported chain as `null`. Production routing does not embed
the partner key in the gateway URL — the key is sent in the request
`Authorization: Bearer` header against the key-free gateway URL
(`https://gateway.thegraph.com/api/subgraphs/id/<id>`), so `prod_config`, the
dispatched path, and the `transport.dispatch` span endpoint are key-free by
construction. A `base_urls` override dispatches its URL verbatim with no
SDK-injected auth header.

`IpfsConfig` (`crates/app-data/src/types/ipfs.rs`) stores `uri` and `read_uri`
as `Option<Redacted<String>>`, so `Debug` and serialization emit the marker;
`IpfsFetchPolicy::from_config` unwraps only at the read dispatch seam.

`crates/core/src/config/hosts.rs` owns `ExternalHostPolicy` and
`HostPolicyError`. Builders validate explicit endpoint overrides against
canonical hosts by default; local fixtures and private mirrors require explicit
opt-in. Rejections retain only the host wrapped in `Redacted<String>`, parse
failures collapse to a `UrlParseFailureClass`, and unsupported schemes use
sanitized static labels, so error output cannot echo URL credentials, paths,
queries, or fragments.

### Per-Error-Family Redaction

Public error variants carrying provider, signer, RPC, transport, response-body,
orderbook-rejection, subgraph-context, or caller-input payloads use
`Redacted<String>`, `Redacted<serde_json::Value>`, or
`Redacted<ResponseBody>` for the credential-bearing field, while `Debug`,
`Display`, and existing `Serialize` emit the shared marker. Typed diagnostics
(chain ids, schema versions, environment names, HTTP status codes, field names,
validation classes, sanitized rejection tags) stay visible; the orderbook `Api`
fallback surfaces the HTTP status while its free-form body stays redacted. The
SDK facade regression test constructs every reviewed family with URL,
bearer-token, private-key-shaped, and PEM-shaped payloads and verifies no
secret substring appears in public renderings. `From<reqwest::Error>` on the
orderbook surface delegates to the canonical core classifier
(`cow_sdk_core::transport::classify_reqwest_error`), which calls `without_url`
and tags the typed class via the upstream `is_timeout`, `is_connect`,
`is_redirect`, `is_decode`, `is_body`, `is_builder`, `is_status`, and
`is_request` set before wrapping, adding a layer below the config-level
`Redacted<T>` storage. The subgraph surface never sees a raw `reqwest::Error`:
its transport seam hands it an already-redacted `TransportError`.

The orderbook, app-data, and contracts JSON decode failures each carry a
structured `{ category, line, column }` triple rather than the raw
`serde_json::Error`, capturing only the serde category (`syntax`, `data`,
`eof`, `io`) and the 1-based position, so a malformed body never reaches
`Display`, `Debug`, or `Serialize`. `AppDataParams` lifts caller-supplied
`metadata.signer`, `metadata.flashloan`, and `metadata.hooks` through scoped
deserializers that map malformed values to fixed field-tagged messages naming
only the public wire key. `AppDataError::Calculation` renders only the stable
`appDataHex calculation failed` label, keeping its typed source behind the
`#[source]` chain. App-data validation
([ADR 0064](../adr/0064-app-data-typed-validation.md)) surfaces typed
`AppDataError` values whose `Display` names only the offending public wire
field and the canonical `ValidationReason`; `InvalidSchemaVersion` wraps the
rejected version string in `Redacted<String>`. `cow-sdk-trading` keeps
partner-fee inputs typed until explicit app-data translation, rejecting invalid
raw metadata before quote or post transport proceeds. The COW Shed signing
helper (`crates/contracts/src/cow_shed/`) captures the owner-resolution and
`ExecuteHooks` signing messages from an arbitrary consumer `Signer` behind
`Redacted<String>`, matching `ContractsError::Eip1271Provider`, so a custom
signer's error text cannot reach `Display` or `Debug` unredacted.

#### Subgraph `Display` Non-Tautology

Every diagnostic `SubgraphError` variant carrying a
`SubgraphRequestErrorContext` interpolates **both** the redacted `context.api`
and the plaintext `context.chain_id`. `Transport` adds the typed
`TransportErrorClass` label; `HttpStatus` adds the numeric status; the
`Serialization` variant adds `body.as_inner().len()`; `GraphQl` adds
`errors.len()` and, when present, the first error's first source location as
`at line:column` via `first_graphql_location_suffix` (GraphQL document
positions that cannot carry credential content). `TransportConfiguration`
carries no context — returned before any request is assembled — so it pairs the
typed `class` label as its plaintext diagnostic, rendering as
`subgraph transport configuration error (<class>): <placeholder>`. The
`subgraph_display_carries_plaintext_structural_diagnostic` sweep asserts every
chain-scoped variant carries at least one ASCII digit, forbidding a regression
that collapses the rendering to a tautological `for [redacted]` shape. No
`Display` template invokes `.as_inner()` on any redacted field, including the
free-form `errors[].message` payload on `GraphQl`; consumers reach
upstream-authored GraphQL text only through explicit typed access on the
carried `errors` vector, which the `GraphQl` rustdoc documents as a doctest.

### Native Alloy Adapter Opaque Debug

The native Alloy provider, signer, umbrella, and facade adapters store
configured RPC URLs behind redacting state and use hand-written opaque `Debug`
so provider URLs, private-key material, signer internals, transport details,
and pending-transaction details never print. The adapter error types
(`ProviderError`, `SignerError`, `AlloyClientError`) hold their `Validation` and
`Internal` detail behind `Redacted<String>`, so the `thiserror`-derived `Display`
renders the placeholder from the type rather than from a hand-written literal;
each error keeps its hand-written `Debug` so the credential firewall does not
depend on a derive. The wasm surface extends the
contract to JavaScript: `WasmError` (`crates/wasm/src/exports/errors.rs`)
exposes typed discriminants and low-cardinality fields, maps `TransportError`
through `Display` and `cow_sdk_core::redact_response_body`, and never exposes a
`Redacted<T>` secret to JS without re-redacting it: the response body is read
via `as_inner()` and immediately passed back through `redact_response_body`, so
URL credentials and secret-shaped response snippets stay redacted across
`Debug`, `Display`, and serialized output. The guard test bans `into_inner`
escapes in the module source.

### Response-Body Scanner Detection Layers

`cow_sdk_core::redact_response_body` runs a single-pass byte-offset scanner
that replaces every credential-shaped span with the sanitized placeholder. The
layers run in a fixed order so a more specific pattern is never reclassified as
a more general one:

1. JWT-shaped tokens (a run of at least 23 credential-value characters
   beginning with the `eyJ` prefix) are matched first so an opaque JWT
   surrounded by URL syntax cannot ship verbatim ahead of userinfo redaction.
2. `Bearer <token>` schemes match anywhere with no word-boundary constraint, so
   `someBearer secret-...` still has its trailing token redacted.
3. Strict URLs (`scheme://userinfo@host`, IANA-shaped scheme) redact the
   userinfo span while retaining scheme and host.
4. Bare userinfo (`://user:pass@host` with no preceding scheme word) is a
   separate pass so a mangled or non-ASCII scheme prefix still strips userinfo.
5. Credential-keyed values (`key=value`, `key:value`) match when the normalized
   key contains `apikey`, `token`, `secret`, `password`, `authorization`, or
   `bearer`, or matches the canonical name exactly; the key prefix is
   recursively redacted before the value, so a key carrying an embedded JWT or
   URL userinfo also sheds its inner credential.

## Evidence

Primary implementation points:

- `crates/core/src/redaction/wrappers.rs`
- `crates/core/src/redaction/body.rs`
- `crates/core/src/config/hosts.rs`
- `crates/core/src/errors.rs`
- `crates/core/src/transport/error.rs`
- `crates/orderbook/src/builder.rs`
- `crates/orderbook/src/types/mod.rs`
- `crates/orderbook/src/error.rs`
- `crates/orderbook/src/rejection.rs`
- `crates/orderbook/src/request.rs`
- `crates/subgraph/src/api.rs`
- `crates/subgraph/src/builder.rs`
- `crates/subgraph/src/error.rs` (`SubgraphError` enum, per-variant `#[error(...)]` templates, `first_graphql_location_suffix`)
- `crates/contracts/src/errors.rs`
- `crates/signing/src/errors.rs`
- `crates/trading/src/error.rs`
- `crates/trading/src/types/params.rs`
- `crates/trading/src/quote.rs`
- `crates/trading/src/app_data.rs`
- `crates/trading/src/post.rs`
- `crates/trading/src/slippage.rs`
- `crates/app-data/src/types/ipfs.rs`
- `crates/app-data/src/types/params.rs`
- `crates/app-data/src/errors.rs`
- `crates/app-data/src/fetch.rs`
- `crates/sdk/src/lib.rs`
- `crates/wasm/src/exports/errors.rs`

Primary regression coverage:

- `crates/core/tests/redaction_contract.rs`
- `crates/core/tests/config_contract.rs::external_host_policy_accepts_canonical_and_explicit_hosts_only`
- `crates/orderbook/tests/builder_contract.rs::builder_debug_redacts_partner_api_key`
- `crates/orderbook/tests/builder_contract.rs::builder_debug_redacts_base_url_credentials`
- `crates/orderbook/tests/builder_contract.rs::builder_debug_redacts_userinfo_in_custom_base_url_overrides`
- `crates/orderbook/tests/api_contract.rs::api_debug_redacts_context_base_url_credentials`
- `crates/orderbook/tests/types_contract.rs::quote_request_supports_buy_side_and_context_overrides`
- `crates/orderbook/tests/host_policy_contract.rs`
- `crates/subgraph/tests/builder_contract.rs::builder_debug_redacts_partner_api_key`
- `crates/subgraph/tests/builder_contract.rs::builder_debug_redacts_base_url_credentials`
- `crates/subgraph/tests/builder_contract.rs::builder_debug_redacts_userinfo_in_custom_endpoint_url`
- `crates/subgraph/tests/api_contract.rs::config_debug_and_serialize_redact_custom_base_url_credentials`
- `crates/subgraph/tests/api_contract.rs::recording_transport::production_routing_carries_the_key_in_the_authorization_header_not_the_url`
- `crates/subgraph/tests/host_policy_contract.rs`
- `crates/subgraph/tests/error_contract.rs::graphql_display_includes_chain_id`
- `crates/subgraph/tests/error_contract.rs::graphql_display_includes_first_location_when_present`
- `crates/subgraph/tests/error_contract.rs::graphql_display_omits_location_when_absent`
- `crates/subgraph/tests/error_contract.rs::graphql_display_does_not_leak_message_content`
- `crates/subgraph/tests/error_contract.rs::serialization_display_includes_body_byte_count`
- `crates/subgraph/tests/error_contract.rs::httpstatus_display_includes_chain_id_and_status_code`
- `crates/subgraph/tests/error_contract.rs::transport_variant_carries_typed_class_and_sanitized_detail`
- `crates/app-data/tests/ipfs_config_redaction_contract.rs`
- `crates/app-data/tests/schema_contract.rs::non_semver_version_is_rejected_without_leaking_the_value`
- `crates/sdk/tests/error_redaction_contract.rs`
- `crates/sdk/tests/error_redaction_contract.rs::orderbook_serialization_error_drops_decoded_response_bytes`
- `crates/sdk/tests/error_redaction_contract.rs::app_data_and_contracts_serialization_errors_drop_decoded_bytes`
- `crates/sdk/tests/error_redaction_contract.rs::app_data_metadata_parse_failures_do_not_echo_caller_input`
- `crates/sdk/tests/error_redaction_contract.rs::app_data_calculation_error_does_not_render_boxed_source`
- `crates/sdk/tests/error_redaction_contract.rs::subgraph_errors_and_contexts_redact_serialized_request_payloads`
- `crates/sdk/tests/error_redaction_contract.rs::subgraph_display_carries_plaintext_structural_diagnostic`
- `crates/sdk/tests/public_api.rs`
- `crates/wasm/tests/wasm_redaction_contract.rs::transport_connect_error_uses_redacted_message`
- `crates/wasm/tests/wasm_redaction_contract.rs::http_status_error_redacts_headers_and_body`
- `crates/wasm/tests/wasm_redaction_contract.rs::display_format_of_redacted_transport_error_does_not_expose_secret`
- `crates/wasm/tests/wasm_redaction_contract.rs::errors_module_does_not_unwrap_redacted_values`

Validation surface:

```text
cargo fmt --all --check
cargo test -p cow-sdk-core --test redaction_contract
cargo test -p cow-sdk-core --test config_contract
cargo test -p cow-sdk-core
cargo test -p cow-sdk-app-data --test ipfs_config_redaction_contract
cargo test -p cow-sdk-app-data
cargo test -p cow-sdk-orderbook --test builder_contract
cargo test -p cow-sdk-orderbook --test api_contract
cargo test -p cow-sdk-orderbook --test host_policy_contract
cargo test -p cow-sdk-orderbook
cargo test -p cow-sdk-subgraph --test builder_contract
cargo test -p cow-sdk-subgraph --test api_contract
cargo test -p cow-sdk-subgraph --test host_policy_contract
cargo test -p cow-sdk-subgraph --test error_contract
cargo test -p cow-sdk-trading
cargo test -p cow-sdk --test error_redaction_contract
cargo test -p cow-sdk --test error_redaction_contract --all-features
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --target wasm32-unknown-unknown -p cow-sdk
cargo build --target wasm32-unknown-unknown -p cow-sdk-app-data
```
