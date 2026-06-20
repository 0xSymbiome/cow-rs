# ADR 0050: EIP-1271 Signature Blob Encoding

- Status: Accepted
- Date: 2026-05-15
- Last reviewed: 2026-06-15
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: eip-1271, signature-encoding, composable, safe-muxer, erc1271-forwarder
- Related: [ADR 0014](0014-eip1271-verification-cache.md), [ADR 0048](0048-composable-conditional-order-framework.md), [ADR 0049](0049-cow-shed-account-abstraction-proxy.md), [ADR 0051](0051-signing-owned-eip1271-signature-provider-trait.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

> **Partially shipped.** Shape B (the raw `ERC1271Forwarder`
> `abi.encode(order, payload)` layout) ships today in `cow_sdk_signing::eip1271`
> (`OrderAndSignature` + `SolValue::abi_encode_sequence`, consumed by
> `eip1271_signature_payload`). Shape A (the Safe-muxer encoder), the
> `cow-sdk-composable` hosting crate, and the `parity/fixtures/composable/*`
> vectors are **deferred** with the composable capability (ADR 0048). The live
> guard for the no-shape-flag rule is the `eip1271-shape-flag` fence in
> `xtask/src/policy/fences.rs`. Present-tense claims about composable below
> describe the planned shape, not shipped code.

## Context

Composable conditional orders and COW Shed account-abstraction flows submit
opaque signature bytes to the CoW Protocol orderbook. The on-chain
`isValidSignature(bytes32 hash, bytes signature)` callback must accept those
bytes and route them to the registered handler or forwarder. The signature
bytes are not a raw ECDSA tuple: they are an ABI-encoded payload that
identifies the calling shape and carries the order plus the verification
payload.

Two payload shapes ship in the first release. The first is the Safe muxer
shape (Shape A): the bytes begin with a 4-byte selector for
`safeSignature(bytes32,bytes32,bytes,bytes)` and continue with the
ABI-encoded conditional-order parameters and EIP-1271 inner payload. Safe
multisigs use this shape because the muxer dispatches the signature to a
designated module before the Safe's normal signature path runs. The second
is the raw `ERC1271Forwarder` shape (Shape B): the bytes carry the
ABI-encoded `(GPv2Order.Data, bytes payload)` tuple with no selector
prefix. Custom smart-account signers that own their entire
`isValidSignature` callback use this shape because they decode the tuple
directly without a muxer.

The selector prefix is load-bearing for Shape A. If the prefix is omitted,
the Safe muxer cannot dispatch to the right module and the signature
verification reverts at the Safe layer rather than the handler. The
selector prefix is forbidden for Shape B. If the prefix is included, the
forwarder's ABI decode fails because the leading four bytes shift every
field offset by four.

The ABI type strings carry no whitespace between commas in declaration
order, because the upstream Solidity sources encode them that way and any
deviation would change the EIP-712 struct hash.

## Decision

The SDK recognizes exactly two EIP-1271 payload shapes and produces them
through two distinct encoder entry points. Custom smart-account signers
select the shape that matches their callback at construction time.

### Shape A — Safe Muxer Payload

The Shape A encoder produces:

```text
safeSignature(...)_selector(4 bytes) || abi.encode(domain_separator, type_hash, order, payload)
```

The selector is the 4-byte function selector for the muxer's
`safeSignature(bytes32 domain_separator, bytes32 type_hash, bytes order, bytes payload)`
entry point, matching the `safeSignature(bytes32,bytes32,bytes,bytes)`
signature named in the Context section. The selector value matches `forge methodIdentifiers` for the
canonical Safe muxer ABI. The encoder rejects callers that omit the
selector at construction time.

### Shape B — Raw `ERC1271Forwarder` Payload

The Shape B encoder produces:

```text
abi.encode(GPv2Order.Data, payload)
```

with no selector prefix. The encoder rejects callers that include a
selector prefix at construction time.

### Type Strings and Byte-Identity

The EIP-712 type strings used inside both shapes carry no whitespace between
commas in declaration order. Fixture parity tests assert byte-identity
against the pinned upstream test vectors at
`parity/fixtures/composable/safe_muxer_signature_blob.json` (Shape A) and
`parity/fixtures/composable/forwarder_signature_blob.json` (Shape B). Each
fixture carries at least five rows covering a range of order and payload
shapes; every row must encode byte-identically to the pinned vector.

### Public Surface Boundary

The signature-shape decision lives in `cow-sdk-composable` because the
encoder needs typed access to `ConditionalOrderParams`. The trait that
custom smart-account signers implement to plug their callback into the
trading submission path lives in `cow-sdk-signing` per
[ADR 0051](0051-signing-owned-eip1271-signature-provider-trait.md). Trading
consumes the trait and routes provider failures to `TradingError` at the
call site.

## Why

Two shapes cover the realistic deployment surface. Safe multisigs are by
far the most common smart account; they require the muxer prefix because
the muxer is a separate contract that dispatches to a module. Custom
smart-account signers that own their entire callback use the raw tuple
because they decode it directly without a muxer. A single-shape encoder
would force every custom signer to wrap a no-op muxer module just to
satisfy the encoder, and would produce verification failures whenever the
muxer module address changed.

Two distinct encoder entry points (rather than a single shape-flag
parameter) make the decision visible at the call site. A caller that picks
the wrong shape fails at construction rather than at verification. The
fixture parity tests make the byte-layout a regression target rather than
an implementation detail.

The whitespace-free type strings carry the byte-identity contract upstream.
The strings are not free to vary: they appear inside an EIP-712 struct
hash, and any whitespace change would shift every downstream byte. The
fixture parity assertions catch any future whitespace creep.

## Must Remain True

- Public surface: the two encoder entry points are distinct. Shape A
  always emits the muxer selector prefix; Shape B never emits it. Callers
  do not pass a shape flag.
- Runtime and support: the EIP-712 type strings stay whitespace-free
  between commas in declaration order, byte-identically with the pinned
  upstream test vectors.
- Validation and review: the shipped Shape-B layout is exercised by the
  signing-crate tests around `eip1271_signature_payload`, and the no-shape-flag
  rule is enforced by the `eip1271-shape-flag` fence. When composable lands, the
  fixture parity tests at
  `parity/fixtures/composable/safe_muxer_signature_blob.json` and
  `forwarder_signature_blob.json` must stay byte-exact against the pinned
  upstream vectors (those files do not exist yet).
- Crate graph: the trait that lets custom signers plug into the trading
  submission path lives in `cow-sdk-signing` per ADR 0051. When composable
  lands it consumes that canonical signing path directly; no parallel trait
  definition exists.
- Cost: any future shape addition requires a new ADR and a new fixture
  file; the two-shape boundary is not silently extendable.

## Alternatives Rejected

- Single-shape encoder with a `shape: ShapeKind` parameter: the wrong
  shape would fail at verification, not at construction. Two distinct
  entry points surface the mistake earlier.
- Always emit the muxer selector prefix: custom smart-account signers that
  do not run a muxer would fail every verification because the prefix
  shifts every field offset.
- Never emit the muxer selector prefix: Safe multisigs would fail every
  verification because the muxer would not dispatch to the right module.
- Move the encoder into `cow-sdk-signing`: the encoder needs typed access
  to `ConditionalOrderParams` from `cow-sdk-composable`; moving it to
  signing would force signing to depend on composable and break the leaf
  ordering.
- Tolerate whitespace between commas in EIP-712 type strings: the
  resulting struct hash would diverge from upstream and every signature
  would fail verification.

## Links

- [Architecture](../architecture.md)
- [ADR 0014](0014-eip1271-verification-cache.md)
- [ADR 0048](0048-composable-conditional-order-framework.md)
- [ADR 0049](0049-cow-shed-account-abstraction-proxy.md)
- [ADR 0051](0051-signing-owned-eip1271-signature-provider-trait.md)
