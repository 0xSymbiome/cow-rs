# Trading Order-Bounds Validator Audit

Status: Current
Last reviewed: 2026-04-25
Owning surface: `cow-sdk-trading` `OrderBoundsValidator`,
`OrderValidityBounds`, `SubmissionClass`, `ClientRejection`,
`AmountSide`, and the `TradingError::ClientRejected` lifting variant.
Refresh trigger: Changes to the `validate` signature, the
`ClientRejection` variant set, the `OrderValidityBounds::SERVICES_DEFAULT`
constants, the `TradingSdkBuilder::with_order_bounds` plumbing, the
eth-flow `is_eth_flow` skip rule, the WETH-paired-with-native-buy
guard, or the offline `TradeParameters::validate` /
`LimitTradeParameters::validate` builder-level subset.
Related docs:
- [ADR 0015](../adr/0015-client-side-order-bounds-validator.md)
- [Architecture](../architecture.md)
- [Verification Guide](../verification-guide.md)

## Scope

This audit covers:

- the typed `OrderBoundsValidator` and the public `validate` entry
  point on `cow-sdk-trading`
- the `OrderValidityBounds` policy struct, its `SERVICES_DEFAULT`
  constant, and the `SubmissionClass` discriminator
- the `ClientRejection` enum and the `TradingError::ClientRejected`
  lifting variant
- the validator wiring on every public submission seam
  (`post_swap_order`, `post_swap_order_async`, `post_limit_order`,
  `post_limit_order_async`, `post_swap_order_from_quote`,
  `post_swap_order_from_quote_async`,
  `post_sell_native_currency_order`, the matching `_with_bounds`
  variants, and the central `post_cow_protocol_trade_async` sink)
- the `TradingSdkBuilder::with_order_bounds` setter and the
  `TradingSdk` field that carries the configured policy
- the offline `TradeParameters::validate` and
  `LimitTradeParameters::validate` builder-level subset
- the `cow_sdk_core::Amount::is_zero` predicate consumed by
  zero-amount checks

It does not cover the orderbook authoritative validation surface,
the off-chain cancellation pipeline, or the on-chain settlement
encoder.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Validator signature | `validate(&order, scheme, app_data_signer: Option<Address>, now: u64, is_eth_flow: bool) -> Result<(), ClientRejection>` is the canonical entry point | Conforms |
| Variant coverage | Every reviewed services protocol-invariant rejection has a typed `ClientRejection` variant; the enum is `#[non_exhaustive]` | Conforms |
| Default policy | `OrderValidityBounds::SERVICES_DEFAULT` matches the published 60 s minimum, 3 h market maximum, and 1 y limit-class ceiling | Conforms |
| Builder plumbing | `TradingSdkBuilder::with_order_bounds` flows through `TradingSdk` to every submission seam; the configured policy is honoured at validation time | Conforms |
| EthFlow skip rule | `is_eth_flow: true` skips the native-currency-sentinel sell-token check and runs every other invariant | Conforms |
| WETH-paired guard | A WETH-bound validator rejects `sell_token = WETH` paired with `buy_token = native sentinel` as `SameBuyAndSellToken { token: weth }` | Conforms |
| Purity | The validator reads no system clock or environment, performs no I/O, and is idempotent for a given input tuple | Conforms |
| Scope framing | The public validator documentation frames the local checks as defence-in-depth and names services-side rejection classes outside SDK pre-check coverage | Conforms |

## Current Contract

### Validator Signature

`OrderBoundsValidator::validate` lives at
`crates/trading/src/validation.rs`. The entry point accepts the
`OrderCreation` payload, the `SigningScheme`, the typed
`Option<Address>` declared signer carried inside the app-data
metadata envelope, the caller-supplied UNIX-seconds `now`, and the
`is_eth_flow` flag. Returning `Result<(), ClientRejection>` keeps
the typed error channel observable for pattern matching.

### Scope Framing

The `OrderBoundsValidator` documentation describes the validator as a
client-side defence-in-depth guard. A successful local validation means
the order does not violate the reviewed SDK-side invariants; it does
not guarantee services acceptance. The documentation explicitly leaves
deny-list, transferability, gas budget, banned-users, market-class
classification, signing-scheme/onchain pairings, and other services-side
rejection classes to the authoritative orderbook services surface.

### Default Policy And Submission Class

`OrderValidityBounds::SERVICES_DEFAULT` carries `min = 60 s`,
`max_market = 10_800 s`, and `max_limit = 31_536_000 s` matching
the reviewed services production configuration. The
`SubmissionClass` discriminator selects between `Market`, `Limit`,
and `Liquidity`. `PreSign` scheme and `Liquidity` class bypass the
maximum-lifetime check so reviewed corner cases stay valid.

### Variant Set

`ClientRejection` is `#[non_exhaustive]` and ships every
protocol-invariant rejection the reviewed services validator
surfaces:

- `ValidToInsufficient { valid_to, now, min_seconds }`
- `ValidToExcessive { valid_to, now, max_seconds }`
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

`TradingSdk` carries an `order_bounds: OrderValidityBounds` field
populated from the builder's `with_order_bounds` setter (default
`SERVICES_DEFAULT`). Every public `TradingSdk` post method forwards
`self.order_bounds` to the matching `_with_bounds` companion on the
module-level helper. The central `post_cow_protocol_trade_async`
sink constructs the validator from the supplied bounds, attaches
the chain-specific WETH address through `with_weth_address`, and
runs the `validate` call between order construction and the HTTP
upload.

### EthFlow Skip Rule And WETH-Paired Guard

`post_sell_native_currency_order_async` invokes the validator with
`is_eth_flow: true` so the native-currency-sentinel sell-token
check is skipped while every other invariant (zero amount, same
token, owner mismatch, lifetime bounds) still fires. When the
validator is configured with the chain's wrapped-native address
through `with_weth_address`, the paired sell-WETH /
buy-native-sentinel case rejects locally as `SameBuyAndSellToken {
token: weth_address }`, mirroring the reviewed services token-pair
guard. Without a configured WETH address the exact-match guard
still fires for identical sell and buy tokens.

### Purity

The validator never reads `SystemTime::now`, never opens a network
connection, and never inspects environment variables. The pure
shape keeps deterministic regression tests reproducible across
machines and replays.

## Evidence

Primary implementation points:

- `crates/trading/src/validation.rs`
- `crates/trading/src/parameters.rs`
- `crates/trading/src/post.rs`
- `crates/trading/src/sdk.rs`
- `crates/core/src/types.rs` (`Amount::is_zero`)

Primary regression coverage:

- `crates/trading/tests/validation_contract.rs`
- `crates/trading/tests/post_contract.rs`
- `crates/trading/tests/post_contract.rs::post_swap_order_appdata_from_mismatch_does_not_upload_or_sign`
- `crates/trading/tests/post_contract.rs::post_swap_order_same_buy_sell_token_does_not_upload_or_sign`
- `crates/trading/tests/post_contract.rs::post_swap_order_zero_amount_does_not_upload_or_sign`
- `crates/trading/tests/parity_contract.rs`

Validation surface:

```text
cargo test -p cow-sdk-trading --all-features
cargo clippy -p cow-sdk-trading --all-targets --all-features -- -D warnings
```
