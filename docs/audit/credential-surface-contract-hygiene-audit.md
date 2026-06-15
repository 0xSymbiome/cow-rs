# Credential Surface Contract Hygiene Audit

Status: Current
Last reviewed: 2026-06-01
Owning surface: Cross-cutting credential redaction and typed partner-fee public boundary across core, app-data, orderbook, subgraph, and trading
Refresh trigger: Changes to public credential-bearing configs, URL-bearing public configuration fields, subgraph route identity or request-failure context, the `Redacted<T>` newtype contract, external host-policy validation, the transport `From<reqwest::Error>` conversion classifiers, the `redact_response_body` token-detection layers, or typed partner-fee request boundaries
Related docs:
- [ADR 0005](../adr/0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0006](../adr/0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md)
- [ADR 0010](../adr/0010-runtime-neutral-async-and-transport-posture.md)
- [ADR 0025](../adr/0025-workspace-url-redaction-convention.md)
- [Credential Surface Audit](credential-surface-audit.md)
- [URL Credential Redaction Audit](url-credential-redaction-audit.md)
- [Architecture](../architecture.md)
- [Verification Guide](../verification.md)
- [Verification Matrix](../verification.md)
- [Alloy Umbrella Adapter Audit](alloy-umbrella-adapter-audit.md)

## Scope

This audit covers:

- stable subgraph route identity and typed request-failure context
- default diagnostic and serialized behavior for credential-bearing config
  structs in `cow-sdk-core`, `cow-sdk-orderbook`, and `cow-sdk-app-data`
- URL-bearing public configuration fields and host-policy failures that could
  otherwise echo private endpoints or credentials
- user-facing partner-fee policy on the `cow-sdk-trading` request surface

It does not cover browser-wallet session management, unrelated transport-policy
questions, or future capability crates that are still outside the active SDK
surface.

The native Alloy adapter family is now inside the active SDK surface and is
covered by the same redaction and facade error probes.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Subgraph route identity | Keep Graph API credentials out of stable metadata and typed failure context | Conforms |
| Credential-bearing config diagnostics | Redact secret material in default `Debug`, `Display`, and serialized forms while preserving explicit inputs | Conforms |
| URL-bearing configuration | Store configured endpoint URLs in redacting wrappers and unwrap only at dispatch seams | Conforms |
| Host-policy failures | Fail closed on non-canonical orderbook and subgraph hosts without echoing raw URL credentials | Conforms |
| `Redacted<T>` secret wrapper | Type-level redaction in `Debug`, `Display`, and `Serialize` with an explicit `into_inner` escape | Conforms |
| Transport error redaction | `From<reqwest::Error>` on orderbook and subgraph classifies via the upstream kind checkers and strips the URL before wrapping | Conforms |
| Trading partner-fee policy | Keep user-facing partner-fee inputs typed until explicit app-data translation | Conforms |

## Current Contract

### Subgraph Route Identity

`cow-sdk-subgraph` keeps the Graph API key private to request routing. The
stable production route map is redacted, and typed request failures expose only
non-secret route identity. Custom override routes remain explicit, but the
public failure context is sanitized to a public origin or a generic override
marker instead of echoing a credential-bearing URL.

### Credential-Bearing Config Diagnostics

`ApiContext`, `ApiContextOverride`, and `IpfsConfig` continue to accept
explicit credential input, but their default `Debug` and serialized forms now
redact secret material. This keeps routine diagnostics and generic
serialization from turning partner API keys into ordinary log output.
`IpfsConfig` read-URI display output follows the same redaction rule.

### URL-Bearing Configuration

`ApiContext`, `ApiContextOverride`, `OrderbookApiBuilder`,
`SubgraphApiBuilder`, `WalletChainParameters`, and `IpfsConfig` store
credential-bearing URLs in redacting wrappers. Map keys and unsupported-chain
markers remain reviewable, while configured endpoint bytes serialize and format
as `[redacted]`. Raw URL access stays confined to orderbook, subgraph,
wallet-chain, and IPFS dispatch seams. Custom orderbook and subgraph endpoint
debug output redacts userinfo-bearing URLs before they cross a diagnostic
boundary.

### Host-Policy Failures

Orderbook and subgraph builders validate explicit endpoint overrides through
`ExternalHostPolicy`. Default policy accepts canonical service hosts, local
fixtures and private mirrors require explicit opt-in, and typed
`HostPolicyError` values retain sanitized host or failure-class data instead
of raw URLs.

### `Redacted<T>` Secret Wrapper

`cow-sdk-core` exposes `Redacted<T>`, a generic newtype whose `Debug`,
`Display`, and `Serialize` implementations emit the literal `[redacted]`
placeholder. Consumers reach the wrapped secret through an explicit
`into_inner` escape. Secret-bearing configuration fields (`ApiContext`,
`ApiContextOverride`, `IpfsConfig`, and the internal subgraph API key
slot) carry `Redacted<String>` at the type level so accidental logging,
default serialization, and ad-hoc diagnostics cannot print the secret.

### Transport Error Redaction

`From<reqwest::Error>` on the orderbook and subgraph error surfaces
classifies failures through the upstream `is_timeout`, `is_connect`,
`is_decode`, `is_body`, `is_redirect`, `is_builder`, `is_request`, and
`is_status` set and calls `reqwest::Error::without_url` before wrapping.
Partner routes and their query-string credentials cannot leak through
the wrapped `Display` output, adding a second layer of defense below the
config-level `Redacted<T>` migration.

### Typed Partner-Fee Boundary

`cow-sdk-trading` accepts typed partner-fee policy values on its public request
types. Raw JSON remains confined to explicit app-data metadata translation
boundaries, and invalid raw metadata is rejected before quote or posting
transport proceeds.

## Evidence

Primary implementation points:

- `crates/core/src/config/hosts.rs`
- `crates/core/src/redaction/wrappers.rs`
- `crates/app-data/src/types/ipfs.rs`
- `crates/orderbook/src/error.rs`
- `crates/orderbook/src/types/mod.rs`
- `crates/orderbook/src/builder.rs`
- `crates/subgraph/src/api.rs`
- `crates/subgraph/src/builder.rs`
- `crates/subgraph/src/error.rs`
- `crates/browser-wallet/src/wallet/chain.rs`
- `crates/trading/src/types/options.rs`
- `crates/trading/src/quote.rs`
- `crates/trading/src/app_data.rs`
- `crates/trading/src/post/generic.rs`
- `crates/trading/src/slippage/policy.rs`

Primary regression coverage:

- `crates/core/tests/config_contract.rs`
- `crates/core/tests/redaction_contract.rs`
- `crates/orderbook/tests/types_contract.rs`
- `crates/orderbook/tests/builder_contract.rs`
- `crates/orderbook/tests/builder_contract.rs::builder_debug_redacts_userinfo_in_custom_base_url_overrides`
- `crates/orderbook/tests/host_policy_contract.rs`
- `crates/subgraph/tests/api_contract.rs`
- `crates/subgraph/tests/builder_contract.rs`
- `crates/subgraph/tests/builder_contract.rs::builder_debug_redacts_userinfo_in_custom_endpoint_url`
- `crates/subgraph/tests/host_policy_contract.rs`
- `crates/browser-wallet/tests/wallet_contract.rs`
- `crates/app-data/tests/ipfs_config_redaction_contract.rs`
- `crates/trading/tests/quote_contract.rs`
- `crates/trading/tests/post_contract.rs`
- `crates/trading/tests/property_contract.rs`
- `crates/trading/tests/slippage_contract.rs`
- `crates/sdk/tests/public_api.rs`

Validation surface:

```text
cargo fmt --all --check
cargo test -p cow-sdk-core --test redaction_contract
cargo test -p cow-sdk-core --test config_contract
cargo test -p cow-sdk-core
cargo test -p cow-sdk-app-data
cargo test -p cow-sdk-orderbook
cargo test -p cow-sdk-subgraph
cargo test -p cow-sdk-browser-wallet
cargo test -p cow-sdk-trading
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --target wasm32-unknown-unknown -p cow-sdk
cargo build --target wasm32-unknown-unknown -p cow-sdk-app-data
```
