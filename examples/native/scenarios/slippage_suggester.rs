//! Custom slippage suggestion through the public `SlippageSuggester` seam.
//!
//! Implements `SlippageSuggester` and wires it through
//! `TradeAdvancedSettings::with_slippage_suggester`, then quotes with
//! `Trading::quote_results` against the `cow_sdk::testing` doubles. The report
//! contrasts the SDK's default suggestion with the consumer-supplied one to show
//! the seam takes effect; leaving the price quality at its default keeps the
//! suggester in the quote path (an explicit `Fast` quality would bypass it).

use std::error::Error;

use serde_json::json;

use cow_sdk::core::SupportedChainId;
use cow_sdk::trading::{
    SlippageSuggester, SlippageToleranceRequest, SlippageToleranceResponse, TradeAdvancedSettings,
    Trading, TradingError, async_trait,
};

use cow_sdk::testing::{MockOrderbook, MockSigner};
use cow_sdk_examples_native::support::{OWNER, sample_quote_response, sample_trade_parameters};

/// A consumer slippage policy that always suggests a fixed tolerance.
struct StaticSlippageProvider {
    bps: u32,
}

#[async_trait]
impl SlippageSuggester for StaticSlippageProvider {
    async fn slippage_suggestion(
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
    let signer = MockSigner::builder().address(OWNER).build();
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("cow-rs-native-examples")
        .orderbook(orderbook)
        .build()?;

    // Baseline: no suggester, so the SDK derives its own default suggestion.
    let baseline = trading
        .quote_results(sample_trade_parameters(), &signer, None)
        .await?;

    // With a consumer-supplied suggester wired through the advanced settings.
    let advanced =
        TradeAdvancedSettings::new().with_slippage_suggester(StaticSlippageProvider { bps: 200 });
    let suggested = trading
        .quote_results(sample_trade_parameters(), &signer, Some(&advanced))
        .await?;

    let report = json!({
        "surface": "cow_sdk::trading::SlippageSuggester",
        "mode": "simulated-transport",
        "providerSuggestionBps": 200,
        "defaultSuggestedSlippageBps": baseline.suggested_slippage_bps,
        "customSuggestedSlippageBps": suggested.suggested_slippage_bps
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
