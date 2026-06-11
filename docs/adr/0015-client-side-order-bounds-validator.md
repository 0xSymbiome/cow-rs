# ADR 0015: Typed Client-Side Order-Bounds Validator On Every Trading Submission Seam

- Status: Accepted (amended)
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
the exact, operator-tunable order-validity window to services (see the
2026-06-07 amendment). The chain-aware
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

## Amendment 2026-05-22: canonical primitive layer (per ADR 0052)

The `Address`-typed payload fields on `ClientRejection`
(`AppdataFromMismatch { appdata_signer: Address, from: Address }`,
`SameBuyAndSellToken { token: Address }`, and
`OwnerMismatch { expected: Address, recovered: Address }`) and the
`app_data_signer: Option<Address>` parameter on
`OrderBoundsValidator::validate` resolve through the cow-owned
`#[repr(transparent)]` newtype around `alloy_primitives::Address` per
[ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md). The
wire-form preservation (lowercase `0x`-prefixed hex) is locked through
the cow-owned `Display`/`Serialize`/`Deserialize` impls on `Address`.

## Amendment 2026-05-27: single submission seam, services-default validator

`cow-sdk-trading` ships one async entry point per public submission
operation — `post_swap_order`, `post_limit_order`,
`post_swap_order_from_quote`, and `post_sell_native_currency_order`
for the eth-flow path — each bounded on `cow_sdk_core::Signer`. The
validator runs at the reviewed `OrderValidityBounds::SERVICES_DEFAULT`
policy on every seam. The chain-aware default constructor
`OrderBoundsValidator::services_default_for_chain(chain_id)` attaches
the chain's wrapped-native-token address for the same-token paired
guard. The validator's `validate` entry point and the
`ClientRejection` typed channel are unchanged; the
`SERVICES_DEFAULT` constant, `EthFlow` skip rule, `PreSign` and
`Liquidity` exemptions, and `Amount::is_zero` predicate all remain
in force.

## Amendment 2026-06-07: stable, provider-independent invariants only

The validator no longer mirrors the orderbook's configurable order-validity
window. The reviewed services minimum (60 s), market maximum (3 h), and
limit-class ceiling (1 y) are operator-tunable deployment configuration, not
protocol constants: a client that pins them drifts whenever an operator retunes
the deployment, and it is additionally sensitive to clock skew between the
caller-supplied `now` and the server's wall clock. On the public submission
seams every order also routes through the limit class, so the maximum-lifetime
ceilings were effectively unreachable in normal use.

The validator now enforces only the stable, provider-independent validity
invariant: an order whose `valid_to` is at or before `now` is already expired
and is rejected as `ClientRejection::ValidToInPast { valid_to, now }`. Services
remains authoritative for the exact minimum and maximum lifetimes.

The validator's public surface changes accordingly:

- `OrderValidityBounds` and `SubmissionClass` are removed.
- `ClientRejection::ValidToInsufficient` and `ClientRejection::ValidToExcessive`
  are removed and replaced by `ClientRejection::ValidToInPast { valid_to, now }`.
- `OrderBoundsValidator::validate` drops its `scheme` parameter, which existed
  only to bypass the maximum-lifetime ceiling for `PreSign`; its signature is now
  `validate(order: &OrderData, from: Address, app_data_signer: Option<Address>,
  now: u64, is_eth_flow: bool)`.
- `OrderBoundsValidator::bounds`, `OrderBoundsValidator::class`, and the
  `PreSign` / `Liquidity` maximum-lifetime exemptions are removed.

The same-token guard is unchanged in behavior and is now documented as mirroring
the services `AllowSell` policy specifically (buy-side same-token rejected,
sell-side accepted, including the wrapped-native / native-sentinel pair) — the
production deployment's policy — rather than the services same-token policy in
general, which would also cover the `Disallow` and `Allow` configurations the SDK
does not attempt to track.

The remaining invariants — present owner, non-native sell token outside eth-flow,
buy-side same-token, non-zero amounts, app-data-signer agreement, and the
recoverable-owner check via `assert_owner_matches_signer` — are unchanged, as are
the pure caller-supplied-`now` posture and the `TradingError::ClientRejected`
typed channel.

## Amendment 2026-06-11: the owner check recovers the signer from the signature

The recoverable-owner check now actually recovers. Previously it compared the
declared owner against the signer's self-reported `address()` before signing —
a self-report the SDK cannot verify. A signer that reports address A but signs
with key B (a hardware wallet on the wrong derivation path, a misloaded key, a
buggy or adversarial `Signer`) produced a valid signature recovering to B while
the order declared `from = A`; the self-report check passed it, and only the
services submission-side recovery (`WrongOwner`) caught it after a full round
trip.

`post_cow_protocol_trade` now recovers the signer from the produced ECDSA
signature and the order's EIP-712 digest after signing and before submission,
and rejects `ClientRejection::OwnerMismatch { expected, recovered }` when the
recovered address is not the declared owner — the client-side mirror of the
services `WrongOwner` check. `expected` is the declared owner; `recovered` is
the address the signature actually recovers to.

- The check is ECDSA-only (`Eip712`/`EthSign`). EIP-1271 and pre-sign
  authorizations carry no recoverable ECDSA signature and are verified by their
  own mechanisms (on-chain EIP-1271 verification; the owner setting the
  pre-signature flag). Recovery is scheme-aware (EthSign uses the EIP-191
  prehash) through `RecoverableSignature::recover`, with ADR 0022
  canonicalization enforced before any recovery runs.
- The pre-sign self-report comparison is retained as a cheap fast-fail that
  rejects an explicit owner ≠ `signer.address()` before wasting an app-data
  upload and a signature; the post-sign recovery is the authoritative check
  that inspects the signature itself. The gate fires before `send_order`, so a
  mismatched order never reaches the wire.
- No public surface changes: the check reuses the existing
  `ClientRejection::OwnerMismatch` variant and `assert_owner_matches_signer`
  helper. The `recovered` field — already its name — now carries the address
  recovered from the signature rather than the signer's self-report.

## Amendment 2026-06-11: chain coherence joins the pre-submission defense-in-depth checks

Chain coherence now joins owner recovery in the pre-submission defense-in-depth
checks. A signer carries an optional, defaulted `chain_id()` hint
(`cow_sdk_core::Signer::chain_id`) reporting the chain it is statically bound to.
`post_cow_protocol_trade` consults it before signing: when the hint is `Some` and
disagrees with the trading client's canonical chain, the flow fails closed
locally with `TradingError::ChainMismatch { signer, trading }` before any
app-data upload, signing, or submission.

This catches a signer bound to the wrong chain — whose EIP-712 signature would
carry the wrong domain separator — without the orderbook round-trip that would
otherwise reject it. It is the chain-domain complement to the owner-recovery
gate above: both fast-fail a domain mismatch before the order reaches the wire.

- The hint is optional. Signers that learn their chain at runtime (EIP-1193
  wallets, browser wallets, recording doubles, the pre-sign placement stand-in)
  return `None` and the gate is a no-op, so the order posts as before. Local
  key signers bound at construction (`LocalAlloySigner`, the Alloy client signer
  handle) report `Some`.
- `chain_id()` is synchronous and defaulted on the trait: it reports a
  construction-time fact, not a runtime query, so a signer adopts the trait
  without implementing it. `ChainMismatch` classifies as a non-retryable
  caller-side validation error, consistent with the other configuration faults.
