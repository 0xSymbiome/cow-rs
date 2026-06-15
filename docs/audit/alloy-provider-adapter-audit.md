# Alloy Provider Adapter Audit

Status: Current
Last reviewed: 2026-06-15
Owning surface: `cow-sdk-alloy-provider` `RpcAlloyProvider`, its builder, and its `Provider` implementation
Refresh trigger: ADR 0038 - rich receipt population, or changes to the provider public API, the `Provider` trait, transport classification, the `read_contract` algorithm, the opt-in `retry` seam or its `RetryConfig`, the inter-crate seam entries consumed by sibling Alloy adapters, the workspace Alloy runtime pin, or the crate dependency boundary
Related docs:
- [ADR 0035](../adr/0035-alloy-provider-adapter.md)
- [ADR 0038](../adr/0038-transaction-lifecycle-types.md)
- [ADR 0024](../adr/0024-asyncprovider-asyncsigningprovider-capability-split.md)
- [ADR 0025](../adr/0025-workspace-url-redaction-convention.md)
- [ADR 0026](../adr/0026-alloy-major-release-absorption-plan.md)
- [URL Credential Redaction Audit](url-credential-redaction-audit.md)
- [Typestate Builder Contract Audit](typestate-builder-contract-audit.md)

## Scope

This audit covers:

- the `RpcAlloyProvider` public type and its `Provider` implementation
- the `RpcAlloyProviderBuilder` HTTP typestate builder and builder error type
- the opt-in `retry` seam and its `RetryConfig`
- the `ProviderError` and `ProviderErrorClass` surfaces
- conversion between `cow-sdk-core` domain types and Alloy RPC values
- the `read_contract` ABI encode, dispatch, decode, and JSON result path
- the doc-hidden helper seam used by sibling Alloy adapter crates,
  including the `execute_read_contract` entry that the umbrella adapter
  consumes for its own `Provider::read_contract` implementation
- dependency boundaries for the read-only provider crate

It does not cover upstream Alloy internals, signer or wallet support, WS or IPC
transport support, browser-wallet behavior, or transaction submission.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Public API exposure | Documented provider and builder methods expose SDK-owned domain types; upstream Alloy values remain internal apart from the doc-hidden sibling seam, which is not semver-guaranteed consumer API | Conforms |
| Trait coverage | `RpcAlloyProvider` implements all six `Provider` methods from `cow-sdk-core` | Conforms |
| Negative capability boundary | Compile-fail tests assert the provider is not a `SigningProvider` or `Signer` | Conforms |
| Builder typestate | `build()` is callable only on the HTTP-selected builder state; transport state stores the URL through `Redacted<reqwest::Url>` | Conforms |
| RPC retry seam | Retry is off by default (one request per call); `retry(RetryConfig)` wraps the JSON-RPC client in `alloy`'s bounded backoff layer and transparently retries a transient rate-limited request | Conforms |
| Error classification | `ProviderError::class()` covers validation, transport, remote, cancelled, and internal failures | Conforms |
| Credential redaction | Invalid URL errors carry no input detail, provider debug output redacts the transport, and transport details use `Redacted<String>` | Conforms |
| `read_contract` | The adapter loads the ABI, resolves a single function, parses JSON arguments, ABI-encodes, dispatches `eth_call`, decodes the response, serializes supported JSON value strings, and rejects unsupported decoded shapes as validation errors | Conforms |
| Dependency boundary | The crate declares no direct signer-family dependency and the resolved normal graph excludes `alloy-signer-local`; upstream Alloy's internal `alloy-signer` dependency does not enable local signing | Conforms |

## Current Contract

### Public Surface

`cow-sdk-alloy-provider` exposes `RpcAlloyProvider`,
`RpcAlloyProviderBuilder`, sealed transport-state marker names,
`RpcAlloyProviderBuilderError`, `RetryConfig`, `ProviderError`, and
`ProviderErrorClass`. The provider stores the upstream `DynProvider` in
private state and keeps raw transport labels out of debug output.

The `__seam` module is `#[doc(hidden)]` and exists only for sibling `cow-rs`
Alloy adapter crates. Its conversion, classification, and read-contract
functions (including `execute_read_contract`) are not part of the
documented consumer API and are not semver-guaranteed for downstream
consumers.

### Provider Methods

The adapter implements `get_chain_id`, `get_code`,
`get_transaction_receipt`, `call`, `read_contract`, and `get_block`. Each RPC
method converts caller-owned SDK types to Alloy values before dispatch and
converts the result back to `cow-sdk-core` types before returning.

`get_transaction_receipt` converts Alloy receipts into the rich
`TransactionReceipt` shape. Status comes from
`receipt.inner.status_or_post_state().as_eip658()`, so a pre-Byzantium
post-state receipt surfaces `status: None` rather than coerced success.
Block number, block hash, gas used, sender, and recipient are populated when
the Alloy receipt carries them; contract creation leaves `to` unset.

### RPC Retry Seam

Retry is opt-in. The default builder path issues each request once and surfaces
a transient transport failure — such as a public-endpoint `429` — directly to
the caller, preserving the runtime-neutral default. Passing a `RetryConfig`
(maximum retry count and initial backoff) through `retry` wraps the
JSON-RPC client in `alloy`'s rate-limit backoff layer, which transparently
retries a rate-limited request up to the configured attempt count. Only the
SDK-owned `RetryConfig` is public; the underlying transport layer and its
internal compute-units budget stay private, and the umbrella `AlloyClient`
reuses the same layer constructor through the doc-hidden seam so the policy is
defined once. Per ADR 0035's amendment, the existing REST `TransportPolicy` is
not reused here because its retry signal is keyed on HTTP status codes that
JSON-RPC errors do not cleanly surface.

### `read_contract`

`read_contract` is the ABI boundary. It parses `ContractCall::abi_json` with
`alloy-json-abi`, rejects overloaded function names, parses `args_json` as JSON,
maps argument JSON into `DynSolValue`, encodes calldata with
`alloy-dyn-abi`, dispatches `eth_call`, decodes return data, and serializes the
decoded value to the JSON string required by `cow-sdk-core`.

Malformed ABI, malformed JSON arguments, malformed addresses, bad integer
values, unsupported ABI types, and decode failures return typed errors instead
of panicking.

### Error And Cancellation

`ProviderError` has validation, transport, remote, cancelled, and internal
classes. Transport errors carry a shared `TransportErrorClass` plus redacted
detail. Remote JSON-RPC errors keep their code and message because those fields
are the peer's structured protocol response.

`From<cow_sdk_core::Cancelled>` allows consumer code using
`Cancellable::cancel_with(...).await?` to propagate cancellation through the
provider error type.

### Dependency Boundary

The crate depends on Alloy runtime provider crates and direct `reqwest` with
rustls TLS. It does not declare direct `alloy-signer` or
`alloy-signer-local` dependencies, and tests assert the resolved provider graph
does not include the local private-key signer crate.

## Evidence

Primary implementation points:

- `crates/alloy-provider/src/lib.rs`
- `crates/alloy-provider/src/provider.rs`
- `crates/alloy-provider/src/builder.rs`
- `crates/alloy-provider/src/client.rs`
- `crates/alloy-provider/src/retry.rs`
- `crates/alloy-provider/src/error.rs`
- `crates/alloy-provider/src/conversion.rs`
- `crates/alloy-provider/src/read_contract.rs`
- `crates/alloy-provider/Cargo.toml`

Primary regression coverage:

- `crates/alloy-provider/tests/provider_contract.rs`
- `crates/alloy-provider/tests/builder_contract.rs`
- `crates/alloy-provider/tests/retry_contract.rs`
- `crates/alloy-provider/tests/error_class_contract.rs`
- `crates/alloy-provider/tests/seam_contract.rs`
- `crates/alloy-provider/src/conversion.rs::tests::alloy_to_cow_receipt_populates_status_success`
- `crates/alloy-provider/src/conversion.rs::tests::alloy_to_cow_receipt_populates_status_reverted`
- `crates/alloy-provider/src/conversion.rs::tests::alloy_to_cow_receipt_returns_none_status_for_post_state_receipt`
- `crates/alloy-provider/src/conversion.rs::tests::alloy_to_cow_receipt_handles_contract_creation_no_to`
- `crates/alloy-provider/tests/read_contract_parity.rs`
- `crates/alloy-provider/tests/read_contract_no_panic.rs`
- `tests/alloy_read_contract_parity_invariant.rs`
- `crates/alloy-provider/tests/redaction_contract.rs`
- `crates/alloy-provider/tests/cancellation_contract.rs`
- `crates/alloy-provider/tests/dependency_boundary_contract.rs`
- `crates/alloy-provider/tests/compile_fail.rs`
- `crates/alloy-provider/tests/trybuild/no_signing_provider.rs`
- `crates/alloy-provider/tests/trybuild/no_signer.rs`
- `crates/alloy-provider/tests/trybuild/external_marker_construction_fails.rs`

Validation surface:

```text
cargo fmt --all --check
cargo clippy -p cow-sdk-alloy-provider --all-targets -- -D warnings
cargo test -p cow-sdk-alloy-provider --all-features
cargo test -p cow-rs-workspace-tests --test alloy_read_contract_parity_invariant
cargo test -p cow-sdk-alloy-provider --test compile_fail
RUSTDOCFLAGS="-D warnings" cargo doc -p cow-sdk-alloy-provider --no-deps
cargo check-property-citations
cargo check-alloy-provider-invariant
```
