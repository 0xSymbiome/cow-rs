//! Typed sub-metadata shapes carried inside the app-data envelope.
//!
//! Every sub-module in this namespace narrows one of the reviewed metadata
//! sections from a free-form JSON object into a typed Rust value. The
//! typed values serialize byte-identically with the reviewed wire form and
//! carry construction-time validation so invalid hints are caught at the
//! client before a document would fail the reviewed schema.

/// Flash-loan hints consumed by the app-data metadata envelope.
pub mod flashloan;

pub use flashloan::FlashloanHints;
