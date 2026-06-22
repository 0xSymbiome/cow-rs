# ADR 0070: On-Chain Transaction Helper Boundary And Native-Asset Wrapping

- Status: Accepted
- Date: 2026-06-19
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: trading, contracts, public-surface, scope
- Related: [ADR 0002](0002-dedicated-trading-orchestration-crate.md), [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md), [ADR 0069](0069-layered-trading-operation-surface-and-signing-free-transport.md)

## Decision

_Amended 2026-06-22: `cow-sdk-contracts` owns the complete gas-free on-chain
transaction builders — `pre_sign_transaction`, `invalidate_order_transaction`, and
`ethflow_create_order_transaction`, each returning a gas-free
`UnsignedTransaction { to, data, value }` (the upstream services `eth::Tx` shape) —
together with the shared override-or-registry target resolver
(`resolve_contract_address`). `cow-sdk-trading` wraps these with signer-bound gas
estimation and submission; `cow-sdk-wasm` surfaces them through its existing
`buildPresignTx` / `buildCancelOrderTx` / `buildSellNativeCurrencyTx` exports. The
pure `wrap_transaction` / `unwrap_transaction` builders and the `setPreSignature` /
`invalidateOrder` call-data encoders also live in `cow-sdk-contracts`; the paragraphs
below reflect that placement._

The SDK ships one deterministic, single-call, parity-pinned builder for each
on-chain transaction a trader sends directly: approve the vault relayer, wrap and
unwrap the native asset, place an eth-flow native sell, pre-sign, and cancel
on-chain. It ships no long-running orchestration, no solver settlement path, and no
wallet or provider management.

Native-asset wrapping joins this boundary. The free functions
`wrap_transaction(chain_id, amount)` and `unwrap_transaction(chain_id, amount)` are
pure and signing-free, so they live in `cow-sdk-contracts` — the lean layer that
already owns the `wrap_interaction` / `unwrap_interaction` primitives — and are
re-exported from `cow-sdk-trading`, leaving the trader-facing free-function surface
unchanged. Each resolves the chain's canonical wrapped-native token through
`cow_sdk_core::wrapped_native_token` and returns a `TransactionRequest`. They are
infallible: a typed `SupportedChainId`, a construction-validated `Amount`, and the
fixed `deposit()` / `withdraw(uint256)` calldata leave no failure mode, so — unlike
the signing- and registry-bound builders — they do not return `Result`. The
`wrap_interaction` / `unwrap_interaction` primitives stay public for interaction
composition. The wasm `trading` surface mirrors the helpers as `buildWrapTx` /
`buildUnwrapTx` and exposes `wrappedNativeToken(chainId)` for wrap-pair detection
and display.

The pre-sign, settlement-cancel, and eth-flow native-sell steps build through the
gas-free `cow-sdk-contracts` builders `pre_sign_transaction`,
`invalidate_order_transaction`, and `ethflow_create_order_transaction` (over the
`IGPv2Settlement` `encode_set_pre_signature` / `encode_invalidate_order` and
`CoWSwapEthFlow` `createOrder` encoders). Each resolves its deployment through the
shared override-or-registry resolver and returns a gas-free `UnsignedTransaction`
mirroring the upstream services `eth::Tx` shape, leaving gas to the caller. The
signer-bound `cow-sdk-trading` flows (`pre_sign_transaction`,
`onchain_cancellation_transaction`, `eth_flow_transaction`) and the wasm
`buildPresignTx` / `buildCancelOrderTx` / `buildSellNativeCurrencyTx` exports delegate
the resolve-and-encode step to these builders — adding only signer-bound gas
estimation, or the gas-defaulted wire DTO — so one byte-locked source backs the
calldata and one resolver backs the target across the native and browser surfaces.
The eth-flow on-chain cancellation is the sole exception: its orderbook-`Order`
projection encoder stays in `cow-sdk-trading` (which sits above `cow-sdk-contracts`),
so it delegates only the target resolution.

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

- Public surface: each on-chain trade step has one single-call transaction builder.
  The complete gas-free builders (`pre_sign_transaction`,
  `invalidate_order_transaction`, `ethflow_create_order_transaction`) and the shared
  override-or-registry resolver live in `cow-sdk-contracts` and return a gas-free
  `UnsignedTransaction`; the pure `wrap_transaction` / `unwrap_transaction` builders
  live there too (re-exported from `cow-sdk-trading`) and stay infallible. The
  `wrap_interaction` / `unwrap_interaction` primitives and the
  `encode_set_pre_signature` / `encode_invalidate_order` settlement call-data encoders
  stay public and parity-pinned. A missing deployment is a typed
  `ContractsError::DeploymentNotFound`, never a panic.
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
