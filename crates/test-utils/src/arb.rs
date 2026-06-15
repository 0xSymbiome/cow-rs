//! Shared `proptest` strategies for the workspace property suites.
//!
//! Only the strategies that are identical across crates live here; the
//! per-crate variants (different ranges, value types, or value shapes) stay
//! local to their `property_contract.rs`.

use cow_sdk_core::{Address, Amount, AppDataHash, SupportedChainId};
use proptest::prelude::*;

/// A strategy emitting an [`Address`] with a non-zero low byte, so downstream
/// helpers never observe the canonical-zero address boundary they reject
/// outside the property under test.
///
/// # Panics
/// Never panics — every sampled value is a valid 20-byte hex address.
pub fn arb_address() -> impl Strategy<Value = Address> {
    any::<[u8; 20]>().prop_map(|mut bytes| {
        if bytes.iter().all(|byte| *byte == 0) {
            bytes[19] = 1;
        }
        Address::new(alloy_primitives::hex::encode_prefixed(bytes))
            .expect("byte-derived address is valid")
    })
}

/// A strategy emitting an [`Amount`] with at least one non-zero byte, so
/// amount inputs stay outside the all-zero boundary.
///
/// # Panics
/// Never panics — every sampled value is a valid 32-byte hex amount.
pub fn arb_amount() -> impl Strategy<Value = Amount> {
    any::<[u8; 32]>().prop_map(|mut bytes| {
        if bytes.iter().all(|byte| *byte == 0) {
            bytes[31] = 1;
        }
        Amount::new(alloy_primitives::hex::encode_prefixed(bytes))
            .expect("byte-derived amount is valid")
    })
}

/// A strategy emitting an [`AppDataHash`] payload.
///
/// # Panics
/// Never panics — every sampled value is a valid 32-byte hex digest.
pub fn arb_app_data_hex() -> impl Strategy<Value = AppDataHash> {
    any::<[u8; 32]>().prop_map(|bytes| {
        AppDataHash::new(alloy_primitives::hex::encode_prefixed(bytes))
            .expect("byte-derived app-data hex is valid")
    })
}

/// A strategy emitting every supported chain id.
pub fn arb_supported_chain_id() -> impl Strategy<Value = SupportedChainId> {
    prop::sample::select(SupportedChainId::ALL.to_vec())
}
