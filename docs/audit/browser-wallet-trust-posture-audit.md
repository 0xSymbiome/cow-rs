# Browser Wallet Trust Posture Audit

Status: Current
Last reviewed: 2026-05-14
Owning surface: `cow-sdk-browser-wallet` EIP-1193 provider construction and wallet chain-management URL payloads
Refresh trigger: Changes to EIP-1193 provider construction, EIP-6963 discovery metadata, wallet origin handling, chain-management URL validation, or browser-wallet error redaction
Related docs:
- [ADR 0007](../adr/0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md)
- [ADR 0024](../adr/0024-asyncprovider-asyncsigningprovider-capability-split.md)
- [ADR 0028](../adr/0028-account-abstraction-integration-plan.md)
- [Browser Wallet Chain Coherence Audit](browser-wallet-chain-coherence-audit.md)
- [URL Credential Redaction Audit](url-credential-redaction-audit.md)
- [Verification Matrix](../verification-matrix.md)

## Scope

This audit covers:

- `Eip1193ProviderBuilder` origin trust checks
- EIP-6963 discovery metadata flowing into browser-wallet construction
- explicit trust for anonymous EIP-1193 transports
- wallet chain-management URL payload boundaries

It does not cover vendor-specific browser extension behavior, live wallet UI
prompts, or RPC endpoint safety after a wallet accepts
`wallet_addEthereumChain`.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Provider trust | EIP-6963-discovered providers carry a detected origin into construction, while anonymous providers require `with_trusted_origin(...)` | Conforms |
| Origin schemes | Trusted origins accept the documented browser-wallet schemes and reject unsupported schemes before provider construction | Conforms |
| Error redaction | Untrusted provider-origin errors and trust telemetry use redacted origin values | Conforms |
| Wallet URL payloads | `rpc_urls`, `block_explorer_urls`, and `icon_urls` stay wallet payload data and are not governed by SDK service-host policy | Conforms |
| Regression depth | Provider-builder tests pin anonymous rejection, explicit trust, and session preservation | Conforms |

## Current Contract

### Provider Construction

`Eip1193ProviderBuilder` is the trust-aware construction path for typed
EIP-1193 providers. Providers selected from EIP-6963 discovery carry their
reverse-DNS identifier into the builder as detected origin metadata. A
transport without EIP-6963 metadata is anonymous and fails construction with
`BrowserWalletError::UntrustedProviderOrigin` unless the caller adds an
explicit reviewed origin through `with_trusted_origin(...)`.
Reviewed trusted origins accept reverse-DNS identifiers plus the documented
`http`, `https`, `test`, and `transport` schemes. Other schemes fail before a
provider can be constructed from the supplied origin.

### Telemetry And Errors

Provider-origin trust warnings use the `cow_sdk::trust` tracing target and
record redacted origin fields. Public error display and debug output do not
emit raw origin strings for untrusted anonymous providers.

### Wallet URL Payloads

`WalletChainParameters` still validates `wallet_addEthereumChain` URL strings
for basic `http` or `https` shape and stores them in redacting wrappers for
public debug and serialization. Those URLs are payloads sent to the selected
wallet and are not SDK-owned orderbook or subgraph service endpoints, so
`ExternalHostPolicy` is intentionally not applied to `rpc_urls`,
`block_explorer_urls`, or `icon_urls`.

## Evidence

Primary implementation points:

- `crates/browser-wallet/src/provider/builder.rs`
- `crates/browser-wallet/src/provider/origin.rs`
- `crates/browser-wallet/src/wallet/detect.rs`
- `crates/browser-wallet/src/wallet/mod.rs`
- `crates/browser-wallet/src/error.rs`

Primary regression coverage:

- `crates/browser-wallet/tests/provider_contract.rs::anonymous_provider_builder_requires_trusted_origin`
- `crates/browser-wallet/tests/provider_contract.rs::provider_builder_accepts_explicit_trusted_origin`
- `crates/browser-wallet/tests/provider_contract.rs::trusted_origin_accepts_documented_schemes_and_rejects_others`
- `crates/browser-wallet/tests/provider_contract.rs::wallet_add_chain_payload_urls_are_not_subject_to_external_host_policy`
- `crates/browser-wallet/tests/wallet_contract.rs::chain_parameters_public_debug_and_serialize_redact_url_credentials`
- `crates/browser-wallet/tests/origin_contract.rs`
- `crates/browser-wallet/tests/async_signing_provider_contract.rs`
- `crates/browser-wallet/tests/signer_contract.rs`

Validation surface:

```text
cargo test -p cow-sdk-browser-wallet --test provider_contract
cargo test -p cow-sdk-browser-wallet --test wallet_contract
cargo test --workspace --all-features
bun run --cwd e2e/browser-wallet test
```
