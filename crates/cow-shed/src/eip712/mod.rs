//! COW Shed EIP-712 helpers.

mod domain;
mod hash;
mod type_hashes;

pub use domain::cow_shed_domain_separator;
pub use hash::{execute_hooks_message_hash, hash_to_sign};
pub use type_hashes::{CALL_TYPE_HASH, EIP712_DOMAIN_TYPE_HASH, EXECUTE_HOOKS_TYPE_HASH};
