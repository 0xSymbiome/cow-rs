# Alloy Umbrella Adapter Audit

Status: Current
Last reviewed: 2026-06-11
Owning surface: `cow-sdk-alloy` `AlloyClient`, its builder, its `Provider` and `LogProvider` implementations, and its owned signer handle
Refresh trigger: ADR 0038 - transaction lifecycle types, or changes to the umbrella public API, `Provider`, `SigningProvider`, `LogProvider`, `Signer`, wallet-filler transaction submission, the opt-in `with_retry` seam consumed from the provider leaf, typed-data conversion, chain-coherence validation, read-contract and log-fetch consumption from the provider seam, error redaction, cancellation propagation, or the Alloy provider/signer dependency boundaries
Related docs:
- [ADR 0037](../adr/0037-alloy-umbrella-adapter.md)
- [ADR 0024](../adr/0024-asyncprovider-asyncsigningprovider-capability-split.md)
- [ADR 0025](../adr/0025-workspace-url-redaction-convention.md)
- [ADR 0026](../adr/0026-alloy-major-release-absorption-plan.md)
- [ADR 0035](../adr/0035-alloy-provider-adapter.md)
- [ADR 0036](../adr/0036-alloy-signer-adapter.md)
- [ADR 0038](../adr/0038-transaction-lifecycle-types.md)
- [ADR 0057](../adr/0057-log-provider-capability-trait.md)
- [Alloy Provider Adapter Audit](alloy-provider-adapter-audit.md)
- [Alloy Signer Adapter Audit](alloy-signer-adapter-audit.md)

## Scope

This audit covers:

- the `AlloyClient` public type, typestate builder, and native-only support
  posture
- the `Provider`, `LogProvider`, and `SigningProvider` implementations on
  `AlloyClient`
- the opt-in `with_retry` seam, which reuses the provider leaf's `RetryConfig`
  and backoff-layer constructor through the doc-hidden seam
- the owned `AlloyClientSignerHandle` returned by `create_signer`
- EIP-191, EIP-712, transaction submission, gas estimation, and raw
  transaction-signing behavior on the handle
- `AlloyClientError` classification, redaction, and cancellation propagation
- the provider-leaf and signer-leaf inter-crate seams consumed by the
  umbrella crate for read-contract dispatch, typed-data conversion, and
  signature normalization
- dependency allow-lists for the native Alloy provider and signer-local family

It does not cover upstream Alloy internals, browser-wallet behavior, live RPC
operator reliability, or smart-account signing.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Public API exposure | `AlloyClient`, its builder, the signer handle, and errors expose SDK-owned types; upstream Alloy state remains private and redacted | Conforms |
| Builder typestate | HTTP transport, private-key source, and chain id are selected before `build()` is available; marker states remain sealed | Conforms |
| Chain coherence | `build_checked()` rejects configured-chain and remote-chain mismatches directly, while `verify_chain_id().await` exposes the same check for clients built through `build()` | Conforms |
| Provider coverage | Every `Provider` method delegates through the inner Alloy provider with SDK-owned conversions | Conforms |
| Read-contract parity | The umbrella's read-contract path consumes the provider leaf's `execute_read_contract` entry through the doc-hidden inter-crate seam and lifts the provider's error variants through the `From<ProviderError> for AlloyClientError` impl. The workspace `alloy_read_contract_parity_invariant` integration test continues to assert byte-for-byte equality between the umbrella and the provider for pinned ABI fixtures as a regression pin against any future re-fork. | Conforms |
| Log-provider coverage | `AlloyClient` implements `LogProvider`, issuing one bounded `eth_getLogs` over the composed provider and reusing the leaf's `LogQuery` / `RawLog` conversions through the doc-hidden seam, so a consumer fetches event logs without constructing a second provider | Conforms |
| RPC retry seam | `with_retry(RetryConfig)` is off by default; when configured it routes the wallet-filler provider through a JSON-RPC client carrying the leaf's shared backoff layer (built via the doc-hidden seam), so umbrella and leaf share one retry policy | Conforms |
| Signing-provider coverage | `create_signer` returns an owned handle that survives parent client drop | Conforms |
| Typed-data signing | Canonical payload signing preserves the caller's primary type and matches the CoW order reference vector | Conforms |
| Transaction behavior | `send_transaction` uses the Alloy wallet-filler provider and reads the broadcast hash through `*pending.tx_hash()` without waiting for confirmation; returns `TransactionBroadcast`. `get_transaction_receipt` delegates to the provider crate, which populates rich receipt fields from the Alloy receipt. `estimate_gas` delegates to the provider. | Conforms |
| Raw transaction deferral | `sign_transaction` returns `UnsupportedTransactionRequest` without dispatching HTTP | Conforms |
| Error and cancellation | Error classes cover validation, transport, remote, signing, pending transaction, unsupported request, cancelled, and internal failures; sensitive details are redacted | Conforms |
| Stability boundary | Documented client, builder, trait, signer-handle, and error-class surfaces are consumer API; the doc-hidden inter-crate seams on both the provider leaf (`cow_sdk_alloy_provider::__seam`) and the signer leaf (`cow_sdk_alloy_signer::__seam`) that the umbrella consumes are sibling-crate internals and not semver-guaranteed | Conforms |
| Dependency boundary | The umbrella is the only crate, alongside the provider and signer leaves, allowed to consume the native Alloy provider and signer-local families | Conforms |

## Evidence

- `crates/alloy/tests/provider_contract.rs`
- `crates/alloy/tests/signing_provider_contract.rs`
- `crates/alloy/tests/log_provider_contract.rs`
- `crates/alloy/tests/builder_contract.rs`
- `crates/alloy/tests/error_contract.rs`
- `crates/alloy/tests/read_contract_contract.rs`
- `crates/alloy/tests/eip712_reference_vectors.rs`
- `crates/alloy/tests/no_broadcast_for_sign_transaction.rs`
- `crates/alloy/tests/send_transaction_does_not_wait_for_confirmation.rs`
- `crates/alloy/tests/chain_coherence_mismatch.rs`
- `crates/alloy/tests/handle_survives_drop.rs`
- `crates/alloy/tests/cancellation_contract.rs`
- `crates/alloy/tests/redaction_contract.rs`
- `crates/alloy/tests/compile_fail.rs`
- `tests/alloy_umbrella_composition.rs`
- `tests/transaction_lifecycle_cross_adapter_invariant.rs`
- `tests/alloy_read_contract_parity_invariant.rs`
- `examples/native/scenarios/alloy_trading_full_flow.rs`
- `xtask/src/policy/dependency_invariant.rs`

## Residual Risk

The adapter inherits upstream Alloy behavior for transaction filling and RPC
transport details. The SDK boundary narrows this risk by keeping upstream types
private, pinning the reviewed dependency families, and running a scheduled
canary against configurable Alloy refs.

## Validation

```text
cargo test -p cow-sdk-alloy --all-features
cargo test -p cow-sdk-alloy --test log_provider_contract
cargo test -p cow-sdk-alloy --test chain_coherence_mismatch
cargo test -p cow-rs-workspace-tests --test alloy_umbrella_composition
cargo test -p cow-rs-workspace-tests --test alloy_read_contract_parity_invariant
cargo run -p cow-sdk-examples-native --example alloy_trading_full_flow --features alloy
cargo check-alloy-provider-invariant
cargo check-alloy-signer-invariant
RUSTDOCFLAGS="-D warnings" cargo doc -p cow-sdk-alloy --no-deps
```
