# ADR 0002: Dedicated Trading Orchestration Crate

**Status:** Accepted  
**Date:** 2026-04-09  
**Author:** 0xSymbiotic  

## 1. Context and Problem Statement

Quote-to-order workflows span orderbook transport, contract helpers, signing, app-data, and approvals. That logic needs a stable home.

## 2. Alternatives Considered

- Put orchestration into `cow-sdk-orderbook`
- Put orchestration into the root `cow-sdk` facade
- Create a dedicated `cow-sdk-trading` crate

## 3. Decision

Place all user-facing trading workflows in `cow-sdk-trading`.

## 4. Rationale

This keeps the orderbook crate focused on typed transport, keeps the root facade thin, and gives higher-level consumers a single workflow layer for quote, sign, submit, cancel, and approval flows.

## 5. Protocol and Runtime Implications

- **Determinism:** Pure order construction, slippage logic, and app-data merge rules stay testable and explicit.
- **Security:** Approval and cancellation flows remain visible and chain-aware rather than hidden behind convenience wrappers.
- **Runtime:** Async wallet-backed flows can be added without changing transport or hashing crates.
- **Dependencies:** `cow-sdk-trading` depends on stable leaf crates instead of duplicating their behavior.

## 6. Consequences

- **Positive:** One clear home for high-level workflows and SDK ergonomics.
- **Negative:** The trading crate becomes the main integration point and must be disciplined about not absorbing unrelated low-level logic.
