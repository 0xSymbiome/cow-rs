# ADR 0017: Typed `OrderbookRejection` Parser With Permanent Unknown-Tag Fallback

- Status: Accepted (amended)
- Date: 2026-04-21
- Last reviewed: 2026-05-31
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: orderbook, errors, rejections, transport, error-typing
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md), [ADR 0013](0013-http-transport-injection-and-typestate-builders.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Decision

Non-2xx orderbook responses classify through a typed
`OrderbookRejection` enum at `cow_sdk_orderbook::rejection`. The
enum carries a `#[non_exhaustive]` variant for every published
`errorType` tag the orderbook surfaces across order submission,
quoting, cancellation, and price-estimation routes, plus a
permanent tail variant `Unknown { code, message }` that preserves
forward compatibility whenever a new tag ships. The single
data-carrying variant `SellAmountDoesNotCoverFee { fee_amount:
Amount }` lifts the typed payload through the same parser. The
public free function `parse_rejection(status: http::StatusCode,
body: &[u8]) -> Option<OrderbookRejection>` exposes the same
classification at the byte-slice level, and
`OrderbookError::Rejected { status, rejection, source: Box<OrderbookApiError> }`
promotes the typed payload onto the per-call error tree whenever
the response body carries a recognisable rejection envelope. The
prior stringly-typed `OrderbookApiError::error_type() -> Option<&str>`
helper is retired in favour of the typed channel.

## Why

Stringly-typed rejection inspection forces every consumer to
hard-code a tag-name string match and re-do the inspection on every
call site. A typed enum lets consumers pattern-match on the
classification without re-parsing free-form strings, lets metrics
and telemetry partition on a typed key, and lets the SDK promote a
new tag through the dedicated `Unknown { code, message }` fallback
without the orderbook needing a coordinated SDK release. Putting
the parser at the byte-slice level keeps it usable from consumers
that hold a raw HTTP response and never go through the
`OrderbookApiError` envelope, and the
`OrderbookError::Rejected` promotion keeps the original transport
envelope reachable for telemetry while exposing the typed payload
on the happy diagnostic path.

## Must Remain True

- Public surface: `OrderbookRejection` is `#[non_exhaustive]` and
  carries a typed variant for every published `errorType` tag the
  orderbook surfaces. The tail variant `Unknown { code: String,
  message: String }` is permanent and is the canonical home for any
  new tag the orderbook ships before the SDK adopts a typed
  variant. `parse_rejection(status, body) -> Option<OrderbookRejection>`
  classifies a raw `http::StatusCode` plus byte slice; it returns
  `None` whenever the envelope fails to deserialize so the
  `From<OrderbookApiError>` promotion in `error.rs` falls back to
  `OrderbookError::Api(Box<OrderbookApiError>)` (preserving the
  decoded `ResponseBody` — including the `Text` variant for
  plain-text bodies — and the derived public message) instead of
  silently coercing unknown payloads into a default rejection.
  `OrderbookError::Rejected { status, rejection, source: Box<OrderbookApiError> }`
  is the typed promotion path on the per-call error tree, wired
  through `From<OrderbookApiError>` whenever the response body
  carries a recognisable rejection envelope. The retired
  `OrderbookApiError::error_type()` accessor does not exist on the
  shipped surface.
- Runtime and support: the parser is pure. It performs no network
  I/O, reads no environment, and returns `None` rather than
  panicking on shape drift. The `SellAmountDoesNotCoverFee`
  variant pulls the `fee_amount` payload through a typed
  `cow_sdk_core::Amount` field; if the payload shape ever drifts
  the parser falls back to `Unknown` rather than coercing a bogus
  value. The multi-environment order-lookup fallback in the
  orderbook client honours both `Api` and `Rejected` on a 404 so
  an environment retry sees the same rejection class either way.
- Validation and review: a fixture suite at
  `crates/orderbook/tests/rejection_contract.rs` exercises every
  tag, the `SellAmountDoesNotCoverFee` typed payload, the
  `Unknown` fallback for unknown and malformed inputs, the
  `DuplicateOrder` historical typo regression that now classifies
  through `Unknown`, the `None`-on-malformed-body path, and the
  `From<OrderbookApiError>` promotion. The
  `cow_sdk::SdkError::class` classification still lifts a
  `Rejected` response onto `ErrorClass::Remote` so downstream
  telemetry partitions remain stable.
- Cost: one new module (`crates/orderbook/src/rejection.rs`), one
  typed variant on `OrderbookError`, one byte-slice-level public
  function, and one re-export from the `cow-sdk` facade prelude.
  The retired stringly-typed accessor is the only contract removal.

## Alternatives Rejected

- Keep the stringly-typed `error_type()` accessor and add the
  typed enum on top: shorter migration, but leaves two competing
  classification surfaces and lets callers ignore the typed one.
- Make the parser fall back to a default rejection on shape
  drift instead of `Unknown`: lossier, because the shape drift
  becomes invisible and consumers cannot tell whether the new
  payload was a known variant or a forward-compatibility case.
- Skip the byte-slice entry point and require every consumer to
  go through `OrderbookApiError`: simpler module, but blocks
  consumers that hold a raw HTTP response from reaching the typed
  classification.
- Spread the rejection tags across multiple unrelated enums by
  route: matches the orderbook OpenAPI grouping more closely,
  but forces every consumer to maintain N parallel enums when one
  taxonomy already partitions the wire surface cleanly.

## Links

- [Architecture](../architecture.md)
- [Verification Guide](../verification-guide.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md)
- [ADR 0013](0013-http-transport-injection-and-typestate-builders.md)

## Amendment 2026-05-22: canonical primitive layer (per ADR 0052)

The typed `fee_amount: Amount` field carried by
`OrderbookRejection::SellAmountDoesNotCoverFee` resolves through the
cow-owned `#[repr(transparent)]` newtype around `alloy_primitives::U256`
per
[ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md). The
decimal-string wire format is preserved through the cow-owned
`Serialize`/`Deserialize` impls on `Amount`; the strict-decimal-only
fail-closed contract on the `Deserialize` boundary rejects radix-prefixed
payloads that alloy's underlying `ruint::Uint::FromStr` would otherwise
accept.

## Amendment 2026-05-31: coarse category accessor

`OrderbookRejection::category()` returns a coarse, action-oriented
`OrderbookRejectionCategory` partition (`Authorization`, `InsufficientFunds`,
`InvalidOrder`, `NotFound`, `Conflict`, `Unfulfillable`, `Server`, `Unknown`).
It is an **additive accessor**: the typed per-tag taxonomy and the permanent
`Unknown` fallback are unchanged, and the partition itself is
`#[non_exhaustive]`. The mapping is exhaustive over the typed tags with no
wildcard arm, so a newly added wire tag must be assigned a category at the
source rather than being silently misclassified. The category carries no `code`
or `message`, so it never re-exposes a redacted rejection payload and is safe to
log or partition telemetry on directly.
