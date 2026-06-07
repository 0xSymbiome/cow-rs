//! Consumer-facing receipt-polling lifecycle against the published test doubles.
//!
//! Drives `submit_and_wait_for_receipt` through `cow_sdk::testing` doubles
//! (`MockSigner` + `MockProvider`), whose `receipt_sequence` scripts each poll —
//! so a downstream test covers every receipt outcome offline: a receipt that
//! arrives after several polls, a reverted receipt surfaced as
//! `WaitError::Reverted`, and a never-mined transaction surfaced as
//! `WaitError::Timeout`. Short poll/timeout windows keep the run deterministic.

use std::error::Error;
use std::time::Duration;

use cow_sdk::core::{TransactionReceipt, TransactionRequest, TransactionStatus};
use cow_sdk::prelude::{Address, Amount};
use cow_sdk::testing::{MockProvider, MockSigner, defaults};
use cow_sdk::trading::{WaitError, WaitOptions, submit_and_wait_for_receipt};
use serde_json::json;

// Fast, deterministic polling so the example finishes promptly offline.
fn fast_wait(require_success: bool) -> WaitOptions {
    WaitOptions::new(Duration::from_millis(5), Duration::from_millis(200))
        .with_require_success(require_success)
}

fn mined_receipt(status: TransactionStatus) -> TransactionReceipt {
    TransactionReceipt::new(defaults::transaction_hash())
        .with_status(status)
        .with_block_number(1_234)
}

fn self_transfer() -> TransactionRequest {
    TransactionRequest::new(
        Some(Address::ZERO),
        None,
        Some(Amount::ZERO),
        Some(Amount::from(21_000u32)),
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let signer = MockSigner::new();
    let tx = self_transfer();

    // A. Mined after two "not yet" polls: the third scripted poll returns success.
    let provider = MockProvider::builder()
        .receipt_sequence([None, None, Some(mined_receipt(TransactionStatus::Success))])
        .build();
    let mined = submit_and_wait_for_receipt(&signer, &provider, &tx, fast_wait(true)).await?;

    // B. Reverted: success is required, so a reverted receipt surfaces as an error.
    let provider = MockProvider::builder()
        .receipt_sequence([Some(mined_receipt(TransactionStatus::Reverted))])
        .build();
    let reverted_outcome =
        match submit_and_wait_for_receipt(&signer, &provider, &tx, fast_wait(true)).await {
            Err(WaitError::Reverted { receipt }) => {
                format!("reverted at block {:?}", receipt.block_number)
            }
            other => format!("unexpected: {other:?}"),
        };

    // C. Never mined: the provider yields no receipt, so the wait times out.
    let provider = MockProvider::new();
    let timeout_outcome =
        match submit_and_wait_for_receipt(&signer, &provider, &tx, fast_wait(false)).await {
            Err(WaitError::Timeout { elapsed, .. }) => format!("timed out after {elapsed:?}"),
            other => format!("unexpected: {other:?}"),
        };

    let report = json!({
        "surface": "cow-sdk::trading::submit_and_wait_for_receipt with cow_sdk::testing doubles",
        "mode": "scripted-receipt-sequence",
        "minedAfterPolls": {
            "status": format!("{:?}", mined.status),
            "blockNumber": mined.block_number
        },
        "revertedOutcome": reverted_outcome,
        "timeoutOutcome": timeout_outcome
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
