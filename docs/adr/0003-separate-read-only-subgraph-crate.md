# ADR 0003: Separate Read-Only Subgraph Crate

**Status:** Accepted  
**Date:** 2026-04-09  
**Author:** 0xSymbiotic  

## 1. Context and Problem Statement

Subgraph queries serve analytics and reporting use cases, but they are operationally different from order submission and orderbook transport.

## 2. Alternatives Considered

- Re-export subgraph helpers from the root `cow-sdk` facade by default
- Fold GraphQL access into `cow-sdk-orderbook`
- Keep a dedicated `cow-sdk-subgraph` crate

## 3. Decision

Expose subgraph functionality through a separate, read-only `cow-sdk-subgraph` crate.

## 4. Rationale

This keeps GraphQL concerns separate from trading flows, preserves a trading-first root facade, and avoids turning analytics helpers into hidden dependencies for every SDK consumer.

## 5. Protocol and Runtime Implications

- **Determinism:** Query documents and typed result models stay explicit and reviewable.
- **Security:** The crate remains read-only and does not gain order submission or mutation behavior.
- **Runtime:** GraphQL transport stays isolated from HTTP orderbook transport and wallet flows.
- **Dependencies:** Consumers who do not need analytics do not pull in subgraph code.

## 6. Consequences

- **Positive:** Cleaner scope, clearer mental model, easier future expansion if the upstream subgraph surface grows.
- **Negative:** Consumers who want one import path for everything need to add one extra crate.
