use serde::{Deserialize, Serialize};

use cow_sdk_core::{Address, OrderDigest, OrderUid};

/// Structured order UID components.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderUidParams {
    /// Order digest.
    pub order_digest: OrderDigest,
    /// Order owner address.
    pub owner: Address,
    /// Order expiration timestamp.
    pub valid_to: u32,
}

/// EIP-712 message body for order cancellations.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderCancellations {
    /// Order UIDs being cancelled.
    pub order_uids: Vec<OrderUid>,
}

impl OrderUidParams {
    /// Creates structured order UID components.
    #[must_use]
    pub const fn new(order_digest: OrderDigest, owner: Address, valid_to: u32) -> Self {
        Self {
            order_digest,
            owner,
            valid_to,
        }
    }
}

impl OrderCancellations {
    /// Creates an order-cancellation payload.
    #[must_use]
    pub const fn new(order_uids: Vec<OrderUid>) -> Self {
        Self { order_uids }
    }
}
