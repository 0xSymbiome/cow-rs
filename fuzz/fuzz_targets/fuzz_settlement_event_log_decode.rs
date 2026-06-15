#![no_main]

//! Fuzz target for the `GPv2Settlement` event log decoder.
//!
//! The target maps arbitrary bytes onto an event log (up to four 32-byte topics
//! drawn from the input head, the remainder as the data body) and feeds it to
//! [`decode_settlement_log`]. The decoder must always return `Ok`/`Err` and
//! never panic — the fail-closed contract for adversarial on-chain log input.
//! Five extra passes force each canonical settlement event topic-0 so the
//! fuzzer also drives the ABI-decode and order-UID length-check paths with
//! hostile bodies.

use alloy_primitives::{B256, Bytes, LogData};
use alloy_sol_types::SolEvent;
use cow_sdk_contracts::{IGPv2SettlementEvents, decode_settlement_log};
use libfuzzer_sys::fuzz_target;

/// Builds a candidate log from `topic0` plus up to three further topics drawn
/// from the input head, with the remaining bytes as the data body. Returns
/// `None` only when [`LogData::new`] rejects the topic set (never panics).
fn build_log(topic0: Option<B256>, rest: &[u8]) -> Option<LogData> {
    let mut topics: Vec<B256> = Vec::new();
    if let Some(topic) = topic0 {
        topics.push(topic);
    }
    let mut body = rest;
    while topics.len() < 4 && body.len() >= 32 {
        let (head, tail) = body.split_at(32);
        topics.push(B256::from_slice(head));
        body = tail;
    }
    LogData::new(topics, Bytes::copy_from_slice(body))
}

fuzz_target!(|data: &[u8]| {
    // 1. Fully arbitrary topics + body.
    if let Some(log) = build_log(None, data) {
        let _ = decode_settlement_log(&log);
    }

    // 2. Force each canonical settlement event topic-0 to drive the ABI-decode
    //    and order-UID length-check paths with adversarial bodies.
    for topic0 in [
        IGPv2SettlementEvents::Trade::SIGNATURE_HASH,
        IGPv2SettlementEvents::Interaction::SIGNATURE_HASH,
        IGPv2SettlementEvents::Settlement::SIGNATURE_HASH,
        IGPv2SettlementEvents::OrderInvalidated::SIGNATURE_HASH,
        IGPv2SettlementEvents::PreSignature::SIGNATURE_HASH,
    ] {
        if let Some(log) = build_log(Some(topic0), data) {
            let _ = decode_settlement_log(&log);
        }
    }
});
