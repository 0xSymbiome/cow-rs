//! Typed client-side validator for every public trading submission seam.
//!
//! The validator is client-side defence-in-depth for orders before
//! submission. It covers eight reviewed protocol invariants (zero-amount,
//! buy-side same-token, app-data-from mismatch, and others) and is not a replacement
//! for the broader services-side rejection set. Services may still reject for
//! reasons the SDK cannot pre-check, including deny-list, transferability,
//! gas budget, banned-users, market-class classification, and
//! signing-scheme/onchain pairings. A passing local validation only means the
//! order does not violate any of the eight reviewed invariants; it does not
//! guarantee services will accept the order.
//!
//! The public entry point is [`OrderBoundsValidator::validate`]. The helper
//! is pure: `now` is a caller-supplied UNIX-seconds timestamp and no
//! `SystemTime::now` is read inside the validator, so every observation is
//! deterministic and reproducible across replays.

use std::time::Duration;

use cow_sdk_core::{
    Address, Amount, EVM_NATIVE_CURRENCY_ADDRESS, OrderData, OrderKind, SupportedChainId,
};
use cow_sdk_orderbook::SigningScheme;
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
    /// `valid_to` is closer to `now` than the configured minimum lifetime.
    #[error("validTo is insufficient: valid_to={valid_to}, now={now}, min_seconds={min_seconds}")]
    ValidToInsufficient {
        /// Submitted `valid_to` timestamp.
        valid_to: u64,
        /// Caller-supplied current UNIX-seconds timestamp.
        now: u64,
        /// Minimum lifetime in seconds enforced by the configured bounds.
        min_seconds: u64,
    },
    /// `valid_to` exceeds the configured maximum lifetime for the submission class.
    #[error("validTo is excessive: valid_to={valid_to}, now={now}, max_seconds={max_seconds}")]
    ValidToExcessive {
        /// Submitted `valid_to` timestamp.
        valid_to: u64,
        /// Caller-supplied current UNIX-seconds timestamp.
        now: u64,
        /// Maximum lifetime in seconds enforced by the configured bounds.
        max_seconds: u64,
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

/// Configured lifetime bounds enforced by [`OrderBoundsValidator`].
///
/// The reviewed services production defaults are exposed as the public
/// constant [`OrderValidityBounds::SERVICES_DEFAULT`]; the validator
/// applied at every public submission seam uses those defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderValidityBounds {
    /// Minimum lifetime enforced for every submission class.
    pub min: Duration,
    /// Maximum lifetime enforced for market-class orders.
    pub max_market: Duration,
    /// Maximum lifetime enforced for limit-class orders.
    pub max_limit: Duration,
}

impl OrderValidityBounds {
    /// Lifetime bounds that match the reviewed services production config
    /// (`min = 60s`, `max_market = 3h`, `max_limit = 1y`).
    pub const SERVICES_DEFAULT: Self = Self {
        min: Duration::from_secs(60),
        max_market: Duration::from_secs(10_800),
        max_limit: Duration::from_secs(31_536_000),
    };
}

impl Default for OrderValidityBounds {
    fn default() -> Self {
        Self::SERVICES_DEFAULT
    }
}

/// Submission class routed through [`OrderBoundsValidator::validate`].
///
/// Every cow-rs submission seam currently routes through `Limit` because
/// the reviewed services validator classifies any order carrying a
/// zero-`fee_amount` on the wire as a limit order. The `Market` variant is
/// exposed so offline helpers that assemble a signing order outside the
/// hot path can still exercise the tighter market-class bound.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmissionClass {
    /// Market-class order, subject to `max_market`.
    Market,
    /// Limit-class order, subject to `max_limit`.
    Limit,
    /// Liquidity-class order, exempt from the maximum-lifetime bound.
    Liquidity,
}

/// Pure client-side validator that enforces the reviewed services protocol
/// invariants on a signing order ([`cow_sdk_core::OrderData`]) and its
/// submission owner before the order reaches the orderbook.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderBoundsValidator {
    bounds: OrderValidityBounds,
    class: SubmissionClass,
    weth_address: Option<Address>,
}

impl OrderBoundsValidator {
    /// Creates a validator with the reviewed services-default bounds and
    /// the limit-class lifetime ceiling.
    #[must_use]
    pub const fn services_default() -> Self {
        Self {
            bounds: OrderValidityBounds::SERVICES_DEFAULT,
            class: SubmissionClass::Limit,
            weth_address: None,
        }
    }

    /// Creates a validator with the reviewed services-default bounds, the
    /// limit-class lifetime ceiling, and the chain-specific wrapped-native
    /// token address attached for the same-token paired guard.
    #[must_use]
    pub fn services_default_for_chain(chain_id: SupportedChainId) -> Self {
        Self::services_default()
            .with_weth_address(cow_sdk_core::wrapped_native_token(chain_id).address)
    }

    /// Creates a validator with the supplied bounds and submission class.
    #[cfg(test)]
    const fn new(bounds: OrderValidityBounds, class: SubmissionClass) -> Self {
        Self {
            bounds,
            class,
            weth_address: None,
        }
    }

    /// Returns a copy of this validator configured with the chain-specific
    /// wrapped-native token address. When supplied, the validator rejects
    /// `sell_token == weth_address` paired with `buy_token == BUY_ETH_ADDRESS`
    /// through [`ClientRejection::SameBuyAndSellToken`] on buy-side orders
    /// to mirror the reviewed services token-pair guard.
    #[must_use]
    pub const fn with_weth_address(mut self, weth_address: Address) -> Self {
        self.weth_address = Some(weth_address);
        self
    }

    /// Returns the configured lifetime bounds.
    #[must_use]
    pub const fn bounds(&self) -> OrderValidityBounds {
        self.bounds
    }

    /// Returns the configured submission class.
    #[must_use]
    pub const fn class(&self) -> SubmissionClass {
        self.class
    }

    /// Returns the configured chain-specific wrapped-native address, if any.
    #[must_use]
    pub const fn weth_address(&self) -> Option<&Address> {
        self.weth_address.as_ref()
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
    /// (zero amount, buy-side same token, owner mismatch, `valid_to` bounds) still
    /// runs.
    ///
    /// # Errors
    ///
    /// Returns [`ClientRejection`] on the first invariant violation so the
    /// caller can surface a typed error before any HTTP transport runs.
    pub fn validate(
        &self,
        order: &OrderData,
        from: Address,
        scheme: SigningScheme,
        app_data_signer: Option<Address>,
        now: u64,
        is_eth_flow: bool,
    ) -> Result<(), ClientRejection> {
        if from == zero_address() {
            return Err(ClientRejection::MissingFrom);
        }

        let valid_to = u64::from(order.valid_to);
        let remaining = valid_to.saturating_sub(now);
        let min_seconds = self.bounds.min.as_secs();
        if remaining < min_seconds {
            return Err(ClientRejection::ValidToInsufficient {
                valid_to,
                now,
                min_seconds,
            });
        }
        if !self.lifetime_is_unbounded(scheme) {
            let max_seconds = self.max_seconds();
            if remaining > max_seconds {
                return Err(ClientRejection::ValidToExcessive {
                    valid_to,
                    now,
                    max_seconds,
                });
            }
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

        let _ = scheme;

        Ok(())
    }

    fn validate_token_bounds(
        &self,
        sell_token: &Address,
        buy_token: &Address,
        kind: OrderKind,
        is_eth_flow: bool,
    ) -> Result<(), ClientRejection> {
        if !is_eth_flow {
            let native = native_sentinel();
            if sell_token == &native {
                return Err(ClientRejection::InvalidNativeSellToken);
            }
        }
        if sell_token == buy_token && kind == OrderKind::Buy {
            return Err(ClientRejection::SameBuyAndSellToken { token: *sell_token });
        }
        if let Some(weth) = self.weth_address.as_ref()
            && sell_token == weth
            && buy_token == &native_sentinel()
            && kind == OrderKind::Buy
        {
            return Err(ClientRejection::SameBuyAndSellToken { token: *weth });
        }
        Ok(())
    }

    const fn lifetime_is_unbounded(&self, scheme: SigningScheme) -> bool {
        matches!(scheme, SigningScheme::PreSign) || matches!(self.class, SubmissionClass::Liquidity)
    }

    const fn max_seconds(&self) -> u64 {
        match self.class {
            SubmissionClass::Market => self.bounds.max_market.as_secs(),
            SubmissionClass::Limit => self.bounds.max_limit.as_secs(),
            SubmissionClass::Liquidity => u64::MAX,
        }
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

/// Returns the native-currency sentinel address used by order validation.
///
/// # Panics
///
/// Panics only if the shared native-currency sentinel literal stops being a
/// valid EVM address.
fn native_sentinel() -> Address {
    // SAFETY: EVM_NATIVE_CURRENCY_ADDRESS is a crate-owned protocol sentinel
    // literal validated through the shared Address constructor.
    Address::new(EVM_NATIVE_CURRENCY_ADDRESS)
        .expect("EVM_NATIVE_CURRENCY_ADDRESS must remain a valid address literal")
}

const fn zero_address() -> Address {
    Address::from_bytes([0u8; 20])
}

/// Asserts `sell_token` is not the native-currency sentinel.
///
/// Runs the equivalent of [`OrderBoundsValidator::validate`] for the
/// post-routing assertion that a pre-HTTP order does not carry the native
/// sentinel as `sell_token` unless it is routed through the reviewed
/// eth-flow submission path. Returns
/// [`ClientRejection::InvalidNativeSellToken`] when the submission path
/// carries the native sentinel outside eth-flow routing.
///
/// # Errors
///
/// See [`ClientRejection::InvalidNativeSellToken`].
pub fn assert_non_native_sell_token(sell_token: &Address) -> Result<(), ClientRejection> {
    if sell_token == &native_sentinel() {
        return Err(ClientRejection::InvalidNativeSellToken);
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

#[cfg(test)]
mod tests {
    use super::{ClientRejection, OrderBoundsValidator, OrderValidityBounds, SubmissionClass};
    use cow_sdk_core::{
        Address, Amount, AppDataHash, BuyTokenDestination, OrderData, OrderKind, SellTokenSource,
    };
    use cow_sdk_orderbook::SigningScheme;

    const FROM: &str = "0x1111111111111111111111111111111111111111";
    const SELL_TOKEN: &str = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const BUY_TOKEN: &str = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const NOW: u64 = 1_700_000_000;
    const VALID_TO: u32 = 1_700_003_600;

    fn address(hex: &str) -> Address {
        Address::new(hex).expect("fixture address must be valid")
    }

    fn order() -> OrderData {
        OrderData::new(
            address(SELL_TOKEN),
            address(BUY_TOKEN),
            address(FROM),
            Amount::new("1000000000000000000").expect("test amount literal must be valid"),
            Amount::new("1000000").expect("test amount literal must be valid"),
            VALID_TO,
            app_data_hash(),
            Amount::ZERO,
            OrderKind::Sell,
            false,
            SellTokenSource::Erc20,
            BuyTokenDestination::Erc20,
        )
    }

    fn app_data_hash() -> AppDataHash {
        AppDataHash::new("0x0000000000000000000000000000000000000000000000000000000000000000")
            .expect("app-data hash literal must be valid")
    }

    #[test]
    fn liquidity_class_bypasses_the_lifetime_ceiling() {
        let validator = OrderBoundsValidator::new(
            OrderValidityBounds::SERVICES_DEFAULT,
            SubmissionClass::Liquidity,
        );
        let mut order = order();
        order.valid_to = u32::try_from(NOW + 31_536_001).expect("valid_to must fit in u32");
        validator
            .validate(
                &order,
                address(FROM),
                SigningScheme::Eip712,
                None,
                NOW,
                false,
            )
            .expect("Liquidity class must bypass the lifetime ceiling");
    }

    #[test]
    fn market_class_rejects_valid_to_above_three_hours() {
        let validator = OrderBoundsValidator::new(
            OrderValidityBounds::SERVICES_DEFAULT,
            SubmissionClass::Market,
        );
        let mut order = order();
        order.valid_to = u32::try_from(NOW + 10_801).expect("valid_to must fit in u32");
        let error = validator
            .validate(
                &order,
                address(FROM),
                SigningScheme::Eip712,
                None,
                NOW,
                false,
            )
            .expect_err("market class must enforce the 3h ceiling");
        assert!(matches!(
            error,
            ClientRejection::ValidToExcessive {
                max_seconds: 10_800,
                ..
            }
        ));
    }

    #[test]
    fn validator_constructed_with_tighter_min_rejects_short_lifetime() {
        let tighter = OrderValidityBounds {
            min: std::time::Duration::from_secs(120),
            ..OrderValidityBounds::SERVICES_DEFAULT
        };
        let validator = OrderBoundsValidator::new(tighter, SubmissionClass::Limit);
        let mut order = order();
        order.valid_to = u32::try_from(NOW + 60).expect("valid_to must fit in u32");
        let error = validator
            .validate(
                &order,
                address(FROM),
                SigningScheme::Eip712,
                None,
                NOW,
                false,
            )
            .expect_err("custom 120s minimum must reject a 60s lifetime");
        assert!(matches!(
            error,
            ClientRejection::ValidToInsufficient {
                min_seconds: 120,
                ..
            }
        ));
    }
}
