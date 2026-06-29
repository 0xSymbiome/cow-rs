//! TWAP conditional-order transaction builders (wasm-bindgen).
//!
//! A TWAP is a `ComposableCoW` conditional order: the owner authorizes it once
//! on chain, and the watch tower posts each discrete part to the order book as it
//! becomes tradeable. These builders produce the authorization and cancellation
//! transactions; the owner — a smart-contract wallet (a Safe) — submits them.

use cow_sdk_contracts::composable::{TwapData, twap_create_transaction, twap_remove_transaction};
use cow_sdk_core::{Address, Amount, AppDataHash, Hash32, TransactionRequest};
use wasm_bindgen::prelude::*;

use crate::dto::{TwapCreateParams, TwapCreateResult, to_js_value};
use crate::exports::{envelope::WasmEnvelope, errors::JsResultExt};

/// Builds the `ComposableCoW` transaction that authorizes a TWAP order.
///
/// Start-at-mining-time orders route through `createWithContext` with the
/// block-timestamp factory; start-at-epoch orders route through `create`. The
/// transaction targets `ComposableCoW` with zero value, and the owner Safe
/// submits it. The result also carries the conditional-order id for tracking the
/// discrete parts the watch tower posts.
///
/// @param params Totals, parts, interval, salt, app-data, and optional start,
///   duration, and receiver.
/// @returns A versioned envelope with the transaction and the order id.
/// @throws CowError when an address, amount, hash, or TWAP rule is invalid.
#[wasm_bindgen(
    js_name = "buildTwapCreateTransaction",
    unchecked_return_type = "WasmEnvelope<TwapCreateResult>"
)]
pub fn build_twap_create_transaction(params: TwapCreateParams) -> Result<JsValue, JsValue> {
    let twap = twap_from_params(&params)?;
    let salt = Hash32::new(&params.salt).map_js()?;
    let unsigned = twap_create_transaction(&twap, salt).map_js()?;
    let order_id = twap.order_id(salt).map_js()?;
    let result = TwapCreateResult {
        transaction: TransactionRequest::from(unsigned),
        order_id: order_id.to_string(),
    };
    to_js_value(&WasmEnvelope::v1(result))
}

/// Builds the `ComposableCoW` transaction that cancels a TWAP by its order id.
///
/// @param orderId The conditional-order id returned by
///   `buildTwapCreateTransaction`.
/// @returns A versioned envelope with the cancellation transaction.
/// @throws CowError when the order id is not a valid 32-byte hash.
#[wasm_bindgen(
    js_name = "buildTwapRemoveTransaction",
    unchecked_return_type = "WasmEnvelope<TransactionRequest>"
)]
pub fn build_twap_remove_transaction(
    #[wasm_bindgen(js_name = orderId)] order_id: String,
) -> Result<JsValue, JsValue> {
    let order_id = Hash32::new(&order_id).map_js()?;
    let unsigned = twap_remove_transaction(order_id);
    to_js_value(&WasmEnvelope::v1(TransactionRequest::from(unsigned)))
}

/// Lowers the boundary params into a validated [`TwapData`].
fn twap_from_params(params: &TwapCreateParams) -> Result<TwapData, JsValue> {
    let mut builder = TwapData::builder()
        .sell(
            Address::new(&params.sell_token).map_js()?,
            Amount::new(&params.sell_amount).map_js()?,
        )
        .buy(
            Address::new(&params.buy_token).map_js()?,
            Amount::new(&params.buy_amount).map_js()?,
        )
        .parts(params.number_of_parts)
        .every(params.time_between_parts)
        .app_data(AppDataHash::new(&params.app_data).map_js()?);
    builder = match params.start_epoch {
        Some(epoch) => builder.start_at_epoch(epoch),
        None => builder.start_at_mining_time(),
    };
    builder = match params.limit_duration {
        Some(seconds) => builder.limit_duration(seconds),
        None => builder.auto_duration(),
    };
    if let Some(receiver) = &params.receiver {
        builder = builder.receiver(Address::new(receiver).map_js()?);
    }
    builder.build().map_js()
}
