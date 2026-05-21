use serde::{Deserialize, Deserializer, Serialize, de::Error as DeError};

use crate::error::OrderbookError;

use super::{
    Address, Amount, AppDataHash, BuyTokenDestination, OrderKind, SellTokenSource,
    enums::{PriceQuality, SigningScheme},
};

/// Encodes the mutually exclusive buy-side or sell-side amount on quote requests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QuoteSide {
    /// Whether the quote is sell-driven or buy-driven.
    pub kind: OrderKind,
    /// Sell amount before fee for sell quotes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_amount_before_fee: Option<Amount>,
    /// Buy amount after fee for buy quotes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_amount_after_fee: Option<Amount>,
}

impl QuoteSide {
    /// Creates a sell-side quote request amount.
    #[must_use]
    pub const fn sell(amount: Amount) -> Self {
        Self {
            kind: OrderKind::Sell,
            sell_amount_before_fee: Some(amount),
            buy_amount_after_fee: None,
        }
    }

    /// Creates a buy-side quote request amount.
    #[must_use]
    pub const fn buy(amount: Amount) -> Self {
        Self {
            kind: OrderKind::Buy,
            sell_amount_before_fee: None,
            buy_amount_after_fee: Some(amount),
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
#[non_exhaustive]
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
    pub sell_token_balance: SellTokenSource,
    /// Buy-token balance destination.
    #[serde(default)]
    pub buy_token_balance: BuyTokenDestination,
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
            sell_token_balance: SellTokenSource::Erc20,
            buy_token_balance: BuyTokenDestination::Erc20,
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
    pub const fn with_receiver(mut self, receiver: Address) -> Self {
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
    pub const fn with_app_data_hash(mut self, app_data_hash: AppDataHash) -> Self {
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
    pub const fn with_sell_token_balance(mut self, balance: SellTokenSource) -> Self {
        self.sell_token_balance = balance;
        self
    }

    /// Returns a copy of this request with a new buy-token balance destination.
    #[must_use]
    pub const fn with_buy_token_balance(mut self, balance: BuyTokenDestination) -> Self {
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

    /// Validates local quote preconditions before dispatching the HTTP request.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::InvalidQuoteRequest`] when the quote side is
    /// not well-formed or `verificationGasLimit` is paired with a non-EIP-1271
    /// scheme. Returns [`OrderbookError::IncompatibleSigningScheme`] when an
    /// ECDSA signing scheme is marked as an on-chain order.
    pub fn validate(&self) -> Result<(), OrderbookError> {
        if !self.is_valid() {
            return Err(OrderbookError::InvalidQuoteRequest {
                field: "side",
                reason: cow_sdk_core::ValidationReason::Precondition {
                    details: "exactly one of sellAmountBeforeFee or buyAmountAfterFee must be set",
                },
            });
        }

        if self.verification_gas_limit.is_some() && self.signing_scheme != SigningScheme::Eip1271 {
            return Err(OrderbookError::InvalidQuoteRequest {
                field: "verificationGasLimit",
                reason: cow_sdk_core::ValidationReason::Precondition {
                    details: "only eip1271 quote signing supports verificationGasLimit",
                },
            });
        }

        match (self.signing_scheme, self.onchain_order) {
            (SigningScheme::Eip712 | SigningScheme::EthSign, true) => {
                Err(OrderbookError::IncompatibleSigningScheme {
                    signing_scheme: self.signing_scheme,
                    onchain_order: self.onchain_order,
                })
            }
            (
                SigningScheme::Eip712
                | SigningScheme::EthSign
                | SigningScheme::Eip1271
                | SigningScheme::PreSign,
                _,
            ) => Ok(()),
        }
    }
}

/// Quote order data returned by the orderbook API.
///
/// This is a wire DTO, not the user-domain signing order and not the contract
/// ABI order. It accepts the orderbook's full-app-data echo shape and resolves
/// that into the app-data hash used by downstream order creation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QuoteData {
    /// Sell-token address.
    pub sell_token: Address,
    /// Buy-token address.
    pub buy_token: Address,
    /// Optional receiver override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    /// Sell amount in the upstream decimal-string wire shape.
    pub sell_amount: Amount,
    /// Buy amount in the upstream decimal-string wire shape.
    pub buy_amount: Amount,
    /// Absolute UNIX expiry timestamp.
    pub valid_to: u32,
    /// Effective app-data hash derived from the orderbook response.
    pub app_data: AppDataHash,
    /// Network-cost amount echoed by the orderbook `/quote` response.
    ///
    /// Stored under the upstream wire name `feeAmount` so the deterministic
    /// JSON schema stays aligned with the services contract; consumers read
    /// the value through [`QuoteData::network_cost_amount`] and configure it
    /// through [`QuoteData::with_network_cost_amount`].
    fee_amount: Amount,
    /// Order kind.
    pub kind: OrderKind,
    /// Whether partial fills are allowed.
    #[serde(default)]
    pub partially_fillable: bool,
    /// Sell-token balance source.
    #[serde(default)]
    pub sell_token_balance: SellTokenSource,
    /// Buy-token balance destination.
    #[serde(default)]
    pub buy_token_balance: BuyTokenDestination,
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
            sell_amount: Amount,
            buy_amount: Amount,
            valid_to: u32,
            app_data: String,
            #[serde(default)]
            app_data_hash: Option<AppDataHash>,
            fee_amount: Amount,
            kind: OrderKind,
            #[serde(default)]
            partially_fillable: bool,
            #[serde(default)]
            sell_token_balance: SellTokenSource,
            #[serde(default)]
            buy_token_balance: BuyTokenDestination,
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

impl QuoteData {
    /// Creates a quote-data payload with the required trade fields.
    ///
    /// Optional fields (receiver, partial-fill, balance sources, network-cost
    /// amount) can be attached through the `with_*` setters. The
    /// network-cost amount defaults to `"0"` and is populated from the
    /// orderbook wire on deserialization.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        sell_token: Address,
        buy_token: Address,
        sell_amount: Amount,
        buy_amount: Amount,
        valid_to: u32,
        app_data: AppDataHash,
        kind: OrderKind,
    ) -> Self {
        Self {
            sell_token,
            buy_token,
            receiver: None,
            sell_amount,
            buy_amount,
            valid_to,
            app_data,
            fee_amount: Amount::ZERO,
            kind,
            partially_fillable: false,
            sell_token_balance: SellTokenSource::Erc20,
            buy_token_balance: BuyTokenDestination::Erc20,
        }
    }

    /// Returns the network-cost amount echoed by the orderbook `/quote`
    /// response.
    #[must_use]
    pub const fn network_cost_amount(&self) -> &Amount {
        &self.fee_amount
    }

    /// Returns a copy of this payload with an explicit network-cost amount.
    #[must_use]
    pub const fn with_network_cost_amount(mut self, value: Amount) -> Self {
        self.fee_amount = value;
        self
    }

    /// Sets the network-cost amount echoed by the orderbook `/quote`
    /// response, mutating the payload in place.
    pub const fn set_network_cost_amount(&mut self, value: Amount) {
        self.fee_amount = value;
    }

    /// Returns a copy of this payload with an explicit receiver.
    #[must_use]
    pub const fn with_receiver(mut self, receiver: Address) -> Self {
        self.receiver = Some(receiver);
        self
    }

    /// Returns a copy of this payload with the partial-fill flag set.
    #[must_use]
    pub const fn with_partially_fillable(mut self, partially_fillable: bool) -> Self {
        self.partially_fillable = partially_fillable;
        self
    }

    /// Returns a copy of this payload with an explicit sell-token balance source.
    #[must_use]
    pub const fn with_sell_token_balance(mut self, balance: SellTokenSource) -> Self {
        self.sell_token_balance = balance;
        self
    }

    /// Returns a copy of this payload with an explicit buy-token balance destination.
    #[must_use]
    pub const fn with_buy_token_balance(mut self, balance: BuyTokenDestination) -> Self {
        self.buy_token_balance = balance;
        self
    }
}

/// Quote response DTO returned by `/api/v1/quote`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OrderQuoteResponse {
    /// Resolved quote payload.
    pub quote: QuoteData,
    /// Effective owner used for the quote, when returned by the API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    /// Quote expiration timestamp rendered by the orderbook.
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

impl OrderQuoteResponse {
    /// Creates a quote response from the resolved quote payload and its expiration timestamp.
    #[must_use]
    pub fn new(quote: QuoteData, expiration: impl Into<String>, verified: bool) -> Self {
        Self {
            quote,
            from: None,
            expiration: expiration.into(),
            id: None,
            verified,
            protocol_fee_bps: None,
        }
    }

    /// Returns a copy of this response with an explicit owner address.
    #[must_use]
    pub const fn with_from(mut self, from: Address) -> Self {
        self.from = Some(from);
        self
    }

    /// Returns a copy of this response with an explicit quote id.
    #[must_use]
    pub const fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    /// Returns a copy of this response with explicit protocol-fee basis points.
    #[must_use]
    pub fn with_protocol_fee_bps(mut self, value: impl Into<String>) -> Self {
        self.protocol_fee_bps = Some(value.into());
        self
    }
}
