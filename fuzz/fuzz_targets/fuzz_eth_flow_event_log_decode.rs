#![no_main]

//! Fuzz target for the eth-flow lifecycle event log decoder.
//!
//! The target maps arbitrary bytes onto an event log — up to four 32-byte topics
//! drawn from the input head, the remainder as the data body — and feeds it to
//! [`decode_eth_flow_log`]. Three extra passes force each canonical eth-flow
//! topic-0 (`OrderPlacement`, `OrderInvalidation`, `OrderRefund`) so the
//! ABI-decode, marker-mapping, owner-resolution, and order-UID length-check
//! paths are also driven. The decoder must always return `Ok`/`Err` and never
//! panic — the fail-closed contract for adversarial on-chain log input.

use alloy_primitives::{B256, Bytes, LogData};
use alloy_sol_types::SolEvent;
use cow_sdk_contracts::{ICoWSwapEthFlowEvents, ICoWSwapOnchainOrders, decode_eth_flow_log};
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
        let _ = decode_eth_flow_log(&log);
    }

    // 2. Force each canonical eth-flow lifecycle topic-0 to drive the per-event
    //    decode paths with adversarial bodies.
    for topic0 in [
        ICoWSwapOnchainOrders::OrderPlacement::SIGNATURE_HASH,
        ICoWSwapOnchainOrders::OrderInvalidation::SIGNATURE_HASH,
        ICoWSwapEthFlowEvents::OrderRefund::SIGNATURE_HASH,
    ] {
        if let Some(log) = build_log(Some(topic0), data) {
            let _ = decode_eth_flow_log(&log);
        }
    }
});
