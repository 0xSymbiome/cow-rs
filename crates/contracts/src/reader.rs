use serde::{Deserialize, Serialize};

use cow_sdk_core::{Address, Amount, ContractCall, OrderBalance, Provider, SignedAmount};

use crate::{
    ContractsError,
    interaction::{InteractionLike, normalize_interactions},
    primitives::{balance_name, keccak256_hex, zero_address},
    settlement::InteractionStage,
};

#[derive(Debug, Clone)]
pub struct AllowListReader<P> {
    pub allow_list_address: Address,
    pub allow_list_abi_json: String,
    pub reader_address: Address,
    pub reader_abi_json: String,
    pub provider: P,
}

#[derive(Debug, Clone)]
pub struct SettlementReader<P> {
    pub settlement_address: Address,
    pub settlement_abi_json: String,
    pub reader_address: Address,
    pub reader_abi_json: String,
    pub provider: P,
}

#[derive(Debug, Clone)]
pub struct TradeSimulator<P> {
    pub settlement_address: Address,
    pub settlement_abi_json: String,
    pub simulator_address: Address,
    pub simulator_abi_json: String,
    pub provider: P,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeSimulation {
    pub sell_token: Address,
    pub buy_token: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    pub sell_amount: Amount,
    pub buy_amount: Amount,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_token_balance: Option<OrderBalance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_token_balance: Option<OrderBalance>,
    pub owner: Address,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeSimulationBalanceDelta {
    pub sell_token_delta: SignedAmount,
    pub buy_token_delta: SignedAmount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeSimulationResult {
    pub gas_used: Amount,
    pub executed_buy_amount: Amount,
    pub contract_balance: TradeSimulationBalanceDelta,
    pub owner_balance: TradeSimulationBalanceDelta,
}

impl<P> AllowListReader<P>
where
    P: Provider,
    P::Error: std::fmt::Display,
{
    pub fn are_solvers(&self, solvers: &[Address]) -> Result<bool, ContractsError> {
        let raw = read_storage(
            &self.provider,
            &self.allow_list_address,
            &self.allow_list_abi_json,
            &self.reader_address,
            &self.reader_abi_json,
            "areSolvers",
            &serde_json::to_value(solvers)
                .map_err(|error| ContractsError::Serialization(error.to_string()))?,
        )?;
        serde_json::from_str(&raw).map_err(|error| ContractsError::Decode(error.to_string()))
    }
}

impl<P> SettlementReader<P>
where
    P: Provider,
    P::Error: std::fmt::Display,
{
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
            &serde_json::to_value(order_uids)
                .map_err(|error| ContractsError::Serialization(error.to_string()))?,
        )?;
        serde_json::from_str(&raw).map_err(|error| ContractsError::Decode(error.to_string()))
    }
}

impl<P> TradeSimulator<P>
where
    P: Provider,
    P::Error: std::fmt::Display,
{
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
        serde_json::from_str(&raw).map_err(|error| ContractsError::Decode(error.to_string()))
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
        .map_err(|error| ContractsError::Provider(error.to_string()))
}

fn balance_id(balance: OrderBalance) -> String {
    keccak256_hex(balance_name(balance).as_bytes())
}
