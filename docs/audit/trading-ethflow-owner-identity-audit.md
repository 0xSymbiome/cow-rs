# Trading EthFlow Owner Identity Audit

Status: Current
Last reviewed: 2026-05-26
Owning surface: `cow-sdk-trading` EthFlow submission seam,
including the `EthFlowTransaction` bundle shape, the
`get_eth_flow_transaction` owner resolution, and the
`post_sell_native_currency_order` pre-HTTP validation
preview that feeds `OrderBoundsValidator::validate`.
Refresh trigger: Changes to the `EthFlowTransaction` public
field set or constructor signature; changes to the
`get_eth_flow_transaction` owner resolution; any change
that lets `preview_from` diverge from `tx.from` on the
submission seam; any extension to
`OrderBoundsValidator::validate` that reads a different
identity for the `AppdataFromMismatch` check on the EthFlow
path; changes to the EthFlow-aware invocation of the validator
(the `is_eth_flow: true` skip rule).
Related docs:
- [ADR 0020](../adr/0020-ethflow-owner-threading.md)
- [ADR 0015](../adr/0015-client-side-order-bounds-validator.md)
- [Trading Order-Bounds Validator Audit](trading-order-bounds-validator-audit.md)
- [Contract Bindings Parity Audit](contract-bindings-parity-audit.md) — covers the contracts-crate construction-time receiver invariant adjacent to the trading-layer owner threading
- [Architecture](../architecture.md)

## Scope

This audit covers:

- the `cow_sdk_trading::EthFlowTransaction` bundle and its
  public `from: cow_sdk_core::Address` field
- the `EthFlowTransaction::new` constructor and the owner
  parameter it accepts
- the `get_eth_flow_transaction` helper and the owner
  resolution that populates `EthFlowTransaction.from`
- the `post_sell_native_currency_order` submission seam
  and its `preview_from = tx.from.clone()` read when building
  the preview `OrderCreation` for
  `OrderBoundsValidator::validate`
- the EthFlow-aware invocation of the validator
  (`is_eth_flow: true`) and the owner-versus-receiver
  identity contract on the `AppdataFromMismatch { appdata_signer,
  from }` payload

It does not cover the EthFlow transaction encoding (covered by
the contracts crate), the payout-side `receiver` semantics beyond
the identity clarification, the off-chain cancellation pipeline,
or the orderbook authoritative server-side validation.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Bundle shape | `EthFlowTransaction` carries a public typed `from: Address` field populated at construction | Conforms |
| Owner resolution | `get_eth_flow_transaction` resolves the owner through `Signer::get_address` exactly once and stores the value on the returned bundle | Conforms |
| Submission preview | `post_sell_native_currency_order` reads `preview_from = tx.from.clone()` when building the validator preview; no receiver-as-owner fallback remains | Conforms |
| Identity on rejections | `ClientRejection::AppdataFromMismatch { appdata_signer, from }` reports the owner in `from`, not the payout receiver | Conforms |
| EthFlow-aware invariants | The validator still fires for zero amount, same token, owner mismatch, and lifetime bounds on the EthFlow path; only the native-currency-sentinel sell-token check is skipped | Conforms |
| Receiver semantics | Receiver continues to carry the payout-recipient role and may legitimately differ from owner without triggering false rejections | Conforms |

## Current Contract

### Bundle Shape

`cow_sdk_trading::EthFlowTransaction` is a `#[non_exhaustive]`
struct with `order_id: OrderUid`, `transaction:
TransactionRequest`, `order_to_sign: UnsignedOrder`, and a
typed `from: cow_sdk_core::Address`. The `from` field carries
the signer-derived owner captured at transaction construction
and documents the owner-versus-receiver distinction in its
doc-comment. `EthFlowTransaction::new` accepts the owner as a
required parameter so every construction path populates the
field explicitly.

### Owner Resolution

`get_eth_flow_transaction` resolves the owner through a
single `signer.get_address().await` call near the top of the
helper. The resolved value is threaded into `OrderToSignParams`
for order-body derivation and forwarded onto the returned
`EthFlowTransaction` bundle via the typed `from` field. No
second signer round-trip happens on the submission seam.

### Submission Preview

`post_sell_native_currency_order` reads
`let preview_from = tx.from.clone()` when constructing the
preview `OrderCreation` for
`OrderBoundsValidator::validate`. The previous assignment that
sourced `preview_from` from `tx.order_to_sign.receiver` is gone.
Receiver semantics are unchanged: the `OrderCreation.receiver`
field continues to carry the payout address, and the on-chain
EthFlow encoding continues to honor the receiver through the
`cow_sdk_contracts::eth_flow::EthFlowOrderData` surface.

### Identity On Rejections

`OrderBoundsValidator::validate` compares `app_data_signer`
against `order.from`. Because the preview `from` now carries
the owner, the typed `AppdataFromMismatch { appdata_signer,
from }` payload reports the owner identity in its `from`
field both on success-side consistency checks and on
rejection-side diagnostics. Consumers pattern-matching on the
rejection see the owner identity rather than the payout
identity.

### EthFlow-Aware Invariants

The validator is invoked with `is_eth_flow: true` on the
native-currency submission path. The native-currency-sentinel
sell-token check is skipped (the sentinel is expected on this
path), and every other invariant — zero amount,
same-token, owner mismatch, and lifetime bounds (min and
class-specific max) — still fires. The typed
`ClientRejection` variants surface unchanged from the
non-EthFlow submission seams.

### Receiver Semantics

`tx.order_to_sign.receiver` continues to mean the native-currency
payout recipient. The EthFlow transaction encoding, the on-chain
order hash, and the reviewed services authority all treat the
receiver as payout-only. The only change from the prior state
is the submission-seam preview source: `tx.from` is the owner,
and receiver may legitimately differ from owner without
triggering a false rejection.

## Evidence

Primary implementation points:

- `crates/trading/src/onchain.rs`
- `crates/trading/src/post/native.rs`
- `crates/trading/src/validation.rs`
- `crates/core/src/types/identity.rs` (`Address`)

Primary regression coverage:

- `crates/trading/tests/post_contract.rs`
- `crates/trading/tests/validation_contract.rs`

Validation surface:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p cow-sdk-trading --test post_contract
cargo test -p cow-sdk-trading --test validation_contract
cargo check --workspace --all-features --target wasm32-unknown-unknown
```
