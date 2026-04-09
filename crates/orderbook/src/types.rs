use serde::{Deserialize, Deserializer, Serialize, de::Error as DeError};

pub use cow_sdk_core::{
    Address, ApiBaseUrls, ApiContext, AppDataHash, CowEnv, ENVS_LIST, EVM_NATIVE_CURRENCY_ADDRESS,
    OrderBalance, OrderKind, OrderUid, QuoteAmountsAndCosts, SupportedChainId,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApiContextOverride {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<SupportedChainId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_urls: Option<ApiBaseUrls>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EnvBaseUrlOverrides {
    pub prod: Option<String>,
    pub staging: Option<String>,
}

impl EnvBaseUrlOverrides {
    pub fn set(&mut self, env: CowEnv, base_url: impl Into<String>) {
        match env {
            CowEnv::Prod => self.prod = Some(base_url.into()),
            CowEnv::Staging => self.staging = Some(base_url.into()),
        }
    }

    pub fn get(&self, env: CowEnv) -> Option<&str> {
        match env {
            CowEnv::Prod => self.prod.as_deref(),
            CowEnv::Staging => self.staging.as_deref(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PriceQuality {
    Fast,
    Optimal,
    #[default]
    Verified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SigningScheme {
    #[default]
    Eip712,
    EthSign,
    Eip1271,
    PreSign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EcdsaSigningScheme {
    #[default]
    Eip712,
    EthSign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OrderClass {
    #[default]
    Market,
    Limit,
    Liquidity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum OrderStatus {
    PresignaturePending,
    #[default]
    Open,
    Fulfilled,
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteSide {
    pub kind: OrderKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_amount_before_fee: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_amount_after_fee: Option<String>,
}

impl QuoteSide {
    pub fn sell(amount: impl Into<String>) -> Self {
        Self {
            kind: OrderKind::Sell,
            sell_amount_before_fee: Some(amount.into()),
            buy_amount_after_fee: None,
        }
    }

    pub fn buy(amount: impl Into<String>) -> Self {
        Self {
            kind: OrderKind::Buy,
            sell_amount_before_fee: None,
            buy_amount_after_fee: Some(amount.into()),
        }
    }

    pub fn is_sell(&self) -> bool {
        self.kind == OrderKind::Sell
    }

    pub fn is_buy(&self) -> bool {
        self.kind == OrderKind::Buy
    }

    pub fn is_valid(&self) -> bool {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderQuoteRequest {
    pub sell_token: Address,
    pub buy_token: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_for: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data_hash: Option<AppDataHash>,
    #[serde(default)]
    pub partially_fillable: bool,
    #[serde(default)]
    pub sell_token_balance: OrderBalance,
    #[serde(default)]
    pub buy_token_balance: OrderBalance,
    pub from: Address,
    #[serde(default)]
    pub price_quality: PriceQuality,
    #[serde(default)]
    pub signing_scheme: SigningScheme,
    #[serde(default)]
    pub onchain_order: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_gas_limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    #[serde(flatten)]
    pub side: QuoteSide,
}

impl OrderQuoteRequest {
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

    pub fn with_receiver(mut self, receiver: Address) -> Self {
        self.receiver = Some(receiver);
        self
    }

    pub fn with_valid_to(mut self, valid_to: u32) -> Self {
        self.valid_to = Some(valid_to);
        self
    }

    pub fn with_valid_for(mut self, valid_for: u32) -> Self {
        self.valid_for = Some(valid_for);
        self
    }

    pub fn with_app_data(mut self, app_data: impl Into<String>) -> Self {
        self.app_data = Some(app_data.into());
        self
    }

    pub fn with_app_data_hash(mut self, app_data_hash: AppDataHash) -> Self {
        self.app_data_hash = Some(app_data_hash);
        self
    }

    pub fn with_price_quality(mut self, quality: PriceQuality) -> Self {
        self.price_quality = quality;
        self
    }

    pub fn with_signing_scheme(mut self, scheme: SigningScheme) -> Self {
        self.signing_scheme = scheme;
        self
    }

    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_onchain_order(mut self) -> Self {
        self.onchain_order = true;
        self
    }

    pub fn with_verification_gas_limit(mut self, verification_gas_limit: u64) -> Self {
        self.verification_gas_limit = Some(verification_gas_limit);
        self
    }

    pub fn with_partially_fillable(mut self) -> Self {
        self.partially_fillable = true;
        self
    }

    pub fn with_sell_token_balance(mut self, balance: OrderBalance) -> Self {
        self.sell_token_balance = balance;
        self
    }

    pub fn with_buy_token_balance(mut self, balance: OrderBalance) -> Self {
        self.buy_token_balance = balance;
        self
    }

    pub fn is_sell(&self) -> bool {
        self.side.is_sell()
    }

    pub fn is_buy(&self) -> bool {
        self.side.is_buy()
    }

    pub fn is_valid(&self) -> bool {
        self.side.is_valid()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteData {
    pub sell_token: Address,
    pub buy_token: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderQuoteResponse {
    pub quote: QuoteData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    pub expiration: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub verified: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_fee_bps: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderCreation {
    pub sell_token: Address,
    pub buy_token: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    pub sell_amount: String,
    pub buy_amount: String,
    pub valid_to: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data_hash: Option<AppDataHash>,
    pub fee_amount: String,
    pub kind: OrderKind,
    #[serde(default)]
    pub partially_fillable: bool,
    #[serde(default)]
    pub sell_token_balance: OrderBalance,
    #[serde(default)]
    pub buy_token_balance: OrderBalance,
    #[serde(default)]
    pub signing_scheme: SigningScheme,
    pub signature: String,
    pub from: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<i64>,
}

impl OrderCreation {
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

    pub fn with_quote_id(mut self, quote_id: i64) -> Self {
        self.quote_id = Some(quote_id);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderCancellations {
    pub order_uids: Vec<OrderUid>,
    pub signature: String,
    #[serde(default)]
    pub signing_scheme: EcdsaSigningScheme,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthflowData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_tx_hash: Option<String>,
    pub user_valid_to: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub sell_token: Address,
    pub buy_token: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
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
    #[serde(default)]
    pub signing_scheme: SigningScheme,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<i64>,
    #[serde(default)]
    pub class: OrderClass,
    pub owner: Address,
    pub uid: OrderUid,
    #[serde(skip_serializing_if = "Option::is_none", alias = "creationTime")]
    pub creation_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_balance: Option<String>,
    #[serde(default)]
    pub executed_sell_amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executed_sell_amount_before_fees: Option<String>,
    #[serde(default)]
    pub executed_buy_amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executed_fee_amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executed_fee: Option<String>,
    #[serde(default)]
    pub invalidated: bool,
    #[serde(default)]
    pub status: OrderStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_fee_amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onchain_user: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ethflow_data: Option<EthflowData>,
    #[serde(default)]
    pub total_fee: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetOrdersRequest {
    pub owner: Address,
    #[serde(default)]
    pub offset: u32,
    #[serde(default = "default_orders_limit")]
    pub limit: u32,
}

const fn default_orders_limit() -> u32 {
    1_000
}

impl GetOrdersRequest {
    pub fn new(owner: Address) -> Self {
        Self {
            owner,
            offset: 0,
            limit: default_orders_limit(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTradesRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_uid: Option<OrderUid>,
    #[serde(default)]
    pub offset: u32,
    #[serde(default = "default_trades_limit")]
    pub limit: u32,
}

const fn default_trades_limit() -> u32 {
    10
}

impl GetTradesRequest {
    pub fn by_owner(owner: Address) -> Self {
        Self {
            owner: Some(owner),
            order_uid: None,
            offset: 0,
            limit: default_trades_limit(),
        }
    }

    pub fn by_order_uid(order_uid: OrderUid) -> Self {
        Self {
            owner: None,
            order_uid: Some(order_uid),
            offset: 0,
            limit: default_trades_limit(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.owner.is_some() ^ self.order_uid.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    pub block_number: u64,
    pub log_index: u64,
    pub order_uid: OrderUid,
    pub owner: Address,
    pub sell_token: Address,
    pub buy_token: Address,
    pub sell_amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_amount_before_fees: Option<String>,
    pub buy_amount: String,
    #[serde(alias = "txHash")]
    pub transaction_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativePriceResponse {
    pub price: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TotalSurplus {
    pub total_surplus: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppDataObject {
    pub full_app_data: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuctionOrder {
    pub uid: OrderUid,
    pub sell_token: Address,
    pub buy_token: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    pub sell_amount: String,
    pub buy_amount: String,
    pub valid_to: u32,
    pub app_data: AppDataHash,
    pub fee_amount: String,
    pub kind: OrderKind,
    #[serde(default)]
    pub partially_fillable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Address>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Auction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub block: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_settlement_block: Option<u64>,
    #[serde(default)]
    pub orders: Vec<AuctionOrder>,
    #[serde(default)]
    pub prices: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CompetitionOrderStatusKind {
    Open,
    Scheduled,
    Active,
    Solved,
    Executing,
    Traded,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolverExecution {
    pub solver: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executed_sell_amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executed_buy_amount: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompetitionOrderStatus {
    #[serde(rename = "type")]
    pub kind: CompetitionOrderStatusKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Vec<SolverExecution>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompetitionAuction {
    #[serde(default)]
    pub orders: Vec<String>,
    #[serde(default)]
    pub prices: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolverSettlement {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranking: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solver_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_score: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clearing_prices: Option<std::collections::BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_winner: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filtered_out: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolverCompetitionResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auction_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auction_start_block: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auction_deadline_block: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_hashes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auction: Option<CompetitionAuction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solutions: Option<Vec<SolverSettlement>>,
}
