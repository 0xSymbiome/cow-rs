use std::fmt;

use serde::{Deserialize, Serialize};

use crate::errors::{CoreError, ValidationError};

pub type ChainId = u64;

pub const EVM_ADDRESS_HEX_CHARS: usize = 40;
pub const APP_DATA_HASH_HEX_CHARS: usize = 64;
pub const ORDER_UID_HEX_CHARS: usize = 112;

pub const ORDER_TYPE_FIELD_NAMES: [&str; 12] = [
    "sellToken",
    "buyToken",
    "receiver",
    "sellAmount",
    "buyAmount",
    "validTo",
    "appData",
    "feeAmount",
    "kind",
    "partiallyFillable",
    "sellTokenBalance",
    "buyTokenBalance",
];

pub const QUOTE_AMOUNT_STAGE_NAMES: [&str; 7] = [
    "beforeAllFees",
    "beforeNetworkCosts",
    "afterProtocolFees",
    "afterNetworkCosts",
    "afterPartnerFees",
    "afterSlippage",
    "amountsToSign",
];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Address(String);

impl Address {
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        validate_hex_field("address", &value, EVM_ADDRESS_HEX_CHARS)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn normalized_key(&self) -> String {
        self.0.to_ascii_lowercase()
    }
}

impl TryFrom<String> for Address {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Address {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_owned())
    }
}

impl From<Address> for String {
    fn from(value: Address) -> Self {
        value.0
    }
}

impl AsRef<str> for Address {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AppDataHash(String);

impl AppDataHash {
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        validate_hex_field("app_data_hash", &value, APP_DATA_HASH_HEX_CHARS)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for AppDataHash {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for AppDataHash {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_owned())
    }
}

impl From<AppDataHash> for String {
    fn from(value: AppDataHash) -> Self {
        value.0
    }
}

impl AsRef<str> for AppDataHash {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for AppDataHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub type AppDataHex = AppDataHash;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct OrderUid(String);

impl OrderUid {
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        validate_hex_field("order_uid", &value, ORDER_UID_HEX_CHARS)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for OrderUid {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for OrderUid {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_owned())
    }
}

impl From<OrderUid> for String {
    fn from(value: OrderUid) -> Self {
        value.0
    }
}

impl AsRef<str> for OrderUid {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for OrderUid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderKind {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OrderBalance {
    #[default]
    Erc20,
    External,
    Internal,
}

impl OrderBalance {
    pub fn normalize_for_buy(self) -> Self {
        match self {
            Self::Internal => Self::Internal,
            Self::Erc20 | Self::External => Self::Erc20,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    pub chain_id: ChainId,
    pub address: Address,
    pub decimals: u8,
    pub symbol: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
}

pub fn addresses_equal(left: &Address, right: &Address) -> bool {
    left.normalized_key() == right.normalized_key()
}

pub fn token_id(chain_id: ChainId, address: &Address) -> String {
    format!("{chain_id}:{}", address.normalized_key())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnsignedOrder {
    pub sell_token: Address,
    pub buy_token: Address,
    pub receiver: Address,
    pub sell_amount: String,
    pub buy_amount: String,
    pub valid_to: u32,
    pub app_data: AppDataHash,
    pub fee_amount: String,
    pub kind: OrderKind,
    #[serde(default)]
    pub partially_fillable: bool,
    #[serde(default)]
    pub sell_token_balance: OrderBalance,
    #[serde(default)]
    pub buy_token_balance: OrderBalance,
}

impl UnsignedOrder {
    pub fn normalized_buy_token_balance(&self) -> OrderBalance {
        self.buy_token_balance.normalize_for_buy()
    }

    pub fn field_names() -> &'static [&'static str; ORDER_TYPE_FIELD_NAMES.len()] {
        &ORDER_TYPE_FIELD_NAMES
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    #[serde(flatten)]
    pub unsigned: UnsignedOrder,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<OrderUid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    pub order_uid: OrderUid,
    pub executed_sell_amount: String,
    pub executed_buy_amount: String,
}

pub type TradeModel = Trade;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderModel {
    pub kind: OrderKind,
    pub sell_token: Address,
    pub buy_token: Address,
    pub receiver: Address,
    pub owner: Address,
    pub app_data_hex: AppDataHash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub kind: OrderKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_token: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_token: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data_hash: Option<AppDataHash>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    pub kind: OrderKind,
    pub sell_amount: String,
    pub buy_amount: String,
    pub fee_amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_uid: Option<OrderUid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amounts_and_costs: Option<QuoteAmountsAndCosts<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuoteModel {
    pub kind: OrderKind,
    pub sell_amount: String,
    pub buy_amount: String,
    pub fee_amount: String,
    pub order_uid: Option<OrderUid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Amounts<T> {
    pub sell_amount: T,
    pub buy_amount: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkFee<T> {
    pub amount_in_sell_currency: T,
    pub amount_in_buy_currency: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeComponent<T> {
    pub amount: T,
    pub bps: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Costs<T> {
    pub network_fee: NetworkFee<T>,
    pub partner_fee: FeeComponent<T>,
    pub protocol_fee: FeeComponent<T>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteAmountsAndCosts<T = String> {
    pub is_sell: bool,
    pub costs: Costs<T>,
    pub before_all_fees: Amounts<T>,
    pub before_network_costs: Amounts<T>,
    pub after_protocol_fees: Amounts<T>,
    pub after_network_costs: Amounts<T>,
    pub after_partner_fees: Amounts<T>,
    pub after_slippage: Amounts<T>,
    pub amounts_to_sign: Amounts<T>,
}

impl<T> QuoteAmountsAndCosts<T> {
    pub fn stage_names() -> &'static [&'static str; QUOTE_AMOUNT_STAGE_NAMES.len()] {
        &QUOTE_AMOUNT_STAGE_NAMES
    }
}

fn validate_hex_field(
    field: &'static str,
    value: &str,
    expected_hex_chars: usize,
) -> Result<(), CoreError> {
    if value.is_empty() {
        return Err(ValidationError::EmptyField { field }.into());
    }

    let Some(hex_data) = value.strip_prefix("0x") else {
        return Err(ValidationError::InvalidHexPrefix { field }.into());
    };

    if hex_data.len() != expected_hex_chars {
        return Err(ValidationError::InvalidHexLength {
            field,
            expected: expected_hex_chars,
        }
        .into());
    }

    if hex::decode(hex_data).is_err() {
        return Err(ValidationError::InvalidHexCharacters { field }.into());
    }

    Ok(())
}
