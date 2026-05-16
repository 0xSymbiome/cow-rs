//! Public COW Shed helper types.

mod call;
mod deadline;
mod nonce;

pub use call::Call;
pub use cow_sdk_app_data::{Hook, HookList};
pub use deadline::Deadline;
pub use nonce::Nonce;

/// COW Shed proxy address type.
pub type ProxyAddress = alloy_primitives::Address;
