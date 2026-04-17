use std::fmt;

use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, de::Error as DeError};

pub use cow_sdk_core::{
    Address, ApiBaseUrls, ApiContext, AppDataHash, CowEnv, ENVS_LIST, EVM_NATIVE_CURRENCY_ADDRESS,
    OrderBalance, OrderKind, OrderUid, QuoteAmountsAndCosts, REDACTED_PLACEHOLDER, Redacted,
    SupportedChainId,
};

/// Partial override applied to an [`ApiContext`] when cloning an orderbook client.
#[derive(Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApiContextOverride {
    /// Replacement chain id for endpoint resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<SupportedChainId>,
    /// Replacement deployment environment for endpoint resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    /// Replacement explicit base URL map keyed by numeric chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_urls: Option<ApiBaseUrls>,
    /// Replacement partner API key used for request headers and endpoint selection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<Redacted<String>>,
}

impl fmt::Debug for ApiContextOverride {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiContextOverride")
            .field("chain_id", &self.chain_id)
            .field("env", &self.env)
            .field("base_urls", &self.base_urls)
            .field(
                "api_key",
                &self.api_key.as_ref().map(|_| REDACTED_PLACEHOLDER),
            )
            .finish()
    }
}

impl Serialize for ApiContextOverride {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("ApiContextOverride", 4)?;

        if let Some(chain_id) = &self.chain_id {
            state.serialize_field("chainId", chain_id)?;
        }
        if let Some(env) = &self.env {
            state.serialize_field("env", env)?;
        }
        if let Some(base_urls) = &self.base_urls {
            state.serialize_field("baseUrls", base_urls)?;
        }
        if self.api_key.is_some() {
            state.serialize_field("apiKey", REDACTED_PLACEHOLDER)?;
        }

        state.end()
    }
}

/// Per-environment base URL overrides applied ahead of [`ApiContext`] resolution.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EnvBaseUrlOverrides {
    /// Explicit production base URL.
    pub prod: Option<String>,
    /// Explicit staging base URL.
    pub staging: Option<String>,
}

impl EnvBaseUrlOverrides {
    /// Sets the explicit base URL for `env`.
    pub fn set(&mut self, env: CowEnv, base_url: impl Into<String>) {
        match env {
            CowEnv::Prod => self.prod = Some(base_url.into()),
            CowEnv::Staging => self.staging = Some(base_url.into()),
        }
    }

    /// Returns the explicit base URL for `env`, if one is configured.
    #[must_use]
    pub fn get(&self, env: CowEnv) -> Option<&str> {
        match env {
            CowEnv::Prod => self.prod.as_deref(),
            CowEnv::Staging => self.staging.as_deref(),
        }
    }
}

/// Quote-quality mode accepted by the orderbook quote endpoint.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PriceQuality {
    /// Prefer the fastest available quote.
    Fast,
    /// Prefer the best available quote, allowing additional search.
    Optimal,
    /// Require the orderbook's verified quote mode.
    #[default]
    Verified,
}

/// Signature scheme encoded in orderbook wire DTOs.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SigningScheme {
    /// EIP-712 typed-data signature.
    #[default]
    Eip712,
    /// `eth_sign` / personal-sign style signature.
    EthSign,
    /// EIP-1271 smart-account signature.
    Eip1271,
    /// Pre-signed order recorded on-chain.
    PreSign,
}

/// ECDSA signing schemes accepted by order-cancellation payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EcdsaSigningScheme {
    /// EIP-712 typed-data signature.
    #[default]
    Eip712,
    /// `eth_sign` / personal-sign style signature.
    EthSign,
}

/// Order class surfaced by the orderbook API.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OrderClass {
    /// Market order.
    #[default]
    Market,
    /// Limit order.
    Limit,
    /// Liquidity order.
    Liquidity,
}

/// Order lifecycle status returned by the orderbook API.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum OrderStatus {
    /// Waiting for a pre-signature to become valid.
    PresignaturePending,
    /// Open and fillable.
    #[default]
    Open,
    /// Fully or terminally fulfilled.
    Fulfilled,
    /// Cancelled by the owner or protocol.
    Cancelled,
    /// Expired because `valid_to` has passed.
    Expired,
}

/// Encodes the mutually exclusive buy-side or sell-side amount on quote requests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteSide {
    /// Whether the quote is sell-driven or buy-driven.
    pub kind: OrderKind,
    /// Sell amount before fee for sell quotes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_amount_before_fee: Option<String>,
    /// Buy amount after fee for buy quotes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_amount_after_fee: Option<String>,
}

impl QuoteSide {
    /// Creates a sell-side quote request amount.
    #[must_use]
    pub fn sell(amount: impl Into<String>) -> Self {
        Self {
            kind: OrderKind::Sell,
            sell_amount_before_fee: Some(amount.into()),
            buy_amount_after_fee: None,
        }
    }

    /// Creates a buy-side quote request amount.
    #[must_use]
    pub fn buy(amount: impl Into<String>) -> Self {
        Self {
            kind: OrderKind::Buy,
            sell_amount_before_fee: None,
            buy_amount_after_fee: Some(amount.into()),
        }
    }

    /// Returns `true` when this quote side is sell-driven.
    #[must_use]
    pub fn is_sell(&self) -> bool {
        self.kind == OrderKind::Sell
    }

    /// Returns `true` when this quote side is buy-driven.
    #[must_use]
    pub fn is_buy(&self) -> bool {
        self.kind == OrderKind::Buy
    }

    /// Returns `true` when exactly one side amount matches the declared order kind.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        matches!(
            (
                &self.kind,
                self.sell_amount_before_fee.as_ref(),
                self.buy_amount_after_fee.as_ref()
            ),
            (OrderKind::Sell, Some(_), None) | (OrderKind::Buy, None, Some(_))
        )
    }
}

/// Quote request DTO for the orderbook `/api/v1/quote` endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderQuoteRequest {
    /// Sell-token address.
    pub sell_token: Address,
    /// Buy-token address.
    pub buy_token: Address,
    /// Optional explicit receiver override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    /// Relative validity duration in seconds when supported by the upstream API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_for: Option<u32>,
    /// Absolute UNIX expiry timestamp override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u32>,
    /// Full app-data payload or literal app-data string when sent inline.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data: Option<String>,
    /// App-data hash when app-data is provided separately.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data_hash: Option<AppDataHash>,
    /// Whether partial fills are allowed.
    #[serde(default)]
    pub partially_fillable: bool,
    /// Sell-token balance source.
    #[serde(default)]
    pub sell_token_balance: OrderBalance,
    /// Buy-token balance destination.
    #[serde(default)]
    pub buy_token_balance: OrderBalance,
    /// Effective order owner used for quote verification.
    pub from: Address,
    /// Quote-quality mode.
    #[serde(default)]
    pub price_quality: PriceQuality,
    /// Signature scheme expected for the eventual order submission.
    #[serde(default)]
    pub signing_scheme: SigningScheme,
    /// Whether the eventual order is expected to be on-chain.
    #[serde(default)]
    pub onchain_order: bool,
    /// Optional gas limit supplied for verification-aware quoting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_gas_limit: Option<u64>,
    /// Optional request timeout override in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// Mutually exclusive buy-side or sell-side amount fields.
    #[serde(flatten)]
    pub side: QuoteSide,
}

impl OrderQuoteRequest {
    /// Creates a quote request with stable orderbook defaults.
    ///
    /// The default app-data is the zero hash, the signing scheme is EIP-712,
    /// and both token balances default to ERC-20 balances.
    #[must_use]
    pub fn new(sell_token: Address, buy_token: Address, from: Address, side: QuoteSide) -> Self {
        Self {
            sell_token,
            buy_token,
            receiver: None,
            valid_for: None,
            valid_to: None,
            app_data: Some(format!("0x{}", "0".repeat(64))),
            app_data_hash: None,
            partially_fillable: false,
            sell_token_balance: OrderBalance::Erc20,
            buy_token_balance: OrderBalance::Erc20,
            from,
            price_quality: PriceQuality::Verified,
            signing_scheme: SigningScheme::Eip712,
            onchain_order: false,
            verification_gas_limit: None,
            timeout: None,
            side,
        }
    }

    /// Returns a copy of this request with an explicit receiver.
    #[must_use]
    pub fn with_receiver(mut self, receiver: Address) -> Self {
        self.receiver = Some(receiver);
        self
    }

    /// Returns a copy of this request with an absolute expiry timestamp.
    #[must_use]
    pub const fn with_valid_to(mut self, valid_to: u32) -> Self {
        self.valid_to = Some(valid_to);
        self
    }

    /// Returns a copy of this request with a relative validity duration.
    #[must_use]
    pub const fn with_valid_for(mut self, valid_for: u32) -> Self {
        self.valid_for = Some(valid_for);
        self
    }

    /// Returns a copy of this request with inline app-data content.
    #[must_use]
    pub fn with_app_data(mut self, app_data: impl Into<String>) -> Self {
        self.app_data = Some(app_data.into());
        self
    }

    /// Returns a copy of this request with an explicit app-data hash.
    #[must_use]
    pub fn with_app_data_hash(mut self, app_data_hash: AppDataHash) -> Self {
        self.app_data_hash = Some(app_data_hash);
        self
    }

    /// Returns a copy of this request with a new quote-quality mode.
    #[must_use]
    pub const fn with_price_quality(mut self, quality: PriceQuality) -> Self {
        self.price_quality = quality;
        self
    }

    /// Returns a copy of this request with a new signing scheme.
    #[must_use]
    pub const fn with_signing_scheme(mut self, scheme: SigningScheme) -> Self {
        self.signing_scheme = scheme;
        self
    }

    /// Returns a copy of this request with a timeout override in milliseconds.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Returns a copy of this request marked as an on-chain order.
    #[must_use]
    pub const fn with_onchain_order(mut self) -> Self {
        self.onchain_order = true;
        self
    }

    /// Returns a copy of this request with an explicit verification gas limit.
    #[must_use]
    pub const fn with_verification_gas_limit(mut self, verification_gas_limit: u64) -> Self {
        self.verification_gas_limit = Some(verification_gas_limit);
        self
    }

    /// Returns a copy of this request marked as partially fillable.
    #[must_use]
    pub const fn with_partially_fillable(mut self) -> Self {
        self.partially_fillable = true;
        self
    }

    /// Returns a copy of this request with a new sell-token balance source.
    #[must_use]
    pub const fn with_sell_token_balance(mut self, balance: OrderBalance) -> Self {
        self.sell_token_balance = balance;
        self
    }

    /// Returns a copy of this request with a new buy-token balance destination.
    #[must_use]
    pub const fn with_buy_token_balance(mut self, balance: OrderBalance) -> Self {
        self.buy_token_balance = balance;
        self
    }

    /// Returns `true` when the embedded side is sell-driven.
    #[must_use]
    pub fn is_sell(&self) -> bool {
        self.side.is_sell()
    }

    /// Returns `true` when the embedded side is buy-driven.
    #[must_use]
    pub fn is_buy(&self) -> bool {
        self.side.is_buy()
    }

    /// Returns `true` when the quote-side shape is valid for the declared order kind.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.side.is_valid()
    }
}

/// Quote order data returned by the orderbook API.
///
/// This is a wire DTO, not the user-domain signing order and not the contract
/// ABI order. It accepts the orderbook's full-app-data echo shape and resolves
/// that into the app-data hash used by downstream order creation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteData {
    /// Sell-token address.
    pub sell_token: Address,
    /// Buy-token address.
    pub buy_token: Address,
    /// Optional receiver override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    /// Sell amount as an upstream decimal string.
    pub sell_amount: String,
    /// Buy amount as an upstream decimal string.
    pub buy_amount: String,
    /// Absolute UNIX expiry timestamp.
    pub valid_to: u32,
    /// Effective app-data hash derived from the orderbook response.
    pub app_data: AppDataHash,
    /// Fee amount as an upstream decimal string.
    pub fee_amount: String,
    /// Order kind.
    pub kind: OrderKind,
    /// Whether partial fills are allowed.
    #[serde(default)]
    pub partially_fillable: bool,
    /// Sell-token balance source.
    #[serde(default)]
    pub sell_token_balance: OrderBalance,
    /// Buy-token balance destination.
    #[serde(default)]
    pub buy_token_balance: OrderBalance,
}

impl<'de> Deserialize<'de> for QuoteData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct QuoteDataWire {
            sell_token: Address,
            buy_token: Address,
            receiver: Option<Address>,
            sell_amount: String,
            buy_amount: String,
            valid_to: u32,
            app_data: String,
            #[serde(default)]
            app_data_hash: Option<AppDataHash>,
            fee_amount: String,
            kind: OrderKind,
            #[serde(default)]
            partially_fillable: bool,
            #[serde(default)]
            sell_token_balance: OrderBalance,
            #[serde(default)]
            buy_token_balance: OrderBalance,
        }

        let wire = QuoteDataWire::deserialize(deserializer)?;
        let app_data = match wire.app_data_hash {
            Some(hash) => hash,
            None => AppDataHash::new(wire.app_data).map_err(D::Error::custom)?,
        };

        Ok(Self {
            sell_token: wire.sell_token,
            buy_token: wire.buy_token,
            receiver: wire.receiver,
            sell_amount: wire.sell_amount,
            buy_amount: wire.buy_amount,
            valid_to: wire.valid_to,
            app_data,
            fee_amount: wire.fee_amount,
            kind: wire.kind,
            partially_fillable: wire.partially_fillable,
            sell_token_balance: wire.sell_token_balance,
            buy_token_balance: wire.buy_token_balance,
        })
    }
}

/// Quote response DTO returned by `/api/v1/quote`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderQuoteResponse {
    /// Resolved quote payload.
    pub quote: QuoteData,
    /// Effective owner used for the quote, when returned by the API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    /// Quote expiration timestamp rendered by the orderbook.
    pub expiration: String,
    /// Quote identifier used when submitting the corresponding order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// Whether the quote was verified by the orderbook.
    pub verified: bool,
    /// Optional protocol fee basis points for the quote.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_fee_bps: Option<String>,
}

/// Orderbook order submission DTO.
///
/// This is kept separate from `QuoteData` because submission adds signature,
/// signer, signing-scheme, and optional quote-id fields while preserving the
/// orderbook wire shape expected by `/api/v1/orders`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderCreation {
    /// Sell-token address.
    pub sell_token: Address,
    /// Buy-token address.
    pub buy_token: Address,
    /// Optional receiver override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    /// Sell amount as an upstream decimal string.
    pub sell_amount: String,
    /// Buy amount as an upstream decimal string.
    pub buy_amount: String,
    /// Absolute UNIX expiry timestamp.
    pub valid_to: u32,
    /// Inline app-data payload when supplied instead of an app-data hash.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data: Option<String>,
    /// App-data hash for the submission payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data_hash: Option<AppDataHash>,
    /// Fee amount as an upstream decimal string.
    pub fee_amount: String,
    /// Order kind.
    pub kind: OrderKind,
    /// Whether partial fills are allowed.
    #[serde(default)]
    pub partially_fillable: bool,
    /// Sell-token balance source.
    #[serde(default)]
    pub sell_token_balance: OrderBalance,
    /// Buy-token balance destination.
    #[serde(default)]
    pub buy_token_balance: OrderBalance,
    /// Signature scheme used for `signature`.
    #[serde(default)]
    pub signing_scheme: SigningScheme,
    /// Raw signature string encoded for the upstream API.
    pub signature: String,
    /// Effective order owner.
    pub from: Address,
    /// Optional quote id from a prior quote response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<i64>,
}

impl OrderCreation {
    /// Creates an order-submission payload from a quote response.
    #[must_use]
    pub fn from_quote(
        quote: &QuoteData,
        from: Address,
        receiver: Option<Address>,
        signing_scheme: SigningScheme,
        signature: impl Into<String>,
    ) -> Self {
        Self {
            sell_token: quote.sell_token.clone(),
            buy_token: quote.buy_token.clone(),
            receiver: receiver.or_else(|| quote.receiver.clone()),
            sell_amount: quote.sell_amount.clone(),
            buy_amount: quote.buy_amount.clone(),
            valid_to: quote.valid_to,
            app_data: None,
            app_data_hash: Some(quote.app_data.clone()),
            fee_amount: quote.fee_amount.clone(),
            kind: quote.kind,
            partially_fillable: quote.partially_fillable,
            sell_token_balance: quote.sell_token_balance,
            buy_token_balance: quote.buy_token_balance,
            signing_scheme,
            signature: signature.into(),
            from,
            quote_id: None,
        }
    }

    /// Returns a copy of this submission payload with an attached quote id.
    #[must_use]
    pub const fn with_quote_id(mut self, quote_id: i64) -> Self {
        self.quote_id = Some(quote_id);
        self
    }
}

/// Signed order-cancellation payload for `/api/v1/orders`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderCancellations {
    /// Order UIDs to cancel.
    pub order_uids: Vec<OrderUid>,
    /// Cancellation signature string.
    pub signature: String,
    /// ECDSA signing scheme used for `signature`.
    #[serde(default)]
    pub signing_scheme: EcdsaSigningScheme,
}

/// `EthFlow`-specific orderbook metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthflowData {
    /// Transaction hash for the refund path, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_tx_hash: Option<String>,
    /// User-facing validity timestamp for the `EthFlow` order.
    pub user_valid_to: u32,
}

/// Orderbook order response DTO.
///
/// This response includes status, owner, uid, execution totals, and `EthFlow`
/// metadata that are not part of the user-domain signing order or contract ABI
/// hashing payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// Sell-token address.
    pub sell_token: Address,
    /// Buy-token address.
    pub buy_token: Address,
    /// Optional receiver override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    /// Sell amount as an upstream decimal string.
    pub sell_amount: String,
    /// Buy amount as an upstream decimal string.
    pub buy_amount: String,
    /// Absolute UNIX expiry timestamp.
    pub valid_to: u32,
    /// App-data hash attached to the order.
    pub app_data: AppDataHash,
    /// Fee amount as an upstream decimal string.
    pub fee_amount: String,
    /// Order kind.
    pub kind: OrderKind,
    /// Whether partial fills are allowed.
    #[serde(default)]
    pub partially_fillable: bool,
    /// Sell-token balance source.
    #[serde(default)]
    pub sell_token_balance: OrderBalance,
    /// Buy-token balance destination.
    #[serde(default)]
    pub buy_token_balance: OrderBalance,
    /// Signature scheme used for `signature`.
    #[serde(default)]
    pub signing_scheme: SigningScheme,
    /// Raw signature string.
    pub signature: String,
    /// Effective owner field returned by the API, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    /// Quote id used when the order originated from a quote.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<i64>,
    /// Order class.
    #[serde(default)]
    pub class: OrderClass,
    /// Canonical owner surfaced by the orderbook response.
    pub owner: Address,
    /// Order UID.
    pub uid: OrderUid,
    /// Creation timestamp string returned by the API.
    #[serde(skip_serializing_if = "Option::is_none", alias = "creationTime")]
    pub creation_date: Option<String>,
    /// Available remaining balance, when returned by the API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_balance: Option<String>,
    /// Executed sell amount.
    #[serde(default)]
    pub executed_sell_amount: String,
    /// Executed sell amount before fees, when returned separately.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executed_sell_amount_before_fees: Option<String>,
    /// Executed buy amount.
    #[serde(default)]
    pub executed_buy_amount: String,
    /// Executed fee amount component, when provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executed_fee_amount: Option<String>,
    /// Additional executed fee component, when provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executed_fee: Option<String>,
    /// Whether the order was invalidated by the protocol.
    #[serde(default)]
    pub invalidated: bool,
    /// Order lifecycle status.
    #[serde(default)]
    pub status: OrderStatus,
    /// Full fee amount, when returned by the endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_fee_amount: Option<String>,
    /// On-chain user for `EthFlow`-style orders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onchain_user: Option<Address>,
    /// `EthFlow`-specific metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ethflow_data: Option<EthflowData>,
    /// Total fee normalized by the SDK transform layer.
    #[serde(default)]
    pub total_fee: String,
}

/// Request DTO for listing an account's orders.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetOrdersRequest {
    /// Account owner whose orders should be listed.
    pub owner: Address,
    /// Pagination offset.
    #[serde(default)]
    pub offset: u32,
    /// Pagination limit.
    #[serde(default = "default_orders_limit")]
    pub limit: u32,
}

const fn default_orders_limit() -> u32 {
    1_000
}

impl GetOrdersRequest {
    /// Creates an order-list request with the upstream default pagination.
    #[must_use]
    pub const fn new(owner: Address) -> Self {
        Self {
            owner,
            offset: 0,
            limit: default_orders_limit(),
        }
    }
}

/// Request DTO for listing trades by owner or order UID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTradesRequest {
    /// Optional owner filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Address>,
    /// Optional order-UID filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_uid: Option<OrderUid>,
    /// Pagination offset.
    #[serde(default)]
    pub offset: u32,
    /// Pagination limit.
    #[serde(default = "default_trades_limit")]
    pub limit: u32,
}

const fn default_trades_limit() -> u32 {
    10
}

impl GetTradesRequest {
    /// Creates a trades request filtered by owner.
    #[must_use]
    pub const fn by_owner(owner: Address) -> Self {
        Self {
            owner: Some(owner),
            order_uid: None,
            offset: 0,
            limit: default_trades_limit(),
        }
    }

    /// Creates a trades request filtered by order UID.
    #[must_use]
    pub const fn by_order_uid(order_uid: OrderUid) -> Self {
        Self {
            owner: None,
            order_uid: Some(order_uid),
            offset: 0,
            limit: default_trades_limit(),
        }
    }

    /// Returns `true` when exactly one of `owner` or `order_uid` is set.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.owner.is_some() ^ self.order_uid.is_some()
    }
}

/// Trade DTO returned by the orderbook trades endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    /// Block number containing the trade event.
    pub block_number: u64,
    /// Log index within the block.
    pub log_index: u64,
    /// Order UID associated with the trade.
    pub order_uid: OrderUid,
    /// Owner address.
    pub owner: Address,
    /// Sell-token address.
    pub sell_token: Address,
    /// Buy-token address.
    pub buy_token: Address,
    /// Executed sell amount as an upstream decimal string.
    pub sell_amount: String,
    /// Executed sell amount before fees, when returned separately.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_amount_before_fees: Option<String>,
    /// Executed buy amount as an upstream decimal string.
    pub buy_amount: String,
    /// Settlement transaction hash.
    #[serde(alias = "txHash")]
    pub transaction_hash: String,
}

/// Native-price response from `/api/v1/token/{token}/native_price`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativePriceResponse {
    /// Token price quoted in the chain's native asset.
    pub price: f64,
}

/// Total-surplus response from `/api/v1/users/{owner}/total_surplus`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TotalSurplus {
    /// Total surplus value as an upstream decimal string.
    pub total_surplus: String,
}

/// Full app-data response from the orderbook app-data endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppDataObject {
    /// Full serialized app-data payload.
    pub full_app_data: String,
}

/// Order entry inside an auction snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuctionOrder {
    /// Order UID.
    pub uid: OrderUid,
    /// Sell-token address.
    pub sell_token: Address,
    /// Buy-token address.
    pub buy_token: Address,
    /// Optional receiver override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    /// Sell amount as an upstream decimal string.
    pub sell_amount: String,
    /// Buy amount as an upstream decimal string.
    pub buy_amount: String,
    /// Absolute UNIX expiry timestamp.
    pub valid_to: u32,
    /// App-data hash.
    pub app_data: AppDataHash,
    /// Fee amount as an upstream decimal string.
    pub fee_amount: String,
    /// Order kind.
    pub kind: OrderKind,
    /// Whether partial fills are allowed.
    #[serde(default)]
    pub partially_fillable: bool,
    /// Optional owner value when provided by the auction endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Address>,
}

/// Auction snapshot returned by the orderbook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Auction {
    /// Auction id, when exposed by the endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// Current auction block number.
    pub block: u64,
    /// Latest settlement block, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_settlement_block: Option<u64>,
    /// Auction orders.
    #[serde(default)]
    pub orders: Vec<AuctionOrder>,
    /// Clearing prices keyed by token address.
    #[serde(default)]
    pub prices: std::collections::BTreeMap<String, String>,
}

/// Competition-status kind returned by `/api/v1/orders/{uid}/status`.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CompetitionOrderStatusKind {
    /// Open but not yet scheduled.
    Open,
    /// Scheduled for competition.
    Scheduled,
    /// Actively competing.
    Active,
    /// Solved by at least one solver.
    Solved,
    /// Currently executing.
    Executing,
    /// Traded successfully.
    Traded,
    /// Cancelled before execution.
    Cancelled,
}

/// Solver execution entry nested inside competition-status responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolverExecution {
    /// Solver identifier or address rendered by the API.
    pub solver: String,
    /// Executed sell amount for this solver path, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executed_sell_amount: Option<String>,
    /// Executed buy amount for this solver path, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executed_buy_amount: Option<String>,
}

/// Competition-status response for an order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompetitionOrderStatus {
    /// High-level competition status kind.
    #[serde(rename = "type")]
    pub kind: CompetitionOrderStatusKind,
    /// Optional solver execution payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Vec<SolverExecution>>,
}

/// Nested auction snapshot inside solver-competition responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompetitionAuction {
    /// Order UIDs participating in the competition.
    #[serde(default)]
    pub orders: Vec<String>,
    /// Clearing prices keyed by token address.
    #[serde(default)]
    pub prices: std::collections::BTreeMap<String, String>,
}

/// Settlement candidate nested inside solver-competition responses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolverSettlement {
    /// Optional settlement ranking score.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranking: Option<f64>,
    /// Solver address, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solver_address: Option<String>,
    /// Settlement score as rendered by the API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<String>,
    /// Reference score used for comparison.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_score: Option<String>,
    /// Settlement transaction hash, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,
    /// Clearing prices keyed by token address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clearing_prices: Option<std::collections::BTreeMap<String, String>>,
    /// Whether this settlement was the winning one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_winner: Option<bool>,
    /// Whether the settlement was filtered out by the API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filtered_out: Option<bool>,
}

/// Solver-competition response returned by the orderbook.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolverCompetitionResponse {
    /// Auction id, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auction_id: Option<i64>,
    /// Start block of the auction, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auction_start_block: Option<u64>,
    /// Deadline block of the auction, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auction_deadline_block: Option<u64>,
    /// Settlement transaction hashes associated with the competition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_hashes: Option<Vec<String>>,
    /// Nested auction payload, when returned by the endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auction: Option<CompetitionAuction>,
    /// Settlement candidates, when returned by the endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solutions: Option<Vec<SolverSettlement>>,
}
