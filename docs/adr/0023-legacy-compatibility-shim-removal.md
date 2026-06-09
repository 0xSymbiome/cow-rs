# ADR 0023: Remove Legacy Compatibility Shims That Produced Protocol-Incorrect Order Digests

- Status: Accepted (amended)
- Date: 2026-04-24
- Last reviewed: 2026-05-30
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: contracts, core, hashing, compatibility
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Decision

`cow_sdk_contracts::hash_order` and `cow_sdk_contracts::compute_order_uid`
operating on the concrete `cow_sdk_core::OrderData` are the only supported
contracts-boundary order-identity helpers. Callers construct
`cow_sdk_core::OrderData` and compute digests or UIDs directly from that
canonical shape. The legacy `OrderModel`, `QuoteModel`,
`hash_order_for_contract`, `uid_for_contract`, and `compatibility_order`
shim surface is removed from the public API.

## Why

The legacy compatibility path converted `OrderModel` into a
contracts-crate order by zeroing `sell_amount`, `buy_amount`,
`valid_to`, and `fee_amount` before hashing. That made the resulting
digest detached from the actual order economics and expiry. Order
digests are protocol identities, so a helper that emits structurally
valid but semantically meaningless digests is more dangerous than a
missing helper. Removing the compatibility surface is safer than
preserving an incorrect digest path, because the canonical typed
unsigned-order boundary already carries the full order payload needed
for protocol-correct hashing.

## Must Remain True

- Public surface: contracts-boundary digest and UID computation flows
  only through the concrete `OrderData`, with no compatibility model or
  shim helper re-exported from `cow-sdk-core`, `cow-sdk-contracts`, or
  the `cow-sdk` facade.
- Runtime and support: every surviving caller constructs the canonical
  typed order shape before hashing, so the digest always includes the
  real amount, fee, and expiry fields reviewed by the protocol.
- Validation and review:
  `crates/contracts/tests/order_contract.rs::canonical_unsigned_order_path_matches_upstream_signing_fixture_digest_and_uid`
  pins the canonical Sepolia digest and UID against the upstream signing
  fixture anchor, and `crates/contracts/tests/order_digest_parity_contract.rs`
  locks the per-chain digest rows; the removed shim symbols are absent
  from the public surface. `PROP-CON-006` in
  [PROPERTIES.md](../../PROPERTIES.md) registers the invariant.
- Cost: callers that used the removed compatibility surface must
  construct the canonical typed unsigned order before hashing or UID
  packing.

## Alternatives Rejected

- Promote `OrderModel` into a full amount-bearing order shape: would
  preserve a second contracts-boundary order model with overlapping
  semantics, which keeps compatibility debt alive when the canonical
  `OrderData` path already exists.
- Keep the shim but mark it deprecated: leaves a known incorrect digest
  path callable and review-visible, which is unacceptable for a
  protocol-identity helper.
- Delete only the contracts helpers and retain `QuoteModel`: reduces part
  of the bad surface but still preserves the legacy compatibility family
  in `cow-sdk-core` and the SDK facade.

## Links

- [Architecture](../architecture.md)
- [Verification Guide](../verification.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md)
- `PROP-CON-006` in [PROPERTIES.md](../../PROPERTIES.md)

## Amendment 2026-05-22: canonical primitive layer (per ADR 0052)

`cow_sdk_contracts::hash_order` and
`cow_sdk_contracts::compute_order_uid` route the canonical
`OrderData` digest path through
`alloy_sol_types::SolStruct::eip712_signing_hash` on the macro-emitted
internal `crate::order::sol::Order` codec struct per
[ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md). The
EIP-712 hashing seam uses `alloy_sol_types::Eip712Domain` (constructed
through the `TypedDataDomain::into_alloy_domain()` adapter on the cow
`TypedDataDomain` struct) and ultimately consumes
`alloy_primitives::keccak256` for every digest. The byte-typed identity
fields on the cow `OrderData` struct (`Address`, `Amount`, `OrderUid`) are
cow-owned `#[repr(transparent)]` newtypes around their alloy primitives
per ADR 0052; the digest output is byte-identical against every pinned
upstream signing fixture.

## Amendment 2026-05-30: hash the concrete `OrderData` directly (per ADR 0059)

The supported path no longer threads a contracts-crate order type. The
public `cow_sdk_contracts::Order` and `cow_sdk_contracts::NormalizedOrder`
types are removed; `hash_order` and `compute_order_uid` take the concrete
`cow_sdk_core::OrderData` by reference and map it straight onto the
crate-internal `sol_types::Order` codec struct, with no optional-to-concrete
normalization step. This preserves every guarantee in this ADR — the shim
surface stays removed and the digest still carries the full order economics —
while collapsing the order-type topology to a single concrete type. See
[ADR 0059](0059-hash-concrete-orderdata-directly.md).
