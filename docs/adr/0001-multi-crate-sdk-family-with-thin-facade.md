# ADR 0001: Multi-Crate SDK Family With Thin Facade

**Status:** Accepted  
**Date:** 2026-04-09  
**Author:** 0xSymbiotic  

## 1. Context and Problem Statement

The Rust SDK needs to cover low-level protocol transforms, transport clients, trading workflows, and browser support without collapsing into one oversized crate.

## 2. Alternatives Considered

- Put the entire SDK into a single crate
- Use a root crate that hides most implementation details internally
- Split the SDK into focused leaf crates with a small root facade

## 3. Decision

Use a multi-crate workspace where leaf crates own behavior and `cow-sdk` remains a thin public facade.

## 4. Rationale

This keeps semver boundaries clear, limits dependency spread, makes targeted testing easier, and lets advanced consumers adopt only the crates they need.

## 5. Protocol and Runtime Implications

- **Determinism:** Protocol transforms stay isolated in dedicated crates instead of being reimplemented in multiple places.
- **Security:** Smaller crates reduce hidden behavior and make review easier.
- **Runtime:** Native, async, and WASM surfaces can evolve without forcing one runtime model on every consumer.
- **Dependencies:** Browser-only dependencies stay out of the default root surface.

## 6. Consequences

- **Positive:** Clear package boundaries, direct leaf-crate reuse, cleaner publication story.
- **Negative:** More crate coordination and more public package surfaces to maintain.
