//! Typestate `TradingBuilder` walkthrough.
//!
//! This example shows the compile-time-checked ready-state terminal on
//! [`cow_sdk_trading::TradingBuilder`]:
//!
//! - [`cow_sdk_trading::TradingBuilder::build`] is only callable once the
//!   builder has reached the `<ChainIdSet, AppCodeSet>` typestate through the
//!   explicit [`cow_sdk_trading::TradingBuilder::with_chain_id`] and
//!   [`cow_sdk_trading::TradingBuilder::with_app_code`] setters. Calling it
//!   before both prerequisites are supplied is a compile error.
//!
//! Chain-bound helper flows that need no app code — allowance reads, approval
//! submission, pre-sign transaction construction, and on-chain cancellation —
//! use the crate's free functions directly (for example `get_cow_protocol_allowance`
//! and `approval_transaction`), so they require no trading client at all.
//!
//! The example compiles without RPC credentials because the terminal used here
//! operates entirely on the builder state that the example itself constructs.
//!
//! Run with:
//!
//! ```text
//! cargo run -p cow-sdk-trading --example typestate_builder_example
//! ```

use cow_sdk_core::SupportedChainId;
use cow_sdk_trading::TradingBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ready-state path: chain id and app code satisfy the compile-time
    // prerequisites for `build`, which only runs the injected
    // orderbook-binding validator at runtime.
    let ready_sdk = TradingBuilder::new()
        .with_chain_id(SupportedChainId::Mainnet)
        .with_app_code("cow-rs/typestate-example")
        .build()?;
    println!(
        "ready sdk built through the typestate path for chain {:?}",
        ready_sdk.trader_defaults().chain_id
    );

    Ok(())
}
