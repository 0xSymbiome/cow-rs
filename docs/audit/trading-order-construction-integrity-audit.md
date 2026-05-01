# Trading Order Construction Integrity Audit

Status: Current
Last reviewed: 2026-05-01
Owning surface: `cow-sdk-trading` order assembly, injected-orderbook builder terminal parity, and recoverable-signature posting boundary
Refresh trigger: Changes to quote-derived or direct order construction, `TradingSdk` builder terminals with injected orderbooks, or recoverable-signature posting validation
Related docs:
- [ADR 0002](../adr/0002-dedicated-trading-orchestration-crate.md)
- [Architecture](../architecture.md)
- [Verification Guide](../verification-guide.md)
- [Verification Matrix](../verification-matrix.md)

## Scope

This audit covers:

- order construction and submission helpers in `cow-sdk-trading`
- receiver fallback when the caller leaves the receiver unset or set to the
  zero address
- quote-derived order assembly and direct posting flows
- `TradingSdk` builder terminals that accept injected orderbook context
- local signature validation before orderbook submission

It does not cover browser-wallet session management, approval flows, or
unrelated leaf-crate transport policy.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Order construction balance semantics | Preserve reviewed `sellTokenBalance` and `buyTokenBalance` values end to end | Conforms |
| Receiver fallback | Signing payload construction falls back to the effective `from` address when `receiver` is unset or zero-address | Conforms |
| `TradingSdk` injected-orderbook terminals | Typestate and total-input builder terminals enforce one fail-fast authority contract | Conforms |
| Recoverable signature posting | Reject explicit owner or signer mismatch before submission | Conforms |

## Current Contract

### Balance Semantics

`cow-sdk-trading` preserves reviewed `sellTokenBalance` and
`buyTokenBalance` semantics across quote overrides, quote-derived order
assembly, direct order construction, signing payload generation, and final
submission. Non-default balance selections remain part of the signed order
contract rather than being normalized during helper composition.

### Receiver Fallback

`get_order_to_sign` treats both an absent receiver and the zero address as
unset and emits the effective `from` address as the receiver in the signing
payload. This matches the reviewed upstream helper behavior and avoids signing
an order with a placeholder receiver when caller intent is to receive proceeds
at the owner address.

### Builder Terminal Parity

Typestate and total-input builder terminals for `TradingSdk` share the same
injected-orderbook validation boundary. If explicit trader or quoter defaults
conflict with the injected orderbook context, SDK construction fails before the
surface is exposed.

### Recoverable Signature Boundary

Posting flows for recoverable signature schemes reject explicit owner or signer
mismatch before app-data upload, signing, or orderbook submission. `PreSign`
and `Eip1271` remain separate non-recoverable contracts.

## Evidence

Primary implementation points:

- `crates/trading/src/error.rs`
- `crates/trading/src/order.rs`
- `crates/trading/src/post.rs`
- `crates/trading/src/quote.rs`
- `crates/trading/src/sdk.rs`
- `crates/trading/src/types.rs`

Primary regression coverage:

- `crates/trading/tests/order_contract.rs`
- `crates/trading/tests/order_contract.rs::order_to_sign_receiver_falls_back_to_from_when_zero_or_unset`
- `crates/trading/tests/post_contract.rs`
- `crates/trading/tests/quote_contract.rs`
- `crates/trading/tests/quote_contract.rs::order_id_collision_retries_with_new_salt_until_success_or_cap`
- `crates/trading/tests/sdk_contract.rs`

Validation surface:

```text
cargo fmt --all --check
cargo test -p cow-sdk-trading
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
