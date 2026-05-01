# EIP-1271 Verification Cache Audit

Status: Current
Last reviewed: 2026-05-01
Owning surface: `cow-sdk-contracts` `Eip1271VerificationCache` trait and its `NoopEip1271VerificationCache` and `InMemoryEip1271VerificationCache` default implementations shipped from `cow-sdk-signing::cache`
Refresh trigger: Changes to the trait signature, the caching semantics (what is cached and what is not), the `verify_eip1271_signature_async` call shape, the verification tracing fields, the default TTL or capacity on the in-memory implementation, the clock injection seam, the platform time-source selection, or the thread-safety posture; a new canonical implementation that ships in the workspace
Related docs:
- [ADR 0014](../adr/0014-eip1271-verification-cache.md)
- [ADR 0027](../adr/0027-post-quantum-signing-absorption-plan.md)
- [ADR 0028](../adr/0028-account-abstraction-integration-plan.md)
- [Verification Guide](../verification-guide.md)
- [Architecture](../architecture.md)

## Scope

This audit covers:

- the `Eip1271VerificationCache` trait defined in `cow-sdk-contracts`
- the trait re-export from `cow-sdk-signing::cache`
- the `NoopEip1271VerificationCache` and
  `InMemoryEip1271VerificationCache` canonical implementations
- the conservative caching semantics on
  `verify_eip1271_signature_async` — which outcomes are cached and
  which are not
- the `verify.eip1271` tracing span and event fields emitted by the
  verification path
- the controlled-clock seam and exact TTL boundary on native and
  wasm32 targets
- the thread-safety contract on the in-memory implementation

It does not cover the ECDSA signing surface, EIP-712 typed-data
construction, or the recoverable-signature posting boundary (each
covered by its own contract).

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Trait contract | `get(verifier, digest) -> Option<bool>` and `put(verifier, digest, result)` with `Send + Sync + 'static` | Conforms |
| Conservative caching | Only `Ok(())` (magic-value match) and `Eip1271MagicValueMismatch` outcomes are cached; every other error class re-hits the chain | Conforms |
| Pre-interaction scope | The sync and async verification helpers document that they do not simulate order pre-interactions before checking EIP-1271 signatures | Conforms |
| Verification telemetry | `verify_eip1271_signature_async` emits `verify.eip1271` tracing with cache, RPC, and final magic-value outcome fields | Conforms |
| Shipped implementations | `NoopEip1271VerificationCache` (zero-sized, always miss) and `InMemoryEip1271VerificationCache` (bounded capacity, TTL-expiring) | Conforms |
| Platform time source | `InMemoryEip1271VerificationCache` defaults to wall-clock `Instant::now`, accepts an injected clock for deterministic TTL checks, and uses `web_time::Instant` on `wasm32` | Conforms |
| TTL boundary | A 5-minute TTL cache hits at 4m59s999ms and misses at 5m1ms under controlled time on native and wasm32 targets | Conforms |
| Thread-safety | `InMemoryEip1271VerificationCache` sustains concurrent inserts against the same key space without losing writes | Conforms |

## Current Contract

### Trait Definition

`Eip1271VerificationCache` lives at `crates/contracts/src/verify.rs`
co-located with its sole consumer `verify_eip1271_signature_async` so
no reverse dependency on the signing crate is introduced. The trait is
re-exported from `crates/signing/src/cache.rs` where the default
implementations live, so consumers that reach for caching through
`cow-sdk-signing::cache` find the trait and its impls in one place.

The trait is narrow:

- `fn get(&self, verifier: Address, digest: [u8; 32]) -> Option<bool>`
- `fn put(&self, verifier: Address, digest: [u8; 32], result: bool)`

It is `Send + Sync + 'static` so consumers hold the cache across tokio
tasks without lifetime juggling.

### Conservative Caching Semantics

`verify_eip1271_signature_async` takes `&impl Eip1271VerificationCache`
as a required parameter. On a cache hit of `Some(true)` the function
returns `Ok(())` without a chain call; on a cache hit of `Some(false)`
it returns `Err(Eip1271MagicValueMismatch { .. })` without a chain
call. On a cache miss the function dispatches the on-chain
`isValidSignature` call; on `Ok(())` it writes `true` back and on
`Eip1271MagicValueMismatch` it writes `false` back. Every other error
class — transport failure, missing contract code, serialization error,
hex decode error, provider error — bypasses the write-back so a
transient network failure cannot pin a signer into a permanent
`Rejected` state and a stale `false` cannot block a signer whose
on-chain state has since changed.

The verifier's cacheability branch is an exhaustive contracts-crate
match over the current `ContractsError` variants. External consumers
still see `ContractsError` as non-exhaustive and must include a
wildcard arm; the signing crate carries compile-fail coverage for that
public posture.

Both the sync `verify_eip1271_signature` helper and the async
`verify_eip1271_signature_async` helper document the reviewed scope
boundary: they call the verifier against the current provider state and
do not run the order's pre-interactions first. Consumers that need the
same pre-interaction-aware state used by the upstream order-placement
service run that simulation at their own RPC seam before calling the
helper.

### Verification Telemetry

`verify_eip1271_signature_async` carries a `verify.eip1271` tracing span under
the `cow_sdk::verify_eip1271` target. The contracts-layer span records cache
hit or miss state, chain-RPC dispatch outcome, and final magic-value match
state without recording signature payload bytes or provider internals.

### Shipped Implementations

`NoopEip1271VerificationCache` is a zero-sized `Default + Clone + Copy`
unit struct. `get` returns `None`, `put` is a no-op. Consumers that do
not want caching pass a reference to it and pay zero allocation or
synchronization overhead.

`InMemoryEip1271VerificationCache` is a bounded in-memory cache backed
by `parking_lot::RwLock<HashMap<(Address, [u8; 32]), CacheEntry>>` with
a default 5-minute TTL and a default 1024-entry capacity. Past-capacity
inserts evict the oldest entry through a linear scan; the default
capacity keeps the scan cheap. Consumers with much larger key spaces
are expected to compose a proper LRU-backed implementation of the trait
rather than scale the capacity on this struct.

The default constructor preserves wall-clock behaviour. The
`with_clock` constructor accepts an injected `Clock` implementation for
deterministic tests or embedders that centralize time elsewhere.

### Platform Time Source

The in-memory cache obtains timestamps through the `Clock` trait on both
the miss path (`get`) and the write path (`put`). `SystemClock` calls
`Instant::now()` and remains the default. On native targets the instant
type is `std::time::Instant`; on `wasm32-unknown-unknown` it is
`web_time::Instant`, matching the rest of the workspace's time-bearing
cache modules. This keeps the documented wasm32 support posture honest:
constructing the cache, probing a miss, writing a hit, and checking TTL
boundaries all stay non-panicking in browser runtimes.

### Thread-Safety

A hammer regression drives concurrent `put` calls across many tokio
tasks against the same key space and asserts every key written by a
racing task is observable through `get` after the tasks join. Linear
value ordering between racing writers is not required — only that no
write is lost.

## Evidence

Primary implementation points:

- `crates/contracts/src/verify.rs`
- `crates/signing/src/cache.rs`

Primary regression coverage:

- `crates/contracts/tests/verify_telemetry_contract.rs`
- `crates/signing/tests/eip1271_cache_contract.rs::cache_skips_every_non_cacheable_error_class`
- `crates/signing/tests/eip1271_cache_contract.rs::cache_ttl_boundary_holds_at_minus_one_and_misses_at_plus_one`
- `crates/signing/tests/wasm_cache_contract.rs::cache_ttl_boundary_holds_at_minus_one_and_misses_at_plus_one_on_wasm32`
- `crates/signing/tests/ui.rs::eip1271_error_match_requires_wildcard`
- `crates/signing/tests/ui/eip1271_error_match_requires_wildcard.rs`

Validation surface:

```text
cargo test -p cow-sdk-contracts --test verify_telemetry_contract --features tracing
cargo test -p cow-sdk-signing --test eip1271_cache_contract
cargo test -p cow-sdk-signing --test ui
cargo test -p cow-sdk-contracts -p cow-sdk-signing --all-features
cargo check -p cow-sdk-signing --target wasm32-unknown-unknown
wasm-pack test --node crates/signing
```
