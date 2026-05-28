//! Typestate `TradingSdkBuilder` walkthrough.
//!
//! This example shows the two compile-time-checked terminals on
//! [`cow_sdk_trading::TradingSdkBuilder`]:
//!
//! - [`cow_sdk_trading::TradingSdkBuilder::build_ready`] is only callable once
//!   the builder has reached the `<ChainIdSet, AppCodeSet>` typestate through
//!   explicit [`cow_sdk_trading::TradingSdkBuilder::with_chain_id`] and
//!   [`cow_sdk_trading::TradingSdkBuilder::with_app_code`] setters.
//! - [`cow_sdk_trading::TradingSdkBuilder::build_helper_only`] unlocks once a
//!   chain id is set and returns [`cow_sdk_trading::HelperOnlySdk`], a
//!   narrower type that exposes only chain-bound helpers.
//!
//! The example compiles without RPC credentials because every terminal used
//! here operates entirely on the builder state that the example itself
//! constructs.
//!
//! Run with:
//!
//! ```text
//! cargo run -p cow-sdk-trading --example typestate_builder_example
//! ```

use cow_sdk_core::SupportedChainId;
use cow_sdk_trading::TradingSdkBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ready-state path: chain id and app code satisfy the compile-time
    // prerequisites for `build_ready`, which only runs the injected
    // orderbook-binding validator at runtime.
    let ready_sdk = TradingSdkBuilder::new()
        .with_chain_id(SupportedChainId::Mainnet)
        .with_app_code("cow-rs/typestate-example")
        .build_ready()?;
    println!(
        "ready sdk built through the typestate path for chain {:?}",
        ready_sdk.trader_defaults().chain_id
    );

    // Helper-only path: only chain id is required. The returned SDK can drive
    // allowance reads, approval submission, pre-sign transaction
    // construction, and on-chain cancellation without ever exposing quote,
    // post, order-lookup, or off-chain cancellation methods.
    let helper_sdk = TradingSdkBuilder::new()
        .with_chain_id(SupportedChainId::Mainnet)
        .build_helper_only()?;
    assert_eq!(
        helper_sdk.trader_defaults().chain_id,
        Some(SupportedChainId::Mainnet)
    );
    assert!(helper_sdk.trader_defaults().app_code.is_none());
    println!("helper-only sdk exposes only chain-bound helpers");

    Ok(())
}
