# Trading Order-Bounds Validator Audit

Status: Current
Last reviewed: 2026-06-07
Owning surface: `cow-sdk-trading` `OrderBoundsValidator`,
`ClientRejection`, `AmountSide`, and the `TradingError::ClientRejected`
lifting variant.
Refresh trigger: Changes to the `validate` signature, the
`ClientRejection` variant set, the not-expired (`ValidToInPast`) check, the
`OrderBoundsValidator::services_default_for_chain`
constructor, the eth-flow `is_eth_flow` skip rule, upstream services
`crates/shared/src/order_validation.rs` same-token semantics, the
WETH-paired-with-native-buy guard, or the offline `TradeParams::validate`
/ `LimitTradeParams::validate` builder-level subset.
Related docs:
- [ADR 0015](../adr/0015-client-side-order-bounds-validator.md)
- [Architecture](../architecture.md)
- [Verification Guide](../verification.md)

## Scope

This audit covers:

- the typed `OrderBoundsValidator` and the public `validate` entry
  point on `cow-sdk-trading`
- the not-expired validity invariant (`ClientRejection::ValidToInPast`)
- the `ClientRejection` enum and the `TradingError::ClientRejected`
  lifting variant
- the validator wiring on every public submission seam:
  `post_swap_order`, `post_limit_order`,
  `post_swap_order_from_quote`, and `post_sell_native_currency_order`,
  routed through the central `post_cow_protocol_trade` sink. Each
  public seam is a single async entry point bounded on
  `cow_sdk_core::Signer`.
- the chain-aware default validator constructed by
  `OrderBoundsValidator::services_default_for_chain`, which attaches
  the chain-specific wrapped-native-token address for the same-token
  paired guard
- the offline `TradeParams::validate` and
  `LimitTradeParams::validate` builder-level subset
- the `cow_sdk_core::Amount::is_zero` predicate consumed by
  zero-amount checks

It does not cover the orderbook authoritative validation surface,
the off-chain cancellation pipeline, or the on-chain settlement
encoder.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Validator signature | `validate(order: &OrderData, from: Address, app_data_signer: Option<Address>, now: u64, is_eth_flow: bool) -> Result<(), ClientRejection>` is the canonical entry point | Conforms |
| Variant coverage | Every stable invariant the validator enforces has a typed `ClientRejection` variant; the enum is `#[non_exhaustive]` | Conforms |
| Validity invariant | The validator rejects an order whose `valid_to` is at or before `now` (`ValidToInPast`) and leaves the exact, operator-tunable validity window to services | Conforms |
| Submission-seam policy | Every public submission seam constructs the validator via `OrderBoundsValidator::services_default_for_chain` and runs `validate` between order construction and HTTP upload | Conforms |
| EthFlow skip rule | `is_eth_flow: true` skips the native-currency-sentinel sell-token check and runs every other invariant | Conforms |
| Same-token policy | Mirrors the reviewed services `AllowSell` policy: exact same-token and WETH-paired-with-native-sentinel orders accept on sell-side and reject on buy-side with `SameBuyAndSellToken` | Conforms |
| WETH-paired guard | A WETH-bound validator rejects buy-side `sell_token = WETH` paired with `buy_token = native sentinel` as `SameBuyAndSellToken { token: weth }` and accepts the sell-side pair | Conforms |
| Purity | The validator reads no system clock or environment, performs no I/O, and is idempotent for a given input tuple | Conforms |
| Time-source determinism | Property coverage compares validation classifications at `now` and `now + delta` while the order is not expired at either observation | Conforms |
| Timestamp extremes | `valid_to = u32::MAX` resolves to typed validation outcomes at `u32::MAX` and `u64::MAX` timestamp boundaries without panicking | Conforms |
| Gas overhead | EthFlow and pre-sign transaction helpers apply the documented 20% gas overhead with floor integer rounding | Conforms |
| Cancellation gas fallback | On-chain cancellation transaction construction falls back to `DEFAULT_GAS_LIMIT` when signer gas estimation is unavailable | Conforms |
| Fuzz harness | `fuzz_order_bounds_validator` carries a documented seed-class contract covering validator rejection classes and timestamp/token sentinels; the working corpus stays local-only (gitignored) | Conforms |
| Scope framing | The public validator documentation frames the local checks as defence-in-depth and names services-side rejection classes outside SDK pre-check coverage | Conforms |

## Current Contract

### Validator Signature

`OrderBoundsValidator::validate` lives at
`crates/trading/src/validation.rs`. The entry point accepts the
signing order (`cow_sdk_core::OrderData`), the submission owner
(`from: Address`, threaded separately because the signing order
carries no owner field), the typed `Option<Address>` declared signer
carried inside the app-data metadata envelope, the caller-supplied
UNIX-seconds `now`, and the `is_eth_flow` flag. Returning `Result<(), ClientRejection>` keeps
the typed error channel observable for pattern matching.

### Scope Framing

The `OrderBoundsValidator` documentation describes the validator as a
client-side defence-in-depth guard. A successful local validation means
the order does not violate the reviewed SDK-side invariants; it does
not guarantee services acceptance. The documentation explicitly leaves
deny-list, transferability, gas budget, banned-users, market-class
classification, signing-scheme/onchain pairings, and other services-side
rejection classes to the authoritative orderbook services surface.

### Validity Invariant

The validator checks only the stable, provider-independent validity
invariant: an order whose `valid_to` is at or before the caller-supplied
`now` is already expired and rejects as
`ClientRejection::ValidToInPast { valid_to, now }`. The exact minimum and
maximum order lifetimes are orderbook-operator configuration, so the SDK
leaves them to the authoritative services surface rather than pinning a
window that would drift when an operator retunes the deployment.

### Variant Set

`ClientRejection` is `#[non_exhaustive]` and ships a typed variant for
every stable invariant the validator enforces:

- `ValidToInPast { valid_to, now }`
- `MissingFrom`
- `AppdataFromMismatch { appdata_signer, from }`
- `SameBuyAndSellToken { token }`
- `InvalidNativeSellToken`
- `ZeroAmount { side: AmountSide }` with `AmountSide::{Sell, Buy}`
- `OwnerMismatch { expected, recovered }`

`TradingError::ClientRejected(ClientRejection)` lifts every
variant onto the public trading error surface so callers and
downstream telemetry see the typed payload without parsing free-form
strings.

### Submission-Seam Plumbing

Every public submission entry point constructs the chain-aware
default validator through `OrderBoundsValidator::services_default_for_chain(chain_id)`
on the orderbook's canonical chain id, runs `validate` between order
construction and the HTTP upload, and surfaces failures through
`TradingError::ClientRejected(ClientRejection)`. The central
`post_cow_protocol_trade` sink is the shared submission helper; the
eth-flow native-currency seam routes through
`post_sell_native_currency_order` with `is_eth_flow: true`. No
caller-side configuration of the validator policy is exposed on the
public surface.

### Same-Token And Native-Sentinel Parity

`OrderBoundsValidator::validate` mirrors the reviewed services
`AllowSell` same-token policy in `cow-sdk-trading`. Exact same-token
orders and WETH-paired-with-native-sentinel orders are accepted
when `OrderKind::Sell` is submitted and rejected when
`OrderKind::Buy` is submitted. Buy-side rejections surface through
`ClientRejection::SameBuyAndSellToken { token }`. The
reviewed-services configuration in production deployments runs the
same `AllowSell` mode (the `Disallow` and `Allow` modes are
upstream policy variants out of scope for `cow-sdk-trading`).

`TradeParams::validate` and `LimitTradeParams::validate` apply the
same buy-only exact same-token rule at the chain-agnostic builder layer. The
chain-specific WETH/native-sentinel pairing remains owned by the order-level
validator because it requires the wrapped-native token address for the
selected chain.

### EthFlow Skip Rule And WETH-Paired Guard

`post_sell_native_currency_order` invokes the validator with
`is_eth_flow: true` so the native-currency-sentinel sell-token
check is skipped while every other invariant (zero amount, same
token buy-side rejection, owner mismatch, expired validTo) still fires. When
the validator is configured with the chain's wrapped-native address through
`with_weth_address`, the paired sell-WETH / buy-native-sentinel case rejects
locally for buy-side orders as `SameBuyAndSellToken { token: weth_address }`,
while sell-side orders validate. Without a configured WETH address the
exact-match guard still applies for identical sell and buy tokens.

### Purity

The validator never reads `SystemTime::now`, never opens a network
connection, and never inspects environment variables. The pure
shape keeps deterministic regression tests reproducible across
machines and replays.

### Property And Fuzz Evidence

`crates/trading/tests/property_contract.rs` pins deterministic validation
under caller-supplied time by comparing the typed result classification at
`now` and `now + delta` while the order is not expired at either timestamp.
The same file covers `valid_to = u32::MAX` with `now` values around
`u32::MAX` and `u64::MAX`, asserting typed results rather than relying on an
implicit non-panic test.

`fuzz/fuzz_targets/fuzz_order_bounds_validator.rs` maps arbitrary bytes into
the validator tuple shape and checks that every outcome remains a typed
`Result`. Its harness header documents the seed-class contract —
happy-path, rejection-class, timestamp-extreme, and WETH/native sentinel
seeds — while the working corpus stays local-only (gitignored).

### Gas Overhead Evidence

The EthFlow and pre-sign transaction helpers apply the same documented gas
overhead as the trading utility: `gas + (gas * 20) / 100`. The boundary tests
pin small floor-rounding cases and a large `u64::MAX / 2` estimate so future
changes cannot silently switch multiplier or rounding behavior.

On-chain cancellation transaction construction keeps a separate fallback
contract: if signer gas estimation fails, the helper uses the documented
`DEFAULT_GAS_LIMIT` constant rather than surfacing an estimation-only error
before callers can sign or inspect the cancellation transaction.

## Evidence

Primary implementation points:

- `crates/trading/src/onchain.rs`
- `crates/trading/src/slippage/amounts.rs`
- `crates/trading/src/validation.rs`
- `crates/trading/src/parameters.rs`
- `crates/trading/src/post/generic.rs`
- `crates/trading/src/post.rs`
- `crates/core/src/types/amount.rs` (`Amount::is_zero`)
- `fuzz/fuzz_targets/fuzz_order_bounds_validator.rs`

Primary regression coverage:

- `crates/trading/tests/validation_contract.rs`
- `crates/trading/tests/validation_contract.rs::validate_same_token_matches_services_allow_sell_policy`
- `crates/trading/tests/parameters_contract.rs::tradeparameters_validate_mirrors_services_allow_sell`
- `crates/trading/tests/parameters_contract.rs::limittradeparameters_validate_mirrors_services_allow_sell`
- `crates/trading/tests/property_contract.rs::validator_classification_is_stable_while_order_stays_in_the_future`
- `crates/trading/tests/property_contract.rs::validator_handles_u32_max_validto_without_overflow`
- `crates/trading/tests/onchain_contract.rs::eth_flow_gas_estimate_applies_documented_floor_overhead`
- `crates/trading/tests/onchain_contract.rs::pre_sign_gas_estimate_applies_documented_floor_overhead`
- `crates/trading/tests/cancel_contract.rs::cancellation_gas_estimation_fallback_uses_documented_constant`
- `crates/trading/tests/post_contract.rs`
- `crates/trading/tests/post_contract.rs::post_swap_order_appdata_from_mismatch_does_not_upload_or_sign`
- `crates/trading/tests/post_contract.rs::post_swap_order_same_buy_sell_token_does_not_upload_or_sign`
- `crates/trading/tests/post_contract.rs::post_swap_order_sell_side_same_buy_sell_token_uploads_signs_and_submits`
- `crates/trading/tests/post_contract.rs::post_swap_order_zero_amount_does_not_upload_or_sign`
- `crates/trading/tests/parity_contract.rs`

Validation surface:

```text
cargo test -p cow-sdk-trading --test onchain_contract
cargo test -p cow-sdk-trading --test validation_contract
cargo test -p cow-sdk-trading --test parameters_contract
cargo test -p cow-sdk-trading --test property_contract
cargo test -p cow-sdk-trading --all-features
cargo clippy -p cow-sdk-trading --all-targets --all-features -- -D warnings
cargo +nightly fuzz build --fuzz-dir fuzz fuzz_order_bounds_validator
cargo +nightly fuzz run fuzz_order_bounds_validator --fuzz-dir fuzz -- -runs=1024
cargo run -p xtask -- policy check-property-citations
```
