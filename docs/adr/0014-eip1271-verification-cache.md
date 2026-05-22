# ADR 0014: Pluggable EIP-1271 Verification Cache With Conservative Caching Semantics

- Status: Accepted (amended)
- Date: 2026-04-21
- Last reviewed: 2026-05-22
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: signing, eip1271, caching, security
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0006](0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Decision

EIP-1271 signature verification threads an `Eip1271VerificationCache`
trait through `verify_eip1271_signature_async`. The trait is defined in
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

- Public surface: `Eip1271VerificationCache` defines a narrow trait with
  `get(verifier: Address, digest: [u8; 32]) -> Option<bool>` and
  `put(verifier: Address, digest: [u8; 32], result: bool)` as the sole
  methods, and `Send + Sync + 'static` as the bound. The function
  `verify_eip1271_signature_async` takes `&impl Eip1271VerificationCache`
  as a required parameter — there is no overload that defaults the
  cache. `NoopEip1271VerificationCache` and
  `InMemoryEip1271VerificationCache` are the shipped canonical impls;
  third-party impls (Redis, bounded LRU, Postgres) are expected to live
  in downstream code.
- Runtime and support: the cache stores `true` only on a successful
  magic-value match and `false` only on a verified
  `Eip1271MagicValueMismatch` outcome. Every other error class
  (transport failure, missing contract code, serialization error, hex
  decode error, provider error) is never cached — those probes re-hit
  the chain on the next call so a transient network failure cannot pin a
  signer into a permanent "rejected" state and a stale `false` cannot
  block a signer whose on-chain state has since changed. The TTL is the
  second safety rail: `InMemoryEip1271VerificationCache` expires entries
  after five minutes by default so the cache never pretends to be an
  authoritative view of mutable on-chain state.
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
