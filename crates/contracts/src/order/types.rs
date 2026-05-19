use serde::{Deserialize, Serialize};

use cow_sdk_core::{
    Address, Amount, AppDataHash, BuyTokenDestination, OrderDigest, OrderKind, OrderUid,
    SellTokenSource,
};

use super::hash::normalize_order;
use crate::ContractsError;

/// Contract ABI and EIP-712 order payload.
///
/// This type intentionally differs from `cow_sdk_core::UnsignedOrder`: receiver
/// and token-balance fields are optional here because the contract hashing
/// boundary applies `CoW` Protocol defaults during normalization.
///
/// Convert user-domain orders into this type through the canonical
/// [`cow_sdk_core::UnsignedOrder`] boundary.
///
/// ```
/// use cow_sdk_contracts::Order;
/// use cow_sdk_core::{
///     Address, Amount, AppDataHash, BuyTokenDestination, OrderKind, SellTokenSource,
///     UnsignedOrder,
/// };
///
/// let unsigned = UnsignedOrder::new(
///     Address::new("0x1111111111111111111111111111111111111111").unwrap(),
///     Address::new("0x2222222222222222222222222222222222222222").unwrap(),
///     Address::new("0x3333333333333333333333333333333333333333").unwrap(),
///     Amount::new("100").unwrap(),
///     Amount::new("200").unwrap(),
///     1_700_000_000,
///     AppDataHash::new(
///         "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
///     )
///     .unwrap(),
///     Amount::new("5").unwrap(),
///     OrderKind::Sell,
///     false,
///     SellTokenSource::External,
///     BuyTokenDestination::Internal,
/// );
///
/// let order = Order::from(&unsigned);
/// assert_eq!(order.valid_to, unsigned.valid_to);
/// assert_eq!(order.fee_amount, unsigned.fee_amount);
/// ```
#[doc = concat!(
    "```compile_fail\n",
    "use cow_sdk_contracts::{hash_order_for_", "contract, uid_for_", "contract};\n",
    "use cow_sdk_core::{Order", "Model, Quote", "Model};\n",
    "\n",
    "fn main() {}\n",
    "```\n",
)]
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// Sell token address.
    pub sell_token: Address,
    /// Buy token address.
    pub buy_token: Address,
    /// Optional receiver. Missing values normalize to `address(0)`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    /// Sell amount.
    pub sell_amount: Amount,
    /// Buy amount.
    pub buy_amount: Amount,
    /// Expiration timestamp.
    pub valid_to: u32,
    /// App-data hash.
    pub app_data: AppDataHash,
    /// Fee amount.
    pub fee_amount: Amount,
    /// Order side.
    pub kind: OrderKind,
    /// Whether the order is partially fillable.
    pub partially_fillable: bool,
    /// Optional sell-token balance source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_token_balance: Option<SellTokenSource>,
    /// Optional buy-token balance destination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_token_balance: Option<BuyTokenDestination>,
}

/// Canonical contract order used for struct hashing.
///
/// [`normalize_order`] creates this type after applying ABI-level defaults and
/// rejecting invalid receiver state. It is separate from [`Order`] so hashing
/// code cannot accidentally skip normalization.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedOrder {
    /// Sell token address.
    pub sell_token: Address,
    /// Buy token address.
    pub buy_token: Address,
    /// Normalized receiver address.
    pub receiver: Address,
    /// Sell amount.
    pub sell_amount: Amount,
    /// Buy amount.
    pub buy_amount: Amount,
    /// Expiration timestamp.
    pub valid_to: u32,
    /// App-data hash.
    pub app_data: AppDataHash,
    /// Fee amount.
    pub fee_amount: Amount,
    /// Order side.
    pub kind: OrderKind,
    /// Whether the order is partially fillable.
    pub partially_fillable: bool,
    /// Normalized sell-token balance source.
    pub sell_token_balance: SellTokenSource,
    /// Normalized buy-token balance destination.
    pub buy_token_balance: BuyTokenDestination,
}

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

impl Order {
    /// Creates a contract order payload.
    #[must_use]
    // Mirrors the full current public field set so callers can migrate off
    // struct literals without losing explicit control over any wire field.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        sell_token: Address,
        buy_token: Address,
        receiver: Option<Address>,
        sell_amount: Amount,
        buy_amount: Amount,
        valid_to: u32,
        app_data: AppDataHash,
        fee_amount: Amount,
        kind: OrderKind,
        partially_fillable: bool,
        sell_token_balance: Option<SellTokenSource>,
        buy_token_balance: Option<BuyTokenDestination>,
    ) -> Self {
        Self {
            sell_token,
            buy_token,
            receiver,
            sell_amount,
            buy_amount,
            valid_to,
            app_data,
            fee_amount,
            kind,
            partially_fillable,
            sell_token_balance,
            buy_token_balance,
        }
    }

    /// Returns the normalized contract order used for hashing and encoding.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError::ZeroReceiver`] when the receiver is explicitly
    /// set to the zero address.
    pub fn normalize(&self) -> Result<NormalizedOrder, ContractsError> {
        normalize_order(self)
    }
}

impl NormalizedOrder {
    /// Creates a normalized contract order payload.
    #[must_use]
    // Mirrors the full current public field set so callers can migrate off
    // struct literals without losing explicit control over any wire field.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        sell_token: Address,
        buy_token: Address,
        receiver: Address,
        sell_amount: Amount,
        buy_amount: Amount,
        valid_to: u32,
        app_data: AppDataHash,
        fee_amount: Amount,
        kind: OrderKind,
        partially_fillable: bool,
        sell_token_balance: SellTokenSource,
        buy_token_balance: BuyTokenDestination,
    ) -> Self {
        Self {
            sell_token,
            buy_token,
            receiver,
            sell_amount,
            buy_amount,
            valid_to,
            app_data,
            fee_amount,
            kind,
            partially_fillable,
            sell_token_balance,
            buy_token_balance,
        }
    }
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

impl From<&cow_sdk_core::UnsignedOrder> for Order {
    fn from(order: &cow_sdk_core::UnsignedOrder) -> Self {
        Self::new(
            order.sell_token,
            order.buy_token,
            Some(order.receiver),
            order.sell_amount.clone(),
            order.buy_amount.clone(),
            order.valid_to,
            order.app_data.clone(),
            order.fee_amount.clone(),
            order.kind,
            order.partially_fillable,
            Some(order.sell_token_balance),
            Some(order.buy_token_balance),
        )
    }
}
