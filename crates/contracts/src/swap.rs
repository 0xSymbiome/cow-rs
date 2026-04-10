use serde::{Deserialize, Serialize};

use cow_sdk_core::{Address, Amount, TypedDataDomain};

use crate::{
    ContractsError,
    order::Order,
    settlement::{TokenRegistry, Trade, TradeExecution, encode_trade},
    signature::Signature,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Swap {
    pub pool_id: String,
    pub asset_in: Address,
    pub asset_out: Address,
    pub amount: Amount,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_data: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchSwapStep {
    pub pool_id: String,
    pub asset_in_index: usize,
    pub asset_out_index: usize,
    pub amount: Amount,
    pub user_data: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapExecution {
    pub limit_amount: Amount,
}

pub type EncodedSwap = (Vec<BatchSwapStep>, Vec<Address>, Trade);

#[derive(Debug, Clone)]
pub struct SwapEncoder {
    pub domain: TypedDataDomain,
    tokens: TokenRegistry,
    swaps: Vec<BatchSwapStep>,
    trade: Option<Trade>,
}

impl SwapEncoder {
    pub fn new(domain: TypedDataDomain) -> Self {
        Self {
            domain,
            tokens: TokenRegistry::new(),
            swaps: Vec::new(),
            trade: None,
        }
    }

    pub fn tokens(&self) -> Vec<Address> {
        self.tokens.addresses()
    }

    pub fn swaps(&self) -> Vec<BatchSwapStep> {
        self.swaps.clone()
    }

    pub fn trade(&self) -> Result<Trade, ContractsError> {
        self.trade.clone().ok_or(ContractsError::MissingTrade)
    }

    pub fn encode_swap_step(&mut self, swaps: &[Swap]) {
        self.swaps.extend(
            swaps
                .iter()
                .map(|swap| encode_swap_step(&mut self.tokens, swap)),
        );
    }

    pub fn encode_trade(
        &mut self,
        order: &Order,
        signature: &Signature,
        swap_execution: Option<SwapExecution>,
    ) -> Result<(), ContractsError> {
        let order = crate::order::normalize_order(order)?;
        let limit_amount = swap_execution
            .map(|execution| execution.limit_amount)
            .unwrap_or_else(|| match order.kind {
                cow_sdk_core::OrderKind::Sell => order.buy_amount.clone(),
                cow_sdk_core::OrderKind::Buy => order.sell_amount.clone(),
            });
        self.trade = Some(encode_trade(
            &mut self.tokens,
            &order,
            signature,
            &TradeExecution {
                executed_amount: limit_amount,
            },
        )?);
        Ok(())
    }

    pub fn encoded_swap(&self) -> Result<EncodedSwap, ContractsError> {
        Ok((self.swaps(), self.tokens(), self.trade()?))
    }
}

pub fn encode_swap_step(tokens: &mut TokenRegistry, swap: &Swap) -> BatchSwapStep {
    BatchSwapStep {
        pool_id: swap.pool_id.clone(),
        asset_in_index: tokens.index(&swap.asset_in),
        asset_out_index: tokens.index(&swap.asset_out),
        amount: swap.amount.clone(),
        user_data: swap.user_data.clone().unwrap_or_else(|| "0x".to_owned()),
    }
}
