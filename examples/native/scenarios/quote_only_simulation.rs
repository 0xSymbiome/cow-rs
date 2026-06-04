use std::{error::Error, sync::Arc};

use serde_json::json;

use cow_sdk::prelude::{SupportedChainId, TraderParameters, TradingBuilder};
use cow_sdk::trading::TradingOptions;

use cow_sdk::testing::MockOrderbook;
use cow_sdk_examples_native::support::{sample_quote_response, sample_trade_parameters};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let orderbook = MockOrderbook::builder(SupportedChainId::Sepolia)
        .quote(sample_quote_response())
        .build();
    let trading = TradingBuilder::ready(
        TraderParameters::new(SupportedChainId::Sepolia, "cow-rs-quote-only")
            .expect("app code should validate"),
        TradingOptions::new().with_orderbook_client(Arc::new(orderbook.clone())),
    )?;

    let quote = trading.get_quote_only(sample_trade_parameters(), None).await?;
    let request = orderbook
        .recorded()
        .quote_requests
        .last()
        .cloned()
        .expect("example quote request must be captured");

    let report = json!({
        "surface": "cow-sdk::Trading::get_quote_only",
        "mode": "simulated-transport",
        "quote": {
            "id": quote.quote_response.id,
            "suggestedSlippageBps": quote.suggested_slippage_bps,
            "sellAmount": quote.quote_response.quote.sell_amount,
            "buyAmount": quote.quote_response.quote.buy_amount
        },
        "request": {
            "from": request.from.to_hex_string(),
            "receiver": request.receiver.as_ref().map(|address| address.to_hex_string()),
            "priceQuality": format!("{:?}", request.price_quality)
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
