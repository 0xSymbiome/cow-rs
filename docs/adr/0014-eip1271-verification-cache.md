# ADR 0014: Pluggable EIP-1271 Verification Cache With Conservative Caching Semantics

- Status: Accepted (amended)
- Date: 2026-04-21
- Last reviewed: 2026-05-28
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: signing, eip1271, caching, security
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0006](0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Decision

EIP-1271 signature verification threads an `Eip1271VerificationCache`
trait through `verify_eip1271_signature_cached`. The trait is defined in
`cow-sdk-contracts` so the function that consumes it does not pull a
reverse dependency on the signing crate, and it is re-exported from
`cow-sdk-signing::cache` where the default implementations live. Two
canonical implementations ship: `NoopEip1271VerificationCache`
(zero-sized, always misses, used when memoization is not wanted) and
`InMemoryEip1271VerificationCache` (bounded-capacity, TTL-expiring, used
when repeated probes of the same verifier and digest are expected). The
cache is a parameter on every call; the function never silently
memoizes without a caller-supplied cache argument.

## Why

A single EIP-1271 verification probe is an on-chain read: it costs a
full JSON-RPC round trip to a smart-contract wallet every time. Protocol
workflows that re-check the same `(verifier, digest)` pair (composable
orders replaying a conditional, flash-loan flows revalidating a staged
order, bridging flows replaying a signed intent) pay that cost per
probe unless the SDK offers a safe memoization boundary. A default-off
cache keeps the simple path unchanged; an explicit cache argument keeps
the security contract visible at every call site. Putting the trait in
`cow-sdk-contracts` rather than `cow-sdk-signing` preserves the
dependency direction (signing depends on contracts, never the reverse),
and re-exporting it from signing keeps the ergonomic shape consumers
expect — the trait shows up next to the cache implementations, not two
crates away.

## Must Remain True

- Public surface: `Eip1271VerificationCache` defines a narrow positive-only
  set trait with
  `contains_valid(verifier: Address, digest: [u8; 32], signature_hash: [u8; 32]) -> bool`
  and
  `record_valid(verifier: Address, digest: [u8; 32], signature_hash: [u8; 32])`
  as the sole methods, and `Send + Sync + 'static` as the bound (see the
  2026-05-28 amendment for the move from `get`/`put` to this shape). The
  key is the full probe identity including the signature hash; there is
  no `bool` value and no negative cache. The function
  `verify_eip1271_signature_cached` takes `&impl Eip1271VerificationCache`
  as a required parameter — there is no overload that defaults the cache.
  `NoopEip1271VerificationCache` is always available and carries no
  dependencies; `InMemoryEip1271VerificationCache` ships behind the
  opt-in `in-memory-cache` feature (default off). Third-party impls
  (Redis, bounded LRU, Postgres) are expected to live in downstream code.
- Runtime and support: the cache records a probe only on a successful
  magic-value match (`Ok(())`). A magic-value mismatch and every other
  error class (transport failure, missing contract code, serialization
  error, hex decode error, provider error) are never recorded — those
  probes re-hit the chain on the next call, so a transient network
  failure cannot pin a signer into a "rejected" state and a not-yet-valid
  signature that becomes valid on-chain within the TTL is never blocked by
  a stale negative entry. The TTL is the second safety rail:
  `InMemoryEip1271VerificationCache` expires entries after five minutes by
  default so the cache never pretends to be an authoritative view of
  mutable on-chain state, and on-chain settlement re-checks the signature
  regardless.
- Validation and review: a Noop miss contract test asserts every
  `get` against `NoopEip1271VerificationCache` returns `None`. A TTL
  contract test asserts `InMemoryEip1271VerificationCache` drops an
  expired entry on the next probe. A capacity contract test asserts
  past-capacity inserts evict the oldest entry. A thread-safety contract
  test drives concurrent inserts against the same key space and asserts
  no write is lost. Every caller in the signing, trading, examples, and
  e2e surfaces compiles against the three-parameter shape.
- Cost: one trait, two canonical implementations, and one new
  `parking_lot` dependency on `cow-sdk-signing` for the
  `InMemoryEip1271VerificationCache` lock. Callers that do not want
  caching pass `&NoopEip1271VerificationCache::default()` and pay zero
  allocation or synchronization overhead.

## Alternatives Rejected

- Make the cache an implicit global: simpler at the call site, but
  loses the per-instance scoping that keeps the SDK runtime-neutral
  (see ADR 0006) and makes audit boundaries unclear.
- Cache every error class, not only `Ok(())` and
  `Eip1271MagicValueMismatch`: catches more repeated work, but pins
  transient transport failures into the cache and inverts the
  security contract.
- Define the trait in `cow-sdk-signing` and have `cow-sdk-contracts`
  depend on it: matches the ergonomic expectation, but inverts the
  established dependency direction and forces contracts to carry a
  reverse edge no other trait needs.
- Put the cache behind a method overload (`verify` without a cache,
  `verify_cached` with one): shorter default path, but makes the
  caching contract easy to skip silently and splits the call-site
  vocabulary into two near-duplicates.

## Links

- [Architecture](../architecture.md)
- [Verification Guide](../verification-guide.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0006](0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md)

**Proven by:**

- [EIP-1271 Verification Cache Audit](../audit/eip1271-verification-cache-audit.md)

## Amendment 2026-05-22: canonical primitive layer (per ADR 0052)

The `verifier: Address` parameter on `Eip1271VerificationCache::get` and
`Eip1271VerificationCache::put` resolves through the cow-owned
`#[repr(transparent)]` newtype around `alloy_primitives::Address` per
[ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md). The
`digest: [u8; 32]` parameter stays a raw fixed-size byte array on the
trait signature so the trait does not couple to a specific cow newtype
choice; callers that already hold a typed `Hash32` cross the boundary
via `Hash32::as_alloy()` plus `.into()` or via the
`#[repr(transparent)]` layout guarantee on the cow newtype.

## Amendment 2026-05-26: trading quote cache reuses the cache primitive pattern

The `cow-sdk-trading` crate ships an `InMemoryQuoteCache` reference
implementation of its `QuoteCache` trait that mirrors the cache
primitive pattern this ADR established for
`InMemoryEip1271VerificationCache`. The shared pattern covers:

- `parking_lot::RwLock<HashMap<K, V>>` as the storage primitive.
- A `Clock` trait with `now(&self) -> Instant` plus a default
  `SystemClock` ZST and a blanket `Fn() -> Instant` impl, exposed via a
  `with_clock` constructor for deterministic TTL tests.
- A capacity bound enforced through an oldest-first
  `min_by_key(inserted_at)` scan on every insert past the bound, with
  the trade-off documented as "Eviction Trade-Off" in the rustdoc.
- A TTL bound enforced through
  `now.duration_since(inserted_at) > self.ttl`, so entries stay valid
  at the exact boundary and expire strictly after it.
- A 300-second default TTL exposed as a public constant
  (`DEFAULT_QUOTE_CACHE_TTL`).
- A wasm32-compatible time source via `web_time::Instant` behind the
  same `cfg(target_arch = "wasm32")` arm both crates already use.
- A wasm-target contract test asserting round-trip plus
  controlled-clock TTL boundary on `wasm32-unknown-unknown`.

The EIP-1271-specific conservative-cache semantics (only `Ok(())` and
`Eip1271MagicValueMismatch` cached, every other error class never
cached) remain scoped to `InMemoryEip1271VerificationCache` and the
`verify_eip1271_signature_cached` call shape — the trading quote cache
caches every result the `QuoteCache` trait passes through `insert`
because the trading flow caller already decides what is safe to
memoize before it calls `insert`. The pattern primitive is shared; the
caching policy is not.

The trading-side default capacity is `256` rather than the signing-side
`1024` because the trading key fan-out (chain × env × token-pair × side
× amount × balance source/destination) is narrower than the EIP-1271
fan-out (verifier × digest). Both values stay tunable through their
respective two-argument `new(ttl, capacity)` constructors.

No new transitive dependency is added by the extension; `parking_lot`
was already the cache lock primitive this ADR named for the signing
crate, and is now also a direct dependency of `cow-sdk-trading`. The
trading-side lookup path takes a write guard rather than the
signing-side read-then-write split because the trading cache preserves
the existing lazy-expiry-on-lookup property: changing it to a read-only
hot path would silently shift observable behaviour for downstream code
that relies on lookup-driven eviction.

> Superseded by the 2026-05-28 amendment below: the trading `QuoteCache`
> is removed.

## Amendment 2026-05-28: signature-keyed positive-only cache, feature gate, and quote-cache removal

The cache contract was tightened in three ways and the sibling quote cache
was removed.

- **The key now includes the signature.** The trait keys on the full probe
  identity `(verifier, digest, signature_hash)`, where `signature_hash` is
  the `keccak256` of the signature bytes. The on-chain
  `isValidSignature(hash, signature)` verdict, and the upstream off-chain
  signature validator, are functions of the signature as well as the
  digest. The previous `(verifier, digest)` key could return a verdict
  recorded for a *different* signature on the same digest; the
  `verify_eip1271_signature_cached` helper now folds the signature hash
  into the key before consulting the cache.
- **Positive-only set semantics.** The trait is now `contains_valid` /
  `record_valid` rather than `get` / `put` over a `bool`. Only `Ok(())`
  outcomes are recorded; a magic-value mismatch is never recorded. This
  completes the safety goal the original decision already stated — that a
  stale negative "cannot block a signer whose on-chain state has since
  changed" — which caching a genuine `Eip1271MagicValueMismatch` did not
  fully honour. A `contains_valid` miss now means "unknown", never "known
  invalid", so a pre-sign or staged order that becomes valid on-chain
  within the TTL is never blocked by a stale negative entry. Negative
  caching is unrepresentable in the trait.
- **The in-memory implementation sits behind a feature.** The trait and
  the dependency-free `NoopEip1271VerificationCache` are always available;
  the capacity-bounded, TTL-respecting `InMemoryEip1271VerificationCache`
  and its `Clock` machinery now sit behind the opt-in `in-memory-cache`
  feature (default off). That feature is the only reason the signing crate
  pulls `parking_lot` (and, on `wasm32`, `web-time`), so the default build
  and the default wasm bundle carry neither. The facade re-exports
  `InMemoryEip1271VerificationCache` only under its own matching feature.

This supersedes the 2026-05-26 amendment above: the `cow-sdk-trading`
`QuoteCache` trait, its `QuoteCacheKey`, and its `NoopQuoteCache` /
`InMemoryQuoteCache` implementations are removed. That seam was never
consulted by the quote flow, its key omitted quote-determining inputs (the
effective app-data document — including partner fee and hooks — and the
price-quality variant), and a quote's economic value is intrinsically
time-sensitive in a way a five-minute TTL cannot bound without an
authoritative on-chain re-check. The shared cache *primitive* pattern
(lock, clock seam, capacity bound, TTL boundary, wasm time source) remains
documented for the EIP-1271 cache; any future cache built on it must carry
an explicit argument that its key contains every input the cached value
depends on.
