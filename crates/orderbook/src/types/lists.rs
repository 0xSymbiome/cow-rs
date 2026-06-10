use serde::{Deserialize, Serialize};

use super::{Address, Amount, OrderUid, TransactionHash, order::ExecutedProtocolFee};

/// Request DTO for listing an account's orders.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrdersQuery {
    /// Account owner whose orders should be listed.
    pub owner: Address,
    /// Pagination offset.
    #[serde(default)]
    pub offset: u32,
    /// Pagination limit.
    #[serde(default = "default_orders_limit")]
    pub limit: u32,
}

const fn default_orders_limit() -> u32 {
    1_000
}

impl OrdersQuery {
    /// Creates an order-list request with the upstream default pagination.
    #[must_use]
    pub const fn new(owner: Address) -> Self {
        Self {
            owner,
            offset: 0,
            limit: default_orders_limit(),
        }
    }

    /// Returns a copy of this request with an explicit pagination offset.
    #[must_use]
    pub const fn with_offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }

    /// Returns a copy of this request with an explicit pagination limit.
    #[must_use]
    pub const fn with_limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }
}

/// Request DTO for listing trades by owner or order UID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradesQuery {
    /// Optional owner filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Address>,
    /// Optional order-UID filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_uid: Option<OrderUid>,
    /// Pagination offset.
    #[serde(default)]
    pub offset: u32,
    /// Pagination limit.
    #[serde(default = "default_trades_limit")]
    pub limit: u32,
}

const fn default_trades_limit() -> u32 {
    10
}

impl TradesQuery {
    /// Creates a trades request with raw owner and order-UID filter slots.
    ///
    /// Prefer [`by_owner`](Self::by_owner) or
    /// [`by_order_uid`](Self::by_order_uid) for the supported shapes; this
    /// constructor exists so callers can materialize malformed requests for
    /// validation coverage.
    #[must_use]
    pub const fn new(owner: Option<Address>, order_uid: Option<OrderUid>) -> Self {
        Self {
            owner,
            order_uid,
            offset: 0,
            limit: default_trades_limit(),
        }
    }

    /// Creates a trades request filtered by owner.
    #[must_use]
    pub const fn by_owner(owner: Address) -> Self {
        Self::new(Some(owner), None)
    }

    /// Creates a trades request filtered by order UID.
    #[must_use]
    pub const fn by_order_uid(order_uid: OrderUid) -> Self {
        Self::new(None, Some(order_uid))
    }

    /// Returns a copy of this request with an explicit pagination offset.
    #[must_use]
    pub const fn with_offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }

    /// Returns a copy of this request with an explicit pagination limit.
    #[must_use]
    pub const fn with_limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    /// Returns `true` when exactly one of `owner` or `order_uid` is set.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.owner.is_some() ^ self.order_uid.is_some()
    }
}

/// Trade DTO returned by the orderbook trades endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Trade {
    /// Block number containing the trade event.
    pub block_number: u64,
    /// Log index within the block.
    pub log_index: u64,
    /// Order UID associated with the trade.
    pub order_uid: OrderUid,
    /// Owner address.
    pub owner: Address,
    /// Sell-token address.
    pub sell_token: Address,
    /// Buy-token address.
    pub buy_token: Address,
    /// Executed sell amount in the upstream decimal-string wire shape.
    pub sell_amount: Amount,
    /// Executed sell amount before fees.
    #[serde(default)]
    pub sell_amount_before_fees: Amount,
    /// Executed buy amount in the upstream decimal-string wire shape.
    pub buy_amount: Amount,
    /// Protocol fees executed as part of the trade, when services returns them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executed_protocol_fees: Option<Vec<ExecutedProtocolFee>>,
    /// Settlement transaction hash.
    pub tx_hash: Option<TransactionHash>,
}

impl Trade {
    /// Creates a trade DTO with the required identity and execution fields.
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "constructor mirrors the public field set so callers can migrate off struct-literal construction without losing explicit control over any wire field"
    )]
    pub const fn new(
        block_number: u64,
        log_index: u64,
        order_uid: OrderUid,
        owner: Address,
        sell_token: Address,
        buy_token: Address,
        sell_amount: Amount,
        sell_amount_before_fees: Amount,
        buy_amount: Amount,
        tx_hash: Option<TransactionHash>,
    ) -> Self {
        Self {
            block_number,
            log_index,
            order_uid,
            owner,
            sell_token,
            buy_token,
            sell_amount,
            sell_amount_before_fees,
            buy_amount,
            executed_protocol_fees: None,
            tx_hash,
        }
    }

    /// Returns a copy of this trade with explicit executed protocol fees.
    #[must_use]
    pub fn with_executed_protocol_fees(mut self, fees: Vec<ExecutedProtocolFee>) -> Self {
        self.executed_protocol_fees = Some(fees);
        self
    }
}
