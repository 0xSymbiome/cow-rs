use serde::{Deserialize, Serialize};

use cow_sdk_core::{Address, Amount, ContractCall, OrderBalance, Provider, SignedAmount};

use crate::{
    ContractsError,
    interaction::{InteractionLike, normalize_interactions},
    primitives::{balance_name, keccak256_hex, zero_address},
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
    pub sell_token_balance: Option<OrderBalance>,
    /// Optional buy-token balance source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_token_balance: Option<OrderBalance>,
    /// Trade owner address.
    pub owner: Address,
}

/// Token-balance delta pair returned by simulation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeSimulationBalanceDelta {
    /// Delta for the sell token.
    pub sell_token_delta: SignedAmount,
    /// Delta for the buy token.
    pub buy_token_delta: SignedAmount,
}

/// Result contract returned by trade simulation.
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
    pub fn are_solvers(&self, solvers: &[Address]) -> Result<bool, ContractsError> {
        let raw = read_storage(
            &self.provider,
            &self.allow_list_address,
            &self.allow_list_abi_json,
            &self.reader_address,
            &self.reader_abi_json,
            "areSolvers",
            &serde_json::to_value(solvers)?,
        )?;
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
    pub fn filled_amounts_for_orders(
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
        )?;
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
    pub fn simulate_trade(
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
            "receiver": trade.receiver.clone().unwrap_or_else(zero_address),
            "sellAmount": trade.sell_amount,
            "buyAmount": trade.buy_amount,
            "sellTokenBalance": balance_id(trade.sell_token_balance.unwrap_or(OrderBalance::Erc20)),
            "buyTokenBalance": balance_id(trade.buy_token_balance.unwrap_or(OrderBalance::Erc20)),
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
        )?;
        serde_json::from_str(&raw).map_err(ContractsError::from)
    }
}

fn read_storage<P>(
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
        .read_contract(&ContractCall {
            address: reader_address.clone(),
            method: method.to_owned(),
            abi_json: reader_abi_json.to_owned(),
            args_json: serde_json::json!({
                "baseAddress": base_address,
                "baseAbi": base_abi_json,
                "method": method,
                "parameters": parameters_json,
            })
            .to_string(),
        })
        .map_err(|error| ContractsError::Provider {
            operation: "read_contract",
            message: error.to_string(),
        })
}

fn balance_id(balance: OrderBalance) -> String {
    keccak256_hex(balance_name(balance).as_bytes())
}
