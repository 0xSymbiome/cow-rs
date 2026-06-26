# ADR 0014: Pluggable EIP-1271 Verification Cache With Conservative Caching Semantics

- Status: Accepted
- Date: 2026-04-21
- Last reviewed: 2026-06-24
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: signing, eip1271, caching, security
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0006](0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Decision

EIP-1271 signature verification threads an `Eip1271Cache` trait through
`verify_eip1271_signature_cached`, which takes the cache as a **required**
parameter — it never silently memoizes without a caller-supplied cache. A thin
`verify_eip1271_signature` convenience (no cache parameter) delegates to the
cached form with `NoopEip1271Cache`, so the two public entry points are explicit
on both sides: the cached form demands a cache argument, and the uncached form is
named for what it does.

The trait `Eip1271Cache` and the dependency-free `NoopEip1271Cache` are defined
in `cow-sdk-contracts` so the contracts-level verify functions take no reverse
dependency on signing, and are re-exported through `cow-sdk-signing::cache`. The
SDK ships no concrete cache: a consumer that wants memoization implements the
two-method trait over the store of its choice.

`Eip1271Cache` is a positive-only set keyed on the full probe identity
`(verifier, digest, signature_hash)` — `contains_valid(...) -> bool` and
`record_valid(...)`, `Send + Sync + 'static`. There is no `bool` value and no
negative cache; negative caching is unrepresentable in the trait. Only a
successful magic-value match (`Ok(())`) is recorded. Every other outcome — a
magic-value mismatch, a transport failure, missing contract code, a decode error
— is never recorded and re-hits the chain on the next probe.

## Amendment (2026-06-24): in-memory battery withdrawn

An `InMemoryEip1271Cache` battery previously shipped from `cow-sdk-signing`
behind an `in-memory-cache` feature. It was withdrawn. No in-tree caller consumes
the verifier, so the battery had no exercised consumer; and the replay workloads
that motivate caching (composable orders, flash-loan flows, bridging) reach
validity through order pre-interactions that a bare `isValidSignature` probe does
not simulate, so a correct re-verifier for those flows needs the
pre-interaction-aware verification the upstream order-placement service runs, not
a memoized bare probe. The trait seam stays; a concrete cache returns
co-located with a consumer that needs it. Removing the battery also dropped the
`parking_lot` and `web-time` dependencies from `cow-sdk-signing`.

## Why

A single EIP-1271 verification probe is an on-chain read: a full JSON-RPC round
trip to a smart-contract wallet every time. Workflows that re-check the same
`(verifier, digest, signature)` probe (composable orders replaying a conditional,
flash-loan flows revalidating a staged order, bridging flows replaying a signed
intent) pay that cost per call unless the SDK offers a safe memoization boundary.
A required cache argument keeps the security contract visible at every call site.
The positive-only contract guarantees a `contains_valid` miss means "unknown",
never "known invalid", so a pre-sign or staged order that becomes valid on-chain
is never blocked by a stale negative entry, and a transient network failure can
never pin a signer into a rejected state. Putting the trait in `cow-sdk-contracts`
rather than `cow-sdk-signing` preserves the dependency direction (signing depends
on contracts, never the reverse).

## Must Remain True

- Public surface: `Eip1271Cache` is a positive-only set with
  `contains_valid(verifier: Address, digest: [u8; 32], signature_hash: [u8; 32])
  -> bool` and `record_valid(...)` as the sole methods, `Send + Sync + 'static`.
  The key is the full probe identity including the signature hash; there is no
  negative cache. `verify_eip1271_signature_cached` takes `&impl Eip1271Cache` as
  a required parameter; the cache-free `verify_eip1271_signature` is a thin
  wrapper that passes `NoopEip1271Cache`. `NoopEip1271Cache` is always available
  and dependency-free. The SDK ships no concrete cache; consumers implement the
  trait to memoize.
- Runtime and support: the cache records a probe only on `Ok(())`. A magic-value
  mismatch and every error class (transport failure, missing contract code,
  serialization error, hex decode error, provider error) are never recorded —
  those re-hit the chain on the next call, so a transient failure cannot pin a
  signer into a "rejected" state and a not-yet-valid signature that becomes valid
  on-chain is never blocked by a stale entry. A cache implementation's own TTL, if
  any, bounds the only residual staleness — an optimistic VALID surviving an
  on-chain revocation until expiry. The cache is never an authoritative view of
  mutable on-chain state, and on-chain settlement re-checks the signature
  regardless.
- Validation and review: contract tests assert that the uncached path performs one
  verifier dispatch, that a cache hit returns without a chain call, that only a
  magic-value match is recorded (keyed on the signature), and that every error
  class records nothing. Every caller in the signing, trading, examples, and e2e
  surfaces compiles against the cache-required `verify_eip1271_signature_cached`
  shape.
- Cost: one trait and one always-available `NoopEip1271Cache`, with no extra
  dependency on `cow-sdk-signing`. Callers that do not want caching pass
  `&NoopEip1271Cache::default()` (or call `verify_eip1271_signature`) and pay zero
  allocation or synchronization overhead; callers that do implement the trait over
  their own store.

## Alternatives Rejected

- Make the cache an implicit global: simpler at the call site, but loses the
  per-instance scoping that keeps the SDK runtime-neutral (see
  [ADR 0006](0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md))
  and makes audit boundaries unclear.
- Record every outcome, not only `Ok(())`: catches more repeated work, but pins
  transient transport failures and magic-value mismatches into the cache and
  inverts the security contract — a signer whose on-chain state later changes
  would stay blocked by a stale entry.
- Define the trait in `cow-sdk-signing` and have `cow-sdk-contracts` depend on
  it: matches the ergonomic expectation, but inverts the established dependency
  direction and forces contracts to carry a reverse edge no other trait needs.
- A silent overload that defaults the cache (one `verify` that memoizes invisibly
  when a cache happens to be in scope): makes the caching contract easy to skip
  without noticing, and splits the call-site vocabulary into two near-duplicates.
  The shipped pair avoids this by requiring an explicit cache argument on the
  cached entry and naming the uncached entry for exactly what it does.
- Ship a batteries-included in-memory cache by default: convenient, but it would
  be unexercised surface (no in-tree consumer) and would memoize a bare-probe
  verdict that diverges from the pre-interaction-aware verification the upstream
  service runs for the replay workloads above. The trait seam lets a consumer that
  needs memoization supply an implementation aimed at the verdict it actually
  trusts.

## Links

- [Architecture](../architecture.md)
- [Verification](../verification.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0006](0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md)

**Proven by:**

- [EIP-1271 Verification Cache Audit](../audit/eip1271-verification-cache-audit.md)
