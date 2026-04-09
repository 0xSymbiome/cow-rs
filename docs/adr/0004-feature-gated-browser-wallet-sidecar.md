# ADR 0004: Feature-Gated Browser Wallet Sidecar

**Status:** Accepted  
**Date:** 2026-04-09  
**Author:** 0xSymbiotic  

## 1. Context and Problem Statement

WASM support is required, and browser wallets use async, injected EIP-1193 providers that do not fit a native-first runtime model.

## 2. Alternatives Considered

- Treat browser support as raw private-key handling inside examples
- Add browser globals and wallet shims directly to the root SDK
- Build a dedicated `cow-sdk-browser-wallet` sidecar on top of shared async traits

## 3. Decision

Implement browser wallet support in `cow-sdk-browser-wallet` and expose it from `cow-sdk` only behind the `browser-wallet` feature.

## 4. Rationale

This keeps browser-only dependencies out of default builds, avoids hidden global runtime state, and gives mock and injected wallets the same typed async integration surface.

## 5. Protocol and Runtime Implications

- **Determinism:** Signing payload construction remains owned by pure signing code; the browser layer only provides runtime integration.
- **Security:** Wallet connection and request flows stay explicit instead of being triggered implicitly.
- **Runtime:** The browser layer is WASM-focused and async by design, while native consumers keep the default surface lean.
- **Dependencies:** `wasm-bindgen`, `web-sys`, and related browser dependencies stay out of core crates.

## 6. Consequences

- **Positive:** Cleaner runtime separation, better browser ergonomics, and a credible web application story.
- **Negative:** Browser users need an explicit feature or direct dependency on the sidecar crate.
