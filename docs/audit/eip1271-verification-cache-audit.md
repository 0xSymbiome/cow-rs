# EIP-1271 Verification Cache Audit

Status: Current
Last reviewed: 2026-05-28
Owning surface: `cow-sdk-contracts` `Eip1271VerificationCache` trait, the always-available `NoopEip1271VerificationCache`, and the `InMemoryEip1271VerificationCache` implementation shipped from `cow-sdk-signing::cache` behind the opt-in `in-memory-cache` feature
Refresh trigger: Changes to the trait signature, the cache key, the caching policy (what is recorded and what is not), the `verify_eip1271_signature_cached` call shape, the verification tracing fields, the default TTL or capacity on the in-memory implementation, the clock injection seam, the platform time-source selection, the `in-memory-cache` feature gate, or the thread-safety posture; a new canonical implementation that ships in the workspace
Related docs:
- [ADR 0014](../adr/0014-eip1271-verification-cache.md)
- [ADR 0027](../adr/0027-post-quantum-signing-absorption-plan.md)
- [ADR 0028](../adr/0028-account-abstraction-integration-plan.md)
- [Verification Guide](../verification.md)
- [Architecture](../architecture.md)

## Scope

This audit covers:

- the `Eip1271VerificationCache` trait defined in `cow-sdk-contracts`
- the trait re-export from `cow-sdk-signing::cache`
- the always-available `NoopEip1271VerificationCache` and the
  feature-gated `InMemoryEip1271VerificationCache` implementation
- the cache key (the full `(verifier, digest, signature_hash)` probe
  identity) and the positive-only recording policy on
  `verify_eip1271_signature_cached`
- the `verify.eip1271` tracing span and event fields emitted by the
  verification path
- the controlled-clock seam and exact TTL boundary on native and
  wasm32 targets
- the `in-memory-cache` feature gate and the dependency surface it
  governs
- the thread-safety contract on the in-memory implementation

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
| Verification telemetry | `verify_eip1271_signature_cached` emits `verify.eip1271` tracing with cache, RPC, and final magic-value outcome fields | Conforms |
| Shipped implementations | `NoopEip1271VerificationCache` (zero-sized, always miss, always available) and `InMemoryEip1271VerificationCache` (bounded capacity, TTL-expiring, behind the `in-memory-cache` feature) | Conforms |
| Default availability | The default build performs no memoized verification; the only in-tree caller passes `NoopEip1271VerificationCache`, and the in-memory implementation is compiled only when `in-memory-cache` is enabled | Conforms |
| Platform time source | `InMemoryEip1271VerificationCache` defaults to wall-clock `Instant::now`, accepts an injected clock for deterministic TTL checks, and uses `web_time::Instant` on `wasm32` | Conforms |
| TTL boundary | A 5-minute TTL cache hits at 4m59s999ms and misses at 5m1ms under controlled time on native and wasm32 targets | Conforms |
| Thread-safety | `InMemoryEip1271VerificationCache` sustains concurrent records against the same key space without losing writes | Conforms |

## Current Contract

### Trait Definition

`Eip1271VerificationCache` lives at `crates/contracts/src/verify.rs`
co-located with its sole consumer `verify_eip1271_signature_cached` so
no reverse dependency on the signing crate is introduced. The trait is
re-exported from `crates/signing/src/cache.rs` where the default
implementations live, so consumers that reach for caching through
`cow-sdk-signing::cache` find the trait and its impls in one place.

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

`verify_eip1271_signature_cached` takes `&impl Eip1271VerificationCache`
as a required parameter. On a `contains_valid` hit the function returns
`Ok(())` without a chain call. On a miss it dispatches the on-chain
`isValidSignature` call; on `Ok(())` it records the probe and on every
non-`Ok` outcome it records nothing. A magic-value mismatch, a transport
failure, a missing contract code, a serialization error, a hex decode
error, and a provider error all bypass recording, so a transient network
failure cannot pin a signer into a `Rejected` state and a not-yet-valid
signature that becomes valid on-chain within the TTL is never blocked by
a stale negative entry — the next probe observes the live activation.

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
under the `cow_sdk::verify_eip1271` target. The span records cache hit or
miss state, chain-RPC dispatch outcome, and final magic-value match state
without recording signature payload bytes or provider internals. Because
the cache is positive-only, a `hit` is always a VALID outcome; a `store`
is emitted only when a fresh probe verifies VALID; and a mismatch or
error is reported as `skip` and is never recorded.

### Shipped Implementations

`NoopEip1271VerificationCache` is a zero-sized `Default + Clone + Copy`
unit struct and is always available without any feature. `contains_valid`
returns `false`, `record_valid` is a no-op. Consumers that do not want
caching pass a reference to it and pay zero allocation or synchronization
overhead. It carries no dependencies.

`InMemoryEip1271VerificationCache` is a bounded in-memory cache backed by
`parking_lot::RwLock<HashMap<(Address, [u8; 32], [u8; 32]), Instant>>`
with a default 5-minute TTL and a default 1024-entry capacity. It is
compiled only when the `in-memory-cache` feature is enabled, which is the
only reason the signing crate pulls `parking_lot` (and, on `wasm32`,
`web-time`). Past-capacity records evict the oldest entry through a linear
scan; the default capacity keeps the scan cheap. Consumers with much
larger key spaces are expected to compose a proper LRU-backed
implementation of the trait rather than scale the capacity on this
struct.

The default constructor preserves wall-clock behaviour. The `with_clock`
constructor accepts an injected `Clock` implementation for deterministic
tests or embedders that centralize time elsewhere.

### Default Availability

Caching is off by default. `verify_eip1271_signature_cached` takes the
cache as a required argument, the only in-tree caller passes
`NoopEip1271VerificationCache`, and no shipped code path constructs an
in-memory cache. The in-memory implementation and its locking and time
dependencies are excluded from the default build and from the default
wasm bundle; enabling `in-memory-cache` is an explicit, additive opt-in.

### Platform Time Source

The in-memory cache obtains timestamps through the `Clock` trait on both
the read path (`contains_valid`) and the record path (`record_valid`).
`SystemClock` calls `Instant::now()` and remains the default. On native
targets the instant type is `std::time::Instant`; on
`wasm32-unknown-unknown` it is `web_time::Instant`. This keeps the
documented wasm32 support posture honest: constructing the cache, probing
a miss, recording a hit, and checking TTL boundaries all stay
non-panicking in browser runtimes.

### Thread-Safety

A hammer regression drives concurrent `record_valid` calls across many
tokio tasks against the same key space and asserts every key recorded by
a racing task is observable through `contains_valid` after the tasks
join. No write is lost.

## Evidence

Primary implementation points:

- `crates/contracts/src/verify.rs`
- `crates/signing/src/cache.rs`

Primary regression coverage:

- `crates/contracts/tests/verify_telemetry_contract.rs`
- `crates/contracts/tests/signature_contract.rs::async_eip1271_cache_hit_valid_succeeds_without_provider_call`
- `crates/contracts/tests/signature_contract.rs::async_eip1271_verification_records_only_valid_outcomes_keyed_by_signature`
- `crates/signing/tests/eip1271_cache_contract.rs::in_memory_cache_keys_on_signature_so_distinct_signatures_do_not_alias`
- `crates/signing/tests/eip1271_cache_contract.rs::verify_cached_does_not_serve_a_cached_valid_for_a_different_signature`
- `crates/signing/tests/eip1271_cache_contract.rs::verify_cached_never_records_a_mismatch`
- `crates/signing/tests/eip1271_cache_contract.rs::cache_skips_every_non_cacheable_error_class`
- `crates/signing/tests/eip1271_cache_contract.rs::cache_ttl_boundary_holds_at_minus_one_and_misses_at_plus_one`
- `crates/signing/tests/wasm_cache_contract.rs::cache_ttl_boundary_holds_at_minus_one_and_misses_at_plus_one_on_wasm32`
- `crates/signing/tests/ui.rs::eip1271_error_match_requires_wildcard`

Validation surface:

```text
cargo test -p cow-sdk-contracts --test verify_telemetry_contract --features tracing
cargo test -p cow-sdk-contracts --test signature_contract
cargo test -p cow-sdk-signing --features in-memory-cache --test eip1271_cache_contract
cargo test -p cow-sdk-signing --test ui
cargo test -p cow-sdk-contracts -p cow-sdk-signing --all-features
cargo check -p cow-sdk-signing --target wasm32-unknown-unknown --features in-memory-cache
wasm-pack test --node crates/signing --features in-memory-cache
```
