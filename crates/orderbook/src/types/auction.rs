use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{Address, Amount, OrderUid, TransactionHash};

/// Reference prices keyed by token address.
///
/// Maps a token address to its price denominated in the chain's native token.
/// Shared by the auction-side and solver-competition surfaces so one typed
/// contract describes both.
pub type AuctionPrices = BTreeMap<Address, Amount>;

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
    pub const fn with_executed_amounts(mut self, amounts: ExecutedAmounts) -> Self {
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
    pub orders: Vec<OrderUid>,
    /// Clearing prices keyed by token address.
    #[serde(default)]
    pub prices: AuctionPrices,
}

impl CompetitionAuction {
    /// Creates an empty competition-auction snapshot.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy of this snapshot with an explicit order-UID list.
    #[must_use]
    pub fn with_orders(mut self, orders: Vec<OrderUid>) -> Self {
        self.orders = orders;
        self
    }

    /// Returns a copy of this snapshot with an explicit clearing-prices map.
    #[must_use]
    pub fn with_prices(mut self, prices: AuctionPrices) -> Self {
        self.prices = prices;
        self
    }
}

/// A single order touched by a solver's settlement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SolverCompetitionOrder {
    /// Order UID.
    pub id: OrderUid,
    /// Effective sell amount including all fees.
    pub sell_amount: Amount,
    /// Effective buy amount after all fees.
    pub buy_amount: Amount,
    /// Buy-token address, when rendered by the API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buy_token: Option<Address>,
    /// Sell-token address, when rendered by the API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sell_token: Option<Address>,
}

impl SolverCompetitionOrder {
    /// Creates a touched-order entry with its required identity and amounts.
    #[must_use]
    pub const fn new(id: OrderUid, sell_amount: Amount, buy_amount: Amount) -> Self {
        Self {
            id,
            sell_amount,
            buy_amount,
            buy_token: None,
            sell_token: None,
        }
    }

    /// Returns a copy carrying explicit buy- and sell-token addresses.
    #[must_use]
    pub const fn with_tokens(mut self, sell_token: Address, buy_token: Address) -> Self {
        self.sell_token = Some(sell_token);
        self.buy_token = Some(buy_token);
        self
    }
}

/// Settlement candidate nested inside solver-competition responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SolverSettlement {
    /// Address the solver used to execute the settlement on-chain.
    pub solver_address: Address,
    /// Settlement score.
    pub score: Amount,
    /// Position of this solution in the competition ranking.
    pub ranking: i64,
    /// Clearing prices keyed by token address.
    #[serde(default)]
    pub clearing_prices: AuctionPrices,
    /// Orders touched by this solution.
    #[serde(default)]
    pub orders: Vec<SolverCompetitionOrder>,
    /// Whether this solution won the right to be executed.
    pub is_winner: bool,
    /// Whether this solution was filtered out by the competition rules.
    pub filtered_out: bool,
    /// Reference score for this solution, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_score: Option<Amount>,
    /// Transaction in which the solution was executed on-chain, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<TransactionHash>,
}

impl SolverSettlement {
    /// Creates a settlement candidate with its required identity, score, and flags.
    ///
    /// Attach clearing prices, touched orders, the reference score, and the
    /// settlement transaction hash through the `with_*` setters.
    #[must_use]
    pub const fn new(
        solver_address: Address,
        score: Amount,
        ranking: i64,
        is_winner: bool,
    ) -> Self {
        Self {
            solver_address,
            score,
            ranking,
            clearing_prices: AuctionPrices::new(),
            orders: Vec::new(),
            is_winner,
            filtered_out: false,
            reference_score: None,
            tx_hash: None,
        }
    }

    /// Returns a copy carrying explicit clearing prices.
    #[must_use]
    pub fn with_clearing_prices(mut self, clearing_prices: AuctionPrices) -> Self {
        self.clearing_prices = clearing_prices;
        self
    }

    /// Returns a copy carrying the touched orders for this solution.
    #[must_use]
    pub fn with_orders(mut self, orders: Vec<SolverCompetitionOrder>) -> Self {
        self.orders = orders;
        self
    }

    /// Returns a copy with the filtered-out flag set to `filtered_out`.
    #[must_use]
    pub const fn with_filtered_out(mut self, filtered_out: bool) -> Self {
        self.filtered_out = filtered_out;
        self
    }

    /// Returns a copy carrying an explicit reference score.
    #[must_use]
    pub const fn with_reference_score(mut self, reference_score: Amount) -> Self {
        self.reference_score = Some(reference_score);
        self
    }

    /// Returns a copy carrying an explicit settlement transaction hash.
    #[must_use]
    pub const fn with_tx_hash(mut self, tx_hash: TransactionHash) -> Self {
        self.tx_hash = Some(tx_hash);
        self
    }
}

/// Solver-competition response returned by the orderbook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SolverCompetitionResponse {
    /// Identifier of the auction this competition is for.
    pub auction_id: i64,
    /// Block the auction started on.
    pub auction_start_block: u64,
    /// Block deadline by which the auction must be settled.
    pub auction_deadline_block: u64,
    /// Transaction hashes for the winning solutions of this competition.
    #[serde(default)]
    pub transaction_hashes: Vec<TransactionHash>,
    /// Reference score for each winning solver, keyed by solver address.
    #[serde(default)]
    pub reference_scores: BTreeMap<Address, Amount>,
    /// Auction snapshot for the competition.
    pub auction: CompetitionAuction,
    /// Settlement candidates submitted by solvers.
    #[serde(default)]
    pub solutions: Vec<SolverSettlement>,
}

impl SolverCompetitionResponse {
    /// Creates a solver-competition response with its required auction identity.
    ///
    /// Attach transaction hashes, reference scores, and solutions through the
    /// `with_*` setters.
    #[must_use]
    pub const fn new(
        auction_id: i64,
        auction_start_block: u64,
        auction_deadline_block: u64,
        auction: CompetitionAuction,
    ) -> Self {
        Self {
            auction_id,
            auction_start_block,
            auction_deadline_block,
            transaction_hashes: Vec::new(),
            reference_scores: BTreeMap::new(),
            auction,
            solutions: Vec::new(),
        }
    }

    /// Returns a copy of this response with explicit settlement transaction hashes.
    #[must_use]
    pub fn with_transaction_hashes(mut self, hashes: Vec<TransactionHash>) -> Self {
        self.transaction_hashes = hashes;
        self
    }

    /// Returns a copy of this response with explicit per-solver reference scores.
    #[must_use]
    pub fn with_reference_scores(mut self, reference_scores: BTreeMap<Address, Amount>) -> Self {
        self.reference_scores = reference_scores;
        self
    }

    /// Returns a copy of this response with explicit settlement candidates.
    #[must_use]
    pub fn with_solutions(mut self, solutions: Vec<SolverSettlement>) -> Self {
        self.solutions = solutions;
        self
    }
}
