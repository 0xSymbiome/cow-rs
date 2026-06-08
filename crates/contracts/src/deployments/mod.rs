//! Chain-keyed registry of canonical CoW Protocol contract deployments.
//!
//! The module ships the typed authority for resolving a deployed contract
//! address from the `(ContractId, DeploymentChainId, DeploymentEnv)` key triple.
//! [`Registry::default`] loads the manifest committed at
//! `crates/contracts/registry.toml`; [`Registry::address`] is the primary
//! lookup API.
//!
//! The manifest is validated at compile time by `build.rs` (malformed
//! rows surface as a build failure with the offending manifest line) and
//! again at runtime by [`Registry::from_toml_str`], so the same taxonomy
//! of failures is visible to downstream consumers who pipe their own TOML
//! into the loader.

mod chain_id;
mod contract_id;
mod env;
mod registry;
mod verification;

pub use chain_id::{DeploymentChainId, DeploymentChainIdError};
pub use contract_id::{ContractId, ENVIRONMENT_AGNOSTIC_CONTRACTS};
pub use env::DeploymentEnv;
pub use registry::{Registry, RegistryError};
pub use verification::DeploymentVerificationStatus;
