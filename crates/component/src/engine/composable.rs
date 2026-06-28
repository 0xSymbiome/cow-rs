//! TWAP conditional-order encoding for the engine world's `composable` interface.
//!
//! A thin lowering over [`cow_sdk_contracts::composable`]: it parses the WIT
//! `twap-data` strings into the cow types, builds the validated [`TwapData`], and
//! returns the gas-free transaction wire parts. Pure offline encoding — the
//! consumer's smart-contract owner submits the transaction. No host imports and
//! no chain id: the `ComposableCoW` registry is deployed at one address on every
//! chain, so a TWAP encodes the same way everywhere.

use cow_sdk_contracts::UnsignedTransaction;
use cow_sdk_contracts::composable::{
    TwapData, TwapDurationOfPart, TwapStartTime, TwapTiming, twap_create_transaction,
    twap_remove_transaction,
};
use cow_sdk_core::{Address, Amount, AppDataHash, Hash32};

/// The `(to, data, value)` wire parts of a gas-free unsigned transaction.
fn parts(tx: &UnsignedTransaction) -> (String, String, String) {
    (
        tx.to.to_hex_string(),
        tx.data.to_hex_string(),
        tx.value.to_string(),
    )
}

fn addr(value: &str) -> Result<Address, String> {
    Address::new(value).map_err(|error| error.to_string())
}

fn amount(value: &str) -> Result<Amount, String> {
    Amount::new(value).map_err(|error| error.to_string())
}

/// Parses a 32-byte `0x` hex value (a salt or a conditional-order id).
fn hash32(value: &str) -> Result<Hash32, String> {
    Hash32::new(value).map_err(|error| error.to_string())
}

/// Builds and validates a [`TwapData`] from the flat WIT `twap-data` inputs.
///
/// The `start` / `duration` policy variants arrive already mapped to the contract
/// enums by the world adapter; the addresses, amounts, and app-data hash are
/// parsed here, and [`TwapData`] validation runs at build time.
#[allow(
    clippy::too_many_arguments,
    reason = "the parameters mirror the flat WIT twap-data record field-for-field"
)]
pub fn build_twap(
    sell_token: &str,
    buy_token: &str,
    receiver: Option<&str>,
    sell_amount: &str,
    buy_amount: &str,
    number_of_parts: u32,
    time_between_parts: u32,
    start: TwapStartTime,
    duration: TwapDurationOfPart,
    app_data: &str,
) -> Result<TwapData, String> {
    let mut builder = TwapData::builder()
        .sell(addr(sell_token)?, amount(sell_amount)?)
        .buy(addr(buy_token)?, amount(buy_amount)?)
        .parts(number_of_parts)
        .every(time_between_parts)
        .app_data(AppDataHash::new(app_data).map_err(|error| error.to_string())?);
    if let Some(receiver) = receiver {
        builder = builder.receiver(addr(receiver)?);
    }
    builder = match start {
        TwapStartTime::AtMiningTime => builder.start_at_mining_time(),
        TwapStartTime::AtEpoch(epoch) => builder.start_at_epoch(epoch),
    };
    builder = match duration {
        TwapDurationOfPart::Auto => builder.auto_duration(),
        TwapDurationOfPart::LimitDuration(span) => builder.limit_duration(span),
    };
    builder.build().map_err(|error| error.to_string())
}

/// Builds the gas-free `ComposableCoW` authorization transaction for a TWAP.
pub fn create_transaction(twap: &TwapData, salt: &str) -> Result<(String, String, String), String> {
    let tx = twap_create_transaction(twap, hash32(salt)?).map_err(|error| error.to_string())?;
    Ok(parts(&tx))
}

/// Builds the gas-free `remove` transaction for a conditional-order id.
pub fn remove_transaction(order_id: &str) -> Result<(String, String, String), String> {
    Ok(parts(&twap_remove_transaction(hash32(order_id)?)))
}

/// Returns the conditional-order id for a TWAP and salt.
pub fn order_id(twap: &TwapData, salt: &str) -> Result<String, String> {
    Ok(twap
        .order_id(hash32(salt)?)
        .map_err(|error| error.to_string())?
        .to_hex_string())
}

/// Classifies where the TWAP sits at `now`, given the epoch its schedule runs from.
pub fn timing_at(twap: &TwapData, start: u64, now: u64) -> Result<TwapTiming, String> {
    Ok(twap
        .static_input()
        .map_err(|error| error.to_string())?
        .timing_at(start, now))
}
