use cow_sdk_pure_helpers as pure;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::exports::errors::WasmError;

/// Order side accepted by wasm order inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "lowercase")]
pub enum OrderKindDto {
    /// Sell order.
    Sell,
    /// Buy order.
    Buy,
}

impl From<OrderKindDto> for pure::dto::OrderKindDto {
    fn from(value: OrderKindDto) -> Self {
        match value {
            OrderKindDto::Sell => Self::Sell,
            OrderKindDto::Buy => Self::Buy,
        }
    }
}

/// Token-balance mode accepted by wasm order inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "lowercase")]
pub enum TokenBalanceDto {
    /// ERC-20 balance or allowance path.
    Erc20,
    /// External Balancer Vault balance path.
    External,
    /// Internal Balancer Vault balance path.
    Internal,
}

impl From<TokenBalanceDto> for pure::dto::TokenBalanceDto {
    fn from(value: TokenBalanceDto) -> Self {
        match value {
            TokenBalanceDto::Erc20 => Self::Erc20,
            TokenBalanceDto::External => Self::External,
            TokenBalanceDto::Internal => Self::Internal,
        }
    }
}

/// Order input shared by signing and UID exports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct OrderInput {
    /// Sell token address.
    pub sell_token: String,
    /// Buy token address.
    pub buy_token: String,
    /// Optional receiver.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiver: Option<String>,
    /// Sell amount.
    pub sell_amount: String,
    /// Buy amount.
    pub buy_amount: String,
    /// Valid-to timestamp.
    pub valid_to: u32,
    /// App-data hash.
    pub app_data: String,
    /// Fee amount.
    pub fee_amount: String,
    /// Order side.
    pub kind: OrderKindDto,
    /// Partial fill flag.
    pub partially_fillable: bool,
    /// Sell balance source.
    pub sell_token_balance: TokenBalanceDto,
    /// Buy balance destination.
    pub buy_token_balance: TokenBalanceDto,
}

impl From<OrderInput> for pure::dto::OrderInput {
    fn from(value: OrderInput) -> Self {
        Self {
            sell_token: value.sell_token,
            buy_token: value.buy_token,
            receiver: value.receiver,
            sell_amount: value.sell_amount,
            buy_amount: value.buy_amount,
            valid_to: value.valid_to,
            app_data: value.app_data,
            fee_amount: value.fee_amount,
            kind: value.kind.into(),
            partially_fillable: value.partially_fillable,
            sell_token_balance: value.sell_token_balance.into(),
            buy_token_balance: value.buy_token_balance.into(),
        }
    }
}

impl From<&cow_sdk_core::UnsignedOrder> for OrderInput {
    fn from(value: &cow_sdk_core::UnsignedOrder) -> Self {
        Self {
            sell_token: value.sell_token.to_hex_string(),
            buy_token: value.buy_token.to_hex_string(),
            receiver: Some(value.receiver.to_hex_string()),
            sell_amount: value.sell_amount.to_string(),
            buy_amount: value.buy_amount.to_string(),
            valid_to: value.valid_to,
            app_data: value.app_data.as_str().to_owned(),
            fee_amount: value.fee_amount.to_string(),
            kind: match value.kind {
                cow_sdk_core::OrderKind::Sell => OrderKindDto::Sell,
                cow_sdk_core::OrderKind::Buy => OrderKindDto::Buy,
            },
            partially_fillable: value.partially_fillable,
            sell_token_balance: match value.sell_token_balance {
                cow_sdk_core::SellTokenSource::Erc20 => TokenBalanceDto::Erc20,
                cow_sdk_core::SellTokenSource::External => TokenBalanceDto::External,
                cow_sdk_core::SellTokenSource::Internal => TokenBalanceDto::Internal,
                _ => TokenBalanceDto::Erc20,
            },
            buy_token_balance: match value.buy_token_balance {
                cow_sdk_core::BuyTokenDestination::Erc20 => TokenBalanceDto::Erc20,
                cow_sdk_core::BuyTokenDestination::Internal => TokenBalanceDto::Internal,
                _ => TokenBalanceDto::Erc20,
            },
        }
    }
}

pub fn parse_order(input: OrderInput) -> Result<cow_sdk_core::UnsignedOrder, WasmError> {
    let pure: pure::dto::OrderInput = input.into();
    pure.to_unsigned_order().map_err(WasmError::from)
}

pub fn parse_chain(chain_id: u32) -> Result<cow_sdk_core::SupportedChainId, WasmError> {
    pure::chains::supported_chain(chain_id).map_err(WasmError::from)
}

pub fn parse_owner(owner: &str) -> Result<cow_sdk_core::Address, WasmError> {
    pure::dto::parse_address("owner", owner).map_err(WasmError::from)
}
