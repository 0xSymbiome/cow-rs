use serde::{Deserialize, Serialize};

use super::{
    Address, Amount, AppDataHash, BuyTokenDestination, OrderKind, OrderUid, SellTokenSource,
    enums::OrderClass,
    order::{FeePolicy, InteractionData, Quote},
};

/// Order entry inside an auction snapshot.
///
/// Closed internally so the SDK can add auction-side fields additively while
/// external consumers avoid exhaustive destructuring.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AuctionOrder {
    /// Order UID.
    pub uid: OrderUid,
    /// Sell-token address.
    pub sell_token: Address,
    /// Buy-token address.
    pub buy_token: Address,
    /// Receiver override, nullable on the wire.
    pub receiver: Option<Address>,
    /// Sell amount in the upstream decimal-string wire shape.
    pub sell_amount: Amount,
    /// Buy amount in the upstream decimal-string wire shape.
    pub buy_amount: Amount,
    /// Absolute UNIX expiry timestamp.
    pub valid_to: u32,
    /// App-data hash.
    pub app_data: AppDataHash,
    /// Order kind.
    pub kind: OrderKind,
    /// Whether partial fills are allowed.
    pub partially_fillable: bool,
    /// Owner address of the auction order.
    pub owner: Address,
    /// Currently executed amount of the sell or buy token depending on order kind.
    pub executed: Amount,
    /// Interactions executed before the first execution of the order.
    pub pre_interactions: Vec<InteractionData>,
    /// Interactions executed after execution of the order.
    pub post_interactions: Vec<InteractionData>,
    /// Sell-token balance source.
    #[serde(default)]
    pub sell_token_balance: SellTokenSource,
    /// Buy-token balance destination.
    #[serde(default)]
    pub buy_token_balance: BuyTokenDestination,
    /// Auction order class.
    pub class: OrderClass,
    /// Raw signature string.
    pub signature: String,
    /// Protocol-fee policies used for this order.
    pub protocol_fees: Vec<FeePolicy>,
    /// Creation time denominated in epoch seconds.
    pub created: String,
    /// Winning auction-side quote, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<Quote>,
}

/// Auction snapshot returned by the orderbook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Auction {
    /// Auction id, when exposed by the endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// Current auction block number.
    pub block: u64,
    /// Latest settlement block, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_settlement_block: Option<u64>,
    /// Auction orders.
    #[serde(default)]
    pub orders: Vec<AuctionOrder>,
    /// Clearing prices keyed by token address.
    #[serde(default)]
    pub prices: std::collections::BTreeMap<String, String>,
}

impl Auction {
    /// Creates an auction snapshot pinned to the supplied block number.
    ///
    /// Additional fields are attached through the `with_*` setters.
    #[must_use]
    pub const fn new(block: u64) -> Self {
        Self {
            id: None,
            block,
            latest_settlement_block: None,
            orders: Vec::new(),
            prices: std::collections::BTreeMap::new(),
        }
    }

    /// Returns a copy of this snapshot with an explicit auction id.
    #[must_use]
    pub const fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    /// Returns a copy of this snapshot with an explicit latest settlement block.
    #[must_use]
    pub const fn with_latest_settlement_block(mut self, block: u64) -> Self {
        self.latest_settlement_block = Some(block);
        self
    }

    /// Returns a copy of this snapshot with an explicit auction-order list.
    #[must_use]
    pub fn with_orders(mut self, orders: Vec<AuctionOrder>) -> Self {
        self.orders = orders;
        self
    }

    /// Returns a copy of this snapshot with an explicit clearing-prices map.
    #[must_use]
    pub fn with_prices(mut self, prices: std::collections::BTreeMap<String, String>) -> Self {
        self.prices = prices;
        self
    }
}

/// Competition-status kind returned by `/api/v1/orders/{uid}/status`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CompetitionOrderStatusKind {
    /// Open but not yet scheduled.
    Open,
    /// Scheduled for competition.
    Scheduled,
    /// Actively competing.
    Active,
    /// Solved by at least one solver.
    Solved,
    /// Currently executing.
    Executing,
    /// Traded successfully.
    Traded,
    /// Cancelled before execution.
    Cancelled,
}

/// Executed sell and buy amounts for a solver path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ExecutedAmounts {
    /// Executed sell amount.
    pub sell: Amount,
    /// Executed buy amount.
    pub buy: Amount,
}

impl ExecutedAmounts {
    /// Creates executed-amounts data for a solver path.
    #[must_use]
    pub const fn new(sell: Amount, buy: Amount) -> Self {
        Self { sell, buy }
    }
}

/// Solver execution entry nested inside competition-status responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SolverExecution {
    /// Solver identifier or address rendered by the API.
    pub solver: String,
    /// Executed amounts for this solver path, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executed_amounts: Option<ExecutedAmounts>,
}

impl SolverExecution {
    /// Creates a solver-execution entry for the given solver identifier.
    #[must_use]
    pub fn new(solver: impl Into<String>) -> Self {
        Self {
            solver: solver.into(),
            executed_amounts: None,
        }
    }

    /// Returns a copy with explicit executed amounts.
    #[must_use]
    pub fn with_executed_amounts(mut self, amounts: ExecutedAmounts) -> Self {
        self.executed_amounts = Some(amounts);
        self
    }
}

/// Competition-status response for an order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CompetitionOrderStatus {
    /// High-level competition status kind.
    #[serde(rename = "type")]
    pub kind: CompetitionOrderStatusKind,
    /// Optional solver execution payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Vec<SolverExecution>>,
}

impl CompetitionOrderStatus {
    /// Creates a competition-status response for the given kind.
    #[must_use]
    pub const fn new(kind: CompetitionOrderStatusKind) -> Self {
        Self { kind, value: None }
    }

    /// Returns a copy carrying an explicit solver-execution payload.
    #[must_use]
    pub fn with_value(mut self, value: Vec<SolverExecution>) -> Self {
        self.value = Some(value);
        self
    }
}

/// Nested auction snapshot inside solver-competition responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CompetitionAuction {
    /// Order UIDs participating in the competition.
    #[serde(default)]
    pub orders: Vec<String>,
    /// Clearing prices keyed by token address.
    #[serde(default)]
    pub prices: std::collections::BTreeMap<String, String>,
}

impl CompetitionAuction {
    /// Creates an empty competition-auction snapshot.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy of this snapshot with an explicit order-UID list.
    #[must_use]
    pub fn with_orders(mut self, orders: Vec<String>) -> Self {
        self.orders = orders;
        self
    }

    /// Returns a copy of this snapshot with an explicit clearing-prices map.
    #[must_use]
    pub fn with_prices(mut self, prices: std::collections::BTreeMap<String, String>) -> Self {
        self.prices = prices;
        self
    }
}

/// Settlement candidate nested inside solver-competition responses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SolverSettlement {
    /// Optional settlement ranking score.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranking: Option<f64>,
    /// Solver address, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solver_address: Option<String>,
    /// Settlement score as rendered by the API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<String>,
    /// Reference score used for comparison.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_score: Option<String>,
    /// Settlement transaction hash, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,
    /// Clearing prices keyed by token address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clearing_prices: Option<std::collections::BTreeMap<String, String>>,
    /// Whether this settlement was the winning one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_winner: Option<bool>,
    /// Whether the settlement was filtered out by the API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filtered_out: Option<bool>,
}

impl SolverSettlement {
    /// Creates an empty settlement candidate; attach fields through the `with_*` setters.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy of this settlement with an explicit solver address.
    #[must_use]
    pub fn with_solver_address(mut self, address: impl Into<String>) -> Self {
        self.solver_address = Some(address.into());
        self
    }

    /// Returns a copy of this settlement with an explicit score.
    #[must_use]
    pub fn with_score(mut self, score: impl Into<String>) -> Self {
        self.score = Some(score.into());
        self
    }

    /// Returns a copy of this settlement with an explicit settlement transaction hash.
    #[must_use]
    pub fn with_tx_hash(mut self, tx_hash: impl Into<String>) -> Self {
        self.tx_hash = Some(tx_hash.into());
        self
    }

    /// Returns a copy of this settlement with an explicit winner flag.
    #[must_use]
    pub const fn with_is_winner(mut self, is_winner: bool) -> Self {
        self.is_winner = Some(is_winner);
        self
    }
}

/// Solver-competition response returned by the orderbook.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SolverCompetitionResponse {
    /// Auction id, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auction_id: Option<i64>,
    /// Start block of the auction, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auction_start_block: Option<u64>,
    /// Deadline block of the auction, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auction_deadline_block: Option<u64>,
    /// Settlement transaction hashes associated with the competition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_hashes: Option<Vec<String>>,
    /// Nested auction payload, when returned by the endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auction: Option<CompetitionAuction>,
    /// Settlement candidates, when returned by the endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solutions: Option<Vec<SolverSettlement>>,
}

impl SolverCompetitionResponse {
    /// Creates an empty solver-competition response; attach fields through the `with_*` setters.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy of this response with an explicit auction id.
    #[must_use]
    pub const fn with_auction_id(mut self, auction_id: i64) -> Self {
        self.auction_id = Some(auction_id);
        self
    }

    /// Returns a copy of this response with the auction block range.
    #[must_use]
    pub const fn with_auction_block_range(mut self, start: u64, deadline: u64) -> Self {
        self.auction_start_block = Some(start);
        self.auction_deadline_block = Some(deadline);
        self
    }

    /// Returns a copy of this response with explicit settlement transaction hashes.
    #[must_use]
    pub fn with_transaction_hashes(mut self, hashes: Vec<String>) -> Self {
        self.transaction_hashes = Some(hashes);
        self
    }

    /// Returns a copy of this response with an explicit nested auction payload.
    #[must_use]
    pub fn with_auction(mut self, auction: CompetitionAuction) -> Self {
        self.auction = Some(auction);
        self
    }

    /// Returns a copy of this response with explicit settlement candidates.
    #[must_use]
    pub fn with_solutions(mut self, solutions: Vec<SolverSettlement>) -> Self {
        self.solutions = Some(solutions);
        self
    }
}
