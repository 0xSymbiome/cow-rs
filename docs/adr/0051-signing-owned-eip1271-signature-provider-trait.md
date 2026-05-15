# ADR 0051: Signing-Owned EIP-1271 Signature Provider

- Status: Accepted
- Date: 2026-05-15
- Last reviewed: 2026-05-15
- Tags: eip-1271, signing, trait-ownership, additive-leaf-crates
- Related: [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md), [ADR 0014](0014-eip1271-verification-cache.md), [ADR 0048](0048-composable-conditional-order-framework.md), [ADR 0050](0050-eip1271-signature-blob-encoding.md)

## Context

The trait that custom smart-account signers implement to plug their
`isValidSignature` callback into the CoW Protocol trading submission path
was previously declared inside `cow-sdk-trading::types::seams` along with
trading-orchestration-specific seams. That placement created a structural
problem for future composable and COW Shed helpers: those helpers need to
produce EIP-1271-authenticated signature payloads, but they must not depend
on the trading-orchestration crate because trading-orchestration is a
higher-layer leaf that consumes signing, not the other way around.

Keeping the trait in trading would force composable and COW Shed to depend
on trading, which would:

- break the additive-leaf-crates discipline from ADR 0008 (leaves must not
  depend on peer leaves);
- create a dependency cycle if composable ever needs to publish a custom
  signer alongside trading's signer consumer; and
- pull every transitive consumer of composable or COW Shed into the
  trading crate's dependency closure, including the orderbook HTTP
  surface and trade-specific error types.

The trait is also fundamentally a signing concern. The signer's contract
is "take an order hash and produce an EIP-1271 payload"; trading is one
consumer of that payload but not the natural owner of the trait. The
signing crate already owns custom-signature provider semantics for the
non-EIP-1271 paths.

A separate `From<Eip1271SignatureError> for TradingError` bridge would let
trading return `?` over signing failures, but the bridge would silently
collapse the typed signing error into the generic trading error and lose
the per-operation context. Inline `map_err` at every call site keeps the
operation name and the typed signing error visible at the trading layer.

## Decision

`Eip1271SignatureProvider` and `Eip1271SignatureError` live in
`cow_sdk_signing::eip1271`. Trading consumes the signing-owned trait by
importing the canonical path and maps provider failures into `TradingError`
inline at the call site using `map_err` with a per-operation message.

Trading must not re-export the provider trait at any path under
`cow_sdk_trading`. Trading must not add a blanket
`From<Eip1271SignatureError> for TradingError` bridge. Any leaf crate that
needs the trait imports the canonical signing path; no parallel trait
definition exists in any other crate.

The negative-edge invariants `cow-sdk-signing ⇏ cow-sdk-trading`,
`cow-sdk-composable ⇏ cow-sdk-trading`, and
`cow-sdk-cow-shed ⇏ cow-sdk-trading` are asserted via `cargo metadata` and
the `parity-maintainer check-deps` validator in CI. The reverse-edge guard
`cow-sdk-trading ⇒ cow-sdk-signing` continues to hold.

A compile-fail regression test asserts that any future re-export of
`Eip1271SignatureProvider` from `cow_sdk_trading::types::seams` fails to
compile. This regression makes the single-canonical-path contract
checkable at build time.

## Why

Owning the trait in signing aligns trait placement with the trait's
semantic responsibility: signing produces signature payloads, trading
submits them. The semantic placement matches the dependency direction
trading → signing and prevents the cyclic pressure that would arise if
composable or COW Shed needed to depend on trading just to import the
trait.

Inline `map_err` at trading call sites surfaces every EIP-1271 failure
with the operation name visible in the error message. A blanket `From`
bridge would collapse the typed signing context into a generic trading
error and force every consumer to grep the call site to figure out which
operation failed.

The compile-fail regression makes the contract enforceable rather than
aspirational. Reviewers can rely on the type system rather than reading
the ADR every time a trait import lands in a new file.

## Must Remain True

- Public surface: `Eip1271SignatureProvider` and
  `Eip1271SignatureError` are reachable exclusively from
  `cow_sdk_signing::eip1271`. No re-export from any other crate's public
  surface.
- Runtime and support: trading-side call sites that surface EIP-1271
  failures use inline `map_err` with a per-operation message. No
  `From<Eip1271SignatureError> for TradingError` impl exists anywhere in
  the workspace.
- Crate graph: `cargo metadata` continues to prove
  `cow-sdk-signing ⇏ cow-sdk-trading`,
  `cow-sdk-composable ⇏ cow-sdk-trading`,
  `cow-sdk-cow-shed ⇏ cow-sdk-trading`.
- Validation and review: the compile-fail regression at
  `crates/trading/tests/eip1271_signature_provider_no_reexport.rs`
  continues to fail to compile when any future re-export of
  `Eip1271SignatureProvider` lands inside trading.
- Cost: trading-side call sites carry one inline `map_err` per
  EIP-1271-surfacing entry point rather than a blanket `From` impl. This
  is intentional.

## Alternatives Rejected

- Keep the trait inside `cow-sdk-trading::types::seams`: would force
  composable and COW Shed to depend on trading, breaking the
  additive-leaf-crates discipline.
- Add a blanket `From<Eip1271SignatureError> for TradingError` bridge:
  would let trading return `?` over signing failures but collapse the
  typed signing error into the generic trading error and lose
  per-operation context.
- Re-export the trait from trading at the old path: would create two
  canonical paths and confuse downstream callers; any future renaming
  would have to touch both.
- Duplicate the trait definition in `cow-sdk-composable` and
  `cow-sdk-cow-shed`: would let every leaf crate implement its own
  variant and break interop with the trading submission path.

## Links

- [Architecture](../architecture.md)
- [Principles](../principles.md)
- [ADR 0014](0014-eip1271-verification-cache.md)
- [ADR 0048](0048-composable-conditional-order-framework.md)
- [ADR 0050](0050-eip1271-signature-blob-encoding.md)

**Proven by:**

- [Composable Contract Bindings Audit](../audit/composable-contract-bindings-audit.md)
- [COW Shed Contract Bindings Audit](../audit/cow-shed-contract-bindings-audit.md)
