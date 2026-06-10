use serde::{Deserialize, Deserializer, Serialize, de::Error as DeError, ser::SerializeMap};

use super::{
    Address, Amount, AppDataHash, BuyTokenDestination, OrderKind, OrderUid, SellTokenSource,
    TransactionHash,
    enums::{EcdsaSigningScheme, OrderClass, OrderStatus, SigningScheme},
    quote::OrderQuoteResponse,
};

/// Orderbook order submission DTO.
///
/// This is kept separate from `QuoteData` because submission adds signature,
/// signer, signing-scheme, and optional quote-id fields while preserving the
/// orderbook wire shape expected by `/api/v1/orders`.
///
/// The Serialize impl is hand-rolled so the `(app_data, app_data_hash)`
/// pair routes onto the services `OrderCreationAppData` untagged-enum
/// shape. Services accepts three variants for app-data: `Both`
/// (`appData` is the full document string, `appDataHash` is the
/// explicit hash); `Hash` (the hash lives under the `appData` key —
/// no separate `appDataHash` field); and `Full` (`appData` is the
/// document string and services derives the hash). The cow pair maps
/// onto these variants per the table in the Serialize impl below.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderCreation {
    /// Sell-token address.
    pub sell_token: Address,
    /// Buy-token address.
    pub buy_token: Address,
    /// Optional receiver override.
    pub receiver: Option<Address>,
    /// Sell amount in the upstream decimal-string wire shape.
    pub sell_amount: Amount,
    /// Buy amount in the upstream decimal-string wire shape.
    pub buy_amount: Amount,
    /// Absolute UNIX expiry timestamp.
    pub valid_to: u32,
    /// Inline app-data payload when supplied instead of an app-data hash.
    pub app_data: Option<String>,
    /// App-data hash for the submission payload.
    pub app_data_hash: Option<AppDataHash>,
    /// Order-level fee hardcoded to `"0"` on every submission.
    ///
    /// The cow-protocol services backend rejects orders that carry a
    /// non-zero order-level fee (`NonZeroFee`), so the submission path
    /// always wires this component as `"0"` and preserves the EIP-712
    /// struct-hash contract that hashes it as `uint256(0)`.
    fee_amount: Amount,
    /// Opt-in strict balance check flag accepted by the orderbook services.
    full_balance_check: bool,
    /// Order kind.
    pub kind: OrderKind,
    /// Whether partial fills are allowed.
    pub partially_fillable: bool,
    /// Sell-token balance source.
    pub sell_token_balance: SellTokenSource,
    /// Buy-token balance destination.
    pub buy_token_balance: BuyTokenDestination,
    /// Signature scheme used for `signature`.
    pub signing_scheme: SigningScheme,
    /// Raw signature string encoded for the upstream API.
    pub signature: String,
    /// Effective order owner.
    pub from: Address,
    /// Optional quote id from a prior quote response.
    pub quote_id: Option<i64>,
}

impl Serialize for OrderCreation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Hand-rolled wire shape so the `(app_data, app_data_hash)` pair
        // routes onto the services `OrderCreationAppData` untagged-enum
        // variants. The mapping is:
        //
        // | (app_data, app_data_hash)   | wire shape                                                     | services variant matched |
        // | (None, None)                | (both fields omitted; services rejects — programmer error)     | none                     |
        // | (Some(s), None)             | `{"appData": s}` (s is the JSON-encoded app-data document)     | `Full`                   |
        // | (None, Some(h))             | `{"appData": "0x<h hex>"}` (hash lives under the appData key)  | `Hash`                   |
        // | (Some(s), Some(h))          | `{"appData": s, "appDataHash": "0x<h hex>"}`                   | `Both`                   |
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("sellToken", &self.sell_token)?;
        map.serialize_entry("buyToken", &self.buy_token)?;
        if let Some(receiver) = self.receiver.as_ref() {
            map.serialize_entry("receiver", receiver)?;
        }
        map.serialize_entry("sellAmount", &self.sell_amount)?;
        map.serialize_entry("buyAmount", &self.buy_amount)?;
        map.serialize_entry("validTo", &self.valid_to)?;
        super::app_data::serialize_app_data_pair(
            &mut map,
            self.app_data.as_deref(),
            self.app_data_hash.as_ref(),
        )?;
        map.serialize_entry("feeAmount", &self.fee_amount)?;
        if self.full_balance_check {
            map.serialize_entry("fullBalanceCheck", &self.full_balance_check)?;
        }
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("partiallyFillable", &self.partially_fillable)?;
        map.serialize_entry("sellTokenBalance", &self.sell_token_balance)?;
        map.serialize_entry("buyTokenBalance", &self.buy_token_balance)?;
        map.serialize_entry("signingScheme", &self.signing_scheme)?;
        map.serialize_entry("signature", &self.signature)?;
        map.serialize_entry("from", &self.from)?;
        if let Some(quote_id) = self.quote_id.as_ref() {
            map.serialize_entry("quoteId", quote_id)?;
        }
        map.end()
    }
}

const fn order_creation_zero_fee_amount() -> Amount {
    Amount::ZERO
}

const ORDER_CREATION_NON_ZERO_FEE_ERROR: &str =
    "non-zero feeAmount is not accepted for OrderCreation";

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde skip_serializing_if predicates receive a field reference"
)]
const fn is_false(value: &bool) -> bool {
    !*value
}

impl<'de> Deserialize<'de> for OrderCreation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct OrderCreationWire {
            sell_token: Address,
            buy_token: Address,
            receiver: Option<Address>,
            sell_amount: Amount,
            buy_amount: Amount,
            valid_to: u32,
            app_data: Option<String>,
            app_data_hash: Option<AppDataHash>,
            #[serde(default = "order_creation_zero_fee_amount")]
            fee_amount: Amount,
            #[serde(default)]
            full_balance_check: bool,
            kind: OrderKind,
            #[serde(default)]
            partially_fillable: bool,
            #[serde(default)]
            sell_token_balance: SellTokenSource,
            #[serde(default)]
            buy_token_balance: BuyTokenDestination,
            #[serde(default)]
            signing_scheme: SigningScheme,
            signature: String,
            from: Address,
            quote_id: Option<i64>,
        }

        let wire = OrderCreationWire::deserialize(deserializer)?;
        if !wire.fee_amount.is_zero() {
            return Err(D::Error::custom(ORDER_CREATION_NON_ZERO_FEE_ERROR));
        }

        Ok(Self {
            sell_token: wire.sell_token,
            buy_token: wire.buy_token,
            receiver: wire.receiver,
            sell_amount: wire.sell_amount,
            buy_amount: wire.buy_amount,
            valid_to: wire.valid_to,
            app_data: wire.app_data,
            app_data_hash: wire.app_data_hash,
            fee_amount: wire.fee_amount,
            full_balance_check: wire.full_balance_check,
            kind: wire.kind,
            partially_fillable: wire.partially_fillable,
            sell_token_balance: wire.sell_token_balance,
            buy_token_balance: wire.buy_token_balance,
            signing_scheme: wire.signing_scheme,
            signature: wire.signature,
            from: wire.from,
            quote_id: wire.quote_id,
        })
    }
}

impl OrderCreation {
    /// Creates an order-submission payload with the required trade fields.
    ///
    /// Optional and defaulted fields (app-data, balance sources,
    /// partial-fill, receiver, quote id) can be attached through the
    /// `with_*` setters. The order-level fee is always wired as `"0"`
    /// to satisfy the services `NonZeroFee` constraint and the EIP-712
    /// struct-hash contract.
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "constructor mirrors the public field set so callers can migrate off struct-literal construction without losing explicit control over any wire field"
    )]
    pub fn new(
        sell_token: Address,
        buy_token: Address,
        sell_amount: Amount,
        buy_amount: Amount,
        valid_to: u32,
        kind: OrderKind,
        signing_scheme: SigningScheme,
        signature: impl Into<String>,
        from: Address,
    ) -> Self {
        Self {
            sell_token,
            buy_token,
            receiver: None,
            sell_amount,
            buy_amount,
            valid_to,
            app_data: None,
            app_data_hash: None,
            fee_amount: order_creation_zero_fee_amount(),
            full_balance_check: false,
            kind,
            partially_fillable: false,
            sell_token_balance: SellTokenSource::Erc20,
            buy_token_balance: BuyTokenDestination::Erc20,
            signing_scheme,
            signature: signature.into(),
            from,
            quote_id: None,
        }
    }

    /// Creates an order-submission payload from a quote response.
    ///
    /// Copies the economic fields from the resolved quote and threads the
    /// response's quote id straight onto the payload, so the submission
    /// settles against the quote the user actually approved rather than a
    /// fresh server-side rebind. The order-level fee is always wired as `"0"`
    /// on submission; the network-cost component returned on the quote
    /// response does not round-trip into the signed order.
    #[must_use]
    pub fn from_quote(
        response: &OrderQuoteResponse,
        from: Address,
        receiver: Option<Address>,
        signing_scheme: SigningScheme,
        signature: impl Into<String>,
    ) -> Self {
        let quote = &response.quote;
        Self {
            sell_token: quote.sell_token,
            buy_token: quote.buy_token,
            receiver: receiver.or(quote.receiver),
            sell_amount: quote.sell_amount,
            buy_amount: quote.buy_amount,
            valid_to: quote.valid_to,
            app_data: None,
            app_data_hash: Some(quote.app_data),
            fee_amount: order_creation_zero_fee_amount(),
            full_balance_check: false,
            kind: quote.kind,
            partially_fillable: quote.partially_fillable,
            sell_token_balance: quote.sell_token_balance,
            buy_token_balance: quote.buy_token_balance,
            signing_scheme,
            signature: signature.into(),
            from,
            quote_id: response.id,
        }
    }

    /// Creates a `presign` order-submission payload from a quote response.
    ///
    /// Pre-sign placements carry no cryptographic signature: the orderbook
    /// accepts an empty signature for the `presign` scheme, and the order
    /// only becomes fillable once the owner activates the on-chain
    /// pre-signature flag on the settlement contract (`setPreSignature`).
    /// This is the smart-contract-owner path (vaults, DAOs, protocol
    /// treasuries) where the signing identity is the contract itself.
    #[must_use]
    pub fn presign_from_quote(
        response: &OrderQuoteResponse,
        from: Address,
        receiver: Option<Address>,
    ) -> Self {
        Self::from_quote(response, from, receiver, SigningScheme::PreSign, "")
    }

    /// Creates a submission payload from a signed user-domain order.
    ///
    /// This is the canonical `OrderData` to `OrderCreation` conversion: it
    /// copies every signed economic field from `order_to_sign` verbatim so the
    /// submitted body cannot drift from what was signed, wires the order-level
    /// fee as `"0"` (services rejects a non-zero `feeAmount`), and attaches the
    /// signature, signing scheme, owner (`from`), the optional full app-data
    /// document, and the optional quote id. The signing input it mirrors is
    /// [`cow_sdk_core::OrderData`], which is also the EIP-712 hashing view of
    /// the same order.
    ///
    /// The wire `appDataHash` is taken from `order_to_sign.app_data` — the hash
    /// the order was signed against — so the submitted hash cannot diverge from
    /// the signed commitment.
    #[must_use]
    pub fn from_signed(
        order_to_sign: &cow_sdk_core::OrderData,
        signing_scheme: SigningScheme,
        signature: impl Into<String>,
        from: Address,
        app_data_document: Option<String>,
        quote_id: Option<i64>,
    ) -> Self {
        Self {
            sell_token: order_to_sign.sell_token,
            buy_token: order_to_sign.buy_token,
            receiver: Some(order_to_sign.receiver),
            sell_amount: order_to_sign.sell_amount,
            buy_amount: order_to_sign.buy_amount,
            valid_to: order_to_sign.valid_to,
            app_data: app_data_document,
            app_data_hash: Some(order_to_sign.app_data),
            fee_amount: order_creation_zero_fee_amount(),
            full_balance_check: false,
            kind: order_to_sign.kind,
            partially_fillable: order_to_sign.partially_fillable,
            sell_token_balance: order_to_sign.sell_token_balance,
            buy_token_balance: order_to_sign.buy_token_balance,
            signing_scheme,
            signature: signature.into(),
            from,
            quote_id,
        }
    }

    /// Returns a copy of this submission payload with an explicit receiver.
    #[must_use]
    pub const fn with_receiver(mut self, receiver: Address) -> Self {
        self.receiver = Some(receiver);
        self
    }

    /// Returns a copy of this submission payload with inline app-data content.
    #[must_use]
    pub fn with_app_data(mut self, app_data: impl Into<String>) -> Self {
        self.app_data = Some(app_data.into());
        self
    }

    /// Returns a copy of this submission payload with an explicit app-data hash.
    #[must_use]
    pub const fn with_app_data_hash(mut self, app_data_hash: AppDataHash) -> Self {
        self.app_data_hash = Some(app_data_hash);
        self
    }

    /// Returns a copy of this submission payload marked as partially fillable.
    #[must_use]
    pub const fn with_partially_fillable(mut self, partially_fillable: bool) -> Self {
        self.partially_fillable = partially_fillable;
        self
    }

    /// Returns a copy of this submission payload with the strict full-balance check flag.
    #[must_use]
    pub const fn with_full_balance_check(mut self, full_balance_check: bool) -> Self {
        self.full_balance_check = full_balance_check;
        self
    }

    /// Returns a copy of this submission payload with an explicit sell-token balance source.
    #[must_use]
    pub const fn with_sell_token_balance(mut self, balance: SellTokenSource) -> Self {
        self.sell_token_balance = balance;
        self
    }

    /// Returns a copy of this submission payload with an explicit buy-token balance destination.
    #[must_use]
    pub const fn with_buy_token_balance(mut self, balance: BuyTokenDestination) -> Self {
        self.buy_token_balance = balance;
        self
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

impl OrderCancellations {
    /// Creates a cancellation payload from the supplied order UIDs and signature.
    ///
    /// Defaults to the `Eip712` ECDSA signing scheme; use
    /// [`with_signing_scheme`](Self::with_signing_scheme) to override.
    #[must_use]
    pub fn new(order_uids: Vec<OrderUid>, signature: impl Into<String>) -> Self {
        Self {
            order_uids,
            signature: signature.into(),
            signing_scheme: EcdsaSigningScheme::Eip712,
        }
    }

    /// Returns a copy of this payload carrying a different signing scheme.
    #[must_use]
    pub const fn with_signing_scheme(mut self, scheme: EcdsaSigningScheme) -> Self {
        self.signing_scheme = scheme;
        self
    }
}

/// `EthFlow`-specific orderbook metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EthflowData {
    /// Transaction in which the order was refunded, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_tx_hash: Option<TransactionHash>,
    /// User-facing validity timestamp for the `EthFlow` order.
    pub user_valid_to: u32,
}

impl EthflowData {
    /// Creates an `EthFlow` metadata record for the given user validity timestamp.
    #[must_use]
    pub const fn new(user_valid_to: u32) -> Self {
        Self {
            refund_tx_hash: None,
            user_valid_to,
        }
    }

    /// Returns a copy carrying an explicit refund-transaction hash.
    #[must_use]
    pub const fn with_refund_tx_hash(mut self, tx_hash: TransactionHash) -> Self {
        self.refund_tx_hash = Some(tx_hash);
        self
    }
}

/// On-chain order placement metadata returned by the orderbook for orders that
/// originated from an on-chain submission path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OnchainOrderData {
    /// Sender address associated with the on-chain placement.
    pub sender: Address,
    /// Placement error emitted by services, when on-chain placement failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement_error: Option<String>,
}

impl OnchainOrderData {
    /// Creates on-chain order metadata for the required sender address.
    #[must_use]
    pub const fn new(sender: Address) -> Self {
        Self {
            sender,
            placement_error: None,
        }
    }

    /// Returns a copy carrying the placement error reported by services.
    #[must_use]
    pub fn with_placement_error(mut self, placement_error: impl Into<String>) -> Self {
        self.placement_error = Some(placement_error.into());
        self
    }
}

/// Smart-contract interaction payload used by order pre and post hooks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractionData {
    /// Contract address targeted by the interaction.
    pub target: Address,
    /// Native token value sent with the interaction.
    pub value: Amount,
    /// Hex-encoded calldata forwarded to `target`.
    pub call_data: String,
}

impl InteractionData {
    /// Creates an interaction payload from its required wire fields.
    #[must_use]
    pub fn new(target: Address, value: Amount, call_data: impl Into<String>) -> Self {
        Self {
            target,
            value,
            call_data: call_data.into(),
        }
    }
}

/// Optional pre and post interactions attached to an order response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OrderInteractions {
    /// Interactions executed before the order's trade.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre: Option<Vec<InteractionData>>,
    /// Interactions executed after the order's trade.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post: Option<Vec<InteractionData>>,
}

impl OrderInteractions {
    /// Creates an empty interaction envelope.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy carrying pre-trade interactions.
    #[must_use]
    pub fn with_pre(mut self, pre: Vec<InteractionData>) -> Self {
        self.pre = Some(pre);
        self
    }

    /// Returns a copy carrying post-trade interactions.
    #[must_use]
    pub fn with_post(mut self, post: Vec<InteractionData>) -> Self {
        self.post = Some(post);
        self
    }
}

/// Quote metadata stored with an order response when an order was created from
/// a quote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct StoredOrderQuote {
    /// Estimated gas units required to execute the quoted trade.
    pub gas_amount: String,
    /// Estimated gas price at quote time, in wei per gas unit.
    pub gas_price: String,
    /// Sell-token price in native-token atoms per sell-token atom.
    pub sell_token_price: String,
    /// Quoted sell amount.
    pub sell_amount: Amount,
    /// Quoted buy amount.
    pub buy_amount: Amount,
    /// Estimated network fee in sell-token atoms.
    pub fee_amount: Amount,
    /// Solver address that provided the quote.
    pub solver: Address,
    /// Whether the quote was verified through simulation.
    pub verified: bool,
    /// Additional services-provided quote metadata, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl StoredOrderQuote {
    /// Creates stored quote metadata from every required `OpenAPI` field.
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "constructor mirrors the public field set so callers can migrate off struct-literal construction without losing explicit control over any wire field"
    )]
    pub fn new(
        gas_amount: impl Into<String>,
        gas_price: impl Into<String>,
        sell_token_price: impl Into<String>,
        sell_amount: Amount,
        buy_amount: Amount,
        fee_amount: Amount,
        solver: Address,
        verified: bool,
    ) -> Self {
        Self {
            gas_amount: gas_amount.into(),
            gas_price: gas_price.into(),
            sell_token_price: sell_token_price.into(),
            sell_amount,
            buy_amount,
            fee_amount,
            solver,
            verified,
            metadata: None,
        }
    }

    /// Returns a copy carrying services-provided quote metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Opaque protocol-fee policy descriptor returned on trade records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FeePolicy(pub serde_json::Value);

/// Executed protocol-fee metadata returned on trade records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ExecutedProtocolFee {
    /// Fee policy that produced this fee, when services returns it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy: Option<FeePolicy>,
    /// Fee amount taken.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<Amount>,
    /// Token in which the fee was taken.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<Address>,
}

impl ExecutedProtocolFee {
    /// Creates an empty executed protocol-fee payload.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy carrying the fee policy.
    #[must_use]
    pub fn with_policy(mut self, policy: FeePolicy) -> Self {
        self.policy = Some(policy);
        self
    }

    /// Returns a copy carrying the fee amount.
    #[must_use]
    pub const fn with_amount(mut self, amount: Amount) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Returns a copy carrying the fee token.
    #[must_use]
    pub const fn with_token(mut self, token: Address) -> Self {
        self.token = Some(token);
        self
    }
}

/// Orderbook order response DTO.
///
/// This response includes status, owner, uid, execution totals, and `EthFlow`
/// metadata that are not part of the user-domain signing order or contract ABI
/// hashing payload. It is one of two order-shaped types: the signing and
/// EIP-712 hashing pivot is `cow_sdk_core::OrderData`, and this is the
/// orderbook record. Use [`Order::signing_order`] to project a fetched order
/// back into the `cow_sdk_core::OrderData` for client-side digest or UID
/// re-derivation; it fails closed for `EthFlow` orders, whose response fields
/// are rewritten for display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Order {
    /// Sell-token address.
    pub sell_token: Address,
    /// Buy-token address.
    pub buy_token: Address,
    /// Optional receiver override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    /// Sell amount in the upstream decimal-string wire shape.
    pub sell_amount: Amount,
    /// Buy amount in the upstream decimal-string wire shape.
    pub buy_amount: Amount,
    /// Absolute UNIX expiry timestamp.
    pub valid_to: u32,
    /// App-data hash attached to the order.
    pub app_data: AppDataHash,
    /// Optional app-data hash echoed for debugging by the orderbook.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_data_hash: Option<AppDataHash>,
    /// Order-level fee echoed on the orderbook response; always `"0"` in
    /// practice because services rejects non-zero order-level fees.
    ///
    /// Stored under the upstream wire name `feeAmount` so deserialization
    /// preserves services-schema parity; the value is not exposed on the
    /// public Rust surface.
    #[serde(default = "order_creation_zero_fee_amount")]
    fee_amount: Amount,
    /// Strict balance-check flag accepted by services when the order was created.
    #[serde(default, skip_serializing_if = "is_false")]
    pub full_balance_check: bool,
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
    /// Signature scheme used for `signature`.
    #[serde(default)]
    pub signing_scheme: SigningScheme,
    /// Raw signature string.
    pub signature: String,
    /// Effective owner field returned by the API, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    /// Quote id used when the order originated from a quote.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<i64>,
    /// Order class.
    #[serde(default)]
    pub class: OrderClass,
    /// Canonical owner surfaced by the orderbook response.
    pub owner: Address,
    /// Order UID.
    pub uid: OrderUid,
    /// Creation timestamp string returned by the API.
    #[serde(default, alias = "creationTime")]
    pub creation_date: String,
    /// Executed sell amount.
    #[serde(default)]
    pub executed_sell_amount: Amount,
    /// Executed sell amount before fees.
    #[serde(default)]
    pub executed_sell_amount_before_fees: Amount,
    /// Executed buy amount.
    #[serde(default)]
    pub executed_buy_amount: Amount,
    /// Executed fee component, when provided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executed_fee: Option<Amount>,
    /// Deprecated legacy fee value some orderbook responses still emit on
    /// older order payloads alongside [`executed_fee`].
    ///
    /// Surfaced as a read-only sibling so consumers that need the legacy
    /// summation can compute it explicitly as
    /// `executed_fee + executed_fee_amount`. New code should prefer
    /// [`executed_fee`]; [`total_fee`] intentionally does not fold this
    /// field in.
    ///
    /// [`executed_fee`]: Order::executed_fee
    /// [`total_fee`]: Order::total_fee
    #[serde(default, skip_serializing_if = "Amount::is_zero")]
    pub executed_fee_amount: Amount,
    /// Token in which the executed fee was captured, when returned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executed_fee_token: Option<Address>,
    /// Whether the order was invalidated by the protocol.
    #[serde(default)]
    pub invalidated: bool,
    /// Order lifecycle status.
    #[serde(default)]
    pub status: OrderStatus,
    /// Whether services classified the order as a liquidity order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_liquidity_order: Option<bool>,
    /// On-chain user for `EthFlow`-style orders.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub onchain_user: Option<Address>,
    /// `EthFlow`-specific metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ethflow_data: Option<EthflowData>,
    /// On-chain placement metadata, when services returns it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub onchain_order_data: Option<OnchainOrderData>,
    /// Full app-data payload, when services returns it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_app_data: Option<String>,
    /// Settlement contract address against which the order was signed.
    pub settlement_contract: Address,
    /// Stored quote metadata for quote-linked orders.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<StoredOrderQuote>,
    /// Optional pre and post interactions associated with the order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interactions: Option<OrderInteractions>,
    /// Total fee normalized by the SDK transform layer.
    #[serde(default)]
    pub total_fee: Amount,
}

impl Order {
    /// Creates an orderbook order DTO with the minimal identity fields.
    ///
    /// Remaining response fields default to zero/empty; consumers that hand
    /// craft an `Order` for tests or fixtures set additional state through
    /// direct field access on the returned instance.
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "constructor mirrors the public field set so callers can migrate off struct-literal construction without losing explicit control over any wire field"
    )]
    pub fn new(
        sell_token: Address,
        buy_token: Address,
        sell_amount: Amount,
        buy_amount: Amount,
        valid_to: u32,
        app_data: AppDataHash,
        kind: OrderKind,
        signature: impl Into<String>,
        settlement_contract: Address,
        owner: Address,
        uid: OrderUid,
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
            fee_amount: order_creation_zero_fee_amount(),
            full_balance_check: false,
            kind,
            partially_fillable: false,
            sell_token_balance: SellTokenSource::Erc20,
            buy_token_balance: BuyTokenDestination::Erc20,
            signing_scheme: SigningScheme::Eip712,
            signature: signature.into(),
            from: None,
            quote_id: None,
            class: OrderClass::default(),
            owner,
            uid,
            creation_date: String::new(),
            executed_sell_amount: Amount::ZERO,
            executed_sell_amount_before_fees: Amount::ZERO,
            executed_buy_amount: Amount::ZERO,
            executed_fee: None,
            executed_fee_amount: Amount::ZERO,
            executed_fee_token: None,
            invalidated: false,
            status: OrderStatus::default(),
            is_liquidity_order: None,
            onchain_user: None,
            ethflow_data: None,
            onchain_order_data: None,
            full_app_data: None,
            settlement_contract,
            quote: None,
            interactions: None,
            total_fee: Amount::ZERO,
        }
    }

    /// Projects this response order back into the `cow_sdk_core::OrderData`
    /// used for EIP-712 hashing and UID re-derivation, when the response still
    /// reflects the signed order.
    ///
    /// Returns `Some` for ordinary orders: their response fields equal the
    /// signed order, so `cow_sdk_contracts::hash_order` over the result
    /// reproduces the stored `uid`. The response's optional receiver is mapped
    /// `None -> Address::ZERO` — the "receiver same as owner" sentinel the order
    /// was signed against — so the digest re-derives byte-for-byte.
    ///
    /// Returns `None` for `EthFlow` orders. The orderbook response transform
    /// rewrites `valid_to`, `owner`, and `sell_token` to the user-facing view,
    /// while the on-chain order is signed against the wrapped native token with
    /// `validTo = u32::MAX` and the `EthFlow` contract as owner. A projected
    /// order therefore cannot reproduce the on-chain digest or UID, so this
    /// fails closed rather than returning a silently wrong hashing input.
    /// Detect this case directly via `self.ethflow_data`.
    #[must_use]
    pub fn signing_order(&self) -> Option<cow_sdk_core::OrderData> {
        if self.ethflow_data.is_some() {
            return None;
        }
        Some(cow_sdk_core::OrderData::new(
            self.sell_token,
            self.buy_token,
            self.receiver.unwrap_or(Address::ZERO),
            self.sell_amount,
            self.buy_amount,
            self.valid_to,
            self.app_data,
            self.fee_amount,
            self.kind,
            self.partially_fillable,
            self.sell_token_balance,
            self.buy_token_balance,
        ))
    }
}
