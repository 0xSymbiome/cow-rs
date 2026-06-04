//! ABI calldata builders for COW Shed execution paths.

mod execute_hooks;
mod pre_sign;

pub use execute_hooks::{
    compact_signature, encode_execute_hooks_calldata, encode_execute_hooks_calldata_signed,
    encode_execute_hooks_calldata_with_signature, eoa_signature_from_compact,
};
pub use pre_sign::encode_execute_pre_signed_hooks_calldata;
