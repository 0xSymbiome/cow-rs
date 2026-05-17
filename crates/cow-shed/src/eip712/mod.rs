//! COW Shed EIP-712 helpers.

mod domain;
mod hash;
pub mod sol_types;

pub use domain::cow_shed_domain_separator;
pub use hash::{execute_hooks_message_hash, hash_to_sign};
pub use sol_types::{Call as SolCall, ExecuteHooks};
