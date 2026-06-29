//! Time-weighted average price (TWAP) conditional orders.
//!
//! A TWAP splits a trade into `n` equal parts executed one per fixed interval.
//! The consumer thinks in totals ([`TwapData`]); the on-chain handler reads a
//! per-part [`TwapStaticInput`], so the builder divides the totals across the
//! parts and validates against the handler's revert sites before encoding.
//!
//! Field order in [`TwapStaticInput`] is load-bearing — it mirrors the upstream
//! `TWAPOrder.Data` struct exactly, and any reorder changes the ABI encoding the
//! handler decodes.

use alloy_primitives::U256;
use alloy_sol_types::{SolValue, sol};

use cow_sdk_core::{Address, Amount, AppDataHash, Hash32, HexData, address};

use super::{
    COMPOSABLE_COW, ConditionalOrderParams, conditional_order_id, encode_create_calldata,
    encode_create_with_context_calldata, encode_remove_calldata, merkle_leaf,
};
use crate::tx::UnsignedTransaction;

/// TWAP handler contract — a CREATE2 singleton identical on every supported
/// chain. It is the `handler` of every TWAP [`ConditionalOrderParams`].
pub const TWAP_HANDLER: Address = address!("0x6cf1e9ca41f7611def408122793c358a3d11e5a5");

/// `CurrentBlockTimestampFactory` value factory — supplies the start timestamp
/// for a start-at-mining-time TWAP through `createWithContext`.
pub const CURRENT_BLOCK_TIMESTAMP_FACTORY: Address =
    address!("0x52ed56da04309aca4c3fecc595298d80c2f16bac");

/// Seconds in 365 days — the upstream cap on the interval between parts.
const MAX_FREQUENCY_SECONDS: u32 = 365 * 24 * 60 * 60;

sol! {
    // Mirrors cowprotocol/composable-cow `src/types/twap/libraries/TWAPOrder.sol`
    // `Data`; the field order is the ABI contract the handler decodes.
    struct TwapStaticInputAbi {
        address sellToken;
        address buyToken;
        address receiver;
        uint256 partSellAmount;
        uint256 minPartLimit;
        uint256 t0;
        uint256 n;
        uint256 t;
        uint256 span;
        bytes32 appData;
    }
}

/// When the first part becomes valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TwapStartTime {
    /// Start at the block timestamp when the order is authorized (on-chain `t0 =
    /// 0`, filled from the cabinet by `createWithContext`).
    AtMiningTime,
    /// Start at an explicit unix timestamp.
    AtEpoch(u32),
}

/// How long each part stays valid within its interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TwapDurationOfPart {
    /// Valid for the whole interval (on-chain `span = 0`).
    Auto,
    /// Valid for a shorter window at the start of each interval.
    LimitDuration(u32),
}

/// Friendly TWAP inputs in totals across all parts.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TwapData {
    /// Token sold across the whole TWAP.
    pub sell_token: Address,
    /// Token bought across the whole TWAP.
    pub buy_token: Address,
    /// Receiver of the bought token; [`Address::ZERO`] pays the owner.
    pub receiver: Address,
    /// Total sell amount across all parts.
    pub sell_amount: Amount,
    /// Total minimum buy amount across all parts.
    pub buy_amount: Amount,
    /// Number of parts; must be greater than one.
    pub number_of_parts: u32,
    /// Seconds between parts; must be non-zero and at most 365 days.
    pub time_between_parts: u32,
    /// When the first part becomes valid.
    pub start: TwapStartTime,
    /// How long each part stays valid within its interval.
    pub duration: TwapDurationOfPart,
    /// App-data hash applied to every part.
    pub app_data: AppDataHash,
}

/// The validated per-part TWAP input the handler decodes.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TwapStaticInput {
    /// Token sold in each part.
    pub sell_token: Address,
    /// Token bought in each part.
    pub buy_token: Address,
    /// Receiver of each part's bought token.
    pub receiver: Address,
    /// Sell amount per part (`sell_amount / number_of_parts`).
    pub part_sell_amount: Amount,
    /// Minimum buy amount per part (`buy_amount / number_of_parts`).
    pub min_part_limit: Amount,
    /// Start timestamp; `0` reads the block timestamp at authorization.
    pub t0: u32,
    /// Number of parts.
    pub n: u32,
    /// Seconds between parts.
    pub t: u32,
    /// Intra-interval validity window; `0` is the whole interval.
    pub span: u32,
    /// App-data hash applied to every part.
    pub app_data: AppDataHash,
}

/// Where a TWAP sits in its schedule at a given moment.
///
/// Produced by [`TwapStaticInput::timing_at`]. Mirrors the part selection in the
/// upstream `TWAPOrderMathLib.calculateValidTo`: the schedule runs from the start
/// for `n` intervals of `t` seconds and ends at `start + n * t` regardless of the
/// span.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TwapTiming {
    /// No part is valid yet; the first interval opens at `start_epoch` (unix
    /// seconds).
    NotStarted {
        /// Start of the first interval.
        start_epoch: u64,
    },
    /// An interval is open.
    Active {
        /// Zero-based index of the current part.
        part: u32,
        /// `validTo` of this part's discrete order (unix seconds, inclusive). For
        /// a `span > 0` TWAP this can fall behind `now`, meaning the part's span
        /// has closed and nothing is tradeable until `next_part_start`.
        valid_to: u64,
        /// Start of the next interval; equals the schedule end on the last part.
        next_part_start: u64,
        /// Whether this is the final part.
        is_last: bool,
    },
    /// The schedule has ended (`now >= start + n * t`); no part is valid.
    Expired,
}

/// Why a TWAP failed client-side validation, mirroring the upstream
/// `TWAPOrder.validate` revert sites plus the divisibility guards this builder
/// adds so totals are never silently floored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum TwapValidationError {
    /// Sell and buy token are the same.
    #[error("sell and buy token must differ")]
    SameToken,
    /// Sell or buy token is the zero address.
    #[error("sell and buy token must be non-zero")]
    InvalidToken,
    /// The total sell amount is not divisible by the number of parts.
    #[error("sell amount must divide evenly across the parts")]
    IndivisibleSellAmount,
    /// The total buy amount is not divisible by the number of parts.
    #[error("buy amount must divide evenly across the parts")]
    IndivisibleBuyAmount,
    /// The per-part sell amount is zero.
    #[error("per-part sell amount must be non-zero")]
    InvalidPartSellAmount,
    /// The per-part minimum buy amount is zero.
    #[error("per-part minimum buy amount must be non-zero")]
    InvalidMinPartLimit,
    /// The start epoch is zero (use [`TwapStartTime::AtMiningTime`]) or at the
    /// `uint32` ceiling.
    #[error("start epoch must be in (0, u32::MAX)")]
    InvalidStartTime,
    /// The number of parts is not greater than one.
    #[error("number of parts must be greater than one")]
    InvalidNumParts,
    /// The interval between parts is zero or exceeds 365 days.
    #[error("interval between parts must be in (0, 365 days]")]
    InvalidFrequency,
    /// The intra-interval span exceeds the interval.
    #[error("intra-interval span must not exceed the interval")]
    InvalidSpan,
    /// A required builder field was not set.
    #[error("missing required field: {0}")]
    MissingField(&'static str),
}

impl TwapData {
    /// Starts a fluent builder.
    #[must_use]
    pub fn builder() -> TwapBuilder {
        TwapBuilder::default()
    }

    /// Validates the inputs and lowers them to the per-part [`TwapStaticInput`].
    ///
    /// The single chokepoint every other method routes through.
    ///
    /// # Errors
    ///
    /// Returns the matching [`TwapValidationError`] for the first failing
    /// upstream `TWAPOrder.validate` rule or divisibility guard.
    #[allow(
        clippy::similar_names,
        reason = "the lowered per-part variables (sell/span/n) mirror the short upstream TWAP field names"
    )]
    pub fn static_input(&self) -> Result<TwapStaticInput, TwapValidationError> {
        if self.sell_token == self.buy_token {
            return Err(TwapValidationError::SameToken);
        }
        if self.sell_token == Address::ZERO || self.buy_token == Address::ZERO {
            return Err(TwapValidationError::InvalidToken);
        }
        if self.number_of_parts <= 1 {
            return Err(TwapValidationError::InvalidNumParts);
        }
        if self.time_between_parts == 0 || self.time_between_parts > MAX_FREQUENCY_SECONDS {
            return Err(TwapValidationError::InvalidFrequency);
        }

        let n = U256::from(self.number_of_parts);
        let sell = *self.sell_amount.as_u256();
        let buy = *self.buy_amount.as_u256();
        if sell % n != U256::ZERO {
            return Err(TwapValidationError::IndivisibleSellAmount);
        }
        if buy % n != U256::ZERO {
            return Err(TwapValidationError::IndivisibleBuyAmount);
        }
        let part_sell_amount = Amount::from_u256(sell / n);
        let min_part_limit = Amount::from_u256(buy / n);
        if part_sell_amount == Amount::ZERO {
            return Err(TwapValidationError::InvalidPartSellAmount);
        }
        if min_part_limit == Amount::ZERO {
            return Err(TwapValidationError::InvalidMinPartLimit);
        }

        let t0 = match self.start {
            TwapStartTime::AtMiningTime => 0,
            TwapStartTime::AtEpoch(epoch) => {
                if epoch == 0 || epoch == u32::MAX {
                    return Err(TwapValidationError::InvalidStartTime);
                }
                epoch
            }
        };
        let span = match self.duration {
            TwapDurationOfPart::Auto => 0,
            TwapDurationOfPart::LimitDuration(span) => {
                if span > self.time_between_parts {
                    return Err(TwapValidationError::InvalidSpan);
                }
                span
            }
        };

        Ok(TwapStaticInput {
            sell_token: self.sell_token,
            buy_token: self.buy_token,
            receiver: self.receiver,
            part_sell_amount,
            min_part_limit,
            t0,
            n: self.number_of_parts,
            t: self.time_between_parts,
            span,
            app_data: self.app_data,
        })
    }

    /// Returns `abi.encode(staticInput)` — the 320-byte handler input.
    ///
    /// # Errors
    ///
    /// Propagates [`TwapData::static_input`] validation failures.
    pub fn encode_static_input(&self) -> Result<Vec<u8>, TwapValidationError> {
        Ok(self.static_input()?.abi_encode())
    }

    /// Wraps the TWAP into [`ConditionalOrderParams`] under [`TWAP_HANDLER`].
    ///
    /// # Errors
    ///
    /// Propagates [`TwapData::static_input`] validation failures.
    pub fn to_params(&self, salt: Hash32) -> Result<ConditionalOrderParams, TwapValidationError> {
        Ok(ConditionalOrderParams {
            handler: TWAP_HANDLER,
            salt,
            static_input: HexData::from(alloy_primitives::Bytes::from(self.encode_static_input()?)),
        })
    }

    /// Returns the conditional-order id for this TWAP and salt.
    ///
    /// # Errors
    ///
    /// Propagates [`TwapData::static_input`] validation failures.
    pub fn order_id(&self, salt: Hash32) -> Result<Hash32, TwapValidationError> {
        Ok(conditional_order_id(&self.to_params(salt)?))
    }

    /// Returns the contract-canonical merkle leaf for this TWAP and salt.
    ///
    /// # Errors
    ///
    /// Propagates [`TwapData::static_input`] validation failures.
    pub fn merkle_leaf(&self, salt: Hash32) -> Result<Hash32, TwapValidationError> {
        Ok(merkle_leaf(&self.to_params(salt)?))
    }
}

impl TwapStaticInput {
    /// Classifies where the TWAP sits at `now`, given its resolved start time.
    ///
    /// `start` is the epoch the handler runs from: [`TwapStaticInput::t0`] for a
    /// start-at-epoch TWAP, or the value the `CurrentBlockTimestampFactory` wrote
    /// to the cabinet for a start-at-mining-time TWAP (where `t0` is `0`). `start`
    /// and `now` are unix seconds.
    ///
    /// The arithmetic follows `TWAPOrderMathLib.calculateValidTo`: the schedule
    /// ends at `start + n * t` whatever the span, a part is selected by
    /// `(now - start) / t`, and `valid_to` is the interval end for `span == 0` or
    /// the span end otherwise. The widening to `u64` matches the handler computing
    /// these products in `uint256`; the per-part bounds enforced by
    /// [`TwapData::static_input`] keep every term well within range.
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        reason = "part < n <= u32::MAX, so the part index fits u32 without truncation"
    )]
    pub fn timing_at(&self, start: u64, now: u64) -> TwapTiming {
        let n = u64::from(self.n);
        let t = u64::from(self.t);
        let span = u64::from(self.span);

        if now < start {
            return TwapTiming::NotStarted { start_epoch: start };
        }
        let end = start.saturating_add(n.saturating_mul(t));
        if now >= end {
            return TwapTiming::Expired;
        }
        // `now >= start` from the guard above and `t > 0` from validation, so the
        // subtraction and division are both well defined.
        let part = (now - start) / t;
        let next_part_start = start.saturating_add((part + 1).saturating_mul(t));
        let valid_to = if span == 0 {
            next_part_start.saturating_sub(1)
        } else {
            start
                .saturating_add(part.saturating_mul(t))
                .saturating_add(span)
                .saturating_sub(1)
        };
        TwapTiming::Active {
            part: part as u32,
            valid_to,
            next_part_start,
            is_last: part == n - 1,
        }
    }

    /// Returns `abi.encode(staticInput)` — the 320-byte handler input.
    #[must_use]
    pub fn abi_encode(&self) -> Vec<u8> {
        use alloy_sol_types::private::{Address as SolAddress, FixedBytes};

        TwapStaticInputAbi {
            sellToken: SolAddress::from(self.sell_token.into_alloy().0.0),
            buyToken: SolAddress::from(self.buy_token.into_alloy().0.0),
            receiver: SolAddress::from(self.receiver.into_alloy().0.0),
            partSellAmount: *self.part_sell_amount.as_u256(),
            minPartLimit: *self.min_part_limit.as_u256(),
            t0: U256::from(self.t0),
            n: U256::from(self.n),
            t: U256::from(self.t),
            span: U256::from(self.span),
            appData: FixedBytes::from(self.app_data.as_alloy().0),
        }
        .abi_encode()
    }
}

/// Fluent builder for [`TwapData`]; construct through [`TwapData::builder`].
#[derive(Debug, Clone, Default)]
pub struct TwapBuilder {
    sell_token: Option<Address>,
    buy_token: Option<Address>,
    receiver: Option<Address>,
    sell_amount: Option<Amount>,
    buy_amount: Option<Amount>,
    number_of_parts: Option<u32>,
    time_between_parts: Option<u32>,
    start: Option<TwapStartTime>,
    duration: Option<TwapDurationOfPart>,
    app_data: Option<AppDataHash>,
}

impl TwapBuilder {
    /// Sets the sell token and total sell amount.
    #[must_use]
    pub const fn sell(mut self, token: Address, total_amount: Amount) -> Self {
        self.sell_token = Some(token);
        self.sell_amount = Some(total_amount);
        self
    }

    /// Sets the buy token and total minimum buy amount.
    #[must_use]
    pub const fn buy(mut self, token: Address, total_min_amount: Amount) -> Self {
        self.buy_token = Some(token);
        self.buy_amount = Some(total_min_amount);
        self
    }

    /// Sets the number of parts.
    #[must_use]
    pub const fn parts(mut self, number_of_parts: u32) -> Self {
        self.number_of_parts = Some(number_of_parts);
        self
    }

    /// Sets the seconds between parts.
    #[must_use]
    pub const fn every(mut self, seconds_between_parts: u32) -> Self {
        self.time_between_parts = Some(seconds_between_parts);
        self
    }

    /// Starts the first part at the authorization block timestamp.
    #[must_use]
    pub const fn start_at_mining_time(mut self) -> Self {
        self.start = Some(TwapStartTime::AtMiningTime);
        self
    }

    /// Starts the first part at an explicit unix timestamp.
    #[must_use]
    pub const fn start_at_epoch(mut self, epoch: u32) -> Self {
        self.start = Some(TwapStartTime::AtEpoch(epoch));
        self
    }

    /// Keeps each part valid for its whole interval.
    #[must_use]
    pub const fn auto_duration(mut self) -> Self {
        self.duration = Some(TwapDurationOfPart::Auto);
        self
    }

    /// Limits each part's validity to `seconds` at the start of its interval.
    #[must_use]
    pub const fn limit_duration(mut self, seconds: u32) -> Self {
        self.duration = Some(TwapDurationOfPart::LimitDuration(seconds));
        self
    }

    /// Sets the receiver; omit to pay the owner.
    #[must_use]
    pub const fn receiver(mut self, receiver: Address) -> Self {
        self.receiver = Some(receiver);
        self
    }

    /// Sets the app-data hash applied to every part.
    #[must_use]
    pub const fn app_data(mut self, app_data: AppDataHash) -> Self {
        self.app_data = Some(app_data);
        self
    }

    /// Builds and validates the [`TwapData`].
    ///
    /// # Errors
    ///
    /// Returns [`TwapValidationError::MissingField`] for an unset required field
    /// and otherwise the first failing validation rule.
    pub fn build(self) -> Result<TwapData, TwapValidationError> {
        let data = TwapData {
            sell_token: self
                .sell_token
                .ok_or(TwapValidationError::MissingField("sell"))?,
            buy_token: self
                .buy_token
                .ok_or(TwapValidationError::MissingField("buy"))?,
            receiver: self.receiver.unwrap_or(Address::ZERO),
            sell_amount: self
                .sell_amount
                .ok_or(TwapValidationError::MissingField("sell"))?,
            buy_amount: self
                .buy_amount
                .ok_or(TwapValidationError::MissingField("buy"))?,
            number_of_parts: self
                .number_of_parts
                .ok_or(TwapValidationError::MissingField("parts"))?,
            time_between_parts: self
                .time_between_parts
                .ok_or(TwapValidationError::MissingField("every"))?,
            start: self.start.unwrap_or(TwapStartTime::AtMiningTime),
            duration: self.duration.unwrap_or(TwapDurationOfPart::Auto),
            app_data: self
                .app_data
                .ok_or(TwapValidationError::MissingField("app_data"))?,
        };
        // Surface validation failures at build time rather than at encode time.
        data.static_input()?;
        Ok(data)
    }
}

/// Builds the gas-free transaction that authorizes a TWAP on `ComposableCoW`.
///
/// `to` is [`COMPOSABLE_COW`] and `value` is zero. A start-at-mining-time TWAP is
/// authorized through `createWithContext` with the block-timestamp value factory
/// wired in; a start-at-epoch TWAP carries its start time in the static input and
/// is authorized through `create`, the same routing the upstream SDK applies. The
/// owner is a smart-contract account; the consumer submits this through it.
///
/// # Errors
///
/// Propagates [`TwapData::static_input`] validation failures.
pub fn twap_create_transaction(
    twap: &TwapData,
    salt: Hash32,
) -> Result<UnsignedTransaction, TwapValidationError> {
    let params = twap.to_params(salt)?;
    let data = match twap.start {
        TwapStartTime::AtMiningTime => {
            encode_create_with_context_calldata(&params, CURRENT_BLOCK_TIMESTAMP_FACTORY, &[], true)
        }
        TwapStartTime::AtEpoch(_) => encode_create_calldata(&params, true),
    };
    Ok(UnsignedTransaction::new(
        COMPOSABLE_COW,
        HexData::from(alloy_primitives::Bytes::from(data)),
        Amount::ZERO,
    ))
}

/// Builds the gas-free `remove` transaction that cancels a conditional order by
/// its [`TwapData::order_id`].
#[must_use]
pub fn twap_remove_transaction(order_id: Hash32) -> UnsignedTransaction {
    UnsignedTransaction::new(
        COMPOSABLE_COW,
        HexData::from(alloy_primitives::Bytes::from(encode_remove_calldata(
            order_id,
        ))),
        Amount::ZERO,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn weth() -> Address {
        Address::new("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap()
    }
    fn usdc() -> Address {
        Address::new("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap()
    }
    fn app_data() -> AppDataHash {
        AppDataHash::from_bytes([0xab; 32])
    }

    fn sample() -> TwapData {
        TwapData::builder()
            .sell(weth(), Amount::new("12000000000000000000").unwrap())
            .buy(usdc(), Amount::new("30000000000").unwrap())
            .parts(6)
            .every(3600)
            .start_at_mining_time()
            .app_data(app_data())
            .build()
            .unwrap()
    }

    #[test]
    fn lowers_totals_into_per_part_amounts() {
        let si = sample().static_input().unwrap();
        assert_eq!(
            si.part_sell_amount,
            Amount::new("2000000000000000000").unwrap()
        );
        assert_eq!(si.min_part_limit, Amount::new("5000000000").unwrap());
        assert_eq!(si.t0, 0);
        assert_eq!(si.span, 0);
        assert_eq!(si.n, 6);
        assert_eq!(si.t, 3600);
    }

    #[test]
    fn static_input_is_320_bytes_of_static_words() {
        let encoded = sample().encode_static_input().unwrap();
        assert_eq!(
            encoded.len(),
            320,
            "ten static 32-byte words, no dynamic members"
        );
    }

    #[test]
    fn validation_mirrors_upstream_revert_sites() {
        let same = TwapData::builder()
            .sell(weth(), Amount::new("12000000000000000000").unwrap())
            .buy(weth(), Amount::new("12000000000000000000").unwrap())
            .parts(6)
            .every(3600)
            .app_data(app_data())
            .build();
        assert_eq!(same.unwrap_err(), TwapValidationError::SameToken);

        let one_part = TwapData::builder()
            .sell(weth(), Amount::new("12000000000000000000").unwrap())
            .buy(usdc(), Amount::new("30000000000").unwrap())
            .parts(1)
            .every(3600)
            .app_data(app_data())
            .build();
        assert_eq!(one_part.unwrap_err(), TwapValidationError::InvalidNumParts);

        let too_slow = TwapData::builder()
            .sell(weth(), Amount::new("12000000000000000000").unwrap())
            .buy(usdc(), Amount::new("30000000000").unwrap())
            .parts(6)
            .every(MAX_FREQUENCY_SECONDS + 1)
            .app_data(app_data())
            .build();
        assert_eq!(too_slow.unwrap_err(), TwapValidationError::InvalidFrequency);
    }

    #[test]
    fn rejects_indivisible_totals_rather_than_flooring() {
        let indivisible = TwapData::builder()
            .sell(weth(), Amount::new("100").unwrap())
            .buy(usdc(), Amount::new("30000000000").unwrap())
            .parts(7)
            .every(3600)
            .app_data(app_data())
            .build();
        assert_eq!(
            indivisible.unwrap_err(),
            TwapValidationError::IndivisibleSellAmount
        );
    }

    #[test]
    fn mining_time_start_routes_to_create_with_context() {
        // sample() starts at mining time, so the factory-backed createWithContext
        // path applies (selector 0x0d0d9800).
        let tx = twap_create_transaction(&sample(), Hash32::from_bytes([0x01; 32])).unwrap();
        assert_eq!(tx.to, COMPOSABLE_COW);
        assert_eq!(tx.value, Amount::ZERO);
        assert_eq!(&tx.data.as_alloy()[..4], &[0x0d, 0x0d, 0x98, 0x00]);
    }

    #[test]
    fn epoch_start_routes_to_create() {
        // A fixed start epoch needs no value factory, so the plain create path
        // applies, matching the upstream routing.
        let twap = TwapData::builder()
            .sell(weth(), Amount::new("12000000000000000000").unwrap())
            .buy(usdc(), Amount::new("30000000000").unwrap())
            .parts(6)
            .every(3600)
            .start_at_epoch(1_900_000_000)
            .app_data(app_data())
            .build()
            .unwrap();
        let salt = Hash32::from_bytes([0x01; 32]);
        let tx = twap_create_transaction(&twap, salt).unwrap();
        let expected = encode_create_calldata(&twap.to_params(salt).unwrap(), true);
        assert_eq!(tx.to, COMPOSABLE_COW);
        assert_eq!(tx.data.as_alloy().as_ref(), expected.as_slice());
        // Routed to create, not createWithContext.
        assert_ne!(&tx.data.as_alloy()[..4], &[0x0d, 0x0d, 0x98, 0x00]);
    }

    #[test]
    fn timing_before_start_is_not_started() {
        let si = sample().static_input().unwrap();
        let start = 1_000_000;
        assert_eq!(
            si.timing_at(start, start - 1),
            TwapTiming::NotStarted { start_epoch: start }
        );
    }

    #[test]
    fn timing_selects_the_part_and_valid_to() {
        // sample(): n = 6, t = 3600, span = 0 (auto).
        let si = sample().static_input().unwrap();
        let start = 1_000_000;
        // The first part opens exactly at the start.
        assert_eq!(
            si.timing_at(start, start),
            TwapTiming::Active {
                part: 0,
                valid_to: start + 3600 - 1,
                next_part_start: start + 3600,
                is_last: false,
            }
        );
        // A moment inside the third interval selects part 2.
        assert_eq!(
            si.timing_at(start, start + 2 * 3600 + 10),
            TwapTiming::Active {
                part: 2,
                valid_to: start + 3 * 3600 - 1,
                next_part_start: start + 3 * 3600,
                is_last: false,
            }
        );
    }

    #[test]
    fn timing_flags_the_last_part_then_expires() {
        let si = sample().static_input().unwrap();
        let start = 1_000_000;
        // The sixth interval is the last part.
        assert_eq!(
            si.timing_at(start, start + 5 * 3600),
            TwapTiming::Active {
                part: 5,
                valid_to: start + 6 * 3600 - 1,
                next_part_start: start + 6 * 3600,
                is_last: true,
            }
        );
        // The schedule ends at start + n * t; the boundary itself is expired.
        assert_eq!(si.timing_at(start, start + 6 * 3600), TwapTiming::Expired);
        assert_eq!(si.timing_at(start, start + 100 * 3600), TwapTiming::Expired);
    }

    #[test]
    fn timing_span_closes_the_part_early_without_shortening_the_schedule() {
        // A limited-duration TWAP: each 3600s interval is tradeable for its first
        // 600s only. This exercises calculateValidTo's span branch and its
        // span-agnostic schedule end.
        let twap = TwapData::builder()
            .sell(weth(), Amount::new("12000000000000000000").unwrap())
            .buy(usdc(), Amount::new("30000000000").unwrap())
            .parts(6)
            .every(3600)
            .start_at_epoch(1_000_000)
            .limit_duration(600)
            .app_data(app_data())
            .build()
            .unwrap();
        let si = twap.static_input().unwrap();
        let start = 1_000_000;
        // Inside part 0's span, valid_to is the span end.
        assert_eq!(
            si.timing_at(start, start + 100),
            TwapTiming::Active {
                part: 0,
                valid_to: start + 600 - 1,
                next_part_start: start + 3600,
                is_last: false,
            }
        );
        // After the span but still inside the interval: part 0, but valid_to is
        // behind `now` — the gap until the next part opens.
        match si.timing_at(start, start + 1000) {
            TwapTiming::Active { part, valid_to, .. } => {
                assert_eq!(part, 0);
                assert!(valid_to < start + 1000, "the span has closed");
            }
            other => panic!("expected an active part, got {other:?}"),
        }
        // The schedule still ends at start + n * t, span notwithstanding.
        assert_eq!(si.timing_at(start, start + 6 * 3600), TwapTiming::Expired);
    }
}
