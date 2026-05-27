use serde::{Deserialize, Serialize};

use cow_sdk_core::{
    Address, Amount, BuyTokenDestination, ContractCall, Provider, SellTokenSource, SignedAmount,
};

use crate::{
    ContractsError,
    interaction::{InteractionLike, normalize_interactions},
    primitives::{buy_balance_name, sell_balance_name},
    settlement::InteractionStage,
};

/// Read-only helper for allow-list queries.
#[derive(Debug, Clone)]
pub struct AllowListReader<P> {
    /// Allow-list base contract address.
    pub allow_list_address: Address,
    /// JSON ABI for the allow-list base contract.
    pub allow_list_abi_json: String,
    /// Reader contract address.
    pub reader_address: Address,
    /// JSON ABI for the reader contract.
    pub reader_abi_json: String,
    /// Provider used to execute reads.
    pub provider: P,
}

/// Read-only helper for settlement storage queries.
#[derive(Debug, Clone)]
pub struct SettlementReader<P> {
    /// Settlement base contract address.
    pub settlement_address: Address,
    /// JSON ABI for the settlement base contract.
    pub settlement_abi_json: String,
    /// Reader contract address.
    pub reader_address: Address,
    /// JSON ABI for the reader contract.
    pub reader_abi_json: String,
    /// Provider used to execute reads.
    pub provider: P,
}

/// Read-only helper for trade simulation.
#[derive(Debug, Clone)]
pub struct TradeSimulator<P> {
    /// Settlement base contract address.
    pub settlement_address: Address,
    /// JSON ABI for the settlement base contract.
    pub settlement_abi_json: String,
    /// Simulator contract address.
    pub simulator_address: Address,
    /// JSON ABI for the simulator contract.
    pub simulator_abi_json: String,
    /// Provider used to execute reads.
    pub provider: P,
}

/// Input shape for settlement trade simulation.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeSimulation {
    /// Sell token address.
    pub sell_token: Address,
    /// Buy token address.
    pub buy_token: Address,
    /// Optional receiver address. Missing values normalize to the zero address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    /// Sell amount.
    pub sell_amount: Amount,
    /// Buy amount.
    pub buy_amount: Amount,
    /// Optional sell-token balance source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_token_balance: Option<SellTokenSource>,
    /// Optional buy-token balance destination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_token_balance: Option<BuyTokenDestination>,
    /// Trade owner address.
    pub owner: Address,
}

/// Token-balance delta pair returned by simulation.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeSimulationBalanceDelta {
    /// Delta for the sell token.
    pub sell_token_delta: SignedAmount,
    /// Delta for the buy token.
    pub buy_token_delta: SignedAmount,
}

/// Result contract returned by trade simulation.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeSimulationResult {
    /// Gas used by the simulated trade.
    pub gas_used: Amount,
    /// Executed buy amount.
    pub executed_buy_amount: Amount,
    /// Contract-side balance deltas.
    pub contract_balance: TradeSimulationBalanceDelta,
    /// Owner-side balance deltas.
    pub owner_balance: TradeSimulationBalanceDelta,
}

impl TradeSimulation {
    /// Creates a settlement trade-simulation input.
    #[must_use]
    // Mirrors the full current public field set so callers can migrate off
    // struct literals without losing explicit control over any wire field.
    #[expect(
        clippy::too_many_arguments,
        reason = "constructor mirrors the public field set so callers can migrate off struct-literal construction without losing explicit control over any wire field"
    )]
    pub const fn new(
        sell_token: Address,
        buy_token: Address,
        receiver: Option<Address>,
        sell_amount: Amount,
        buy_amount: Amount,
        sell_token_balance: Option<SellTokenSource>,
        buy_token_balance: Option<BuyTokenDestination>,
        owner: Address,
    ) -> Self {
        Self {
            sell_token,
            buy_token,
            receiver,
            sell_amount,
            buy_amount,
            sell_token_balance,
            buy_token_balance,
            owner,
        }
    }
}

impl TradeSimulationBalanceDelta {
    /// Creates a trade-simulation balance delta pair.
    #[must_use]
    pub const fn new(sell_token_delta: SignedAmount, buy_token_delta: SignedAmount) -> Self {
        Self {
            sell_token_delta,
            buy_token_delta,
        }
    }
}

impl TradeSimulationResult {
    /// Creates a trade-simulation result.
    #[must_use]
    pub const fn new(
        gas_used: Amount,
        executed_buy_amount: Amount,
        contract_balance: TradeSimulationBalanceDelta,
        owner_balance: TradeSimulationBalanceDelta,
    ) -> Self {
        Self {
            gas_used,
            executed_buy_amount,
            contract_balance,
            owner_balance,
        }
    }
}

impl<P> AllowListReader<P>
where
    P: Provider,
    P::Error: std::fmt::Display,
{
    /// Returns whether the supplied solver addresses are allow-listed.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError`] if request serialization, provider execution,
    /// or result decoding fails.
    pub async fn are_solvers(&self, solvers: &[Address]) -> Result<bool, ContractsError> {
        let raw = read_storage(
            &self.provider,
            &self.allow_list_address,
            &self.allow_list_abi_json,
            &self.reader_address,
            &self.reader_abi_json,
            "areSolvers",
            &serde_json::to_value(solvers)?,
        )
        .await?;
        serde_json::from_str(&raw).map_err(ContractsError::from)
    }
}

impl<P> SettlementReader<P>
where
    P: Provider,
    P::Error: std::fmt::Display,
{
    /// Returns filled amounts for the supplied order UIDs.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError`] if request serialization, provider execution,
    /// or result decoding fails.
    pub async fn filled_amounts_for_orders(
        &self,
        order_uids: &[cow_sdk_core::OrderUid],
    ) -> Result<Vec<Amount>, ContractsError> {
        let raw = read_storage(
            &self.provider,
            &self.settlement_address,
            &self.settlement_abi_json,
            &self.reader_address,
            &self.reader_abi_json,
            "filledAmountsForOrders",
            &serde_json::to_value(order_uids)?,
        )
        .await?;
        serde_json::from_str(&raw).map_err(ContractsError::from)
    }
}

impl<P> TradeSimulator<P>
where
    P: Provider,
    P::Error: std::fmt::Display,
{
    /// Simulates a trade plus any staged interactions.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError`] if provider execution or response decoding fails.
    pub async fn simulate_trade(
        &self,
        trade: &TradeSimulation,
        interactions: &[(InteractionStage, Vec<InteractionLike>)],
    ) -> Result<TradeSimulationResult, ContractsError> {
        let mut normalized_interactions = [Vec::new(), Vec::new(), Vec::new()];
        for (stage, items) in interactions {
            normalized_interactions[*stage as usize] = normalize_interactions(items);
        }
        let normalized_trade = serde_json::json!({
            "sellToken": trade.sell_token,
            "buyToken": trade.buy_token,
            "receiver": trade.receiver.unwrap_or(Address::ZERO),
            "sellAmount": trade.sell_amount,
            "buyAmount": trade.buy_amount,
            "sellTokenBalance": sell_balance_id(trade.sell_token_balance.unwrap_or_default()),
            "buyTokenBalance": buy_balance_id(trade.buy_token_balance.unwrap_or_default()),
            "owner": trade.owner,
        });
        let raw = read_storage(
            &self.provider,
            &self.settlement_address,
            &self.settlement_abi_json,
            &self.simulator_address,
            &self.simulator_abi_json,
            "simulateTrade",
            &serde_json::json!([normalized_trade, normalized_interactions]),
        )
        .await?;
        serde_json::from_str(&raw).map_err(ContractsError::from)
    }
}

async fn read_storage<P>(
    provider: &P,
    base_address: &Address,
    base_abi_json: &str,
    reader_address: &Address,
    reader_abi_json: &str,
    method: &str,
    parameters_json: &serde_json::Value,
) -> Result<String, ContractsError>
where
    P: Provider,
    P::Error: std::fmt::Display,
{
    provider
        .read_contract(&ContractCall::new(
            *reader_address,
            method.to_owned(),
            reader_abi_json.to_owned(),
            serde_json::json!({
                "baseAddress": base_address,
                "baseAbi": base_abi_json,
                "method": method,
                "parameters": parameters_json,
            })
            .to_string(),
        ))
        .await
        .map_err(|error| ContractsError::Provider {
            operation: "read_contract",
            message: error.to_string().into(),
        })
}

fn sell_balance_id(balance: SellTokenSource) -> String {
    format!(
        "{}",
        alloy_primitives::keccak256(sell_balance_name(balance).as_bytes())
    )
}

fn buy_balance_id(balance: BuyTokenDestination) -> String {
    format!(
        "{}",
        alloy_primitives::keccak256(buy_balance_name(balance).as_bytes())
    )
}
