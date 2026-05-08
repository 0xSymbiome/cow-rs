//! Host-safe protocol helpers for the TypeScript-callable wasm leaf.
//!
//! Every module under this tree compiles for both native and
//! `wasm32-unknown-unknown` targets. None of them use
//! `wasm-bindgen` derives, `tsify` derives, or `JsValue` references.
//! The host gate (`cargo check -p cow-sdk-wasm --no-default-features`)
//! is the structural proof that this constraint holds: any leak
//! breaks the host build immediately.
//!
//! Concrete helper modules are populated by the public-surface
//! follow-up; this module is intentionally empty in the scaffolding
//! commit so the host gate runs against an unambiguous baseline.
