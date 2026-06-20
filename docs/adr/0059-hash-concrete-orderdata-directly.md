# ADR 0059: Hash The Concrete OrderData Directly And Remove The Contracts-Layer Order Type

- Status: Accepted
- Date: 2026-05-30
- Last reviewed: 2026-05-30
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: contracts, core, hashing, dto, surface
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0016](0016-split-sell-and-buy-token-balance-enums.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Decision

EIP-712 order hashing and UID derivation operate directly on the concrete
`cow_sdk_core::OrderData`. `cow_sdk_contracts::hash_order`,
`compute_order_uid`, and the cancellation hashers take `&OrderData` and map it
straight onto the macro-emitted, crate-internal `sol_types::Order` codec
struct. The contracts crate defines no public order type: the former
`cow_sdk_contracts::Order`, `cow_sdk_contracts::NormalizedOrder`, and the
generated `GPv2Order` `sol!` re-export are removed, along with the
optional-to-concrete normalization step they required.

The earlier legacy compatibility shims that produced protocol-incorrect digests
— `OrderModel`, `QuoteModel`, `hash_order_for_contract`, `uid_for_contract`, and
`compatibility_order` — are removed for the same reason: they zeroed
`sell_amount`, `buy_amount`, `valid_to`, and `fee_amount` before hashing,
detaching the digest from the order's real economics. The concrete `OrderData`
is therefore the sole supported order-identity input.

## Why

`OrderData` is already concrete — every field, including `receiver` and the two
token-balance enums, has a single canonical value with no optional wire form.
The intermediate `Order` (optional receiver and balances) and `NormalizedOrder`
(filled-in) types existed only to re-derive that concrete shape, so for a
concrete input the normalization round-trip was value-identity: it computed
nothing and only widened the public surface and duplicated the field set three
times. Collapsing to one concrete type matches the upstream services
`OrderData` one-to-one, removes the duplicated logic, and leaves a single struct
for a reviewer to audit against the protocol type hash.

## Must Remain True

- Public surface: the contracts crate exposes order hashing and UID helpers over
  `cow_sdk_core::OrderData` only; no `Order`, `NormalizedOrder`, or `GPv2Order`
  order type is re-exported, and the EIP-712 codec struct stays crate-internal.
- Runtime and support: a `receiver` of `address(0)` is hashed verbatim as the
  protocol's pay-to-owner (`RECEIVER_SAME_AS_OWNER`) sentinel; the general hash
  path never rejects it. The eth-flow construction path keeps its own
  `ContractsError::ZeroReceiver` guard (ADR 0020), because an eth-flow order
  owner is the contract itself.
- Validation and review: the per-chain digest rows in
  `crates/contracts/tests/order_digest_parity_contract.rs` and the upstream
  Sepolia anchor in `crates/contracts/tests/order_contract.rs` stay
  byte-identical, and the fixture type hash matches `order_eip712_type_hash()`.
  `PROP-CON-023` and `PROP-CON-006` register the invariants.
- Legacy shims removed: no `OrderModel`, `QuoteModel`, `hash_order_for_contract`,
  `uid_for_contract`, or `compatibility_order` helper is re-exported from
  `cow-sdk-core`, `cow-sdk-contracts`, or the facade; every surviving caller
  constructs the canonical `OrderData`, so a digest always carries the real
  amount, fee, and expiry fields.
- Cost: an optional-field order wire shape, if one is ever introduced, must be
  modelled at its own boundary rather than reusing the hashing input.

## Alternatives Rejected

- Keep `Order` plus `NormalizedOrder`: preserves a three-type topology and a
  normalization step that is value-identity for a concrete order, widening the
  public surface for no behavioral guarantee.
- Seal `NormalizedOrder` behind a private constructor: reduces misuse but still
  ships two extra public types and the duplicated field set.

## Links

- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [Architecture](../architecture.md)
- `PROP-CON-006` and `PROP-CON-023` in [PROPERTIES.md](../../PROPERTIES.md)
