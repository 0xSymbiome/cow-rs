# Parity Scope

Last reviewed: 2026-04-10

This document defines the review scope for parity evidence in `cow-rs`. The source lock is the portable authority for upstream producer commits and paths.

## Source Lock

Pinned sources live in `parity/source-lock.yaml`.

| Producer | Pinned role | Used for |
| --- | --- | --- |
| `cowprotocol/cow-sdk` | Primary | SDK ergonomics, trading flows, orderbook client shape, app-data behavior, subgraph query shape, and root SDK facade evidence. |
| `cowprotocol/contracts` | Primary | Contract hashing, order UID packing, signatures, settlement encoding, vault/proxy/reader helpers, and ABI-level behavior. |
| `cowprotocol/services` | Reference-only | Orderbook OpenAPI, order validation behavior, and app-data service behavior where the TypeScript SDK delegates to service contracts. |

Local upstream checkout paths are optional reviewer inputs. They must be independent git checkouts or worktrees at the pinned commits when used for live parity validation.

## Surface Boundaries

| Surface | Rust crate | Pinned evidence |
| --- | --- | --- |
| Core config and runtime contracts | `cow-sdk-core` | Common adapter, address, token, config, and order type sources from `cowprotocol/cow-sdk`. |
| Contracts | `cow-sdk-contracts` | `cowprotocol/contracts` TypeScript helpers, Solidity tests, and `contracts-ts` package tests. |
| Signing | `cow-sdk-signing` | Order signing utilities, typed-data helpers, and contract signing sources. |
| App-data | `cow-sdk-app-data` | App-data API helpers, schema imports, generated TypeScript schema references, and schema regression tests. |
| Orderbook | `cow-sdk-orderbook` | Orderbook TypeScript API/request/types sources plus `cowprotocol/services` OpenAPI and order validation references. |
| Trading | `cow-sdk-trading` | TypeScript trading package workflows and tests. |
| Subgraph | `cow-sdk-subgraph` | TypeScript subgraph API, GraphQL, query, and test sources. |
| SDK facade | `cow-sdk` | TypeScript SDK root package exports and typedoc entrypoint. |

## Schema Evidence Policy

Schema-derived evidence is a test and review aid, not a public API shortcut.

- Orderbook OpenAPI evidence is tied to `cowprotocol/services` entries in `parity/source-lock.yaml`.
- Subgraph evidence is tied to `cowprotocol/cow-sdk` subgraph query and API entries in `parity/source-lock.yaml`.
- Orderbook source-schema review includes `crates/orderbook/openapi.yml`, `crates/shared/src/order_validation.rs`, and `crates/orderbook/src/app_data.rs`.
- Subgraph source-schema review includes `packages/subgraph/src/api.ts`, `packages/subgraph/src/graphql.ts`, and `packages/subgraph/src/queries.ts`.
- Generated or schema-derived Rust mirrors must live in obvious internal or test-only locations.
- Public DTOs remain hand-reviewed SDK contracts.

## Current Schema-Derived Artifacts

No generated or schema-derived Rust mirrors are part of the public SDK API. Current evidence is committed as fixtures, contract tests, and source-lock references.
