//! Internal shared test helpers for the `cow-rs` workspace.
//!
//! This crate is `publish = false` and exists only as a `[dev-dependencies]`
//! target for the workspace's tests. It must never appear in any published
//! crate's normal dependency graph.
//!
//! Modules: `consts` (canonical test constants), `eip712` (an independent
//! keccak/ABI-word oracle), `fixtures` (parity-fixture loaders), `builders`
//! (order/domain/signature fixtures), `mocks` (recording `Signer`),
//! `trace` (a span/event capturing subscriber), and `arb` (shared `proptest`
//! strategies, behind the `proptest` feature).

#[cfg(feature = "proptest")]
pub mod arb;
pub mod builders;
pub mod consts;
pub mod eip712;
pub mod fixtures;
pub mod mocks;
pub mod trace;
