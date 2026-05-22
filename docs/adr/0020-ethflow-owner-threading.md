# ADR 0020: EthFlow Transaction Bundle Carries The Signer-Derived Owner For Pre-HTTP Validation

- Status: Accepted (amended)
- Date: 2026-04-22
- Last reviewed: 2026-05-22
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: trading, eth-flow, validation, client-side, defense-in-depth
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0011](0011-typed-amount-boundary-and-typestate-ready-state-construction.md), [ADR 0015](0015-client-side-order-bounds-validator.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Decision

The EthFlow transaction bundle returned by
`cow_sdk_trading::get_eth_flow_transaction_async` exposes a typed
`from: cow_sdk_core::Address` field carrying the signer-derived
owner resolved during transaction construction via
`AsyncSigner::get_address`. The submission seam
`post_sell_native_currency_order_async` builds its pre-HTTP
validation preview from `tx.from.clone()`, not from
`tx.order_to_sign.receiver.clone()`. Receiver continues to carry
the payout-recipient semantic unchanged; owner is always
signer-derived on the EthFlow path. `OrderBoundsValidator::validate`
checks `app_data_signer` against `OrderCreation.from`, and the
submission preview therefore compares the declared app-data signer
against the owner identity rather than against the payout address.

## Why

Owner and receiver are distinct identities on the EthFlow protocol
surface: a caller may legitimately direct the native-currency
payout to a recipient that is not the signer. The client-side
validator introduced by ADR 0015 enforces the reviewed services
protocol-invariant matrix before any bytes cross the wire, and
that matrix compares the declared app-data signer against the
`OrderCreation.from` owner. Seeding the preview from the receiver
conflates the two identities: a legitimate EthFlow with a custom
receiver and a matching app-data signer raises a false
`AppdataFromMismatch`, and a tampered flow whose crafted app-data
signer equals the receiver slips past the check entirely. The
owner is already resolved inside
`get_eth_flow_transaction_async` through
`signer.get_address().await`; threading it onto the returned
bundle closes both directions without a second signer round-trip.
Surfacing the field as a public, typed member of
`EthFlowTransaction` keeps the identity explicit for reviewers and
lets the submission seam read the owner without reaching back into
the signer.

## Must Remain True

- Public surface: `cow_sdk_trading::EthFlowTransaction` carries a
  public `from: cow_sdk_core::Address` field holding the
  signer-derived owner captured at transaction construction. The
  struct remains `#[non_exhaustive]`, and the
  `EthFlowTransaction::new` constructor accepts the owner as a
  required parameter so every construction path populates the
  field explicitly. `get_eth_flow_transaction_async` and its
  synchronous companion `get_eth_flow_transaction` populate
  `from` with the resolved signer address before returning the
  bundle. Receiver semantics are unchanged: `tx.order_to_sign.receiver`
  remains the payout identity and may legitimately differ from
  `tx.from`.
- Runtime and support: `post_sell_native_currency_order_async`
  reads `let preview_from = tx.from.clone()` when building the
  preview `OrderCreation` for `OrderBoundsValidator::validate`.
  No receiver-as-owner fallback remains on the submission path.
  The owner is resolved exactly once inside
  `get_eth_flow_transaction_async` and forwarded onto the
  bundle; the submission seam does not call
  `signer.get_address()` a second time. The validator continues
  to compare `app_data_signer` against `order.from`, so the
  typed `AppdataFromMismatch { appdata_signer, from }` payload
  carries the owner identity in both the success and the failure
  surfaces.
- Validation and review: regression coverage in
  `crates/trading/tests/post_contract.rs` exercises three
  invariants explicitly — an EthFlow submission with
  `receiver != owner` and a matching app-data signer succeeds;
  an EthFlow submission with `receiver != owner` and a
  mismatched app-data signer rejects as `AppdataFromMismatch
  { appdata_signer, from }` with `from` reporting the owner
  rather than the receiver; a default-receiver EthFlow
  submission with a matching signer continues to succeed. The
  EthFlow-aware invocation of the validator
  (`is_eth_flow: true`) still exercises the zero-amount,
  same-token, owner-mismatch, and lifetime checks introduced
  by ADR 0015.
- Cost: one new typed field on `EthFlowTransaction`, one
  parameter added to the public `EthFlowTransaction::new`
  constructor, one reuse of the existing local `from` inside
  `get_eth_flow_transaction_async` at the return site, and one
  single-line change inside
  `post_sell_native_currency_order_async` replacing the prior
  receiver-as-preview-owner assignment. No change to the public
  validator surface, the payout semantics, or the EthFlow
  transaction encoding.
- Construction-time receiver invariant:
  `cow_sdk_contracts::EthFlowOrderData::new` and
  `EthFlowOrderData::from_unsigned_order` return
  `Result<Self, ContractsError>`, rejecting
  `receiver == Address::ZERO` with `ContractsError::ZeroReceiver`.
  The same predicate is shared with
  `cow_sdk_contracts::order::hash::normalize_order` via a private
  `reject_zero_receiver` helper, so the receiver-rejection rule is
  expressed in one place across the contracts crate. The shared
  rule mirrors the deployed `CoWSwapEthFlow` contract's
  `ReceiverMustBeSet()` revert (selector `0xefc9ccdf`), raised
  from `EthFlowOrder.toCoWSwapOrder` at the calldata-construction
  step in both the `createOrder` and `invalidateOrder` write
  paths through the shared library call. The selector derivation
  is locked by the unit test
  `zero_receiver_invariant_matches_ethflow_on_chain_revert_selector`
  in `crates/contracts/src/eth_flow.rs`, which re-derives
  `keccak256("ReceiverMustBeSet()")[0..4]` via
  `alloy_primitives::keccak256` and asserts equality with the
  hardcoded byte sequence `0xef 0xc9 0xcc 0xdf`. The proptest
  `ethflow_order_data_new_rejects_zero_receiver_iff_address_is_zero`
  in `crates/contracts/tests/property_contract.rs` covers the
  bidirectional invariant under the full 2^160 address space.

## Alternatives Rejected

- Call `signer.get_address()` a second time inside the
  submission seam: duplicates the signer round-trip that the
  transaction-construction path already performed, pays the
  cost twice on HSM-backed signers, and leaves the submission
  seam coupled to the signer interface instead of reading the
  already-captured identity from the typed bundle.
- Add an `owner` field on `OrderCreation` and change the
  validator to compare against it instead of `from`: changes
  the wire shape contract for a purely client-side concern and
  diverges from the reviewed services authority where `from`
  has always meant the submission owner.
- Skip the EthFlow validator branch entirely and rely on the
  on-chain call to reject tampered flows: removes the typed
  client-side defense-in-depth layer ADR 0015 commits to, and
  forces every consumer to diagnose failures from the on-chain
  revert rather than from a typed `ClientRejection` payload.
- Reinterpret `receiver` as the owner identity whenever
  `receiver` is set: overloads the payout semantic and
  introduces a context-dependent reading of a field that is
  already well-defined on the reviewed protocol surface.

## Links

- [Architecture](../architecture.md)
- [Verification Guide](../verification-guide.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0011](0011-typed-amount-boundary-and-typestate-ready-state-construction.md)
- [ADR 0015](0015-client-side-order-bounds-validator.md)

**Proven by:**

- [Trading EthFlow Owner Identity Audit](../audit/trading-ethflow-owner-identity-audit.md)
- [Contract Bindings Parity Audit](../audit/contract-bindings-parity-audit.md)

## Amendment 2026-05-22: canonical primitive layer (per ADR 0052)

The `from: cow_sdk_core::Address` field on `EthFlowTransaction` and the
`Address`-typed `from` parameter on `EthFlowTransaction::new` resolve
through the cow-owned `#[repr(transparent)]` newtype around
`alloy_primitives::Address` per
[ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md). The
owner identity carried onto the bundle and read by the native-currency
submission seam preserves the lowercase `0x`-prefixed hex wire form
through the cow-owned `Display`/`Serialize`/`Deserialize` impls on
`Address`.
