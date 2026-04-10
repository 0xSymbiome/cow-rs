use std::fmt;

use num_bigint::{BigInt, BigUint};
use serde::{Deserialize, Serialize};

use crate::errors::{CoreError, ValidationError};

pub type ChainId = u64;

pub const EVM_ADDRESS_HEX_CHARS: usize = 40;
pub const APP_DATA_HASH_HEX_CHARS: usize = 64;
pub const ORDER_UID_HEX_CHARS: usize = 112;
pub const HASH32_HEX_CHARS: usize = 64;
pub const U256_MAX_BITS: u64 = 256;

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
pub struct HexData(String);

impl HexData {
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = normalize_hex_payload("hex_data", &value.into())?;
        Ok(Self(value))
    }

    pub fn empty() -> Self {
        Self("0x".to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for HexData {
    fn default() -> Self {
        Self::empty()
    }
}

impl TryFrom<String> for HexData {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for HexData {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_owned())
    }
}

impl From<HexData> for String {
    fn from(value: HexData) -> Self {
        value.0
    }
}

impl AsRef<str> for HexData {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for HexData {
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

/// Generic validated 32-byte hash wrapper for user-domain and contract surfaces.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Hash32(String);

impl Hash32 {
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        validate_hex_field("hash32", &value, HASH32_HEX_CHARS)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Hash32 {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Hash32 {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_owned())
    }
}

impl From<Hash32> for String {
    fn from(value: Hash32) -> Self {
        value.0
    }
}

impl AsRef<str> for Hash32 {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Hash32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub type TransactionHash = Hash32;
pub type BlockHash = Hash32;
pub type OrderDigest = Hash32;

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

/// Canonical non-negative uint256 quantity rendered as a base-10 string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Amount(String);

impl Amount {
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let parsed = parse_u256_quantity("amount", &value.into())?;
        Ok(Self(parsed.to_str_radix(10)))
    }

    pub fn zero() -> Self {
        Self("0".to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Amount {
    fn default() -> Self {
        Self::zero()
    }
}

impl TryFrom<String> for Amount {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Amount {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_owned())
    }
}

impl From<Amount> for String {
    fn from(value: Amount) -> Self {
        value.0
    }
}

impl From<u32> for Amount {
    fn from(value: u32) -> Self {
        Self(value.to_string())
    }
}

impl From<u64> for Amount {
    fn from(value: u64) -> Self {
        Self(value.to_string())
    }
}

impl From<usize> for Amount {
    fn from(value: usize) -> Self {
        Self(value.to_string())
    }
}

impl AsRef<str> for Amount {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Canonical signed integer rendered as a base-10 string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SignedAmount(String);

impl SignedAmount {
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let parsed = parse_signed_quantity("signed_amount", &value.into())?;
        Ok(Self(parsed.to_string()))
    }

    pub fn zero() -> Self {
        Self("0".to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SignedAmount {
    fn default() -> Self {
        Self::zero()
    }
}

impl TryFrom<String> for SignedAmount {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for SignedAmount {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_owned())
    }
}

impl From<SignedAmount> for String {
    fn from(value: SignedAmount) -> Self {
        value.0
    }
}

impl AsRef<str> for SignedAmount {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for SignedAmount {
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

/// User-domain order shape prepared for signing and trading workflows.
///
/// This is not an orderbook wire DTO or an ABI struct. Contract hashing converts
/// it into `cow_sdk_contracts::Order`, where receiver and token-balance defaults
/// are normalized for EIP-712 hashing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnsignedOrder {
    pub sell_token: Address,
    pub buy_token: Address,
    pub receiver: Address,
    pub sell_amount: Amount,
    pub buy_amount: Amount,
    pub valid_to: u32,
    pub app_data: AppDataHash,
    pub fee_amount: Amount,
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

/// Optional order envelope used by SDK consumers that need owner or uid context
/// alongside the user-domain unsigned order.
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
    pub executed_sell_amount: Amount,
    pub executed_buy_amount: Amount,
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

/// User-domain quote request shape with validated quantities.
///
/// This is not the orderbook HTTP wire DTO. The orderbook crate keeps the upstream
/// string-based transport contract explicit.
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
    pub sell_amount: Option<Amount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_amount: Option<Amount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_amount: Option<Amount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data_hash: Option<AppDataHash>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u32>,
}

/// User-domain quote response with validated quantities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    pub kind: OrderKind,
    pub sell_amount: Amount,
    pub buy_amount: Amount,
    pub fee_amount: Amount,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_uid: Option<OrderUid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amounts_and_costs: Option<QuoteAmountsAndCosts>,
}

/// Legacy serialized compatibility quote model retained for current workspace consumers.
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
pub struct QuoteAmountsAndCosts<T = Amount> {
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

fn normalize_hex_payload(field: &'static str, value: &str) -> Result<String, CoreError> {
    if value.is_empty() {
        return Err(ValidationError::EmptyField { field }.into());
    }

    let Some(hex_data) = value.strip_prefix("0x") else {
        return Err(ValidationError::InvalidHexPrefix { field }.into());
    };

    let normalized = if hex_data.len() % 2 == 1 {
        format!("0x0{hex_data}")
    } else {
        value.to_owned()
    };

    if hex::decode(normalized.trim_start_matches("0x")).is_err() {
        return Err(ValidationError::InvalidHexCharacters { field }.into());
    }

    Ok(normalized)
}

fn parse_u256_quantity(field: &'static str, value: &str) -> Result<BigUint, CoreError> {
    if value.is_empty() {
        return Err(ValidationError::EmptyField { field }.into());
    }

    let parsed = if let Some(stripped) = value.strip_prefix("0x") {
        BigUint::parse_bytes(stripped.as_bytes(), 16)
    } else {
        BigUint::parse_bytes(value.as_bytes(), 10)
    }
    .ok_or(ValidationError::InvalidNumeric { field })?;

    if parsed.bits() > U256_MAX_BITS {
        return Err(ValidationError::NumericOverflow { field }.into());
    }

    Ok(parsed)
}

fn parse_signed_quantity(field: &'static str, value: &str) -> Result<BigInt, CoreError> {
    if value.is_empty() {
        return Err(ValidationError::EmptyField { field }.into());
    }

    BigInt::parse_bytes(value.as_bytes(), 10)
        .ok_or(ValidationError::InvalidNumeric { field }.into())
}
