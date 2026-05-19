use cow_sdk_core::{Address, OrderDigest, OrderUid, TypedDataDomain};

use super::{ORDER_UID_LENGTH, Order, OrderUidParams, hash::hash_order};
use crate::ContractsError;

/// Computes the encoded order UID for an order and owner.
///
/// # Errors
///
/// Returns [`ContractsError`] if order hashing or UID packing fails.
#[inline]
pub fn compute_order_uid(
    domain: &TypedDataDomain,
    order: &Order,
    owner: &Address,
) -> Result<OrderUid, ContractsError> {
    pack_order_uid_params(&OrderUidParams::new(
        hash_order(domain, order)?,
        *owner,
        order.valid_to,
    ))
}

/// Packs structured order UID components into the compact UID string.
///
/// # Errors
///
/// Returns [`ContractsError`] if the digest or owner cannot be decoded into the
/// fixed byte lengths required by the UID format.
#[inline]
pub fn pack_order_uid_params(params: &OrderUidParams) -> Result<OrderUid, ContractsError> {
    let digest = params.order_digest.into_alloy().0;
    let owner = params.owner.into_alloy().0.0;
    let mut out = [0u8; ORDER_UID_LENGTH];
    out[..32].copy_from_slice(&digest);
    out[32..52].copy_from_slice(&owner);
    out[52..56].copy_from_slice(&params.valid_to.to_be_bytes());
    Ok(OrderUid::from_bytes(out))
}

/// Extracts structured order UID components from a compact UID string.
///
/// # Errors
///
/// Returns [`ContractsError`] if the UID cannot be decoded into the expected format.
#[inline]
pub fn extract_order_uid_params(order_uid: &OrderUid) -> Result<OrderUidParams, ContractsError> {
    let bytes = order_uid.as_slice();
    if bytes.len() != ORDER_UID_LENGTH {
        return Err(ContractsError::InvalidOrderUidLength {
            actual: bytes.len(),
        });
    }

    let order_digest = OrderDigest::new(format!("0x{}", hex::encode(&bytes[..32])))?;
    let owner = Address::new(format!("0x{}", hex::encode(&bytes[32..52])))?;
    let valid_to_bytes: [u8; 4] =
        bytes[52..56]
            .try_into()
            .map_err(|_| ContractsError::InvalidOrderUidLength {
                actual: bytes.len(),
            })?;
    let valid_to = u32::from_be_bytes(valid_to_bytes);

    Ok(OrderUidParams::new(order_digest, owner, valid_to))
}
