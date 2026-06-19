# ADR 0070: On-Chain Transaction Helper Boundary And Native-Asset Wrapping

- Status: Accepted
- Date: 2026-06-19
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: trading, contracts, public-surface, scope
- Related: [ADR 0002](0002-dedicated-trading-orchestration-crate.md), [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md), [ADR 0069](0069-layered-trading-operation-surface-and-signing-free-transport.md)

## Decision

The SDK ships one deterministic, single-call, parity-pinned builder for each
on-chain transaction a trader sends directly: approve the vault relayer, wrap and
unwrap the native asset, place an eth-flow native sell, pre-sign, and cancel
on-chain. It ships no long-running orchestration, no solver settlement path, and no
wallet or provider management.

Native-asset wrapping joins this boundary. `cow-sdk-trading` adds the free functions
`wrap_transaction(chain_id, amount)` and `unwrap_transaction(chain_id, amount)`. Each
resolves the chain's canonical wrapped-native token through
`cow_sdk_core::wrapped_native_token` and returns a `TransactionRequest`. They are
infallible: a typed `SupportedChainId`, a construction-validated `Amount`, and the
fixed `deposit()` / `withdraw(uint256)` calldata leave no failure mode, so — unlike
the signing- and registry-bound builders — they do not return `Result`. The
`cow-sdk-contracts` primitives `wrap_interaction` / `unwrap_interaction` stay public
for interaction composition. The wasm `trading` surface mirrors the helpers as
`buildWrapTx` / `buildUnwrapTx` and exposes `wrappedNativeToken(chainId)` for
wrap-pair detection and display.

## Why

A trader holding native currency wraps it to hold and trade the wrapped form, and
unwraps to convert back. Eth-flow wraps on-chain during order creation, but the
standalone wrap and treasury paths do not, so without a builder the step falls to
each consumer. Re-deriving the per-chain wrapped-native address and the WETH9
`deposit()` / `withdraw(uint256)` calldata in every consumer is error-prone and can
drift from the address the eth-flow order rewriting already resolves. One tested,
byte-locked builder removes that risk and keeps a single address source across the
order path and the standalone path.

Wrapping is the same class of artifact as the approve and eth-flow builders the
surface already carries: a small, fixed on-chain step that recurs across
integrations. It belongs at the on-chain helper boundary, not re-derived beneath it.

## Must Remain True

- Public surface: each on-chain trade step has one single-call transaction builder;
  `wrap_transaction` / `unwrap_transaction` resolve the address from the typed chain
  and stay infallible; the `wrap_interaction` / `unwrap_interaction` primitives stay
  public and parity-pinned.
- Runtime and support: the helpers perform no I/O and add no orchestration,
  settlement, retry, or wallet management; native-currency selling continues to use
  eth-flow rather than a required manual wrap.
- Validation and review: the `deposit()` / `withdraw(uint256)` selectors stay
  byte-locked (`PROP-CON-020`); the resolved target is the same wrapped-native
  address the eth-flow order rewriting uses.

## Alternatives Rejected

- Leave wrapping to consumers: forces every consumer to re-derive the WETH9 calldata
  and a per-chain address map without test coverage, for no surface saving over an
  existing binding.
- Hide the wrap behind an order-flow auto-wrap: violates the off-chain orchestration
  boundary and conflates a wallet-funding step with order placement.
- A fluent wrap builder: wrapping carries no same-typed transposable pair, so it gets
  no fluent builder (ADR 0069).

## Links

- [Architecture](../architecture.md)
- [ADR 0002](0002-dedicated-trading-orchestration-crate.md)
- [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md)
- [ADR 0069](0069-layered-trading-operation-surface-and-signing-free-transport.md)
