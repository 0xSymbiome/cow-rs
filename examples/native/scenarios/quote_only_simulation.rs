use std::{error::Error, sync::Arc};

use serde_json::json;

use cow_sdk::{PartialTraderParameters, SupportedChainId, TradingSdk, TradingSdkOptions};

use cow_sdk_examples_native::support::{
    MockOrderbook, sample_quote_response, sample_trade_parameters,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let orderbook = MockOrderbook::new(SupportedChainId::Sepolia, sample_quote_response());
    let sdk = TradingSdk::new(
        PartialTraderParameters {
            chain_id: Some(SupportedChainId::Sepolia),
            app_code: Some("cow-rs-quote-only".to_owned()),
            owner: None,
            env: None,
            settlement_contract_override: None,
            eth_flow_contract_override: None,
        },
        TradingSdkOptions {
            order_book_api: Some(Arc::new(orderbook.clone())),
        },
    );

    let quote = sdk.get_quote_only(sample_trade_parameters(), None).await?;
    let request = orderbook
        .state()
        .quote_requests
        .last()
        .cloned()
        .expect("example quote request must be captured");

    let report = json!({
        "surface": "cow-sdk::TradingSdk::get_quote_only",
        "mode": "simulated-transport",
        "quote": {
            "id": quote.quote_response.id,
            "suggestedSlippageBps": quote.suggested_slippage_bps,
            "sellAmount": quote.quote_response.quote.sell_amount,
            "buyAmount": quote.quote_response.quote.buy_amount
        },
        "request": {
            "from": request.from.as_str(),
            "receiver": request.receiver.as_ref().map(|address| address.as_str()),
            "priceQuality": format!("{:?}", request.price_quality)
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
