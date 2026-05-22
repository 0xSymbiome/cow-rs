//! Order hashing, UID packing, and EIP-712 metadata.

use serde::{Deserialize, Serialize};

use crate::primitives::ORDER_UID_LENGTH_BYTES;

pub use self::sol_cancellations::OrderCancellations as GPv2OrderCancellations;
pub use self::sol_types::Order as GPv2Order;
pub use self::{hash::*, types::*, uid::*};

pub(crate) mod hash;
pub(crate) mod sol_cancellations;
pub(crate) mod sol_types;
mod types;
mod uid;

/// Sentinel address used by the protocol to represent native ETH buys.
pub const BUY_ETH_ADDRESS: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
/// Encoded order UID length in bytes.
pub const ORDER_UID_LENGTH: usize = ORDER_UID_LENGTH_BYTES;

/// EIP-712 field descriptor used for `CoW` order-type metadata.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderTypeField {
    /// Field name.
    pub name: &'static str,
    /// Solidity field type.
    #[serde(rename = "type")]
    pub kind: &'static str,
}

impl OrderTypeField {
    /// Creates an order-type field descriptor.
    #[must_use]
    pub const fn new(name: &'static str, kind: &'static str) -> Self {
        Self { name, kind }
    }
}

/// Canonical order type fields in struct-hash order.
pub const ORDER_TYPE_FIELDS: [OrderTypeField; 12] = [
    OrderTypeField::new("sellToken", "address"),
    OrderTypeField::new("buyToken", "address"),
    OrderTypeField::new("receiver", "address"),
    OrderTypeField::new("sellAmount", "uint256"),
    OrderTypeField::new("buyAmount", "uint256"),
    OrderTypeField::new("validTo", "uint32"),
    OrderTypeField::new("appData", "bytes32"),
    OrderTypeField::new("feeAmount", "uint256"),
    OrderTypeField::new("kind", "string"),
    OrderTypeField::new("partiallyFillable", "bool"),
    OrderTypeField::new("sellTokenBalance", "string"),
    OrderTypeField::new("buyTokenBalance", "string"),
];

/// Canonical EIP-712 field descriptor for order-cancellation payloads.
pub const CANCELLATIONS_TYPE_FIELDS: [OrderTypeField; 1] =
    [OrderTypeField::new("orderUids", "bytes[]")];
