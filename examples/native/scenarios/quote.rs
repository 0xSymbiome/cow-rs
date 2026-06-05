//! Quote without submission.
//!
//! Requests a price through `Trading::get_quote_only` against a transport-mocked
//! orderbook — the shortest read-only path for a consumer that wants a quote
//! without building, signing, or posting an order.

use std::{error::Error, sync::Arc};

use serde_json::json;

use cow_sdk::prelude::{SupportedChainId, TraderParameters, TradingBuilder};
use cow_sdk::trading::TradingOptions;

use cow_sdk::testing::MockOrderbook;
use cow_sdk_examples_native::support::{sample_quote_response, sample_trade_parameters};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Transport-mocked orderbook seeded with a canned quote response.
    let orderbook = MockOrderbook::builder(SupportedChainId::Sepolia)
        .quote(sample_quote_response())
        .build();

    // Ready-state client with the mock injected. `orderbook.clone()` keeps a
    // handle so we can read what the client sent after the call.
    let trading = TradingBuilder::ready(
        TraderParameters::new(SupportedChainId::Sepolia, "cow-rs-quote-only")
            .expect("app code should validate"),
        TradingOptions::new().with_orderbook_client(Arc::new(orderbook.clone())),
    )?;

    // Quote only — no order is built, signed, or posted.
    let quote = trading.get_quote_only(sample_trade_parameters(), None).await?;

    // Inspect the request the client actually sent to the orderbook.
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
