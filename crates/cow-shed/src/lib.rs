#![cfg_attr(doctest, doc = include_str!("../README.md"))]

//! COW Shed proxy, EIP-712, calldata, and hook-signing helpers.
//!
//! The crate offers two layers over the COW Shed account-abstraction proxy:
//!
//! - **Building blocks** — deterministic, provider-free primitives: proxy
//!   address derivation ([`proxy_of`], [`proxy_for`]), EIP-712 domain and
//!   signing hash ([`cow_shed_eip712_domain`], [`execute_hooks_signing_hash`]),
//!   the typed-data payload ([`execute_hooks_typed_data_payload`]), and ABI
//!   calldata builders ([`encode_execute_hooks_calldata_signed`] for an EOA
//!   signature, [`encode_execute_hooks_calldata_with_signature`] for any owner
//!   including EIP-1271).
//! - **Orchestrator** — [`CowShedHooks`], which composes those blocks plus an
//!   owned [`Signer`](cow_sdk_core::Signer) into a single
//!   [`sign`](CowShedHooks::sign) call returning a ready-to-submit
//!   [`SignedCowShedCall`] that can also be attached to a `CoW` order as a hook.
//!
//! The crate never owns an RPC provider, service loop, persistence, gas
//! estimation, or automatic order submission: signing is delegated to the
//! caller's [`Signer`](cow_sdk_core::Signer), and submission stays the caller's
//! responsibility.

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
/// High-level hook-signing orchestrator.
pub mod hooks;
/// Public helper types.
pub mod types;
/// Supported deployed COW Shed versions.
pub mod version;

pub use address::{cow_shed_factory, cow_shed_implementation, proxy_for, proxy_of};
pub use calls::{
    compact_signature, encode_execute_hooks_calldata, encode_execute_hooks_calldata_signed,
    encode_execute_hooks_calldata_with_signature, encode_execute_pre_signed_hooks_calldata,
    eoa_signature_from_compact,
};
pub use eip712::{
    ExecuteHooks, SolCall, cow_shed_domain_separator, cow_shed_eip712_domain,
    execute_hooks_signing_hash, execute_hooks_typed_data_payload,
};
pub use errors::{CowShedError, SigSource};
pub use hooks::{CowShedHooks, SignedCowShedCall};
pub use types::{Call, Deadline, Hook, HookList, Nonce, ProxyAddress};
pub use version::CowShedVersion;
