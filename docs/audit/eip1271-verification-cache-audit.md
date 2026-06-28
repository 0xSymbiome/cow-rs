# EIP-1271 Verification Cache Audit

Status: Current
Last reviewed: 2026-06-24
Owning surface: the `cow-sdk-contracts` `Eip1271Cache` trait and always-available `NoopEip1271Cache` (both re-exported from `cow-sdk-signing::cache`), the `verify_eip1271_signature_cached` orchestration, and the wasm EIP-1271 signing parity surface
Refresh trigger: Changes to the trait signature, the cache key, the caching policy (what is recorded and what is not), the `verify_eip1271_signature_cached` call shape, the verification tracing fields, or the wasm EIP-1271 payload parity; a concrete cache implementation that ships in the workspace
Related docs:
- [ADR 0014](../adr/0014-eip1271-verification-cache.md)
- [ADR 0027](../adr/0027-post-quantum-signing-absorption-plan.md)
- [ADR 0028](../adr/0028-account-abstraction-integration-plan.md)
- [ADR 0039](../adr/0039-typescript-callable-wasm-sdk-surface.md)
- [ADR 0040](../adr/0040-wallet-provider-callback-boundary-for-js-consumers.md)
- [ADR 0045](../adr/0045-async-signer-trait-narrowing.md)
- [Verification Guide](../verification.md)
- [Architecture](../architecture.md)

## Scope

This audit covers:

- the `Eip1271Cache` trait and the always-available `NoopEip1271Cache`,
  both defined in `cow-sdk-contracts` next to `verify_eip1271_signature_cached`
- the trait and `NoopEip1271Cache` re-export from `cow-sdk-signing::cache`
- the cache key (the full `(verifier, digest, signature_hash)` probe
  identity) and the positive-only recording policy on
  `verify_eip1271_signature_cached`
- the `verify.eip1271` tracing span and event fields emitted by the
  verification path
- the wasm EIP-1271 signing payload parity against native Rust and the
  recorded upstream TypeScript SDK vector

It does not cover the ECDSA signing surface, EIP-712 typed-data
construction, or the recoverable-signature posting boundary (each
covered by its own contract).

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Trait contract | `contains_valid(verifier, digest, signature_hash) -> bool` and `record_valid(verifier, digest, signature_hash)` with `Send + Sync + 'static` | Conforms |
| Signature in the key | The key is the full probe identity; the verify helper folds `keccak256(signature)` into the key before consulting the cache, so a recorded VALID for one signature is never served for a different signature on the same digest | Conforms |
| Positive-only recording | Only a magic-value match (`Ok(())`) is recorded; a mismatch and every other error class are never recorded, so a `contains_valid` miss means "unknown", never "known invalid" | Conforms |
| EOA miss posture | Verifiers with no contract code return a typed error and do not record anything | Conforms |
| Pre-interaction scope | The verification helpers document that they do not simulate order pre-interactions before checking EIP-1271 signatures | Conforms |
| Verification telemetry | `verify_eip1271_signature_cached` emits `verify.eip1271` tracing whose child debug events use two field keys, `cache_status` and `verification_result`; RPC failures surface as `cache_status=skip` / `verification_result=error` rather than a separate field | Conforms |
| Shipped implementation | `NoopEip1271Cache` (zero-sized, always miss, always available) is the only shipped cache; no concrete cache ships, and a consumer that wants memoization implements the trait | Conforms |
| Default availability | The default build performs no memoized verification; the only in-tree caller passes `NoopEip1271Cache` | Conforms |
| Native Rust parity | The wasm EIP-1271 payload equals `cow_sdk_signing::eip1271_signature_payload` for the fixed vector | Conforms |
| TypeScript SDK parity | The fixed vector matches upstream `OrderSigningUtils.getEip1271Signature` output | Conforms |
| Facade-resolves-callback | JavaScript supplies the final signature, while Rust stores only a pure resolved provider | Conforms |
| UID and digest strings | Cross-ABI DTOs reuse canonical `as_str()` output instead of re-encoding bytes | Conforms |
| Signature validation (wasm) | Malformed ECDSA signatures fail before being surfaced as signed orders | Conforms |
| Capability split | Custom EIP-1271 signing uses a dedicated callback and does not require a broad wallet signer object | Conforms |

## Current Contract

### Trait Definition

`Eip1271Cache` lives at `crates/contracts/src/verify.rs`
co-located with its sole consumer `verify_eip1271_signature_cached` so
no reverse dependency on the signing crate is introduced. The trait and
`NoopEip1271Cache` are re-exported from `crates/signing/src/cache.rs`, so
consumers that reach for caching through `cow-sdk-signing::cache` find the seam
in one place.

The trait is a positive-only set keyed on the full probe identity:

- `fn contains_valid(&self, verifier: Address, digest: [u8; 32], signature_hash: [u8; 32]) -> bool`
- `fn record_valid(&self, verifier: Address, digest: [u8; 32], signature_hash: [u8; 32])`

There is no negative cache and no `bool` value: the cache records only
probes observed VALID, so a `contains_valid` miss means the caller must
re-check the chain. It is `Send + Sync + 'static` so consumers hold the
cache across tokio tasks without lifetime juggling.

### Cache Key

The on-chain `isValidSignature(hash, signature)` verdict — and the
upstream off-chain signature validator — are functions of the signature
as well as the digest. The cache key is therefore the full probe
identity `(verifier, digest, signature_hash)`, where `signature_hash` is
the `keccak256` of the signature bytes. `verify_eip1271_signature_cached`
computes that hash before consulting the cache, so a VALID recorded for
one signature is never returned for a different signature on the same
`(verifier, digest)`.

### Positive-Only Recording Policy

`verify_eip1271_signature_cached` takes `&impl Eip1271Cache`
as a required parameter. On a `contains_valid` hit the function returns
`Ok(())` without a chain call. On a miss it dispatches the on-chain
`isValidSignature` call; on `Ok(())` it records the probe and on every
non-`Ok` outcome it records nothing. A magic-value mismatch, a transport
failure, a missing contract code, a serialization error, a hex decode
error, and a provider error all bypass recording, so a transient network
failure cannot pin a signer into a `Rejected` state and a not-yet-valid
signature that becomes valid on-chain is never blocked by a stale negative
entry — the next probe observes the live activation.

The no-code branch is covered explicitly: an EOA verifier miss returns
the typed non-contract error and records nothing.

Both the bare `verify_eip1271_signature` helper and the
`verify_eip1271_signature_cached` orchestration variant document the
reviewed scope boundary: they call the verifier against the current
provider state and do not run the order's pre-interactions first.
Consumers that need the same pre-interaction-aware state used by the
upstream order-placement service run that simulation at their own RPC
seam before calling the helper.

### Verification Telemetry

`verify_eip1271_signature_cached` carries a `verify.eip1271` tracing span
under the `cow_sdk::verify_eip1271` target. The span declares only the
`verifier` field; the cache and magic-value states are emitted as child
debug events with two field keys, `cache_status` (`hit` / `miss` /
`store` / `skip`) and `verification_result` (`valid` / `invalid` /
`error`), and never record signature payload bytes or provider
internals. Because the cache is positive-only, a `hit` is always a VALID
outcome; a `store` is emitted only when a fresh probe verifies VALID; and
a mismatch or any other error is reported as `skip` and is never
recorded.

### Shipped Implementation

`NoopEip1271Cache` is a zero-sized `Default + Clone + Copy`
unit struct and is always available without any feature. `contains_valid`
returns `false`, `record_valid` is a no-op. Consumers that do not want
caching pass a reference to it and pay zero allocation or synchronization
overhead. It carries no dependencies.

No concrete cache ships. A consumer that wants memoization implements the
two-method trait over the store of its choice (for example an LRU keyed on
`(verifier, digest, signature_hash)`), keeping the positive-only contract and
choosing its own TTL and capacity. The trait being `Send + Sync + 'static`
lets the consumer share that store across tasks.

### Default Availability

Caching is off by default. `verify_eip1271_signature_cached` takes the
cache as a required argument, and the only in-tree caller passes
`NoopEip1271Cache`, so no shipped code path memoizes a verification. A
consumer opts in only by implementing the trait and threading it through the
cached entry point.

### WASM EIP-1271 Parity

`eip1271SignaturePayload` and the EIP-1271 order-signing functions
(`signOrderWithEip1271`, `signOrderWithCustomEip1271`) wrap the same Rust
helper used by native signing; the wasm tests compare output against native
Rust and a recorded upstream TypeScript SDK vector for the same order, owner,
verifier, and signature bytes. `signOrderWithCustomEip1271` invokes a
JavaScript callback at the facade boundary that returns the final ABI-encoded
signature, which Rust wraps in a `Send + Sync` resolved provider; no JavaScript
handle or `JsValue` is stored in the trait object. Order IDs and digests
crossing to TypeScript reuse the canonical string stored by the Rust type and
are never reconstructed from raw byte arrays.

## Evidence

Primary implementation points:

- `crates/contracts/src/verify.rs`
- `crates/signing/src/cache.rs`
- `crates/js/src/helpers/signing.rs`
- `crates/js/src/exports/eip1271.rs`
- `crates/js/src/exports/signing.rs`
- `parity/fixtures/signing/eip1271_typescript_vector.json`
- `parity/source-lock.yaml`

Primary regression coverage:

- `crates/contracts/tests/verify_telemetry_contract.rs`
- `crates/contracts/tests/signature_contract.rs::async_eip1271_cache_hit_valid_succeeds_without_provider_call`
- `crates/contracts/tests/signature_contract.rs::async_eip1271_verification_records_only_valid_outcomes_keyed_by_signature`
- `crates/contracts/tests/signature_contract.rs::async_eip1271_verification_fails_closed_for_missing_code_and_transport_errors`
- `crates/contracts/tests/signature_contract.rs::eip1271_verification_reads_contract_code_and_magic_value`
- `crates/signing/tests/ui.rs::eip1271_error_match_requires_wildcard`
- `crates/js/tests/host_pure_helpers.rs::eip1271_payload_matches_signing_module_output_and_vector`
- `crates/js/tests/host_pure_helpers.rs::generated_order_uid_uses_canonical_strings`
- `crates/js/tests/wasm_eip1271_contract.rs::eip1271_payload_matches_native_rust`
- `crates/js/tests/wasm_eip1271_contract.rs::eip1271_payload_matches_recorded_typescript_sdk_vector`
- `crates/js/tests/wasm_eip1271_contract.rs::sign_order_with_eip1271_uid_equals_generated_order_id_as_str`
- `crates/js/tests/wasm_eip1271_contract.rs::custom_eip1271_callback_signature_is_used_verbatim`
- `crates/js/src/exports/eip1271.rs` (compile-time `Send + Sync` assertion on `ResolvedEip1271Provider`)
- `e2e/wasm-typescript/tests/eip1271.spec.ts`

Validation surface:

```text
cargo test -p cow-sdk-contracts --test verify_telemetry_contract --features tracing
cargo test -p cow-sdk-contracts --test signature_contract
cargo test -p cow-sdk-signing --test ui
cargo test -p cow-sdk-contracts -p cow-sdk-signing --all-features
cargo check -p cow-sdk-signing --target wasm32-unknown-unknown
cargo test -p cow-sdk-js --test host_pure_helpers
wasm-pack test crates/js --headless --firefox
pnpm --dir e2e/wasm-typescript test
```
