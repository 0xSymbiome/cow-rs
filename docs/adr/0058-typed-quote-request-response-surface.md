# ADR 0058: Typed Quote Request/Response Surface

- Status: Accepted
- Date: 2026-05-29
- Last reviewed: 2026-06-12
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: orderbook, trading, quote, dto, openapi, compatibility
- Anchors: Forward-Compatible Public Surfaces (primary)

> **Revision (2026-06-11):** the original decision did not bind the quote
> response to the request, on the premise that the client-side bounds validator
> was the defensive layer. On re-review that premise was found incomplete — the
> bounds validator checks order well-formedness, not agreement with the caller's
> intent, and has no access to the request — so a coherent response that altered
> the fixed amount leg or the balance sources reached a signable order
> unchecked. This ADR now binds the request-determined fields of the response
> (the variable price leg stays free); the superseded clauses are marked inline.
>
> **Revision (2026-06-12):** the receiver and app-data echo checks were
> tightened to close two residual gaps. The receiver is reconciled as the
> *effective* receiver — an unset or zero receiver resolves to the owner — so a
> response that fabricates a receiver for a request that pinned none now fails
> closed; the check previously ran only when both sides carried a receiver. The
> app-data hash is reconciled for *every* request form (an explicit pin, the
> keccak digest of a full document, or the zero hash for an omitted pair), not
> only when the request pinned a hash. Both expected values are
> request-derivable and equal what the orderbook itself returns, so the
> tightening adds no false positives.
- Related: [ADR 0031](0031-wire-dto-openapi-driven-with-order-auction-order-split.md), [ADR 0021](0021-orderbook-total-fee-policy.md), [ADR 0015](0015-client-side-order-bounds-validator.md), [ADR 0017](0017-typed-orderbook-rejection-parser.md), [ADR 0011](0011-typed-amount-boundary-and-typestate-ready-state-construction.md)

## Decision

The quote surface follows the source-lock-pinned orderbook OpenAPI as its
authority, consistent with [ADR 0031](0031-wire-dto-openapi-driven-with-order-auction-order-split.md).

The quote response payload is the orderbook `OrderParameters` schema, mirrored
by `cow_sdk_orderbook::QuoteData`. `QuoteData` covers every required
`OrderParameters` field — including `gasAmount`, `gasPrice`, `sellTokenPrice`,
and `signingScheme` — and is enrolled in `parity/openapi/coverage.yaml`; its schema inventory is
expanded in memory from the vendored spec and every required field is checked
against the Rust mirror. The `quote` field of
`OrderQuoteResponse` is therefore validated for field-level fidelity rather than
treated as an opaque object.

The quote network-cost inputs (`feeAmount`, `gasAmount`, `gasPrice`,
`sellTokenPrice`) are read-only on `QuoteData`, consistent with
[ADR 0021](0021-orderbook-total-fee-policy.md). They are populated only by
deserializing the `/quote` response and are surfaced through accessors; no
public builder exposes a setter for them.

`priceQuality` defaults to `optimal` and is always serialized. `optimal` is the
estimate the orderbook intends order creation to use, so it is the correct
default for a request whose result is meant to be signed and submitted;
`verified` remains available for callers that want a simulated estimate and
`fast` for a non-submittable preview.

The quote-amounts projection that derives the signable order from a `/quote`
response matches the orderbook quote-amounts algorithm and is locked by a parity
regression test (`crates/trading/tests/quote_projection_parity.rs`).

The SDK trusts the orderbook for the variable price leg of a quote — the amount
the solver returns for the unfixed side — but binds every request-determined
field of the response back to the request. `OrderbookApi::quote` invokes
`OrderQuoteResponse::ensure_matches` on each response and fails closed with
`OrderbookError::QuoteEchoMismatch` when the token pair, order kind, owner,
partial-fill flag, balance sources, the effective receiver, the app-data hash,
an absolute `validTo`, or the fixed amount leg did not come back unchanged. The
receiver is reconciled as the effective receiver (an unset or zero receiver
resolves to the owner, matching the orderbook settlement rule), and the app-data
hash is reconciled for every request form (an explicit pin, the keccak digest of
a full document, or the zero hash for an omitted pair). The fixed-leg fold
mirrors the services quote arithmetic per side basis (sell-before-fee:
`sellAmount + feeAmount == requested`; sell-after-fee: `sellAmount == requested`;
buy: `buyAmount == requested`). The signed order then binds the caller's
requested balance sources rather than the response echo, and `from_quote` binds
the caller's receiver rather than the echoed value, so the projected order is
still validated through the client-side bounds validator
([ADR 0015](0015-client-side-order-bounds-validator.md)) before submission.

The quote request models the orderbook's quote `oneOf`s as typed Rust so that an
invalid request is unrepresentable rather than rejected at validation time:
`QuoteValidity` carries either `validTo` or `validFor` (never both),
`OrderQuoteSide` carries exactly one side with `SellAmount` distinguishing the
before/after-fee sell amount, and `QuoteSigningScheme` encodes the
scheme-specific constraints (only EIP-1271 has a `verificationGasLimit`, only
EIP-1271 and pre-sign can be on-chain, and an ECDSA scheme can never be on-chain).
App-data on the request stays modeled as the `appData`/`appDataHash` pair
(`QuoteAppData`), consistent with the signed `OrderCreation` payload, and is
serialized through the same hand-rolled routing: a full document serializes
under `appData`, a hash-only request serializes the hash under `appData` (the
services `Hash` form), and both serialize together. The pair is therefore
wire-correct for every form — a hash-only request no longer produces an
`appDataHash`-only body the orderbook rejects. Modeling app-data as a typed
`oneOf` would be a separate change spanning both `OrderCreation` and
`OrderQuoteRequest` for cross-DTO consistency.

## Why

The orderbook returns the full order parameters in a quote response, and the
network fee a caller sees is a function of `gasAmount`, `gasPrice`, and
`sellTokenPrice`. Modeling those fields and validating them through the coverage
inventory keeps the quote response auditable and prevents the response mirror
from silently dropping fields the backend adds or relies on. Leaving the `quote`
payload as an opaque object passed field-presence validation while hiding the
fact that the Rust mirror omitted the network-cost inputs.

Defaulting `priceQuality` to `optimal` matches the estimate the backend expects
order creation to build from, so the unmanaged "quote then sign" path produces a
submittable order by default instead of a simulation-only estimate.

Keeping the network-cost fields read-only stops callers from fabricating quote
economics, the same reasoning that makes order-level `feeAmount` read-only under
[ADR 0021](0021-orderbook-total-fee-policy.md).

Binding the request-determined fields closes a gap the bounds validator does
not cover. The bounds validator checks that the projected order is well-formed —
owner present, not expired, non-zero amounts, token-pair rules — but it has no
access to the request and never compares the order to the caller's intent, so a
coherent response that altered the fixed leg or the balance sources passes it.
The two guarantees are orthogonal: the bounds validator answers "is this order
well-formed?", the echo check answers "does this order match what the caller
asked for?". The fixed leg and the balance sources are the caller's own inputs
echoed back, not data the orderbook authors, so verifying they returned
unchanged is a round-trip integrity check — the same posture the SDK already
applies to the app-data hash through `HashMismatchStage::ServerEcho`. The
variable price leg stays trusted, because it is the answer to the request.

## Must Remain True

- `QuoteData` covers every required `OrderParameters` field, proven by the
  `OrderParameters` coverage entry and `openapi-coverage`.
- The quote response coverage entry stays in `parity/openapi/coverage.yaml`,
  and its required fields are checked against the vendored spec.
- `priceQuality` defaults to `optimal` and is always serialized.
- The quote `expiration` is exposed as the lossless ISO-8601 UTC string and
  cow-rs takes no datetime dependency to parse it; consumers parse with their
  own datetime crate, and the epoch order-validity remains `QuoteData.valid_to`.
- The quote request types its `oneOf`s so invalid combinations are
  unrepresentable: `QuoteValidity` (`validTo` xor `validFor`), `OrderQuoteSide`
  with `SellAmount` (exactly one side; sell before/after fee), and
  `QuoteSigningScheme` (verification gas limit only on EIP-1271; ECDSA never
  on-chain). The signing constraints are enforced on the wire by a `try_from`
  deserialization guard.
- No public builder exposes a setter for the quote network-cost fields
  (`feeAmount`, `gasAmount`, `gasPrice`, `sellTokenPrice`); they are read-only
  accessors populated from the wire.
- The quote-amounts projection has a parity regression test.
- `OrderbookApi::quote` binds every request-determined field of the response to
  the request through `OrderQuoteResponse::ensure_matches`, failing closed with
  `OrderbookError::QuoteEchoMismatch`; the variable price leg stays free, and the
  fixed-leg fold follows the services arithmetic per side basis. The receiver is
  reconciled as the effective receiver (an unset or zero receiver resolves to the
  owner), and the app-data hash is reconciled for every request form (explicit
  pin, full-document digest, or the zero hash for an omitted pair). The signed
  order binds the caller's requested balance sources, `from_quote` binds the
  caller's receiver rather than the echoed value, and the projected order is
  still validated through the bounds validator
  ([ADR 0015](0015-client-side-order-bounds-validator.md)) before submission.
- The quote response DTO remains open to additive upstream fields (no
  `serde(deny_unknown_fields)` in response position, per
  [ADR 0031](0031-wire-dto-openapi-driven-with-order-auction-order-split.md)).

## Alternatives Rejected

- Treat the quote `quote` payload as an opaque object: passes field-presence
  validation but hides whether the Rust mirror is faithful to `OrderParameters`.
- Default `priceQuality` to `verified`: more conservative in isolation, but
  produces a non-submittable default for the "quote then sign" path that the
  backend expects to build from `optimal`.
- Expose public setters for the quote network-cost fields: convenient for
  test construction, but lets callers fabricate quote economics.
- Field-bind *every* response field to the request, including the variable price
  leg: rejected — it would reject every legitimate quote, since the solver-quoted
  side is the answer to the request and necessarily differs from any placeholder.
  The adopted design binds only the request-determined fields and the fixed
  amount leg, leaving the variable leg free.
- Rely on the bounds validator alone for response integrity (the original
  decision): superseded on re-review (2026-06-11). The bounds validator checks
  well-formedness, not intent, and has no access to the request, so it cannot
  catch a coherent response that altered the fixed leg or the balance sources.

## Anchors

This ADR is an anchor for the Forward-Compatible Public Surfaces principle, on
the quote surface.

## Links

- [Principles](../principles.md)
- [Parity Matrix](../parity.md)
- [Quote Response Surface Audit](../audit/quote-response-surface-audit.md)
- [Wire DTO Coverage Audit](../audit/wire-dto-coverage-audit.md)
- `parity/openapi/coverage.yaml`

**Proven by:**

- [Quote Response Surface Audit](../audit/quote-response-surface-audit.md)
- `xtask/src/openapi_coverage.rs`
- `crates/orderbook/tests/wire_contract.rs`
- `crates/orderbook/tests/fee_amount_is_not_a_public_builder_setter.rs`
- `crates/orderbook/tests/quote_echo_contract.rs`
- `crates/trading/tests/quote_projection_parity.rs`
- `crates/trading/tests/post_contract.rs`
