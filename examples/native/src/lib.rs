//! Shared support scaffolding for the native example scenarios.
//!
//! Every example binary in this crate links the helpers in [`support`] to keep
//! each scenario source focused on the SDK calls it demonstrates. This crate is
//! an unpublished workspace member (`publish = false`), so the published-API
//! documentation and pedantic lints that govern the shipped crates are relaxed
//! here: the helpers are example fixtures, not a stable public surface.
#![allow(
    missing_docs,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::unreadable_literal,
    clippy::missing_const_for_fn,
    reason = "unpublished example-support scaffolding, not a shipped public API"
)]

pub mod support;
