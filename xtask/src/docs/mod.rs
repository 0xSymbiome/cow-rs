//! Documentation-agreement gates behind `cargo xtask docs`.
//!
//! Rust ports of the former shell guards: [`agree`] keeps the published
//! release-gate commands identical across the docs and CI sites, and
//! [`audit_index`] keeps the audit index's review dates in lockstep with the
//! per-audit banners.

pub mod agree;
pub mod audit_index;
