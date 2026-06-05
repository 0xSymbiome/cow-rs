//! Custom slippage suggestion through the public `SlippageSuggestionProvider` seam.
//!
//! Implements `SlippageSuggestionProvider` and wires it through
//! `TradeAdvancedSettings::with_slippage_suggester`, then quotes with
//! `Trading::get_quote_results` against the `cow_sdk::testing` doubles. The report
//! contrasts the SDK's default suggestion with the consumer-supplied one to show
//! the seam takes effect; leaving the price quality at its default keeps the
//! suggester in the quote path (an explicit `Fast` quality would bypass it).

use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;

use cow_sdk::prelude::{SupportedChainId, Trading};
use cow_sdk::trading::{
    SlippageSuggestionProvider, SlippageToleranceRequest, SlippageToleranceResponse,
    TradeAdvancedSettings, TradingError,
};

use cow_sdk::testing::{MockOrderbook, MockSigner};
use cow_sdk_examples_native::support::{
    sample_owner, sample_quote_response, sample_trade_parameters,
};

/// A consumer slippage policy that always suggests a fixed tolerance.
struct StaticSlippageProvider {
    bps: u32,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl SlippageSuggestionProvider for StaticSlippageProvider {
    async fn get_slippage_suggestion(
        &self,
        _request: SlippageToleranceRequest,
    ) -> Result<SlippageToleranceResponse, TradingError> {
        Ok(SlippageToleranceResponse::new().with_slippage_bps(self.bps))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let orderbook = MockOrderbook::builder(SupportedChainId::Sepolia)
        .quote(sample_quote_response())
        .build();
    let signer = MockSigner::builder().address(sample_owner()).build();
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("cow-rs-native-examples")
        .orderbook_client(Arc::new(orderbook.clone()))
        .build()?;

    // Baseline: no suggester, so the SDK derives its own default suggestion.
    let baseline = trading
        .get_quote_results(sample_trade_parameters(), &signer, None)
        .await?;

    // With a consumer-supplied suggester wired through the advanced settings.
    let advanced = TradeAdvancedSettings::new()
        .with_slippage_suggester(Arc::new(StaticSlippageProvider { bps: 200 }));
    let suggested = trading
        .get_quote_results(sample_trade_parameters(), &signer, Some(&advanced))
        .await?;

    let report = json!({
        "surface": "cow-sdk::trading::SlippageSuggestionProvider",
        "mode": "simulated-transport",
        "providerSuggestionBps": 200,
        "defaultSuggestedSlippageBps": baseline.suggested_slippage_bps,
        "customSuggestedSlippageBps": suggested.suggested_slippage_bps
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
