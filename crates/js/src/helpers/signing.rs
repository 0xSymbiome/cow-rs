//! Signing payload helpers.

use cow_sdk_core::{Address, OrderData, SupportedChainId, TypedDataPayload};
use cow_sdk_signing::{GeneratedOrderId, SigningError};

/// Builds signer-facing EIP-712 typed data for an order.
///
/// # Errors
///
/// Returns [`SigningError`] when domain construction or serialization fails.
pub fn order_typed_data_payload(
    chain_id: SupportedChainId,
    order: &OrderData,
) -> Result<TypedDataPayload, SigningError> {
    cow_sdk_signing::domain::order_typed_data_payload(chain_id, order, None)
}

/// Generates the compact order UID plus digest through the signing crate.
///
/// # Errors
///
/// Returns [`SigningError`] when domain construction, hashing, or UID packing fails.
pub fn generate_order_id(
    chain_id: SupportedChainId,
    order: &OrderData,
    owner: &Address,
) -> Result<GeneratedOrderId, SigningError> {
    cow_sdk_signing::generate_order_id(chain_id, order, owner, None)
}

/// Encodes an EIP-1271 payload from an existing ECDSA signature.
///
/// # Errors
///
/// Returns [`SigningError`] when order normalization or ABI-style encoding fails.
pub fn eip1271_signature_payload(
    order: &OrderData,
    ecdsa_signature: &str,
) -> Result<String, SigningError> {
    cow_sdk_signing::eip1271_signature_payload(order, ecdsa_signature)
}
