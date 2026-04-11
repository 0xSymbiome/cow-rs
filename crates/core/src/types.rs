use std::fmt;

use num_bigint::{BigInt, BigUint};
use serde::{Deserialize, Serialize};

use crate::errors::{CoreError, ValidationError};

/// Numeric EVM chain id.
pub type ChainId = u64;

/// Hex character count for an EVM address without the `0x` prefix.
pub const EVM_ADDRESS_HEX_CHARS: usize = 40;
/// Hex character count for a 32-byte app-data hash without the `0x` prefix.
pub const APP_DATA_HASH_HEX_CHARS: usize = 64;
/// Hex character count for an order UID without the `0x` prefix.
pub const ORDER_UID_HEX_CHARS: usize = 112;
/// Hex character count for a 32-byte hash without the `0x` prefix.
pub const HASH32_HEX_CHARS: usize = 64;
/// Maximum bit width accepted for unsigned protocol quantities.
pub const U256_MAX_BITS: u64 = 256;

/// Canonical EIP-712 order field names in struct-hash order.
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

/// Canonical quote amount stage names used by [`QuoteAmountsAndCosts`].
pub const QUOTE_AMOUNT_STAGE_NAMES: [&str; 7] = [
    "beforeAllFees",
    "beforeNetworkCosts",
    "afterProtocolFees",
    "afterNetworkCosts",
    "afterPartnerFees",
    "afterSlippage",
    "amountsToSign",
];

/// Validated EVM address string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Address(String);

impl Address {
    /// Creates a validated address from a `0x`-prefixed hexadecimal string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed, has
    /// the wrong length, or contains non-hex characters.
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        validate_hex_field("address", &value, EVM_ADDRESS_HEX_CHARS)?;
        Ok(Self(value))
    }

    /// Returns the original address string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the lowercase key form used for case-insensitive comparisons.
    #[must_use]
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

/// Validated hex payload used for calldata and byte blobs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct HexData(String);

impl HexData {
    /// Creates validated hex data from a `0x`-prefixed hexadecimal string.
    ///
    /// Odd-length payloads are left-padded with one zero nibble so the stored
    /// value remains canonical byte-aligned hex.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed, or
    /// contains non-hex characters.
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = normalize_hex_payload("hex_data", &value.into())?;
        Ok(Self(value))
    }

    /// Returns the canonical empty payload.
    #[must_use]
    pub fn empty() -> Self {
        Self("0x".to_owned())
    }

    /// Returns the original hex string.
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

/// Validated 32-byte app-data hash string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AppDataHash(String);

impl AppDataHash {
    /// Creates a validated app-data hash from a `0x`-prefixed 32-byte hex string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed, has
    /// the wrong length, or contains non-hex characters.
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        validate_hex_field("app_data_hash", &value, APP_DATA_HASH_HEX_CHARS)?;
        Ok(Self(value))
    }

    /// Returns the original hash string.
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

/// Backward-compatible alias for the app-data hash hex representation.
pub type AppDataHex = AppDataHash;

/// Generic validated 32-byte hash wrapper for user-domain and contract surfaces.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Hash32(String);

impl Hash32 {
    /// Creates a validated 32-byte hash from a `0x`-prefixed hex string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed, has
    /// the wrong length, or contains non-hex characters.
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        validate_hex_field("hash32", &value, HASH32_HEX_CHARS)?;
        Ok(Self(value))
    }

    /// Returns the original hash string.
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

/// Transaction hash alias.
pub type TransactionHash = Hash32;
/// Block hash alias.
pub type BlockHash = Hash32;
/// Order digest alias.
pub type OrderDigest = Hash32;

/// Validated CoW order UID string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct OrderUid(String);

impl OrderUid {
    /// Creates a validated order UID from a `0x`-prefixed 56-byte hex string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed, has
    /// the wrong length, or contains non-hex characters.
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        validate_hex_field("order_uid", &value, ORDER_UID_HEX_CHARS)?;
        Ok(Self(value))
    }

    /// Returns the original order UID string.
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

/// Canonical non-negative `uint256` quantity rendered as a base-10 string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Amount(String);

impl Amount {
    /// Creates a canonical non-negative `uint256` quantity.
    ///
    /// Decimal strings and `0x`-prefixed hexadecimal strings are accepted.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, cannot be parsed, or
    /// exceeds `uint256` bounds.
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let parsed = parse_u256_quantity("amount", &value.into())?;
        Ok(Self(parsed.to_str_radix(10)))
    }

    /// Returns the zero quantity.
    #[must_use]
    pub fn zero() -> Self {
        Self("0".to_owned())
    }

    /// Returns the canonical decimal string.
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
    /// Creates a canonical signed integer quantity.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty or cannot be parsed as a
    /// base-10 signed integer.
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let parsed = parse_signed_quantity("signed_amount", &value.into())?;
        Ok(Self(parsed.to_string()))
    }

    /// Returns the zero quantity.
    #[must_use]
    pub fn zero() -> Self {
        Self("0".to_owned())
    }

    /// Returns the canonical decimal string.
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

/// Side of an order relative to the sell token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderKind {
    /// Buy an exact amount of the buy token.
    Buy,
    /// Sell an exact amount of the sell token.
    Sell,
}

/// Token-balance source selection used by CoW orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OrderBalance {
    /// ERC-20 balance directly held by the owner.
    #[default]
    Erc20,
    /// External balance tracked by the settlement contract.
    External,
    /// Internal balance tracked by the settlement contract.
    Internal,
}

impl OrderBalance {
    /// Normalizes buy-balance selection to the protocol-supported value set.
    #[must_use]
    pub fn normalize_for_buy(self) -> Self {
        match self {
            Self::Internal => Self::Internal,
            Self::Erc20 | Self::External => Self::Erc20,
        }
    }
}

/// Token metadata used by user-domain SDK surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    /// Numeric chain id that owns the token.
    pub chain_id: ChainId,
    /// Token contract address.
    pub address: Address,
    /// Token decimals.
    pub decimals: u8,
    /// Display symbol.
    pub symbol: String,
    /// Display name.
    pub name: String,
    /// Optional logo URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
}

/// Compares two addresses using case-insensitive normalization.
#[must_use]
pub fn addresses_equal(left: &Address, right: &Address) -> bool {
    left.normalized_key() == right.normalized_key()
}

/// Builds the canonical `<chain_id>:<lowercase-address>` token identifier.
#[must_use]
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
    /// Sell token address.
    pub sell_token: Address,
    /// Buy token address.
    pub buy_token: Address,
    /// Receiver of the bought tokens.
    pub receiver: Address,
    /// Exact sell amount for sell orders or maximum sell amount for buy orders.
    pub sell_amount: Amount,
    /// Exact buy amount for buy orders or minimum buy amount for sell orders.
    pub buy_amount: Amount,
    /// Expiration timestamp encoded as `uint32`.
    pub valid_to: u32,
    /// App-data hash linked to the order.
    pub app_data: AppDataHash,
    /// Fee amount encoded in sell-token units.
    pub fee_amount: Amount,
    /// Order side.
    pub kind: OrderKind,
    /// Whether the order can be partially filled.
    #[serde(default)]
    pub partially_fillable: bool,
    /// Sell-token balance source.
    #[serde(default)]
    pub sell_token_balance: OrderBalance,
    /// Buy-token balance source.
    #[serde(default)]
    pub buy_token_balance: OrderBalance,
}

impl UnsignedOrder {
    /// Returns the normalized buy-token balance that contract hashing uses.
    #[must_use]
    pub fn normalized_buy_token_balance(&self) -> OrderBalance {
        self.buy_token_balance.normalize_for_buy()
    }

    /// Returns the canonical EIP-712 field ordering for orders.
    #[must_use]
    pub fn field_names() -> &'static [&'static str; ORDER_TYPE_FIELD_NAMES.len()] {
        &ORDER_TYPE_FIELD_NAMES
    }
}

/// Optional order envelope used by SDK consumers that need owner or uid context
/// alongside the user-domain unsigned order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// Unsigned user-domain order payload.
    #[serde(flatten)]
    pub unsigned: UnsignedOrder,
    /// Optional order owner.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Address>,
    /// Optional persisted order UID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<OrderUid>,
}

/// Simplified trade execution view used by SDK consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    /// Order UID that produced the trade.
    pub order_uid: OrderUid,
    /// Executed sell amount.
    pub executed_sell_amount: Amount,
    /// Executed buy amount.
    pub executed_buy_amount: Amount,
}

/// Backward-compatible alias for the user-domain trade model.
pub type TradeModel = Trade;

/// Compatibility order shape consumed by some lower-level contract helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderModel {
    /// Order side.
    pub kind: OrderKind,
    /// Sell token address.
    pub sell_token: Address,
    /// Buy token address.
    pub buy_token: Address,
    /// Receiver address.
    pub receiver: Address,
    /// Owner address.
    pub owner: Address,
    /// App-data hash hex string.
    pub app_data_hex: AppDataHash,
}

/// User-domain quote request shape with validated quantities.
///
/// This is not the orderbook HTTP wire DTO. The orderbook crate keeps the upstream
/// string-based transport contract explicit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    /// Quote side.
    pub kind: OrderKind,
    /// Optional sell token address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_token: Option<Address>,
    /// Optional buy token address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_token: Option<Address>,
    /// Optional receiver address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    /// Optional order owner address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    /// Optional sell amount input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_amount: Option<Amount>,
    /// Optional buy amount input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_amount: Option<Amount>,
    /// Optional explicit fee amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_amount: Option<Amount>,
    /// Optional app-data hash reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data_hash: Option<AppDataHash>,
    /// Optional raw app-data document payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data: Option<String>,
    /// Optional order expiration timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u32>,
}

/// User-domain quote response with validated quantities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    /// Quote side.
    pub kind: OrderKind,
    /// Sell amount returned by the quote.
    pub sell_amount: Amount,
    /// Buy amount returned by the quote.
    pub buy_amount: Amount,
    /// Fee amount returned by the quote.
    pub fee_amount: Amount,
    /// Optional order UID when the quote is tied to a persisted order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_uid: Option<OrderUid>,
    /// Optional price string from the upstream API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    /// Optional quote identifier from the upstream API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<String>,
    /// Optional staged amounts-and-costs breakdown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amounts_and_costs: Option<QuoteAmountsAndCosts>,
}

/// Legacy serialized compatibility quote model retained for current workspace consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuoteModel {
    /// Quote side.
    pub kind: OrderKind,
    /// Sell amount as a stringly typed compatibility value.
    pub sell_amount: String,
    /// Buy amount as a stringly typed compatibility value.
    pub buy_amount: String,
    /// Fee amount as a stringly typed compatibility value.
    pub fee_amount: String,
    /// Optional order UID when present in compatibility paths.
    pub order_uid: Option<OrderUid>,
}

/// Generic sell/buy amount pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Amounts<T> {
    /// Sell-side amount.
    pub sell_amount: T,
    /// Buy-side amount.
    pub buy_amount: T,
}

/// Network-fee amounts expressed in both quote currencies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkFee<T> {
    /// Network fee expressed in sell-token units.
    pub amount_in_sell_currency: T,
    /// Network fee expressed in buy-token units.
    pub amount_in_buy_currency: T,
}

/// Generic fee component represented by amount and basis points.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeComponent<T> {
    /// Fee amount.
    pub amount: T,
    /// Fee in basis points.
    pub bps: u32,
}

/// Full quote cost breakdown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Costs<T> {
    /// Network fee component.
    pub network_fee: NetworkFee<T>,
    /// Partner fee component.
    pub partner_fee: FeeComponent<T>,
    /// Protocol fee component.
    pub protocol_fee: FeeComponent<T>,
}

/// Staged quote amounts and cost components across the quote lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteAmountsAndCosts<T = Amount> {
    /// Whether the source quote was sell-sided.
    pub is_sell: bool,
    /// Cost breakdown for the quote.
    pub costs: Costs<T>,
    /// Amounts before all fees.
    pub before_all_fees: Amounts<T>,
    /// Amounts before network costs.
    pub before_network_costs: Amounts<T>,
    /// Amounts after protocol fees.
    pub after_protocol_fees: Amounts<T>,
    /// Amounts after network costs.
    pub after_network_costs: Amounts<T>,
    /// Amounts after partner fees.
    pub after_partner_fees: Amounts<T>,
    /// Amounts after slippage.
    pub after_slippage: Amounts<T>,
    /// Amounts that should be signed.
    pub amounts_to_sign: Amounts<T>,
}

impl<T> QuoteAmountsAndCosts<T> {
    /// Returns the canonical stage ordering for quote amount breakdowns.
    #[must_use]
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
