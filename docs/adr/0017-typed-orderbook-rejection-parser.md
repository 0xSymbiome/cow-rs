# ADR 0017: Typed `OrderbookRejection` Parser With Permanent Unknown-Tag Fallback

- Status: Accepted
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
  `cow_sdk::CowError::class` classification still lifts a
  `Rejected` response onto `ErrorClass::Remote` so downstream
  telemetry partitions remain stable.
- Coarse category: `OrderbookRejection::category()` returns an additive,
  action-oriented `OrderbookRejectionCategory` partition (`Authorization`,
  `InsufficientFunds`, `InvalidOrder`, `NotFound`, `Conflict`, `Unfulfillable`,
  `Server`, `Unknown`) — `#[non_exhaustive]`, exhaustive over the typed tags with
  no wildcard, and carrying no `code`/`message` so it is safe to log;
  `SellAmountDoesNotCoverFee` categorizes as `Unfulfillable` (an economic,
  re-quotable condition), not `InvalidOrder`.
- Unclassified fallback: `OrderbookError::Api` (taken when `parse_rejection`
  returns `None`) renders the HTTP status on its public message
  (`orderbook request failed (<status>)`) while the body and derived message
  stay redacted on the `#[source]` error per ADR 0025.
- Cost: one new module (`crates/orderbook/src/rejection.rs`), one
  typed variant on `OrderbookError`, one byte-slice-level public
  function, and one re-export through the `cow-sdk` facade `orderbook` module.
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
- Add a dedicated recovery-action classification — an
  `Action { Skip, Retry, Abort, Fix }` axis on rejections — on top of the
  coarse `category()`: rejected. The recovery action is consumer policy, not a
  property of the rejection: the same rejection is a skip for an automated
  strategy loop and an abort for a one-shot call, so the SDK cannot name it
  without serving one of them poorly. The coarse `category()` (the action class)
  and the orderbook retry verdict (`is_retryable()` / `backoff_hint()`) already
  let a consumer derive its own action; a further classification axis would add
  public surface without removing that consumer-side decision.

## Links

- [Architecture](../architecture.md)
- [Verification Guide](../verification.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md)
- [ADR 0013](0013-http-transport-injection-and-typestate-builders.md)
