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
//!   chain id is set and returns an SDK in [`cow_sdk_trading::TradingSdkMode::HelperOnly`]
//!   so quote, post, and off-chain cancellation flows fail closed with
//!   [`cow_sdk_trading::TradingError::HelperOnlyMode`] while chain-bound
//!   helpers remain fully usable.
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

use cow_sdk_core::{Address, SupportedChainId};
use cow_sdk_trading::{TradingError, TradingSdkBuilder, TradingSdkMode};

const OWNER: &str = "0xc8c753ee51e8fc80e199ab297fb575634a1ac1d3";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let owner = Address::new(OWNER)?;

    // Ready-state path: chain id and app code satisfy the compile-time
    // prerequisites for `build_ready`, which only runs the injected
    // orderbook-binding validator at runtime.
    let ready_sdk = TradingSdkBuilder::new()
        .with_chain_id(SupportedChainId::Mainnet)
        .with_app_code("cow-rs/typestate-example")
        .with_owner(owner.clone())
        .build_ready()?;
    assert_eq!(ready_sdk.mode(), TradingSdkMode::Ready);
    println!(
        "ready sdk built through the typestate path for chain {:?}",
        ready_sdk.trader_defaults().chain_id
    );

    // Helper-only path: only chain id is required. The returned SDK can drive
    // allowance reads, approval submission, pre-sign transaction
    // construction, and on-chain cancellation without ever exposing quote or
    // post workflows, which fail closed with `TradingError::HelperOnlyMode`.
    let helper_sdk = TradingSdkBuilder::new()
        .with_chain_id(SupportedChainId::Mainnet)
        .with_owner(owner)
        .build_helper_only()?;
    assert_eq!(helper_sdk.mode(), TradingSdkMode::HelperOnly);

    // A quote invocation on a helper-only sdk surfaces the typed
    // helper-only error rather than silently falling through to a
    // partially-configured flow.
    let trade_parameters = cow_sdk_trading::TradeParameters {
        kind: cow_sdk_core::OrderKind::Sell,
        owner: None,
        sell_token: Address::new("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2")?,
        sell_token_decimals: 18,
        buy_token: Address::new("0x6b175474e89094c44da98b954eedeac495271d0f")?,
        buy_token_decimals: 18,
        amount: cow_sdk_core::Amount::new("1000000000000000000")?,
        env: None,
        settlement_contract_override: None,
        eth_flow_contract_override: None,
        partially_fillable: false,
        sell_token_balance: cow_sdk_core::OrderBalance::Erc20,
        buy_token_balance: cow_sdk_core::OrderBalance::Erc20,
        slippage_bps: None,
        receiver: None,
        valid_for: None,
        valid_to: None,
        partner_fee: None,
    };

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let quote_error = runtime
        .block_on(helper_sdk.get_quote_only(trade_parameters, None))
        .expect_err("helper-only sdk must refuse the quote flow");
    assert!(matches!(quote_error, TradingError::HelperOnlyMode));
    println!("helper-only sdk correctly refused the quote flow");

    Ok(())
}
