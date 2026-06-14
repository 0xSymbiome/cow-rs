use serde::{Deserialize, Deserializer, Serialize, de::Error as DeError};

use crate::error::{OrderbookError, QuoteEchoField};

use super::{
    Address, Amount, AppDataHash, BuyTokenDestination, OrderKind, QuoteAppData, SellTokenSource,
    enums::{PriceQuality, SigningScheme},
};

/// The sell amount on a sell-side quote request, distinguishing whether the
/// network fee is taken before or after the amount.
///
/// Mirrors the orderbook `OrderQuoteSide` sell-amount oneOf: exactly one of
/// `sellAmountBeforeFee` or `sellAmountAfterFee` is present on the wire, so the
/// before/after-fee distinction is a type-level invariant rather than a
/// runtime check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum SellAmount {
    /// Sell amount measured before the network fee is deducted.
    BeforeFee {
        /// The sell amount in sell-token atoms, before fee.
        #[serde(rename = "sellAmountBeforeFee")]
        value: Amount,
    },
    /// Sell amount measured after the network fee is deducted.
    AfterFee {
        /// The sell amount in sell-token atoms, after fee.
        #[serde(rename = "sellAmountAfterFee")]
        value: Amount,
    },
}

impl SellAmount {
    /// Returns the underlying sell amount regardless of the fee basis.
    #[must_use]
    pub const fn amount(&self) -> &Amount {
        match self {
            Self::BeforeFee { value } | Self::AfterFee { value } => value,
        }
    }
}

impl<'de> Deserialize<'de> for SellAmount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Read the two mutually exclusive keys explicitly rather than through an
        // untagged enum so a malformed amount surfaces the `Amount` parse error
        // instead of a generic "did not match any variant" message.
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Wire {
            sell_amount_before_fee: Option<Amount>,
            sell_amount_after_fee: Option<Amount>,
        }

        let wire = Wire::deserialize(deserializer)?;
        match (wire.sell_amount_before_fee, wire.sell_amount_after_fee) {
            (Some(value), None) => Ok(Self::BeforeFee { value }),
            (None, Some(value)) => Ok(Self::AfterFee { value }),
            (None, None) => Err(DeError::custom(
                "missing one of `sellAmountBeforeFee` or `sellAmountAfterFee`",
            )),
            (Some(_), Some(_)) => Err(DeError::custom(
                "must specify at most one of `sellAmountBeforeFee` or `sellAmountAfterFee`",
            )),
        }
    }
}

/// Encodes the mutually exclusive buy-side or sell-side amount on quote
/// requests.
///
/// Mirrors the orderbook `OrderQuoteSide` `kind` oneOf so that a quote request
/// carries exactly one side amount by construction: a sell request carries a
/// [`SellAmount`] (before or after fee) and a buy request carries a buy amount
/// after fee.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
#[non_exhaustive]
pub enum OrderQuoteSide {
    /// Sell-driven quote.
    #[serde(rename_all = "camelCase")]
    Sell {
        /// The sell amount, before or after fee.
        #[serde(flatten)]
        sell_amount: SellAmount,
    },
    /// Buy-driven quote.
    #[serde(rename_all = "camelCase")]
    Buy {
        /// The buy amount after fee, in buy-token atoms.
        buy_amount_after_fee: Amount,
    },
}

impl OrderQuoteSide {
    /// Creates a sell-side quote request for a sell amount measured before the
    /// network fee.
    #[must_use]
    pub const fn sell_before_fee(amount: Amount) -> Self {
        Self::Sell {
            sell_amount: SellAmount::BeforeFee { value: amount },
        }
    }

    /// Creates a sell-side quote request for a sell amount measured after the
    /// network fee.
    #[must_use]
    pub const fn sell_after_fee(amount: Amount) -> Self {
        Self::Sell {
            sell_amount: SellAmount::AfterFee { value: amount },
        }
    }

    /// Creates a sell-side quote request. Alias for [`Self::sell_before_fee`],
    /// the orderbook's default sell-amount basis.
    #[must_use]
    pub const fn sell(amount: Amount) -> Self {
        Self::sell_before_fee(amount)
    }

    /// Creates a buy-side quote request amount.
    #[must_use]
    pub const fn buy(amount: Amount) -> Self {
        Self::Buy {
            buy_amount_after_fee: amount,
        }
    }

    /// Returns the order kind implied by this side.
    #[must_use]
    pub const fn kind(&self) -> OrderKind {
        match self {
            Self::Sell { .. } => OrderKind::Sell,
            Self::Buy { .. } => OrderKind::Buy,
        }
    }

    /// Returns `true` when this quote side is sell-driven.
    #[must_use]
    pub const fn is_sell(&self) -> bool {
        matches!(self, Self::Sell { .. })
    }

    /// Returns `true` when this quote side is buy-driven.
    #[must_use]
    pub const fn is_buy(&self) -> bool {
        matches!(self, Self::Buy { .. })
    }
}

/// Validity window for a quote request.
///
/// Mirrors the orderbook quote validity oneOf: a request carries either an
/// absolute `validTo` UNIX timestamp or a relative `validFor` duration in
/// seconds, never both. Modeling it as an enum makes that mutual exclusion a
/// type-level invariant. When neither is supplied on the wire it defaults to
/// the protocol's 30-minute relative window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum QuoteValidity {
    /// Absolute UNIX expiry timestamp (`validTo`).
    ValidTo(u32),
    /// Relative validity duration in seconds (`validFor`).
    ValidFor(u32),
}

impl Default for QuoteValidity {
    fn default() -> Self {
        // The protocol default quote validity is a 30-minute relative window.
        Self::ValidFor(30 * 60)
    }
}

impl<'de> Deserialize<'de> for QuoteValidity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Wire {
            valid_to: Option<u32>,
            valid_for: Option<u32>,
        }

        let wire = Wire::deserialize(deserializer)?;
        match (wire.valid_to, wire.valid_for) {
            (Some(valid_to), None) => Ok(Self::ValidTo(valid_to)),
            (None, Some(valid_for)) => Ok(Self::ValidFor(valid_for)),
            (None, None) => Ok(Self::default()),
            (Some(_), Some(_)) => Err(DeError::custom(
                "must specify at most one of `validTo` or `validFor`",
            )),
        }
    }
}

impl Serialize for QuoteValidity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct as _;

        let (field, value) = match self {
            Self::ValidTo(valid_to) => ("validTo", valid_to),
            Self::ValidFor(valid_for) => ("validFor", valid_for),
        };
        let mut state = serializer.serialize_struct("QuoteValidity", 1)?;
        state.serialize_field(field, value)?;
        state.end()
    }
}

/// Default EIP-1271 verification gas limit applied when a quote request does
/// not supply one, matching the orderbook default.
#[must_use]
pub const fn default_verification_gas_limit() -> u64 {
    27_000
}

/// Signing scheme for a quote request.
///
/// Mirrors the orderbook `QuoteSigningScheme` oneOf so that the scheme-specific
/// constraints are type-level: only EIP-1271 carries a `verificationGasLimit`,
/// only EIP-1271 and pre-sign can be on-chain orders, and an ECDSA scheme
/// (`eip712`/`ethSign`) can never be marked on-chain.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "signingScheme",
    rename_all = "lowercase",
    try_from = "QuoteSigningSchemeWire"
)]
#[non_exhaustive]
pub enum QuoteSigningScheme {
    /// EIP-712 typed-data signature (default).
    #[default]
    Eip712,
    /// `eth_sign` signature.
    EthSign,
    /// EIP-1271 smart-contract signature.
    Eip1271 {
        /// Whether the eventual order is placed on-chain.
        #[serde(rename = "onchainOrder")]
        onchain_order: bool,
        /// Gas limit reserved for the EIP-1271 verification call.
        #[serde(
            rename = "verificationGasLimit",
            default = "default_verification_gas_limit"
        )]
        verification_gas_limit: u64,
    },
    /// Pre-sign (on-chain authorization) signature.
    PreSign {
        /// Whether the eventual order is placed on-chain.
        #[serde(rename = "onchainOrder")]
        onchain_order: bool,
    },
}

impl QuoteSigningScheme {
    /// Returns the base [`SigningScheme`] for this quote signing scheme.
    #[must_use]
    pub const fn scheme(&self) -> SigningScheme {
        match self {
            Self::Eip712 => SigningScheme::Eip712,
            Self::EthSign => SigningScheme::EthSign,
            Self::Eip1271 { .. } => SigningScheme::Eip1271,
            Self::PreSign { .. } => SigningScheme::PreSign,
        }
    }

    /// Returns whether the eventual order is placed on-chain.
    #[must_use]
    pub const fn is_onchain_order(&self) -> bool {
        matches!(
            self,
            Self::Eip1271 {
                onchain_order: true,
                ..
            } | Self::PreSign {
                onchain_order: true
            }
        )
    }
}

/// Deserialization helper that mirrors the orderbook wire shape and validates
/// the scheme-specific constraints through [`TryFrom`].
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct QuoteSigningSchemeWire {
    #[serde(default)]
    signing_scheme: SigningScheme,
    #[serde(default)]
    verification_gas_limit: Option<u64>,
    #[serde(default)]
    onchain_order: bool,
}

impl TryFrom<QuoteSigningSchemeWire> for QuoteSigningScheme {
    type Error = String;

    fn try_from(wire: QuoteSigningSchemeWire) -> Result<Self, Self::Error> {
        let is_ecdsa = matches!(
            wire.signing_scheme,
            SigningScheme::Eip712 | SigningScheme::EthSign
        );
        match (
            wire.signing_scheme,
            wire.onchain_order,
            wire.verification_gas_limit,
        ) {
            (_, true, None) if is_ecdsa => Err("ECDSA-signed orders cannot be on-chain".to_owned()),
            (SigningScheme::Eip712, _, None) => Ok(Self::Eip712),
            (SigningScheme::EthSign, _, None) => Ok(Self::EthSign),
            (SigningScheme::Eip1271, onchain_order, verification_gas_limit) => Ok(Self::Eip1271 {
                onchain_order,
                verification_gas_limit: verification_gas_limit
                    .unwrap_or_else(default_verification_gas_limit),
            }),
            (SigningScheme::PreSign, onchain_order, None) => Ok(Self::PreSign { onchain_order }),
            (_, _, Some(_)) => {
                Err("only EIP-1271 quote signing supports a verificationGasLimit".to_owned())
            }
        }
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
    /// Validity window: an absolute `validTo` timestamp or a relative
    /// `validFor` duration in seconds, never both. Defaults to the protocol's
    /// 30-minute relative window.
    #[serde(flatten)]
    pub validity: QuoteValidity,
    /// App-data as the `(full document, hash)` pair, routed to a server-valid
    /// wire shape for every combination (see [`QuoteAppData`]).
    #[serde(flatten)]
    pub app_data: QuoteAppData,
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
    /// Signing scheme expected for the eventual order submission. The
    /// scheme-specific on-chain and verification-gas constraints are encoded by
    /// [`QuoteSigningScheme`].
    #[serde(flatten)]
    pub signing_scheme: QuoteSigningScheme,
    /// Optional request timeout override in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// Mutually exclusive buy-side or sell-side amount fields.
    #[serde(flatten)]
    pub side: OrderQuoteSide,
}

impl OrderQuoteRequest {
    /// Creates a quote request with stable orderbook defaults.
    ///
    /// No app-data is attached by default; the orderbook treats an omitted
    /// app-data field as the zero app-data hash. Attach a full app-data
    /// document with [`with_app_data`](Self::with_app_data), an explicit hash
    /// with [`with_app_data_hash`](Self::with_app_data_hash), or the document
    /// plus its expected hash by calling both setters. The signing scheme is
    /// EIP-712 and both token balances default to ERC-20 balances. The price
    /// quality defaults to [`PriceQuality::Optimal`], the mode used for a quote
    /// that will be signed and submitted: it returns a quote identifier for
    /// order placement.
    #[must_use]
    pub fn new(
        sell_token: Address,
        buy_token: Address,
        from: Address,
        side: OrderQuoteSide,
    ) -> Self {
        Self {
            sell_token,
            buy_token,
            receiver: None,
            validity: QuoteValidity::ValidFor(30 * 60),
            app_data: QuoteAppData::default(),
            partially_fillable: false,
            sell_token_balance: SellTokenSource::Erc20,
            buy_token_balance: BuyTokenDestination::Erc20,
            from,
            price_quality: PriceQuality::Optimal,
            signing_scheme: QuoteSigningScheme::Eip712,
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
    ///
    /// Sets the validity to [`QuoteValidity::ValidTo`], replacing any
    /// previously configured relative window.
    #[must_use]
    pub const fn with_valid_to(mut self, valid_to: u32) -> Self {
        self.validity = QuoteValidity::ValidTo(valid_to);
        self
    }

    /// Returns a copy of this request with a relative validity duration.
    ///
    /// Sets the validity to [`QuoteValidity::ValidFor`], replacing any
    /// previously configured absolute timestamp.
    #[must_use]
    pub const fn with_valid_for(mut self, valid_for: u32) -> Self {
        self.validity = QuoteValidity::ValidFor(valid_for);
        self
    }

    /// Returns a copy of this request carrying the full app-data document,
    /// replacing any previously set document.
    ///
    /// On its own this produces the document-only wire form
    /// (`{"appData": <document>}`). Followed by
    /// [`with_app_data_hash`](Self::with_app_data_hash) it produces the
    /// document-plus-hash form (`{"appData": <document>, "appDataHash": ...}`),
    /// pinning the expected hash of the document.
    #[must_use]
    pub fn with_app_data(mut self, app_data: impl Into<String>) -> Self {
        self.app_data.full = Some(app_data.into());
        self
    }

    /// Returns a copy of this request carrying an explicit app-data hash,
    /// replacing any previously set hash.
    ///
    /// On its own this produces the hash-only wire form: the hash travels under
    /// the `appData` key (the orderbook resolves it to the corresponding
    /// document), never as an `appDataHash`-only body that the orderbook
    /// rejects. Combined with [`with_app_data`](Self::with_app_data) it instead
    /// pins the expected hash of that document.
    #[must_use]
    pub const fn with_app_data_hash(mut self, app_data_hash: AppDataHash) -> Self {
        self.app_data.hash = Some(app_data_hash);
        self
    }

    /// Returns a copy of this request with a new quote-quality mode.
    #[must_use]
    pub const fn with_price_quality(mut self, quality: PriceQuality) -> Self {
        self.price_quality = quality;
        self
    }

    /// Returns a copy of this request with a new signing scheme.
    ///
    /// Maps the base [`SigningScheme`] onto the typed [`QuoteSigningScheme`],
    /// carrying any previously configured on-chain flag and verification gas
    /// limit onto the new scheme where they remain meaningful.
    #[must_use]
    pub const fn with_signing_scheme(mut self, scheme: SigningScheme) -> Self {
        let onchain_order = self.signing_scheme.is_onchain_order();
        let verification_gas_limit = match self.signing_scheme {
            QuoteSigningScheme::Eip1271 {
                verification_gas_limit,
                ..
            } => verification_gas_limit,
            _ => default_verification_gas_limit(),
        };
        self.signing_scheme = match scheme {
            SigningScheme::Eip712 => QuoteSigningScheme::Eip712,
            SigningScheme::EthSign => QuoteSigningScheme::EthSign,
            SigningScheme::Eip1271 => QuoteSigningScheme::Eip1271 {
                onchain_order,
                verification_gas_limit,
            },
            SigningScheme::PreSign => QuoteSigningScheme::PreSign { onchain_order },
        };
        self
    }

    /// Returns a copy of this request with an explicit typed signing scheme.
    #[must_use]
    pub const fn with_quote_signing_scheme(mut self, scheme: QuoteSigningScheme) -> Self {
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
    ///
    /// On-chain placement only applies to EIP-1271 and pre-sign schemes; on an
    /// ECDSA scheme (`eip712`/`ethSign`) the request stays off-chain because an
    /// ECDSA-signed order can never be on-chain.
    #[must_use]
    pub const fn with_onchain_order(mut self) -> Self {
        self.signing_scheme = match self.signing_scheme {
            QuoteSigningScheme::Eip1271 {
                verification_gas_limit,
                ..
            } => QuoteSigningScheme::Eip1271 {
                onchain_order: true,
                verification_gas_limit,
            },
            QuoteSigningScheme::PreSign { .. } => QuoteSigningScheme::PreSign {
                onchain_order: true,
            },
            ecdsa => ecdsa,
        };
        self
    }

    /// Returns a copy of this request with an explicit verification gas limit.
    ///
    /// The verification gas limit only applies to the EIP-1271 scheme; on any
    /// other scheme this is a no-op.
    #[must_use]
    pub const fn with_verification_gas_limit(mut self, verification_gas_limit: u64) -> Self {
        if let QuoteSigningScheme::Eip1271 { onchain_order, .. } = self.signing_scheme {
            self.signing_scheme = QuoteSigningScheme::Eip1271 {
                onchain_order,
                verification_gas_limit,
            };
        }
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
    pub const fn is_sell(&self) -> bool {
        self.side.is_sell()
    }

    /// Returns `true` when the embedded side is buy-driven.
    #[must_use]
    pub const fn is_buy(&self) -> bool {
        self.side.is_buy()
    }

    /// Returns `true` when the request passes local pre-dispatch validation.
    ///
    /// The quote side, validity window, and signing scheme are all mutually
    /// exclusive by construction (see [`OrderQuoteSide`], [`QuoteValidity`],
    /// and [`QuoteSigningScheme`]), so a constructed request is always
    /// well-formed.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Validates local quote preconditions before dispatching the HTTP request.
    ///
    /// Every quote-request invariant — exactly one side amount, exactly one
    /// validity form, the EIP-1271-only verification gas limit, and the
    /// ECDSA-cannot-be-on-chain rule — is enforced at the type level by
    /// [`OrderQuoteSide`], [`QuoteValidity`], and [`QuoteSigningScheme`], so a
    /// constructed request is always well-formed. This hook is retained for
    /// pre-dispatch validation and API stability.
    ///
    /// # Errors
    ///
    /// Currently infallible; retained as the fallible pre-dispatch hook.
    pub const fn validate(&self) -> Result<(), OrderbookError> {
        Ok(())
    }
}

/// Quote order data returned by the orderbook API.
///
/// This mirrors the orderbook `OrderParameters` schema — the order
/// parameters payload returned inside a `/quote` response — and is named
/// `QuoteData` for that role (see ADR 0058). It is a wire DTO, not the
/// user-domain signing order (`cow_sdk_core::OrderData`), which is also the
/// contract EIP-712 hashing input. It accepts the orderbook's full-app-data
/// echo shape and resolves that into the app-data hash used by downstream
/// order creation.
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
    /// Explicit app-data hash echoed alongside full app data, present only
    /// when the orderbook response carried both forms. Mirrors the optional
    /// `OrderParameters.appDataHash` wire field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_data_hash: Option<AppDataHash>,
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
    /// Estimated gas units for the quoted trade, in the upstream
    /// decimal-string wire shape. Read-only quote estimate populated from the
    /// orderbook `/quote` response (ADR 0021); empty for a locally constructed
    /// quote. Read through [`QuoteData::gas_amount`].
    #[serde(skip_serializing_if = "String::is_empty")]
    gas_amount: String,
    /// Estimated gas price at quote time (wei per gas unit), in the upstream
    /// decimal-string wire shape. Read-only quote estimate (ADR 0021); read
    /// through [`QuoteData::gas_price`].
    #[serde(skip_serializing_if = "String::is_empty")]
    gas_price: String,
    /// Sell-token price in native-token atoms per sell-token atom, in the
    /// upstream decimal-string wire shape. Read-only quote estimate
    /// (ADR 0021); read through [`QuoteData::sell_token_price`].
    #[serde(skip_serializing_if = "String::is_empty")]
    sell_token_price: String,
    /// Signing scheme for the quoted order. Mirrors
    /// `OrderParameters.signingScheme`, which defaults to `eip712`. Read-only
    /// quote field (ADR 0021); read through [`QuoteData::signing_scheme`].
    #[serde(default)]
    signing_scheme: SigningScheme,
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
            #[serde(default)]
            gas_amount: String,
            #[serde(default)]
            gas_price: String,
            #[serde(default)]
            sell_token_price: String,
            #[serde(default)]
            signing_scheme: SigningScheme,
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
            app_data_hash: wire.app_data_hash,
            fee_amount: wire.fee_amount,
            kind: wire.kind,
            partially_fillable: wire.partially_fillable,
            sell_token_balance: wire.sell_token_balance,
            buy_token_balance: wire.buy_token_balance,
            gas_amount: wire.gas_amount,
            gas_price: wire.gas_price,
            sell_token_price: wire.sell_token_price,
            signing_scheme: wire.signing_scheme,
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
            app_data_hash: None,
            fee_amount: Amount::ZERO,
            kind,
            partially_fillable: false,
            sell_token_balance: SellTokenSource::Erc20,
            buy_token_balance: BuyTokenDestination::Erc20,
            gas_amount: String::new(),
            gas_price: String::new(),
            sell_token_price: String::new(),
            signing_scheme: SigningScheme::Eip712,
        }
    }

    /// Returns the network-cost amount echoed by the orderbook `/quote`
    /// response.
    #[must_use]
    pub const fn network_cost_amount(&self) -> &Amount {
        &self.fee_amount
    }

    /// Returns the estimated gas units echoed by the orderbook `/quote`
    /// response, or an empty string for a locally constructed quote. This is a
    /// read-only quote estimate (ADR 0021).
    #[must_use]
    pub fn gas_amount(&self) -> &str {
        &self.gas_amount
    }

    /// Returns the estimated gas price (wei per gas unit) echoed by the
    /// orderbook `/quote` response. Read-only quote estimate (ADR 0021).
    #[must_use]
    pub fn gas_price(&self) -> &str {
        &self.gas_price
    }

    /// Returns the sell-token price (native-token atoms per sell-token atom)
    /// echoed by the orderbook `/quote` response. Read-only quote estimate
    /// (ADR 0021).
    #[must_use]
    pub fn sell_token_price(&self) -> &str {
        &self.sell_token_price
    }

    /// Returns the signing scheme for the quoted order. Defaults to `eip712`.
    /// Read-only quote field (ADR 0021).
    #[must_use]
    pub const fn signing_scheme(&self) -> SigningScheme {
        self.signing_scheme
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
    /// Quote price/fee expiry as the orderbook's ISO-8601 UTC string (for
    /// example `2026-04-28T10:00:00Z`), exposed losslessly.
    ///
    /// cow-rs intentionally takes no datetime dependency; parse this with your
    /// preferred datetime crate (`chrono::DateTime::parse_from_rfc3339`,
    /// `time::OffsetDateTime::parse`, ...) when a typed value is needed. This is
    /// when the quoted price and fee expire; the eventual order's validity is
    /// the [`QuoteData::valid_to`] UNIX epoch on `quote`.
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

/// Fails closed with [`OrderbookError::QuoteEchoMismatch`] when `echoes` is
/// false, naming the request-determined field that diverged.
fn require(
    field: QuoteEchoField,
    echoes: bool,
    expected: String,
    received: String,
) -> Result<(), OrderbookError> {
    if echoes {
        Ok(())
    } else {
        Err(OrderbookError::QuoteEchoMismatch {
            field,
            expected,
            received,
        })
    }
}

/// Resolves the address the trade proceeds settle to. An unset or zero receiver
/// pays the owner, mirroring the orderbook's own settlement rule, so the quote
/// receiver is reconciled against the request as the effective receiver rather
/// than by raw `Option` shape: owner-equivalent representations agree, and a
/// redirect to any other address fails closed.
fn effective_receiver(receiver: Option<Address>, owner: Address) -> Address {
    receiver
        .filter(|address| *address != Address::ZERO)
        .unwrap_or(owner)
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

    /// Verifies this `/quote` response echoes every request-determined field of
    /// `request` unchanged, returning [`OrderbookError::QuoteEchoMismatch`] on
    /// the first field the orderbook did not return as asked.
    ///
    /// [`OrderbookApi::quote`](crate::OrderbookApi::quote) calls this
    /// automatically, so a quote that would build an order the caller did not
    /// request never reaches a signing path; raw consumers holding a stored
    /// response can call it directly.
    ///
    /// The check is scoped to the fields the request *determines*. The quote's
    /// variable leg — the price the solver returns for the unfixed side — is the
    /// answer to the request and is never constrained.
    ///
    /// Checked: the sell/buy token pair, order kind, the owner `from` (when the
    /// response carries it), the partial-fill flag, both balance sources, the
    /// app-data hash (an explicit pin, the keccak digest of a full document, or
    /// the zero hash for an omitted pair), an absolute `validTo` (only the
    /// `validTo` validity form), the effective receiver (an unset or zero
    /// receiver resolves to the owner, so an owner-equivalent echo agrees and a
    /// redirect fails closed), and the fixed amount leg. The fixed-leg fold
    /// mirrors the services quote arithmetic: a `sellAmountBeforeFee` request
    /// holds `sellAmount + feeAmount == requested`, a `sellAmountAfterFee`
    /// request holds `sellAmount == requested`, and a buy request holds
    /// `buyAmount == requested`.
    ///
    /// Deliberately unchecked: the variable amount leg (the quote itself),
    /// `expiration` and a relative `validFor` (server-computed), the quote `id`,
    /// `verified`, the read-only gas estimate fields, and `protocolFeeBps`.
    ///
    /// # Errors
    ///
    /// Returns [`OrderbookError::QuoteEchoMismatch`] identifying the first
    /// request-determined field the response failed to echo.
    pub fn ensure_matches(&self, request: &OrderQuoteRequest) -> Result<(), OrderbookError> {
        let quote = &self.quote;

        require(
            QuoteEchoField::SellToken,
            request.sell_token == quote.sell_token,
            request.sell_token.to_string(),
            quote.sell_token.to_string(),
        )?;
        require(
            QuoteEchoField::BuyToken,
            request.buy_token == quote.buy_token,
            request.buy_token.to_string(),
            quote.buy_token.to_string(),
        )?;
        require(
            QuoteEchoField::Kind,
            request.side.kind() == quote.kind,
            format!("{:?}", request.side.kind()),
            format!("{:?}", quote.kind),
        )?;
        require(
            QuoteEchoField::PartiallyFillable,
            request.partially_fillable == quote.partially_fillable,
            request.partially_fillable.to_string(),
            quote.partially_fillable.to_string(),
        )?;
        require(
            QuoteEchoField::SellTokenBalance,
            request.sell_token_balance == quote.sell_token_balance,
            format!("{:?}", request.sell_token_balance),
            format!("{:?}", quote.sell_token_balance),
        )?;
        require(
            QuoteEchoField::BuyTokenBalance,
            request.buy_token_balance == quote.buy_token_balance,
            format!("{:?}", request.buy_token_balance),
            format!("{:?}", quote.buy_token_balance),
        )?;

        let requested_receiver = effective_receiver(request.receiver, request.from);
        let returned_receiver = effective_receiver(quote.receiver, request.from);
        require(
            QuoteEchoField::Receiver,
            requested_receiver == returned_receiver,
            requested_receiver.to_string(),
            returned_receiver.to_string(),
        )?;

        if let Some(returned) = self.from {
            require(
                QuoteEchoField::From,
                request.from == returned,
                request.from.to_string(),
                returned.to_string(),
            )?;
        }

        // The expected app-data hash is request-derivable for every form: an
        // explicit pin is the declared hash, a full document hashes as keccak256
        // of its bytes (the digest the services `Both` form echoes and the
        // upload precheck computes), and an omitted pair must echo the zero hash
        // (the services default). Reconciling it for every form binds the
        // app-data the order commits to, including on the raw pre-sign lane
        // where no signature re-derives it.
        let expected_app_data = request.app_data.hash.unwrap_or_else(|| {
            request
                .app_data
                .full
                .as_deref()
                .map_or(AppDataHash::ZERO, AppDataHash::from_full_app_data)
        });
        require(
            QuoteEchoField::AppDataHash,
            quote.app_data == expected_app_data,
            expected_app_data.to_string(),
            quote.app_data.to_string(),
        )?;

        if let QuoteValidity::ValidTo(valid_to) = request.validity {
            require(
                QuoteEchoField::ValidTo,
                quote.valid_to == valid_to,
                valid_to.to_string(),
                quote.valid_to.to_string(),
            )?;
        }

        self.ensure_fixed_leg(&request.side)
    }

    /// Checks the fixed amount leg against the response using the services quote
    /// arithmetic for the request's side basis: a sell request fixes the sell
    /// leg (before-fee folds the network cost back in, after-fee passes through),
    /// a buy request fixes the buy leg. The opposite leg is the quote and is left
    /// free.
    fn ensure_fixed_leg(&self, side: &OrderQuoteSide) -> Result<(), OrderbookError> {
        let quote = &self.quote;
        match side {
            OrderQuoteSide::Sell { sell_amount } => {
                let requested = *sell_amount.amount();
                let returned = match sell_amount {
                    SellAmount::BeforeFee { .. } => {
                        quote.sell_amount.checked_add(*quote.network_cost_amount())
                    }
                    SellAmount::AfterFee { .. } => Some(quote.sell_amount),
                };
                require(
                    QuoteEchoField::FixedSellAmount,
                    returned == Some(requested),
                    requested.to_string(),
                    returned.map_or_else(|| "overflow".to_owned(), |amount| amount.to_string()),
                )
            }
            OrderQuoteSide::Buy {
                buy_amount_after_fee,
            } => require(
                QuoteEchoField::FixedBuyAmount,
                quote.buy_amount == *buy_amount_after_fee,
                buy_amount_after_fee.to_string(),
                quote.buy_amount.to_string(),
            ),
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
