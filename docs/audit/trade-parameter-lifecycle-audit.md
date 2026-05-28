# Trade-Parameter Lifecycle Audit

Status: Current
Last reviewed: 2026-05-27
Owning surface: `cow-sdk-trading` trade-parameter input shape and the lifecycle distinction between pre-quote and post-quote request types
Refresh trigger: Changes to the public `TradeParameters` or `LimitTradeParameters` field set, changes to the `LimitTradeParametersFromQuote` newtype invariant or constructor entry, changes to the `swap_params_to_limit_order_params` return type, or changes that allow a value lacking `quote_id` to reach the `EthFlow` native-currency submission seam or the `EthFlow` transaction helper
Related docs:
- [ADR 0011](../adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md)
- [ADR 0023](../adr/0023-legacy-compatibility-shim-removal.md)
- [Trading Order Construction Integrity Audit](trading-order-construction-integrity-audit.md)
- [Trading EthFlow Owner Identity Audit](trading-ethflow-owner-identity-audit.md)

## Scope

This audit covers:

- the public `TradeParameters` pre-quote request shape and its single `amount` field interpreted by `kind`
- the public `LimitTradeParameters` post-quote / canonical-submission shape and its `sell_amount`, `buy_amount`, and optional `quote_id` fields
- the public `LimitTradeParametersFromQuote` newtype that guarantees a non-`None` `quote_id` by construction
- the `swap_params_to_limit_order_params` bridge from the pre-quote shape to the from-quote newtype
- the `EthFlow` native-currency submission seam and the `EthFlow` transaction helper, both of which accept only `LimitTradeParametersFromQuote` on their public entries
- the shared `with_*` setter implementations that live in one internal definition and emit identical inherent methods on `TradeParameters` and `LimitTradeParameters`

It does not cover the order-bounds validator (covered by the [Trading Order-Bounds Validator Audit](trading-order-bounds-validator-audit.md)), trader / quoter / order-context parameter shapes (covered by the [Trading SDK Runtime Prerequisites Audit](trading-sdk-runtime-prerequisites-audit.md)), or wire DTO coverage at the orderbook boundary (covered by the [Wire DTO Coverage Audit](wire-dto-coverage-audit.md)).

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Pre-quote shape | `TradeParameters` carries one amount interpreted by `kind` plus the optional override fields shared with the post-quote shape and is accepted by quote and one-call swap entries | Conforms |
| Post-quote shape | `LimitTradeParameters` carries `sell_amount`, `buy_amount`, and `quote_id: Option<i64>` plus the optional override fields shared with the pre-quote shape and is accepted by the limit-order submission entry | Conforms |
| From-quote refinement | `LimitTradeParametersFromQuote` wraps `LimitTradeParameters` and guarantees `quote_id` is `Some` by construction; `quote_id()` returns the inner value without an `Option` | Conforms |
| Bridge | `swap_params_to_limit_order_params` is the only public path that produces a `LimitTradeParametersFromQuote` value from a `TradeParameters` plus an orderbook quote response | Conforms |
| EthFlow entry binding | `post_sell_native_currency_order` and `get_eth_flow_transaction` accept only `LimitTradeParametersFromQuote` on their public entries | Conforms |
| Diagnostic preservation | Attempting to construct `LimitTradeParametersFromQuote` from a value with `quote_id = None` returns `TradingError::MissingQuoteId("EthFlow order posting")` with the same diagnostic shape callers observed before the newtype was introduced | Conforms |
| Setter dedup | Shared `with_*` setter bodies live in one internal definition and emit identical inherent methods on both public types; the public surface shape is preserved exactly | Conforms |

## Current Contract

### Pre-Quote And Post-Quote Shapes

`TradeParameters` is the request shape consumers build before the
quote round trip. It carries a single `amount` field interpreted by
`kind` (sell amount for `OrderKind::Sell`, buy amount for
`OrderKind::Buy`) along with the optional override fields shared with
the post-quote shape.

`LimitTradeParameters` is the canonical submission shape that carries
both `sell_amount` and `buy_amount` plus an optional `quote_id`. Both
types share their non-amount optional `with_*` setter implementations
through one internal definition that emits inherent methods on each
target struct. The factoring is private and not part of the public
API; consumer code observes identical method signatures, identical
`#[must_use]` annotations, identical `const fn` qualifiers, and
identical rustdoc text on each public type.

### From-Quote Newtype

`LimitTradeParametersFromQuote` wraps `LimitTradeParameters` and is
the only shape produced by `swap_params_to_limit_order_params`. The
newtype guarantees `quote_id` is `Some` by construction: its
`try_from_limit` constructor returns `TradingError::MissingQuoteId`
when called with a value lacking a quote id. The public `quote_id()`
accessor returns the inner value directly without an `Option`.

`as_limit` returns a reference to the underlying `LimitTradeParameters`
for callers that need access to the other fields, and `into_limit`
consumes the newtype back into the canonical shape. The newtype
implements `AsRef<LimitTradeParameters>` for ergonomic interop with
APIs that take the underlying type by reference.

### EthFlow Submission And Transaction Helper

The `EthFlow` native-currency submission entry
`post_sell_native_currency_order` and the `EthFlow` transaction helper
`get_eth_flow_transaction` accept only `LimitTradeParametersFromQuote`
on their public entries. The quote-id requirement is enforced at the
type system at the public boundary. Internal orchestration through
`post_cow_protocol_trade` continues to support both shapes; the
`EthFlow` branch constructs a `LimitTradeParametersFromQuote` locally
from the adjusted `LimitTradeParameters` before calling into the
`EthFlow` entry, which preserves the `MissingQuoteId` diagnostic for
the case where a consumer reached the orchestration entry without
going through the from-quote constructor.

### Diagnostic Preservation

The public diagnostic shape is preserved exactly. A consumer who
constructs a `LimitTradeParameters` without a `quote_id` and then
tries to lift it through `LimitTradeParametersFromQuote::try_from_limit`
receives `Err(TradingError::MissingQuoteId("EthFlow order posting"))`
with the documented label. The same error variant flows out of the
orchestration internal path if the `EthFlow` branch is reached
through an intermediate `LimitTradeParameters` value without a quote
id.

### Advanced-Settings Bundle

`TradeAdvancedSettings` is the single advanced-settings type accepted
by every public post and quote entry. Limit-order callers leave the
`slippage_suggester` field as `None` because the limit submission
path does not apply slippage in the same shape as swaps; the field
is documented but unused on that flow.

### Owner Field Placement

The `owner: Option<Address>` field lives on `TradeParameters` and
`LimitTradeParameters` and is the sole source of trade-level owner
attribution observed by the SDK. The `OrderTraderParameters` shape
exposes order-context owner identity through its `order_uid` plus
chain id; the trader-defaults bag (`PartialTraderParameters`) holds
no owner field.

Owner precedence in observing helpers is documented in the
[Trading SDK Runtime Prerequisites Audit](trading-sdk-runtime-prerequisites-audit.md).

## Evidence

Primary implementation points:

- `crates/trading/src/types/trade.rs`
- `crates/trading/src/types/advanced.rs`
- `crates/trading/src/order.rs`
- `crates/trading/src/post/native.rs`
- `crates/trading/src/post/generic.rs`
- `crates/trading/src/onchain.rs`
- `crates/trading/src/lib.rs`

Primary regression coverage:

- `crates/trading/tests/limit_from_quote_contract.rs`
- `crates/trading/tests/types_contract.rs`
- `crates/trading/tests/post_contract.rs::native_sell_posting_requires_quote_id_before_signing_or_submission`
- `crates/trading/tests/post_contract.rs`
- `crates/trading/tests/parameters_contract.rs`
- `crates/trading/tests/invariant_contract.rs`

Validation surface:

```text
cargo fmt --all --check
cargo test -p cow-sdk-trading
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-features --target wasm32-unknown-unknown
```
