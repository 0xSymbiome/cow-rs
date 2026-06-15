# ADR 0015: Typed Client-Side Order-Bounds Validator On Every Trading Submission Seam

- Status: Accepted
- Date: 2026-04-21
- Last reviewed: 2026-06-11
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: trading, validation, client-side, defense-in-depth, error-typing
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0006](0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md), [ADR 0011](0011-typed-amount-boundary-and-typestate-ready-state-construction.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Decision

Every public trading submission seam in `cow-sdk-trading` runs the
typed `OrderBoundsValidator` as a mandatory pre-transport step. The
validator is pure — `now` is a caller-supplied UNIX-seconds timestamp
and no `SystemTime::now` is read inside it. Failures raise `TradingError::ClientRejected(ClientRejection)` so the
typed reason is observable without parsing free-form strings. The
validator enforces only stable, provider-independent invariants —
including that an order's `valid_to` is still in the future — and leaves
the exact, operator-tunable order-validity window to services. The chain-aware
default constructor `OrderBoundsValidator::services_default_for_chain`
attaches the chain-specific wrapped-native-token address for the
same-token paired guard so the validator's behavior matches services
end-to-end without any caller-side configuration.

## Why

Without a client-side validator the only enforcement point for the
protocol-invariant matrix is the orderbook itself, which means every
violation costs a full HTTP round trip and surfaces as an opaque
`422` response rather than as a structured Rust error. A pure typed
validator at the submission seam catches the violation locally,
returns a structured payload the caller can pattern-match on, and
preserves the orderbook's authoritative posture as the second
defense line. The explicit `now` parameter keeps the validator
deterministic under replay.

## Must Remain True

- Public surface: `OrderBoundsValidator::validate(order: &OrderData,
  from: Address, app_data_signer: Option<Address>, now: u64,
  is_eth_flow: bool) -> Result<(), ClientRejection>` is the canonical entry
  point. It validates the signing order plus its submission owner (`from`),
  which is threaded separately because the canonical signing order carries no
  owner field.
  `OrderBoundsValidator::services_default()` and
  `OrderBoundsValidator::services_default_for_chain(chain_id)` are
  the public constructors; the latter attaches the chain's
  wrapped-native-token address for the same-token paired guard. The
  `ClientRejection` enum is `#[non_exhaustive]` and ships a typed
  variant for every stable invariant the validator enforces:
  `ValidToInPast`, `MissingFrom`, `AppdataFromMismatch`,
  `SameBuyAndSellToken`, `InvalidNativeSellToken`, `ZeroAmount`
  (discriminated by `AmountSide`), and `OwnerMismatch`. The validator
  runs on every public submission seam: `post_swap_order`,
  `post_limit_order`, `post_swap_order_from_quote`, and
  `post_sell_native_currency_order` for the eth-flow path. Each public
  seam is a single async entry point bounded on `cow_sdk_core::Signer`.
- Runtime and support: the validator is pure. It performs no network
  I/O, reads no environment variables, and no system clock. Callers
  supply the `now` parameter so deterministic regression tests and
  replay tooling stay reproducible. The eth-flow submission path
  invokes the validator with `is_eth_flow: true` so zero-amount,
  same-token, owner-mismatch, and not-expired checks still fire while
  the native-currency-sentinel sell-token check is skipped (the
  sentinel is expected on that path).
- Validation and review: dedicated fixture coverage exists for every
  `ClientRejection` variant in
  `crates/trading/tests/validation_contract.rs`. The paired
  sell-WETH / buy-native-sentinel fixture proves the WETH-bound
  validator rejects locally on buy-side orders and accepts on
  sell-side orders (matching the reviewed production `AllowSell`
  same-token policy). The chain-aware default validator constructed
  by `OrderBoundsValidator::services_default_for_chain` is exercised
  on the submission seam by `crates/trading/tests/post_contract.rs`.
- Cost: one new module (`crates/trading/src/validation.rs`) and one
  typed error variant on `TradingError`. The pure-function shape
  means no runtime overhead beyond the existing `OrderCreation`
  construction. Every public submission seam constructs the
  chain-aware validator internally from
  `OrderBoundsValidator::services_default_for_chain` so the policy
  matches the reviewed services authority without any caller-side
  configuration knob.
- Pre-submission defense-in-depth: beyond the pure validator,
  `post_cow_protocol_trade` recovers the signer from the produced ECDSA
  signature and rejects `ClientRejection::OwnerMismatch { expected, recovered }`
  when recovery disagrees with the declared owner (ECDSA schemes only; EIP-1271
  and pre-sign use their own mechanisms), and fails closed with
  `TradingError::ChainMismatch { signer, trading }` when a signer's optional
  `chain_id()` hint disagrees with the trading client's chain — both before any
  app-data upload, signing, or `send_order`.

## Alternatives Rejected

- Leave validation to the orderbook only: the orderbook stays
  authoritative, but every protocol-invariant violation costs a
  network round trip and surfaces as an opaque `422`. The
  client-side validator is defence-in-depth, not a replacement.
- Read `SystemTime::now()` inside the validator: shorter call
  sites, but the validator becomes non-deterministic under replay
  and complicates fixture pinning. The caller-supplied `now`
  keeps every observation reproducible.
- Hide bounds behind a global static: simpler to read, but inverts
  the per-instance scoping that keeps the SDK runtime-neutral
  (see ADR 0006) and prevents downstream consumers from running
  multiple SDK instances with different policies.
- Spread the rejection variants across multiple unrelated error
  types: matches the existing `TradingError` taxonomy more
  loosely, but loses the typed `ClientRejection` channel that
  consumers pattern-match on for diagnostics and metrics.

## Links

- [Architecture](../architecture.md)
- [Verification Guide](../verification.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0006](0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md)
- [ADR 0011](0011-typed-amount-boundary-and-typestate-ready-state-construction.md)

**Proven by:**

- [Trading Order-Bounds Validator Audit](../audit/trading-order-bounds-validator-audit.md)
- [Trading App-Data Merge Audit](../audit/trading-app-data-merge-audit.md)
- [Trading EthFlow Owner Identity Audit](../audit/trading-ethflow-owner-identity-audit.md)
