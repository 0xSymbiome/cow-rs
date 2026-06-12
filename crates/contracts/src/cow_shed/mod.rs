//! COW Shed proxy, EIP-712, calldata, and hook-signing helpers.
//!
//! This module offers two layers over the COW Shed account-abstraction proxy.
//! It is gated behind the off-by-default `cow-shed` feature of `cow-sdk-contracts`
//! so a consumer that needs only the core contract helpers does not pull the
//! account-abstraction surface or its `cow-sdk-app-data` dependency.
//!
//! - **Building blocks** — deterministic, provider-free primitives: proxy
//!   address derivation ([`proxy_of`](crate::cow_shed::proxy_of),
//!   [`proxy_for`](crate::cow_shed::proxy_for)), EIP-712 domain and signing
//!   hash ([`cow_shed_eip712_domain`](crate::cow_shed::cow_shed_eip712_domain),
//!   [`execute_hooks_signing_hash`](crate::cow_shed::execute_hooks_signing_hash)),
//!   the typed-data payload
//!   ([`execute_hooks_typed_data_payload`](crate::cow_shed::execute_hooks_typed_data_payload)),
//!   and ABI calldata builders
//!   ([`encode_execute_hooks_calldata_signed`](crate::cow_shed::encode_execute_hooks_calldata_signed)
//!   for an EOA signature,
//!   [`encode_execute_hooks_calldata_with_signature`](crate::cow_shed::encode_execute_hooks_calldata_with_signature)
//!   for any owner including EIP-1271).
//! - **Orchestrator** — [`CowShedHooks`](crate::cow_shed::CowShedHooks), which
//!   composes those blocks plus an owned [`Signer`](cow_sdk_core::Signer) into
//!   a single [`sign`](crate::cow_shed::CowShedHooks::sign) call returning a
//!   ready-to-submit [`SignedCowShedCall`](crate::cow_shed::SignedCowShedCall)
//!   that can also be attached to a `CoW` order as a hook.
//!
//! The module never owns an RPC provider, service loop, persistence, gas
//! estimation, or automatic order submission: signing is delegated to the
//! caller's [`Signer`](cow_sdk_core::Signer), and submission stays the caller's
//! responsibility.
//!
//! The deployed factory and implementation are deterministic CREATE2
//! deployments, identical on every supported chain for a given
//! [`CowShedVersion`](crate::cow_shed::CowShedVersion), so the derived proxy
//! address is chain-independent — the same inputs always derive the same
//! proxy address, and chain id matters only for the EIP-712 signing domain:
//!
//! ```
//! use alloy_primitives::address;
//! use cow_sdk_contracts::cow_shed::{CowShedVersion, proxy_for};
//!
//! let user = address!("0x76b0340e50BD9883D8B2CA5fd9f52439a9e7Cf58");
//! let proxy = proxy_for(CowShedVersion::V1_0_1, user);
//! assert_eq!(
//!     proxy,
//!     address!("0x66545B93A314e5BdEC9E5Ff9c4D2C7054e6afb04"),
//! );
//! ```

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

pub use address::{cow_shed_factory, cow_shed_implementation, init_code_hash, proxy_for, proxy_of};
pub use calls::{
    encode_execute_hooks_calldata_signed, encode_execute_hooks_calldata_with_signature,
};
pub use eip712::{
    ExecuteHooks, SolCall, cow_shed_domain_separator, cow_shed_eip712_domain,
    execute_hooks_signing_hash, execute_hooks_typed_data_payload,
};
pub use errors::CowShedError;
pub use hooks::{CowShedHooks, SignedCowShedCall};
pub use types::{Call, Hook, HookList};
pub use version::CowShedVersion;
