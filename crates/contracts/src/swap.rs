use serde::{Deserialize, Serialize};

use cow_sdk_core::{Address, Amount, TypedDataDomain};

use crate::{
    ContractsError,
    order::Order,
    settlement::{TokenRegistry, Trade, TradeExecution, encode_trade},
    signature::Signature,
};

/// Single Balancer batch-swap step input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Swap {
    /// Pool identifier.
    pub pool_id: String,
    /// Input asset address.
    pub asset_in: Address,
    /// Output asset address.
    pub asset_out: Address,
    /// Swap amount.
    pub amount: Amount,
    /// Optional user data encoded as hex.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_data: Option<String>,
}

/// Encoded Balancer batch-swap step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchSwapStep {
    /// Pool identifier.
    pub pool_id: String,
    /// Token registry index for the input asset.
    pub asset_in_index: usize,
    /// Token registry index for the output asset.
    pub asset_out_index: usize,
    /// Swap amount.
    pub amount: Amount,
    /// Encoded user data.
    pub user_data: String,
}

/// Optional trade-execution override for swap encoding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapExecution {
    /// Limit amount that should be used as the executed amount.
    pub limit_amount: Amount,
}

/// Fully encoded swap output.
pub type EncodedSwap = (Vec<BatchSwapStep>, Vec<Address>, Trade);

/// Stateful helper for building encoded swap payloads.
#[derive(Debug, Clone)]
pub struct SwapEncoder {
    /// Typed-data domain used for the encoded trade.
    pub domain: TypedDataDomain,
    tokens: TokenRegistry,
    swaps: Vec<BatchSwapStep>,
    trade: Option<Trade>,
}

impl SwapEncoder {
    /// Creates a new swap encoder.
    #[must_use]
    pub fn new(domain: TypedDataDomain) -> Self {
        Self {
            domain,
            tokens: TokenRegistry::new(),
            swaps: Vec::new(),
            trade: None,
        }
    }

    /// Returns the encoded token registry in index order.
    #[must_use]
    pub fn tokens(&self) -> Vec<Address> {
        self.tokens.addresses()
    }

    /// Returns the encoded swap steps.
    #[must_use]
    pub fn swaps(&self) -> Vec<BatchSwapStep> {
        self.swaps.clone()
    }

    /// Returns the encoded trade.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError::MissingTrade`] if no trade has been encoded yet.
    pub fn trade(&self) -> Result<Trade, ContractsError> {
        self.trade.clone().ok_or(ContractsError::MissingTrade)
    }

    /// Encodes swap steps and appends them to the current encoder state.
    pub fn encode_swap_step(&mut self, swaps: &[Swap]) {
        self.swaps.extend(
            swaps
                .iter()
                .map(|swap| encode_swap_step(&mut self.tokens, swap)),
        );
    }

    /// Encodes the trade associated with the swap sequence.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError`] when order normalization or trade encoding fails.
    pub fn encode_trade(
        &mut self,
        order: &Order,
        signature: &Signature,
        swap_execution: Option<SwapExecution>,
    ) -> Result<(), ContractsError> {
        let order = crate::order::normalize_order(order)?;
        let limit_amount = swap_execution.map_or_else(
            || match order.kind {
                cow_sdk_core::OrderKind::Sell => order.buy_amount.clone(),
                cow_sdk_core::OrderKind::Buy => order.sell_amount.clone(),
            },
            |execution| execution.limit_amount,
        );
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

    /// Returns the fully encoded swap output.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError::MissingTrade`] if no trade has been encoded yet.
    pub fn encoded_swap(&self) -> Result<EncodedSwap, ContractsError> {
        Ok((self.swaps(), self.tokens(), self.trade()?))
    }
}

/// Encodes a single swap step using the shared token registry.
#[must_use]
pub fn encode_swap_step(tokens: &mut TokenRegistry, swap: &Swap) -> BatchSwapStep {
    BatchSwapStep {
        pool_id: swap.pool_id.clone(),
        asset_in_index: tokens.index(&swap.asset_in),
        asset_out_index: tokens.index(&swap.asset_out),
        amount: swap.amount.clone(),
        user_data: swap.user_data.clone().unwrap_or_else(|| "0x".to_owned()),
    }
}
