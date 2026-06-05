//! Shared EIP-712 reference fixtures for the alloy-signer test suite.

#![allow(
    dead_code,
    reason = "each signer test binary imports this shared module but uses only the subset of EIP-712 fixtures its own vectors need"
)]

use cow_sdk_contracts::hash_order;
use cow_sdk_core::{Hash32, OrderData, SupportedChainId};
use cow_sdk_signing::get_domain;

/// The canonical `CoW` order signing vector shared across the signer suites.
pub fn sample_order() -> OrderData {
    cow_sdk_test_utils::builders::OrderBuilder::upstream_signing().build()
}

/// The order digest for `order` under the mainnet `GPv2` domain.
pub fn order_digest(order: &OrderData) -> Hash32 {
    hash_order(&get_domain(SupportedChainId::Mainnet, None).unwrap(), order).unwrap()
}
