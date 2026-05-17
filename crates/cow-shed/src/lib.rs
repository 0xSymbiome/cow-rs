#![cfg_attr(doctest, doc = include_str!("../README.md"))]

//! Pure COW Shed proxy, EIP-712, and calldata helpers.
//!
//! This crate intentionally stops at deterministic building blocks. It does
//! not own RPC providers, service loops, persistence, or automatic order
//! submission.

#![warn(missing_docs)]

/// CREATE2 proxy address derivation.
pub mod address;
/// Generated ABI bindings for the COW Shed contracts.
pub mod bindings;
/// ABI calldata builders.
pub mod calls;
/// EIP-712 domain and message hashing.
pub mod eip712;
/// Error taxonomy for COW Shed helpers.
pub mod errors;
/// Public helper types.
pub mod types;
/// Supported deployed COW Shed versions.
pub mod version;

pub use address::proxy_of;
pub use calls::{
    encode_execute_hooks_calldata, encode_execute_pre_signed_hooks_calldata,
    eoa_signature_from_compact,
};
pub use eip712::{
    ExecuteHooks, SolCall, cow_shed_domain_separator, execute_hooks_message_hash, hash_to_sign,
};
pub use errors::{CowShedError, SigSource};
pub use types::{Call, CallExt, Deadline, Hook, HookList, Nonce, ProxyAddress};
pub use version::CowShedVersion;
