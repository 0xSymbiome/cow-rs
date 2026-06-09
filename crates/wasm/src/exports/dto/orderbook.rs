#[cfg(feature = "orderbook")]
use crate::helpers::errors::PureError;
use serde::{Deserialize, Serialize};
#[cfg(feature = "orderbook")]
use serde_json::Value;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[cfg(feature = "orderbook")]
use super::{OrderKindDto, TokenBalanceDto};
#[cfg(feature = "orderbook")]
use crate::exports::errors::WasmError;

/// Signature scheme carried on posted and returned orders, mirroring
/// `cow_sdk_orderbook::SigningScheme`, whose wire form is the lowercased
/// variant name.
#[cfg(feature = "orderbook")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum SigningSchemeDto {
    /// EIP-712 typed-data signature.
    Eip712,
    /// `eth_sign` style message signature.
    EthSign,
    /// EIP-1271 smart-account signature.
    Eip1271,
    /// Pre-sign on-chain approval.
    PreSign,
}

/// Full app-data document returned by the orderbook app-data endpoint.
#[cfg(feature = "orderbook")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct AppDataObjectDto {
    /// Full serialized app-data payload.
    pub full_app_data: String,
}

#[cfg(feature = "orderbook")]
impl From<cow_sdk_orderbook::AppDataObject> for AppDataObjectDto {
    fn from(value: cow_sdk_orderbook::AppDataObject) -> Self {
        Self {
            full_app_data: value.full_app_data,
        }
    }
}

/// Native-price response from the orderbook native-price endpoint, mirroring
/// `cow_sdk_orderbook::NativePriceResponse`.
#[cfg(feature = "orderbook")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct NativePriceResponseDto {
    /// Token price quoted in the chain's native asset.
    pub price: f64,
}

/// Executed protocol-fee component of a trade, mirroring
/// `cow_sdk_orderbook::ExecutedProtocolFee`.
#[cfg(feature = "orderbook")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct ExecutedProtocolFeeDto {
    /// Fee policy that produced this fee, when services returns it (arbitrary
    /// JSON mirroring the upstream policy shape).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy: Option<Value>,
    /// Fee amount taken.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
    /// Token in which the fee was taken.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

/// Trade returned by the orderbook trades endpoint, mirroring
/// `cow_sdk_orderbook::Trade`.
#[cfg(feature = "orderbook")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct TradeDto {
    /// Block number containing the trade event.
    pub block_number: u64,
    /// Log index within the block.
    pub log_index: u64,
    /// Order UID associated with the trade.
    pub order_uid: String,
    /// Owner address.
    pub owner: String,
    /// Sell-token address.
    pub sell_token: String,
    /// Buy-token address.
    pub buy_token: String,
    /// Executed sell amount in the upstream decimal-string wire shape.
    pub sell_amount: String,
    /// Executed sell amount before fees.
    #[serde(default)]
    pub sell_amount_before_fees: String,
    /// Executed buy amount in the upstream decimal-string wire shape.
    pub buy_amount: String,
    /// Protocol fees executed as part of the trade, when services returns them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executed_protocol_fees: Option<Vec<ExecutedProtocolFeeDto>>,
    /// Settlement transaction hash.
    pub tx_hash: Option<String>,
}

/// Resolved quote payload echoed by the orderbook `/quote` response, mirroring
/// `cow_sdk_orderbook::QuoteData`.
#[cfg(feature = "orderbook")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct QuoteDataDto {
    /// Sell-token address.
    pub sell_token: String,
    /// Buy-token address.
    pub buy_token: String,
    /// Optional receiver override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiver: Option<String>,
    /// Sell amount in the upstream decimal-string wire shape.
    pub sell_amount: String,
    /// Buy amount in the upstream decimal-string wire shape.
    pub buy_amount: String,
    /// Absolute UNIX expiry timestamp.
    pub valid_to: u32,
    /// Effective app-data hash derived from the orderbook response.
    pub app_data: String,
    /// Explicit app-data hash echoed alongside full app data, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_data_hash: Option<String>,
    /// Network-cost amount echoed by the orderbook `/quote` response.
    pub fee_amount: String,
    /// Order kind.
    pub kind: OrderKindDto,
    /// Whether partial fills are allowed.
    pub partially_fillable: bool,
    /// Sell-token balance source.
    pub sell_token_balance: TokenBalanceDto,
    /// Buy-token balance destination.
    pub buy_token_balance: TokenBalanceDto,
    /// Estimated gas units for the quoted trade; empty for a locally
    /// constructed quote.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub gas_amount: String,
    /// Estimated gas price at quote time (wei per gas unit).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub gas_price: String,
    /// Sell-token price in native-token atoms per sell-token atom.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub sell_token_price: String,
    /// Signing scheme for the quoted order.
    pub signing_scheme: SigningSchemeDto,
}

/// Raw orderbook quote response, mirroring
/// `cow_sdk_orderbook::OrderQuoteResponse`.
#[cfg(feature = "orderbook")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct OrderQuoteResponseDto {
    /// Resolved quote payload.
    pub quote: QuoteDataDto,
    /// Effective owner used for the quote, when returned by the API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// Quote price/fee expiry as an ISO-8601 UTC string.
    pub expiration: String,
    /// Quote identifier used when submitting the corresponding order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// Whether the quote was verified by the orderbook.
    pub verified: bool,
    /// Optional protocol fee basis points for the quote.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol_fee_bps: Option<String>,
}

/// Orderbook quote request input.
#[cfg(feature = "orderbook")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct OrderQuoteRequestInput {
    /// Sell-token address.
    pub sell_token: String,
    /// Buy-token address.
    pub buy_token: String,
    /// Optional explicit receiver.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiver: Option<String>,
    /// Quote owner.
    pub from: String,
    /// Quote side.
    pub kind: OrderKindDto,
    /// Sell amount before fee for sell quotes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sell_amount_before_fee: Option<String>,
    /// Buy amount after fee for buy quotes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buy_amount_after_fee: Option<String>,
    /// Relative validity duration in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_for: Option<u32>,
    /// Absolute UNIX expiry timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u32>,
    /// Inline app-data payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_data: Option<String>,
    /// App-data hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_data_hash: Option<String>,
    /// Whether partial fills are allowed.
    #[serde(default)]
    pub partially_fillable: bool,
    /// Sell-token balance source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sell_token_balance: Option<TokenBalanceDto>,
    /// Buy-token balance destination.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buy_token_balance: Option<TokenBalanceDto>,
    /// Quote-quality mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_quality: Option<String>,
    /// Expected signing scheme.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signing_scheme: Option<String>,
    /// Whether the eventual order is expected to be on-chain.
    #[serde(default)]
    pub onchain_order: bool,
    /// Optional verification gas limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_gas_limit: Option<u64>,
    /// Optional request timeout in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

#[cfg(feature = "orderbook")]
impl OrderQuoteRequestInput {
    pub(crate) fn into_value(self) -> Result<Value, WasmError> {
        serde_json::to_value(self).map_err(WasmError::from)
    }
}

/// Orderbook order-creation input.
#[cfg(feature = "orderbook")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct OrderCreationInput {
    /// Sell-token address.
    pub sell_token: String,
    /// Buy-token address.
    pub buy_token: String,
    /// Optional receiver.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiver: Option<String>,
    /// Sell amount.
    pub sell_amount: String,
    /// Buy amount.
    pub buy_amount: String,
    /// Absolute UNIX expiry timestamp.
    pub valid_to: u32,
    /// Inline app-data payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_data: Option<String>,
    /// App-data hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_data_hash: Option<String>,
    /// Order-level fee amount. The orderbook accepts only zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee_amount: Option<String>,
    /// Strict balance-check flag.
    #[serde(default)]
    pub full_balance_check: bool,
    /// Order side.
    pub kind: OrderKindDto,
    /// Whether partial fills are allowed.
    #[serde(default)]
    pub partially_fillable: bool,
    /// Sell-token balance source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sell_token_balance: Option<TokenBalanceDto>,
    /// Buy-token balance destination.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buy_token_balance: Option<TokenBalanceDto>,
    /// Signature scheme.
    pub signing_scheme: String,
    /// Raw signature.
    pub signature: String,
    /// Effective owner.
    pub from: String,
    /// Optional quote id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<i64>,
}

#[cfg(feature = "orderbook")]
impl OrderCreationInput {
    pub(crate) fn into_value(self) -> Result<Value, WasmError> {
        serde_json::to_value(self).map_err(WasmError::from)
    }
}

/// Pagination options shared by orderbook list helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct PaginationOptions {
    /// Pagination offset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    /// Pagination limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// Trades query accepted by `OrderBookClient.getTrades`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct TradesQueryInput {
    /// Owner filter. Set exactly one of `owner` or `orderUid`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Order UID filter. Set exactly one of `owner` or `orderUid`.
    #[serde(rename = "orderUid", default, skip_serializing_if = "Option::is_none")]
    pub order_uid: Option<String>,
    /// Pagination offset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    /// Pagination limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[cfg(feature = "orderbook")]
pub fn orderbook_signing_scheme(
    value: &str,
) -> Result<cow_sdk_orderbook::SigningScheme, WasmError> {
    match value {
        "eip712" | "Eip712" | "EIP712" => Ok(cow_sdk_orderbook::SigningScheme::Eip712),
        "ethsign" | "ethSign" | "EthSign" => Ok(cow_sdk_orderbook::SigningScheme::EthSign),
        "eip1271" | "Eip1271" | "EIP1271" => Ok(cow_sdk_orderbook::SigningScheme::Eip1271),
        "presign" | "preSign" | "PreSign" => Ok(cow_sdk_orderbook::SigningScheme::PreSign),
        other => Err(WasmError::from(PureError::unknown_enum(
            "signingScheme",
            other,
        ))),
    }
}

#[cfg(feature = "orderbook")]
pub fn ecdsa_signing_scheme(
    value: &str,
) -> Result<cow_sdk_orderbook::EcdsaSigningScheme, WasmError> {
    match value {
        "eip712" | "Eip712" | "EIP712" => Ok(cow_sdk_orderbook::EcdsaSigningScheme::Eip712),
        "ethsign" | "ethSign" | "EthSign" => Ok(cow_sdk_orderbook::EcdsaSigningScheme::EthSign),
        other => Err(WasmError::from(PureError::unknown_enum(
            "signingScheme",
            other,
        ))),
    }
}
