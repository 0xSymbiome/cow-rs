# Parity Scope

This document defines the review scope for parity evidence in `cow-rs`.

The source lock is the portable authority for upstream producer commits and
paths.

## Source Lock

Pinned sources live in `parity/source-lock.yaml`.

| Producer | Pinned role | Used for |
| --- | --- | --- |
| `cowprotocol/cow-sdk` | Primary | SDK ergonomics, trading flows, orderbook client shape, app-data behavior, subgraph query shape, and root facade evidence |
| `cowprotocol/contracts` | Primary | Contract hashing, order UID packing, signatures, settlement encoding, and ABI-level behavior |
| `cowprotocol/services` | Reference-only | Orderbook OpenAPI, order validation behavior, and app-data service behavior where the TypeScript SDK delegates to service contracts |

Local upstream checkout paths are optional validation inputs. When they are
used, they must be independent git checkouts or worktrees at the pinned
commits.

## Surface Boundaries

| Surface | Rust crate | Pinned evidence |
| --- | --- | --- |
| Core config and runtime contracts | `cow-sdk-core` | Common adapter, address, token, config, and selected shared type sources from `cowprotocol/cow-sdk` |
| Contracts | `cow-sdk-contracts` | `cowprotocol/contracts` helpers, Solidity tests, and selected `contracts-ts` package tests |
| Signing | `cow-sdk-signing` | Order-signing utilities, typed-data helpers, and contract-signing sources |
| App-data | `cow-sdk-app-data` | App-data helpers, schema imports, generated schema references, and schema regression tests |
| Orderbook | `cow-sdk-orderbook` | TypeScript orderbook sources plus selected `cowprotocol/services` OpenAPI and validation references |
| Trading | `cow-sdk-trading` | TypeScript trading workflows and tests |
| Subgraph | `cow-sdk-subgraph` | TypeScript subgraph API, GraphQL, query, and test sources |
| SDK facade | `cow-sdk` | TypeScript SDK root package exports and typedoc entrypoint |

## Schema Evidence Policy

Schema-derived evidence is a review aid, not a public API shortcut.

- orderbook schema evidence is tied to `cowprotocol/services`, including
  `crates/orderbook/openapi.yml`
- subgraph evidence is tied to `cowprotocol/cow-sdk`, including
  `packages/subgraph/src/queries.ts`
- canonical subgraph query documents live in
  `crates/subgraph/src/query_documents/`
- test-only subgraph schema and codegen evidence lives in
  `crates/subgraph/tests/schema_evidence/`
- generated or schema-derived Rust mirrors must stay non-public or test-only

## Schema-Derived Artifacts

No generated or schema-derived Rust mirrors are part of the public SDK API.

- orderbook evidence is committed as OpenAPI artifacts, fixtures, contract
  tests, and source-lock references
- subgraph evidence is committed as saved query documents, test-only schema
  snapshots, contract tests, and source-lock references

## Intentionally Out-of-Scope

Parity in `cow-rs` is byte-identity on implemented surfaces, not
feature-identity with the TypeScript SDK. The following upstream surfaces
are intentionally excluded from the Rust SDK because they carry no
pre-release user value, re-introduce known protocol footguns, or have
been superseded by a clearer typed boundary. Every exclusion below is
enforced in code (via a negative test or by removing the surface
entirely) so future contributors cannot quietly reintroduce the upstream
shape on the assumption that a missing positive fixture implies a gap.

See [ADR 0011](./adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md)
for the canonical typed-amount decision. The governing parity-scope
discipline is the four-layer defense documented in the architecture
record: a negative test that fails closed, a scope-doc entry that names
the exclusion, a cross-link to the owning ADR, and a risk-register
entry for anyone who later considers reintroducing the surface.

- **CIDv0 (`Qm...`) encoding and decoding** — the cow-protocol services
  backend enforces CIDv1 with the raw multicodec (`0x55`) over a
  keccak-256 multihash (`0x1b`) as the only supported CID shape; legacy
  CIDv0 (dag-pb + sha2-256) paths carry no pre-release user value.
  Consumers that need to resolve historical `Qm`-prefixed values use a
  general-purpose `cid` crate directly. Negative test:
  `crates/app-data/tests/v0_cid_is_out_of_scope.rs`.
- **Order-level `fee_amount` as a public builder setter or DTO field**
  — the cow-protocol services backend rejects orders that carry a
  non-zero order-level fee, so the submission path always wires
  `"feeAmount": "0"` and there is no reason to let a caller construct
  a non-zero value locally. The internal serializer preserves
  `"feeAmount": "0"` for EIP-712 struct-hash compatibility. Negative
  test:
  `crates/orderbook/tests/fee_amount_is_not_a_public_builder_setter.rs`.
- **Legacy quote-response fee descriptors `executedFeeAmount` and
  `fullFeeAmount`** — the current services schema surfaces executed
  fees through the canonical `executedFee` component and quote-response
  protocol fees through `protocolFeeBps`. The retired descriptors are
  not re-emitted on the cow-rs wire. Covered by the same order-response
  wire-shape regressions in
  `crates/orderbook/tests/fee_amount_is_not_a_public_builder_setter.rs`.
- **Legacy wire-string `Amount` wrapper** — the Rust SDK consolidated
  the canonical atomic amount to a single typed newtype
  `cow_sdk_core::Amount(BigUint)` with custom serde that preserves the
  decimal-string wire format. The retired wire-string wrapper is simply
  absent from the workspace; by design, there is no negative test
  because the type does not exist and the Rust compiler itself enforces
  the exclusion at every call site. Governed by
  [ADR 0011](./adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md).
- **`TypedOrder` alias on `cow-sdk-signing`** — the canonical
  pre-signature order state is `cow_sdk_core::UnsignedOrder`; the
  former `TypedOrder = UnsignedOrder` backward-compatibility alias is
  absent from the workspace. As with the retired wire-string `Amount`
  wrapper, there is no negative test because the type does not exist
  and the Rust compiler itself enforces the exclusion at every call
  site.
