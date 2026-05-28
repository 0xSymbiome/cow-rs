#![no_main]

//! Fuzz target for the `CoWSwapOnchainOrders` log decoders.
//!
//! **Property:** `PROP-CON-019`.
//! The target maps arbitrary bytes onto an event log (up to four 32-byte
//! topics drawn from the input head, the remainder as the data body) and feeds
//! it to [`decode_order_placement`] and [`decode_order_invalidation`]. The
//! decoders must always return `Ok`/`Err` and never panic — the fail-closed
//! contract for adversarial on-chain log input. Two extra passes force the
//! canonical `OrderPlacement` / `OrderInvalidation` topic-0 so the fuzzer also
//! drives the ABI-decode, marker-mapping, and owner-resolution paths with
//! hostile bodies.

use alloy_primitives::{B256, Bytes, LogData};
use alloy_sol_types::SolEvent;
use cow_sdk_contracts::{ICoWSwapOnchainOrders, decode_order_invalidation, decode_order_placement};
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
        let _ = decode_order_placement(&log);
        let _ = decode_order_invalidation(&log);
    }

    // 2. Force the canonical OrderPlacement topic-0 to drive the ABI-decode,
    //    marker-mapping, and owner-resolution paths with adversarial bodies.
    if let Some(log) = build_log(
        Some(ICoWSwapOnchainOrders::OrderPlacement::SIGNATURE_HASH),
        data,
    ) && let Ok(placement) = decode_order_placement(&log)
    {
        let _ = placement.resolve_owner();
    }

    // 3. Force the canonical OrderInvalidation topic-0.
    if let Some(log) = build_log(
        Some(ICoWSwapOnchainOrders::OrderInvalidation::SIGNATURE_HASH),
        data,
    ) {
        let _ = decode_order_invalidation(&log);
    }
});
