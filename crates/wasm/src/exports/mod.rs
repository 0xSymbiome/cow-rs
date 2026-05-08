//! `wasm-bindgen` surface for the TypeScript-callable wasm leaf.
//!
//! This module is gated `#[cfg(target_arch = "wasm32")]` and is
//! never compiled for native targets. It hosts the
//! `tsify`-derived DTOs, the `wasm-bindgen` exports, the four
//! typed wallet callback shapes (typed-data, EIP-1193, eth-sign,
//! custom EIP-1271), the JS callback HTTP transport with its
//! `TimerGuard` cleanup pattern, and the fetch-callback registry.
//!
//! Concrete export modules are populated by the public-surface
//! follow-up; this module is intentionally empty in the
//! scaffolding commit so the wasm32 target check runs against an
//! unambiguous baseline.
