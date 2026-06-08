use serde::{Deserialize, Serialize};

use super::{
    amount::Amount,
    identity::{Address, AppDataHash, ChainId},
};
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

/// Sell or buy side of a trade.
///
/// Encoded as `keccak256("buy")` / `keccak256("sell")` in the EIP-712
/// `Order` type. The set of variants is fixed by the protocol; adding a third
/// variant would change the protocol, not the SDK. Classified as
/// `protocol-fixed-exhaustive` in the workspace enum policy manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderKind {
    /// Buy an exact amount of the buy token.
    Buy,
    /// Sell an exact amount of the sell token.
    Sell,
}

/// Source from which the `sellAmount` is drawn upon order fulfillment.
///
/// This mirrors the services `SellTokenSource` enum byte-for-byte on the wire.
/// Orders model the sell-side allowance path independently of the buy-side
/// payout path, which is typed as [`BuyTokenDestination`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum SellTokenSource {
    /// Sell tokens are drawn through the regular ERC-20 allowance granted to
    /// the vault relayer.
    #[default]
    Erc20,
    /// Sell tokens are drawn through the Balancer vault relayer using an
    /// external ERC-20 allowance on the vault.
    External,
    /// Sell tokens are drawn from the user's internal Balancer vault balance.
    Internal,
}

/// Destination to which the `buyAmount` is transferred upon order fulfillment.
///
/// This mirrors the services `BuyTokenDestination` enum byte-for-byte on the
/// wire. The buy-side payout path only accepts the ERC-20 and internal
/// variants; the [`SellTokenSource::External`] variant has no buy-side
/// counterpart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum BuyTokenDestination {
    /// Buy tokens are paid out as a regular ERC-20 transfer.
    #[default]
    Erc20,
    /// Buy tokens are paid out as a Balancer vault internal balance credit.
    Internal,
}

/// Token metadata used by user-domain SDK surfaces.
#[non_exhaustive]
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

impl TokenInfo {
    /// Creates token metadata from the canonical display fields.
    #[inline]
    #[must_use]
    pub const fn new(
        chain_id: ChainId,
        address: Address,
        decimals: u8,
        symbol: String,
        name: String,
        logo_url: Option<String>,
    ) -> Self {
        Self {
            chain_id,
            address,
            decimals,
            symbol,
            name,
            logo_url,
        }
    }
}

/// User-domain order shape prepared for signing and trading workflows.
///
/// It is the canonical signed-order payload and mirrors the upstream services
/// `OrderData` byte-for-byte (the same field set, EIP-712 type hash, and field
/// ordering). This is not an orderbook wire DTO or an ABI struct. It is hashed
/// directly by `cow_sdk_contracts::hash_order` for the EIP-712 digest and UID
/// (a `receiver` of `address(0)` is the legal "pay-to-owner" sentinel). It is
/// submitted to the orderbook as `cow_sdk_orderbook::OrderCreation` and read
/// back as the separate `cow_sdk_orderbook::Order` response record.
///
/// Downstream crates construct orders through [`OrderData::new`] and the
/// chainable `with_*` setters rather than a struct literal so additive fields
/// remain semver-compatible.
///
/// ```compile_fail
/// use cow_sdk_core::{
///     Address, Amount, AppDataHash, BuyTokenDestination, OrderKind, SellTokenSource,
///     OrderData,
/// };
///
/// let _order = OrderData {
///     sell_token: Address::new("0x1111111111111111111111111111111111111111").unwrap(),
///     buy_token: Address::new("0x2222222222222222222222222222222222222222").unwrap(),
///     receiver: Address::new("0x3333333333333333333333333333333333333333").unwrap(),
///     sell_amount: Amount::new("100").unwrap(),
///     buy_amount: Amount::new("200").unwrap(),
///     valid_to: 1_700_000_000,
///     app_data: AppDataHash::new(
///         "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
///     )
///     .unwrap(),
///     fee_amount: Amount::new("5").unwrap(),
///     kind: OrderKind::Sell,
///     partially_fillable: true,
///     sell_token_balance: SellTokenSource::External,
///     buy_token_balance: BuyTokenDestination::Internal,
/// };
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderData {
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
    pub sell_token_balance: SellTokenSource,
    /// Buy-token balance destination.
    #[serde(default)]
    pub buy_token_balance: BuyTokenDestination,
}

impl OrderData {
    /// Creates an unsigned order from the canonical EIP-712 field set.
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "constructor mirrors the canonical EIP-712 field set so callers can migrate off struct-literal construction without losing explicit control over any wire field"
    )]
    pub const fn new(
        sell_token: Address,
        buy_token: Address,
        receiver: Address,
        sell_amount: Amount,
        buy_amount: Amount,
        valid_to: u32,
        app_data: AppDataHash,
        fee_amount: Amount,
        kind: OrderKind,
        partially_fillable: bool,
        sell_token_balance: SellTokenSource,
        buy_token_balance: BuyTokenDestination,
    ) -> Self {
        Self {
            sell_token,
            buy_token,
            receiver,
            sell_amount,
            buy_amount,
            valid_to,
            app_data,
            fee_amount,
            kind,
            partially_fillable,
            sell_token_balance,
            buy_token_balance,
        }
    }

    /// Returns a copy of this order with a different receiver.
    #[must_use]
    pub const fn with_receiver(mut self, receiver: Address) -> Self {
        self.receiver = receiver;
        self
    }

    /// Returns a copy of this order with a different app-data hash.
    #[must_use]
    pub const fn with_app_data(mut self, app_data: AppDataHash) -> Self {
        self.app_data = app_data;
        self
    }

    /// Returns a copy of this order with a different fee amount.
    #[must_use]
    pub const fn with_fee_amount(mut self, fee_amount: Amount) -> Self {
        self.fee_amount = fee_amount;
        self
    }

    /// Returns a copy of this order with an updated partial-fill flag.
    #[must_use]
    pub const fn with_partially_fillable(mut self, partially_fillable: bool) -> Self {
        self.partially_fillable = partially_fillable;
        self
    }

    /// Returns a copy of this order with a different sell-token balance source.
    #[must_use]
    pub const fn with_sell_token_balance(mut self, sell_token_balance: SellTokenSource) -> Self {
        self.sell_token_balance = sell_token_balance;
        self
    }

    /// Returns a copy of this order with a different buy-token balance destination.
    #[must_use]
    pub const fn with_buy_token_balance(mut self, buy_token_balance: BuyTokenDestination) -> Self {
        self.buy_token_balance = buy_token_balance;
        self
    }

    /// Returns the canonical EIP-712 field ordering for orders.
    #[must_use]
    pub const fn field_names() -> &'static [&'static str; ORDER_TYPE_FIELD_NAMES.len()] {
        &ORDER_TYPE_FIELD_NAMES
    }
}
