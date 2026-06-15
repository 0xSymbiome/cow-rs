//! Pure on-chain event-log decoding exports.
//!
//! `decodeSettlementLog` and `decodeEthFlowLog` reconstruct borrowed log bytes
//! from a [`EventLogInput`] and dispatch to the fail-closed, provider-free
//! decoders in `cow-sdk-contracts`. Both are deterministic and perform no I/O,
//! so one implementation serves any JavaScript host that already holds raw
//! chain logs.

use wasm_bindgen::prelude::*;

use crate::exports::{
    dto::{EthFlowEventDto, EventLogInput, SettlementEventDto, to_js_value},
    envelope::WasmEnvelope,
    errors::WasmError,
};

/// Decodes a `GPv2Settlement` event log into a typed settlement event.
///
/// Dispatches on the log's topic-0 across `Trade`, `Interaction`, `Settlement`,
/// `OrderInvalidated`, and `PreSignature`. The decode is fail-closed: the topic
/// set is validated before ABI decoding and every order UID is length-checked,
/// so a malformed or hostile log returns a typed error rather than panicking.
///
/// @param log Raw log with `topics` (0x-prefixed 32-byte hex, topic-0 first)
/// and `data` (0x-prefixed hex, `"0x"` when empty).
/// @returns A versioned envelope containing the decoded settlement event.
/// @throws CowError when the log is malformed or its topic set matches no known
/// settlement event.
#[wasm_bindgen(
    js_name = "decodeSettlementLog",
    unchecked_return_type = "WasmEnvelope<SettlementEventDto>"
)]
pub fn decode_settlement_log(log: EventLogInput) -> Result<JsValue, JsValue> {
    let log_data = log.to_log_data()?;
    let event = cow_sdk_contracts::decode_settlement_log(&log_data)
        .map_err(|error| WasmError::invalid("log", error.to_string()))?;
    let dto = SettlementEventDto::from_event(event)?;
    to_js_value(&WasmEnvelope::v1(dto))
}

/// Decodes an eth-flow on-chain order lifecycle event log into a typed event.
///
/// Dispatches on the log's topic-0 across the `CoWSwapOnchainOrders`
/// `OrderPlacement` / `OrderInvalidation` events and the `CoWSwapEthFlow`
/// `OrderRefund` event. The decode is fail-closed: the topic set and on-chain
/// signing scheme are validated and every order UID is length-checked, so a
/// malformed or hostile log returns a typed error rather than panicking.
///
/// @param log Raw log with `topics` (0x-prefixed 32-byte hex, topic-0 first)
/// and `data` (0x-prefixed hex, `"0x"` when empty).
/// @returns A versioned envelope containing the decoded eth-flow event.
/// @throws CowError when the log is malformed or its topic set matches no known
/// eth-flow lifecycle event.
#[wasm_bindgen(
    js_name = "decodeEthFlowLog",
    unchecked_return_type = "WasmEnvelope<EthFlowEventDto>"
)]
pub fn decode_eth_flow_log(log: EventLogInput) -> Result<JsValue, JsValue> {
    let log_data = log.to_log_data()?;
    let event = cow_sdk_contracts::decode_eth_flow_log(&log_data)
        .map_err(|error| WasmError::invalid("log", error.to_string()))?;
    let dto = EthFlowEventDto::from_event(event)?;
    to_js_value(&WasmEnvelope::v1(dto))
}
