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

- orderbook schema evidence is tied to `cowprotocol/services`
- subgraph evidence is tied to `cowprotocol/cow-sdk`
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
