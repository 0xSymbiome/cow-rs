//! Typed client-side validator for every public trading submission seam.
//!
//! The validator is client-side defence-in-depth for orders before
//! submission. It enforces only invariants that are stable and independent of
//! the orderbook operator's configuration: a present owner, a `valid_to` that
//! is still in the future, a non-native sell token outside the eth-flow path,
//! the buy-side same-token rule under the services `AllowSell` policy, non-zero
//! amounts, and agreement between the app-data signer and the submission owner.
//! It is not a replacement for the broader services-side rejection set.
//! Services remains authoritative for everything that depends on its own state
//! or configuration — deny-list, transferability, gas budget, banned-users,
//! balances, the exact order-validity window, and quote/price checks — so a
//! passing local validation does not guarantee services will accept the order.
//!
//! The public entry point is [`OrderBoundsValidator::validate`]. The helper
//! is pure: `now` is a caller-supplied UNIX-seconds timestamp and no
//! `SystemTime::now` is read inside the validator, so every observation is
//! deterministic and reproducible across replays.

use cow_sdk_core::{
    Address, Amount, NATIVE_CURRENCY_ADDRESS, OrderData, OrderKind, SupportedChainId,
};
use serde::{Deserialize, Serialize};

/// Typed client-side rejection variants produced by
/// [`OrderBoundsValidator::validate`] and offline trade-parameter validation.
///
/// The enum is `#[non_exhaustive]` so future additions to the reviewed
/// rejection surface may be introduced as a minor change without breaking
/// downstream exhaustive matches. The order-bounds variants reflect
/// conditions the reviewed services validator enforces so client-side
/// rejection fires before any bytes cross the wire; parameter-level variants
/// cover SDK policy preconditions enforced before app-data construction.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ClientRejection {
    /// `valid_to` is at or before `now`, so the order is already expired.
    ///
    /// This is the only validity-window check the client enforces: the exact
    /// minimum and maximum lifetimes are orderbook-operator configuration, so
    /// the SDK leaves them to services and pre-checks only the stable invariant
    /// that an order must not already be expired when it is signed.
    #[error("validTo is in the past: valid_to={valid_to}, now={now}")]
    ValidToInPast {
        /// Submitted `valid_to` timestamp.
        valid_to: u64,
        /// Caller-supplied current UNIX-seconds timestamp.
        now: u64,
    },
    /// The submitted owner address (`from`) is the zero address.
    #[error("missing from: order.from must not be the zero address")]
    MissingFrom,
    /// App-data `metadata.signer` disagrees with the submitted owner (`from`).
    #[error("appdata-from mismatch: metadata.signer={appdata_signer}, order.from={from}")]
    AppdataFromMismatch {
        /// Declared signer carried inside `metadata.signer`.
        appdata_signer: Address,
        /// Owner address (`from`) submitted alongside the order.
        from: Address,
    },
    /// A buy-side order's sell and buy tokens collapse to the same address
    /// after native-sentinel resolution.
    #[error("same buy and sell token: {token}")]
    SameBuyAndSellToken {
        /// Address both `sell_token` and `buy_token` resolve to.
        token: Address,
    },
    /// A non-EthFlow submission path carries the native-currency sentinel as
    /// `sell_token`.
    #[error(
        "invalid native sell token: use the eth-flow submission path for native-currency sells"
    )]
    InvalidNativeSellToken,
    /// One of the trade amounts is zero.
    #[error("zero amount on {side:?} side")]
    ZeroAmount {
        /// Side that carries the zero amount.
        side: AmountSide,
    },
    /// The signer-recovered owner does not match the submitted `from`.
    #[error("owner mismatch: expected owner={expected}, recovered signer={recovered}")]
    OwnerMismatch {
        /// Owner address submitted alongside the order.
        expected: Address,
        /// Owner address recovered from the signing backend.
        recovered: Address,
    },
    /// Partner-fee metadata failed the app-data policy preconditions.
    #[error("invalid partner-fee field `{field}`: {reason}")]
    InvalidPartnerFee {
        /// Public field name that failed validation.
        field: &'static str,
        /// Canonical validation-failure mode.
        reason: cow_sdk_core::ValidationReason,
    },
}

/// Discriminator for amount-side rejections.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AmountSide {
    /// Sell-side amount.
    Sell,
    /// Buy-side amount.
    Buy,
}

/// Pure client-side validator that enforces the reviewed services protocol
/// invariants on a signing order ([`cow_sdk_core::OrderData`]) and its
/// submission owner before the order reaches the orderbook.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderBoundsValidator {
    weth_address: Option<Address>,
}

impl OrderBoundsValidator {
    /// Creates a validator with no chain-specific wrapped-native address.
    #[must_use]
    pub const fn services_default() -> Self {
        Self { weth_address: None }
    }

    /// Creates a validator with the chain-specific wrapped-native token address
    /// attached for the same-token paired guard.
    #[must_use]
    pub fn services_default_for_chain(chain_id: SupportedChainId) -> Self {
        Self::services_default()
            .with_weth_address(cow_sdk_core::wrapped_native_token(chain_id).address)
    }

    /// Returns a copy of this validator configured with the chain-specific
    /// wrapped-native token address. When supplied, the validator rejects
    /// `sell_token == weth_address` paired with `buy_token == BUY_ETH_ADDRESS`
    /// through [`ClientRejection::SameBuyAndSellToken`] on buy-side orders
    /// to mirror the services `AllowSell` token-pair guard (buy-side rejected,
    /// sell-side accepted).
    #[must_use]
    pub const fn with_weth_address(mut self, weth_address: Address) -> Self {
        self.weth_address = Some(weth_address);
        self
    }

    /// Validates the supplied signing order ([`cow_sdk_core::OrderData`])
    /// and its submission owner against the reviewed protocol-invariant matrix.
    ///
    /// `from` is the submission owner — the order owner for an ECDSA-signed
    /// order, or the on-chain user for an `EthFlow` order — and is the address
    /// the owner-presence and app-data-signer checks run against. It is threaded
    /// separately because the canonical signing order carries no owner field.
    ///
    /// The helper is pure — `now` is a caller-supplied UNIX-seconds
    /// timestamp and no `SystemTime::now` is read inside the validator.
    ///
    /// `app_data_signer` is the typed `metadata.signer` field extracted
    /// from the app-data payload; pass `None` when the payload has no
    /// declared signer.
    ///
    /// `is_eth_flow` opts into the eth-flow submission-path defence-in-depth
    /// coverage: the native-currency-sentinel sell-token check is skipped
    /// (the sentinel is expected on that path), while every other invariant
    /// (expired `valid_to`, zero amount, buy-side same token, app-data signer)
    /// still runs.
    ///
    /// # Errors
    ///
    /// Returns [`ClientRejection`] on the first invariant violation so the
    /// caller can surface a typed error before any HTTP transport runs.
    pub fn validate(
        &self,
        order: &OrderData,
        from: Address,
        app_data_signer: Option<Address>,
        now: u64,
        is_eth_flow: bool,
    ) -> Result<(), ClientRejection> {
        if from.is_zero() {
            return Err(ClientRejection::MissingFrom);
        }

        let valid_to = u64::from(order.valid_to);
        if valid_to <= now {
            return Err(ClientRejection::ValidToInPast { valid_to, now });
        }

        self.validate_token_bounds(&order.sell_token, &order.buy_token, order.kind, is_eth_flow)?;

        validate_amount("sellAmount", AmountSide::Sell, &order.sell_amount)?;
        validate_amount("buyAmount", AmountSide::Buy, &order.buy_amount)?;

        if let Some(appdata_signer) = app_data_signer
            && appdata_signer != from
        {
            return Err(ClientRejection::AppdataFromMismatch {
                appdata_signer,
                from,
            });
        }

        Ok(())
    }

    fn validate_token_bounds(
        &self,
        sell_token: &Address,
        buy_token: &Address,
        kind: OrderKind,
        is_eth_flow: bool,
    ) -> Result<(), ClientRejection> {
        if !is_eth_flow && *sell_token == NATIVE_CURRENCY_ADDRESS {
            return Err(ClientRejection::InvalidNativeSellToken);
        }
        if sell_token == buy_token && kind == OrderKind::Buy {
            return Err(ClientRejection::SameBuyAndSellToken { token: *sell_token });
        }
        if let Some(weth) = self.weth_address.as_ref()
            && sell_token == weth
            && *buy_token == NATIVE_CURRENCY_ADDRESS
            && kind == OrderKind::Buy
        {
            return Err(ClientRejection::SameBuyAndSellToken { token: *weth });
        }
        Ok(())
    }
}

impl Default for OrderBoundsValidator {
    fn default() -> Self {
        Self::services_default()
    }
}

fn validate_amount(
    _field: &'static str,
    side: AmountSide,
    value: &Amount,
) -> Result<(), ClientRejection> {
    if value.is_zero() {
        return Err(ClientRejection::ZeroAmount { side });
    }
    Ok(())
}

/// Asserts the recovered signer matches the expected owner.
///
/// Compares an owner and a recovered signer for the reviewed
/// recoverable-signature owner check and returns
/// [`ClientRejection::OwnerMismatch`] when they disagree. Address
/// equality is case-insensitive through the reviewed [`Address`]
/// implementation so a mixed-case owner still matches its recovered
/// counterpart.
///
/// # Errors
///
/// See [`ClientRejection::OwnerMismatch`].
pub fn assert_owner_matches_signer(
    expected: &Address,
    recovered: &Address,
) -> Result<(), ClientRejection> {
    if expected != recovered {
        return Err(ClientRejection::OwnerMismatch {
            expected: *expected,
            recovered: *recovered,
        });
    }
    Ok(())
}
