#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Runtime-neutral helpers shared by the SDK wasm surface.
//!
//! This crate hosts the pure helper modules used by `cow-sdk-wasm`. The helpers
//! wrap canonical SDK primitives for chain handling, app-data conversion,
//! signing payloads, and order UID formatting without introducing JavaScript FFI
//! dependencies.

pub mod app_data;
pub mod chains;
pub mod dto;
pub mod errors;
pub mod signing;
pub mod uid;
