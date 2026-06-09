//! Const-backed registry of canonical `CoW` Protocol contract deployments.
//!
//! The module ships the typed authority for resolving a deployed contract
//! address from the `(ContractId, DeploymentChainId, DeploymentEnv)` key
//! triple. [`Registry::default`] returns the canonical registry and
//! [`Registry::address`] is the primary lookup API. The settlement,
//! vault-relayer, and eth-flow contracts are CREATE2 singletons, so the
//! registry is a small const table; each address is pinned to its upstream
//! source repository in `parity/source-lock.yaml` and confirmed on-chain by a
//! read-only `eth_getCode` presence probe.

mod chain_id;
mod contract_id;
mod env;
mod registry;

pub use chain_id::{DeploymentChainId, DeploymentChainIdError};
pub use contract_id::ContractId;
pub use env::DeploymentEnv;
pub use registry::Registry;
