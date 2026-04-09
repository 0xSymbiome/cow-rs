# Open Questions

The repository is reviewable as-is. These are the main scope confirmations worth resolving before calling the first public milestone final.

## 1. Subgraph Scope

The current Rust surface mirrors the public TypeScript subgraph client: a separate, read-only `cow-sdk-subgraph` crate with `getTotals`, `getLastDaysVolume`, `getLastHoursVolume`, and generic custom query execution. The generated GraphQL schema is broader than that public API. Confirm whether Phase One should stay aligned with the current upstream public surface or whether specific higher-level analytics helpers should be added as first-class APIs.

## 2. Browser Wallet Acceptance Boundary

The current browser path is standards-based and async: `cow-sdk-browser-wallet` provides EIP-1193 integration, the root SDK exposes it behind a feature flag, and the WASM examples cover both deterministic mock-wallet proof and injected `window.ethereum` flows. Confirm whether this standards-level wallet boundary is the intended milestone target or whether a broader compatibility matrix should be treated as part of the first acceptance bar.

## 3. Coding Style and Documentation Preferences

The current codebase is intentionally light on inline comments and concentrates most explanation in tests, examples, and top-level docs. Confirm whether upstream prefers to keep Rust implementation files comment-light with documentation focused on public docs and examples, or whether future contributions should target deeper rustdoc and module-level commentary across the crate surfaces.
