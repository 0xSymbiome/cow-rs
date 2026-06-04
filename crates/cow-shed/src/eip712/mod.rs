//! COW Shed EIP-712 helpers.

mod domain;
mod hash;
pub mod sol_types;
mod typed_data;

pub use domain::{cow_shed_domain_separator, cow_shed_eip712_domain};
pub use hash::execute_hooks_signing_hash;
pub use sol_types::{Call as SolCall, ExecuteHooks};
pub use typed_data::execute_hooks_typed_data_payload;
