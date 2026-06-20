# Alloy Adapters Audit

Status: Current
Last reviewed: 2026-06-20
Owning surface: the native Alloy adapter family (`cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`, `cow-sdk-alloy`), the shared transaction-lifecycle / receipt types they convert, and the `LogProvider` capability they implement
Refresh trigger: changes to any adapter's public API; the `Provider`, `Signer`, `SigningProvider`, or `LogProvider` traits; transport classification or the opt-in `retry` seam and its `RetryConfig`; typed-data conversion or signature normalization; chain-coherence validation; the `read_contract` algorithm or the bounded `get_logs` contract; the `TransactionBroadcast` / `TransactionStatus` / `TransactionReceipt` types or Alloy receipt conversion; the `LogQuery` / `RawLog` / `LogMeta` types; the doc-hidden inter-crate seams consumed across adapters; error redaction or cancellation propagation; the workspace Alloy runtime/signer pin; or the adapter dependency boundaries
Related docs:
- [ADR 0035](../adr/0035-alloy-provider-adapter.md)
- [ADR 0038](../adr/0038-transaction-lifecycle-types.md)
- [ADR 0057](../adr/0057-log-provider-capability-trait.md)

## Scope

This audit covers:

- the read-only `RpcAlloyProvider`, its HTTP typestate builder, and its six-method `Provider` implementation
- the `LocalAlloySigner`, its private-key plus chain-id typestate builder, and its `Signer` implementation including EIP-712 typed-data conversion and signature normalization
- the `AlloyClient` umbrella, its typestate builder, chain-coherence validation, and its `Provider`, `SigningProvider`, and `LogProvider` implementations plus the owned `AlloyClientSignerHandle`
- the opt-in `retry` seam and shared `RetryConfig` / backoff-layer constructor
- the `read_contract` ABI encode/dispatch/decode path and its byte-for-byte umbrella/leaf parity
- the shared transaction-lifecycle types (`TransactionBroadcast`, `TransactionStatus`, `TransactionReceipt`) and Alloy receipt conversion, including Alloy status mapping
- the `LogProvider: Provider` capability, the `LogQuery` / `RawLog` / `LogMeta` types, and the single bounded `get_logs` contract
- error classification, credential redaction, cancellation propagation, and the doc-hidden `__seam` modules shared across the family
- dependency allow-lists for the provider, signer, and umbrella crates

It does not cover upstream Alloy internals, WS/IPC transport, smart-account signing, event-log decoding (the fail-closed decoders are reviewed elsewhere), a higher-level wait helper, or live-chain timing.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Public API exposure | Documented provider, signer, umbrella, builder, signer-handle, and error surfaces expose SDK-owned domain types; upstream Alloy values stay private and redacted; the `__seam` modules are `#[doc(hidden)]` sibling-crate internals, not semver-guaranteed | Conforms |
| Trait coverage | `RpcAlloyProvider` implements all six `Provider` methods; `LocalAlloySigner` implements `Signer`; `AlloyClient` implements `Provider`, `SigningProvider`, and `LogProvider`; compile-fail tests assert each leaf's negative capabilities | Conforms |
| Builder typestate | `build()` is reachable only after the required transport / key / chain states are selected; externally constructed marker states cannot bypass the builder | Conforms |
| Chain coherence | `build_checked()` rejects configured-vs-remote chain mismatch; `verify_chain_id().await` exposes the same check for `build()` clients | Conforms |
| RPC retry seam | Retry is off by default (one request per call); `retry(RetryConfig)` wraps the JSON-RPC client in `alloy`'s bounded backoff layer; umbrella and leaf share one policy through the seam. The REST `TransportPolicy` is not reused (its retry signal is keyed on HTTP status codes JSON-RPC errors do not cleanly surface) | Conforms |
| `read_contract` | Loads ABI, resolves a single function (rejecting overloads), parses JSON args, ABI-encodes, dispatches `eth_call`, decodes, serializes supported JSON value strings, and returns typed errors for malformed input instead of panicking. The umbrella consumes the leaf's `execute_read_contract` through the seam; `alloy_read_contract_parity_invariant` pins byte-for-byte equality | Conforms |
| EIP-191 signing | Message signatures match the committed reference vector and recover to the local signer address | Conforms |
| EIP-712 signing | Typed-data signing is payload-only (no flat `(domain, fields, value)` form): order signatures preserve `Order` as the primary type and match the reference vector; nested multi-type payloads with struct-typed fields convert to digests byte-identical to the macro-emitted `SolStruct` envelope; undeclared struct references stay fail-closed; a payload differing only by primary type signs to a different vector | Conforms |
| Signature normalization | All returned ECDSA signatures pass through `cow_sdk_contracts::RecoverableSignature`, aligning with the shared Solidity-compatible recovery-byte contract | Conforms |
| Transaction lifecycle | Signers return `TransactionBroadcast`; providers return rich `TransactionReceipt`. Umbrella `send_transaction` reads the broadcast hash via `*pending.tx_hash()` without waiting for confirmation and never calls `eth_getTransactionReceipt` during broadcast | Conforms |
| Receipt shape | Provider receipt conversion populates `transaction_hash`, `status`, `block_number`, `block_hash`, `gas_used`, `from`, and `to`; status comes from `status_or_post_state().as_eip658()`, so pre-Byzantium post-state receipts map status to `None` and contract creation leaves `to` unset | Conforms |
| LogProvider capability | `LogProvider: Provider` is an opt-in supertrait leaving the frozen `Provider` shape unchanged; `get_logs` issues exactly one bounded `eth_getLogs` (no loop, poll, watch, or range expansion); `AlloyClient` implements it over its held provider and reuses the leaf's `LogQuery` / `RawLog` conversions through the seam | Conforms |
| Error and cancellation | Error classes cover validation, transport, remote, signing, provider-required, pending-transaction, unsupported, cancelled, and internal failures; sensitive detail is redacted; `From<Cancelled>` propagates cancellation through each error type | Conforms |
| Dependency boundary | The provider declares no signer-family dependency; the signer declares no provider/transport dependency; the umbrella, with the two leaves, is the only consumer of the native Alloy provider and signer-local families. Resolved graphs are asserted to honor these allow-lists | Conforms |

## Current Contract

### Read-only provider

`cow-sdk-alloy-provider` exposes `RpcAlloyProvider`, `RpcAlloyProviderBuilder`, sealed transport-state markers, `RpcAlloyProviderBuilderError`, `RetryConfig`, `ProviderError`, and `ProviderErrorClass`. The provider stores the upstream `DynProvider` in private state and keeps raw transport labels out of debug output. `build()` is callable only on the HTTP-selected builder state, which stores the URL through `Redacted<reqwest::Url>`. The adapter implements `get_chain_id`, `get_code`, `get_transaction_receipt`, `call`, `read_contract`, and `get_block`, converting caller-owned SDK types to Alloy values before dispatch and back to `cow-sdk-core` types on return. Retry is opt-in: the default path issues each request once and surfaces a transient failure (such as a `429`) directly; `retry(RetryConfig)` (max retry count plus initial backoff) wraps the JSON-RPC client in `alloy`'s rate-limit backoff layer, with the internal compute-units budget kept private.

`read_contract` is the ABI boundary: it parses `ContractCall::abi_json` with `alloy-json-abi`, rejects overloaded names, parses `args_json`, maps arguments into `DynSolValue`, encodes calldata with `alloy-dyn-abi`, dispatches `eth_call`, decodes the return data, and serializes the decoded value to the JSON string `cow-sdk-core` requires. Malformed ABI, JSON, addresses, integers, unsupported ABI types, and decode failures return typed errors.

### Local signer (typed data incl. nested, normalization)

`cow-sdk-alloy-signer` exposes `LocalAlloySigner`, `LocalAlloySignerBuilder`, sealed builder-state markers, `LocalAlloySignerBuilderError`, `SignerError`, and `SignerErrorClass`. The signer stores the upstream Alloy private-key signer in private state and redacts it from debug output. The builder accepts hex or raw 32-byte private keys plus a `cow_sdk_core::SupportedChainId`; invalid key material returns a typed error that does not echo the input. The adapter implements `address`, `sign_message`, and `sign_typed_data_payload` — the only typed-data entry point. `send_transaction` and `estimate_gas` return `ProviderRequired`: the local signer owns no provider context, so this leaf never fabricates a broadcast hash.

`sign_typed_data_payload` converts the explicit SDK payload into Alloy dynamic typed data without dropping the primary type. The payload is self-contained (domain, full type map, primary-type name, message), so the adapter never synthesizes a placeholder type. A field may reference another struct declared in the type map, directly or as an array (for example a `Call[]` over a `Call` struct), so nested multi-type EIP-712 payloads convert end to end; a field naming an undeclared struct stays fail-closed. All returned ECDSA signatures pass through `cow_sdk_contracts::RecoverableSignature`.

### Umbrella composition + broadcast-only submission

`cow-sdk-alloy` exposes `AlloyClient`, its typestate builder, the `AlloyClientSignerHandle`, and `AlloyClientError`. The builder requires HTTP transport, private-key source, and chain id before `build()`; `build_checked()` rejects configured-vs-remote chain mismatch and `verify_chain_id().await` exposes the same check for `build()` clients. Every `Provider` method delegates through the inner Alloy provider with SDK-owned conversions. `create_signer` returns an owned handle that survives parent-client drop and serves canonical typed-data signing (preserving the caller's primary type and matching the CoW order reference vector). `send_transaction` uses the Alloy wallet-filler provider and reads the broadcast hash through `*pending.tx_hash()` without waiting for confirmation, returning `TransactionBroadcast`; `get_transaction_receipt` and `estimate_gas` delegate to the provider crate. The umbrella's `read_contract` and `LogProvider` paths consume the provider leaf's `execute_read_contract` and log conversions through `cow_sdk_alloy_provider::__seam`, and reuse the leaf's backoff-layer constructor for `retry`. The `__seam` modules on both leaves are sibling-crate internals, not consumer API.

### Transaction lifecycle / receipt shape + Alloy status mapping

`TransactionBroadcast` is the signer submission acknowledgement and carries only `transaction_hash`. `TransactionStatus` carries `Success` or `Reverted`. `TransactionReceipt` carries `transaction_hash` plus optional `status`, `block_number`, `block_hash`, `gas_used`, `from`, and `to`. Signers return `TransactionBroadcast`; providers return `TransactionReceipt` for mined observation. The umbrella reads the broadcast hash via `*pending.tx_hash()` and does not call `eth_getTransactionReceipt` during broadcast. The provider's `alloy_to_cow_receipt` populates all fields, deriving `status` from `receipt.inner.status_or_post_state().as_eip658()`, so a pre-Byzantium post-state receipt surfaces `status: None` rather than coerced success, and contract creation leaves `to` unset.

### LogProvider capability (single bounded get_logs)

`LogProvider: Provider` mirrors the `SigningProvider: Provider` split: read-only adapters implement only `Provider`, while an adapter that can serve `eth_getLogs` additionally implements `LogProvider`. `Provider`'s frozen six-method shape is unchanged; `LogProvider` adds only `get_logs(&LogQuery) -> Result<Vec<RawLog>, Self::Error>`. `get_logs` cannot be derived from existing `Provider` methods, so it lands as its own opt-in supertrait. It issues exactly one backend query over the caller-bounded `[from_block, to_block]` range — never a watcher, iterator, or indexer loop. `LogQuery` mirrors the `eth_getLogs` filter: an address set (single or any-of), four independent topic slots (topic-0 = event signature, topics 1-3 = indexed arguments, each any-of, empty = wildcard), and a `LogBlockSelector` that is an inclusive number range or a single block hash; `Hash32::from_indexed_address` runs the "events for my address" query server-side. `RawLog` carries the emitting address, the indexed-topics-plus-data payload, the reorg `removed` flag, and positional `LogMeta` (block number and hash, optional timestamp, transaction hash and index, log index). None of these types depend on a provider or network, and `RawLog::data` feeds directly to a fail-closed decoder. Both the `RpcAlloyProvider` leaf and the `AlloyClient` umbrella implement the capability; the umbrella reuses the leaf's `cow_log_query_to_alloy_filter` and `alloy_log_to_cow_raw_log` through the seam, so a consumer fetches logs from the same client it trades through.

### Redaction + cancellation + dependency boundary

Each crate's error type partitions failures into typed classes and redacts sensitive detail in public formatting: invalid-URL and invalid-key errors carry no input, provider debug output redacts the transport (`Redacted<String>`), and remote JSON-RPC errors keep their code and message because those fields are the peer's structured protocol response. `From<cow_sdk_core::Cancelled>` lets consumer code using `Cancellable::cancel_with(...).await?` propagate cancellation through every adapter error type. On dependencies: the provider depends on Alloy runtime provider crates plus `reqwest` with rustls but declares no `alloy-signer`/`alloy-signer-local`; the signer depends on the Alloy signer, local-signer, primitives, consensus, network, dynamic-ABI, and Solidity type crates but no `alloy-provider` or transport crates; the umbrella, with the two leaves, is the only crate allowed to consume the native Alloy provider and signer-local families. Resolved graphs are asserted to honor these allow-lists, and a manually-dispatched (`workflow_dispatch`) canary runs against configurable Alloy refs.

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
- `crates/alloy-signer/src/lib.rs`
- `crates/alloy-signer/src/signer.rs`
- `crates/alloy-signer/src/builder.rs`
- `crates/alloy-signer/src/error.rs`
- `crates/alloy-signer/src/conversion.rs`
- `crates/alloy-signer/Cargo.toml`
- `crates/alloy/src/client.rs`
- `crates/alloy/src/handle.rs`
- `crates/core/src/traits/transaction.rs`
- `crates/core/src/traits/provider.rs`
- `crates/core/src/types/logs.rs`
- `xtask/src/policy/dependency_invariant.rs`

Primary regression coverage:

- `crates/alloy-provider/tests/provider_contract.rs`
- `crates/alloy-provider/tests/builder_contract.rs`
- `crates/alloy-provider/tests/retry_contract.rs`
- `crates/alloy-provider/tests/error_class_contract.rs`
- `crates/alloy-provider/tests/seam_contract.rs`
- `crates/alloy-provider/tests/seam_contract.rs::seam_exposes_log_conversions_for_the_umbrella`
- `crates/alloy-provider/tests/read_contract_parity.rs`
- `crates/alloy-provider/tests/read_contract_no_panic.rs`
- `crates/alloy-provider/tests/redaction_contract.rs`
- `crates/alloy-provider/tests/cancellation_contract.rs`
- `crates/alloy-provider/tests/dependency_boundary_contract.rs`
- `crates/alloy-provider/tests/compile_fail.rs`
- `crates/alloy-provider/tests/trybuild/no_signing_provider.rs`
- `crates/alloy-provider/tests/trybuild/no_signer.rs`
- `crates/alloy-provider/tests/trybuild/external_marker_construction_fails.rs`
- `crates/alloy-provider/src/conversion.rs::tests::alloy_to_cow_receipt_populates_status_success`
- `crates/alloy-provider/src/conversion.rs::tests::alloy_to_cow_receipt_populates_status_reverted`
- `crates/alloy-provider/src/conversion.rs::tests::alloy_to_cow_receipt_returns_none_status_for_post_state_receipt`
- `crates/alloy-provider/src/conversion.rs::tests::alloy_to_cow_receipt_handles_contract_creation_no_to`
- `crates/alloy-provider/src/conversion.rs::tests::cow_log_query_to_alloy_filter_sets_caller_bounded_range`
- `crates/alloy-provider/src/conversion.rs::tests::cow_log_query_to_alloy_filter_maps_topics_addresses_and_block_hash`
- `crates/alloy-provider/src/conversion.rs::tests::alloy_log_to_cow_raw_log_maps_address_meta_and_payload`
- `crates/alloy-signer/tests/signer_contract.rs`
- `crates/alloy-signer/tests/eip191_reference_vectors.rs`
- `crates/alloy-signer/tests/eip712_reference_vectors.rs`
- `crates/alloy-signer/tests/redaction_contract.rs`
- `crates/alloy-signer/tests/cancellation_contract.rs`
- `crates/alloy-signer/tests/dependency_boundary_contract.rs`
- `crates/alloy-signer/tests/proptests.rs`
- `crates/alloy-signer/tests/compile_fail.rs`
- `crates/alloy-signer/tests/trybuild/no_provider.rs`
- `crates/alloy-signer/tests/trybuild/no_signing_provider.rs`
- `crates/alloy-signer/tests/trybuild/external_marker_construction_fails.rs`
- `crates/alloy-signer/src/conversion.rs::tests::nested_struct_payload_matches_macro_digest`
- `crates/alloy-signer/src/conversion.rs::tests::undeclared_struct_reference_is_rejected`
- `crates/alloy/tests/provider_contract.rs`
- `crates/alloy/tests/provider_contract.rs::get_transaction_receipt_populates_rich_fields_from_alloy_receipt`
- `crates/alloy/tests/signing_provider_contract.rs`
- `crates/alloy/tests/log_provider_contract.rs`
- `crates/alloy/tests/log_provider_contract.rs::alloy_client_implements_log_provider_and_returns_typed_error_on_unreachable_rpc`
- `crates/alloy/tests/builder_contract.rs`
- `crates/alloy/tests/error_contract.rs`
- `crates/alloy/tests/read_contract_contract.rs`
- `crates/alloy/tests/eip712_reference_vectors.rs`
- `crates/alloy/tests/send_transaction_does_not_wait_for_confirmation.rs`
- `crates/alloy/tests/send_transaction_does_not_wait_for_confirmation.rs::send_transaction_does_not_dispatch_get_transaction_receipt`
- `crates/alloy/tests/chain_coherence_mismatch.rs`
- `crates/alloy/tests/handle_survives_drop.rs`
- `crates/alloy/tests/cancellation_contract.rs`
- `crates/alloy/tests/redaction_contract.rs`
- `crates/alloy/tests/compile_fail.rs`
- `crates/core/tests/traits_contract.rs`
- `crates/core/src/types/logs.rs::tests::builders_populate_addresses_and_topic_slots`
- `crates/core/src/types/logs.rs::tests::from_indexed_address_left_pads_to_a_topic`
- `tests/alloy_umbrella_composition.rs`
- `tests/alloy_read_contract_parity_invariant.rs`
- `tests/transaction_lifecycle_cross_adapter_invariant.rs`
- `examples/native/scenarios/alloy_trading_full_flow.rs`

Validation surface:

```text
cargo fmt --all --check
cargo clippy -p cow-sdk-alloy-provider --all-targets -- -D warnings
cargo clippy -p cow-sdk-alloy-signer --all-targets -- -D warnings
cargo test -p cow-sdk-core --lib logs
cargo test -p cow-sdk-alloy-provider --all-features
cargo test -p cow-sdk-alloy-provider --lib
cargo test -p cow-sdk-alloy-provider --test seam_contract
cargo test -p cow-sdk-alloy-provider --test compile_fail
cargo test -p cow-sdk-alloy-signer --all-features
cargo test -p cow-sdk-alloy-signer --test compile_fail
cargo test -p cow-sdk-alloy --all-features
cargo test -p cow-sdk-alloy --test log_provider_contract
cargo test -p cow-sdk-alloy --test chain_coherence_mismatch
cargo test -p cow-sdk-alloy --test send_transaction_does_not_wait_for_confirmation
cargo test -p cow-rs-workspace-tests --test alloy_umbrella_composition
cargo test -p cow-rs-workspace-tests --test alloy_read_contract_parity_invariant
cargo test -p cow-rs-workspace-tests --test transaction_lifecycle_cross_adapter_invariant
cargo run -p cow-sdk-examples-native --example alloy_trading_full_flow --features alloy
cargo check-property-citations
cargo check-alloy-provider-invariant
cargo check-alloy-signer-invariant
RUSTDOCFLAGS="-D warnings" cargo doc -p cow-sdk-alloy-provider --no-deps
RUSTDOCFLAGS="-D warnings" cargo doc -p cow-sdk-alloy-signer --no-deps
RUSTDOCFLAGS="-D warnings" cargo doc -p cow-sdk-alloy --no-deps
```
