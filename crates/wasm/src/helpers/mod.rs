//! Runtime-neutral protocol helpers shared by the wasm surface.
//!
//! These modules wrap canonical SDK primitives for chain handling, app-data
//! conversion, signing payloads, EIP-1271 payloads, and order UID formatting.
//! They carry no JavaScript FFI bindings, so they compile for both native and
//! `wasm32-unknown-unknown` targets and are exercised by host tests without a
//! wasm runtime. The JavaScript ABI lives in `crate::exports`; this module owns
//! only the deterministic host-safe composition the exports adapt.
//!
//! The FFI-free boundary is enforced by `tests/no_ffi_helpers.rs`, and helper
//! output parity with the wasm-facing modules by `tests/host_pure_helpers.rs`.

pub mod app_data;
pub mod chains;
pub mod dto;
pub mod errors;
pub mod signing;
