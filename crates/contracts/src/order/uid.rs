use cow_sdk_core::{Address, OrderData, OrderDigest, OrderUid, TypedDataDomain};

use super::{ORDER_UID_LENGTH, OrderUidParams, hash::hash_order};
use crate::ContractsError;

/// Computes the encoded order UID for an order and owner.
///
/// # Errors
///
/// Returns [`ContractsError`] if order hashing or UID packing fails.
#[inline]
pub fn compute_order_uid(
    domain: &TypedDataDomain,
    order: &OrderData,
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
///
/// # Panics
///
/// Cannot panic in practice. The function returns early with
/// [`ContractsError::InvalidOrderUidLength`] when the byte length is
/// not exactly [`ORDER_UID_LENGTH`]; after that guard, the internal
/// 32-byte and 20-byte slice-to-array conversions are infallible by
/// construction. The `expect` calls inside the body document the
/// unreachability proof so a future contributor cannot accidentally
/// weaken the guard without removing the proof first.
#[inline]
pub fn extract_order_uid_params(order_uid: &OrderUid) -> Result<OrderUidParams, ContractsError> {
    let bytes = order_uid.as_slice();
    if bytes.len() != ORDER_UID_LENGTH {
        return Err(ContractsError::InvalidOrderUidLength {
            actual: bytes.len(),
        });
    }

    // SAFETY: the `bytes.len() != ORDER_UID_LENGTH` guard above guarantees
    // `bytes.len() == 56` here, so the `[..32]` and `[32..52]` slices are
    // always 32 and 20 bytes respectively and `try_into` cannot fail.
    let order_digest = OrderDigest::from_bytes(
        bytes[..32]
            .try_into()
            .expect("slice length 32 is guaranteed by the ORDER_UID_LENGTH check above"),
    );
    let owner = Address::from_bytes(
        bytes[32..52]
            .try_into()
            .expect("slice length 20 is guaranteed by the ORDER_UID_LENGTH check above"),
    );
    let valid_to_bytes: [u8; 4] =
        bytes[52..56]
            .try_into()
            .map_err(|_| ContractsError::InvalidOrderUidLength {
                actual: bytes.len(),
            })?;
    let valid_to = u32::from_be_bytes(valid_to_bytes);

    Ok(OrderUidParams::new(order_digest, owner, valid_to))
}
