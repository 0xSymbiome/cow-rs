#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    reason = "internal workspace tool: failures surface through anyhow context rather than a documented public API"
)]
#![allow(
    missing_docs,
    missing_debug_implementations,
    reason = "internal workspace tool: `xtask` is publish = false and is not a documented public API surface"
)]

//! Workspace maintenance library behind the `xtask` binary: upstream-parity
//! provenance ([`parity`]) and repository policy checks ([`policy`]). The
//! binary is dispatch only; every capability lives here so the test suite
//! exercises the same code paths CI runs through the cargo aliases.

pub mod docs;
pub mod parity;
pub mod policy;
