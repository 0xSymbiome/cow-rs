use std::{error::Error, sync::Arc};

use serde_json::json;

use cow_sdk::{PartialTraderParameters, SupportedChainId, TradingSdk, TradingSdkOptions};

use cow_sdk_examples_native::support::{
    MockOrderbook, MockSigner, sample_limit_parameters, sample_owner, sample_quote_response,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let orderbook = MockOrderbook::new(SupportedChainId::Sepolia, sample_quote_response());
    let signer = MockSigner::default();
    let sdk = TradingSdk::new(
        PartialTraderParameters {
            chain_id: Some(SupportedChainId::Sepolia),
            app_code: Some("cow-rs-limit-order".to_owned()),
            owner: Some(sample_owner()),
            env: None,
            settlement_contract_override: None,
            eth_flow_contract_override: None,
        },
        TradingSdkOptions::new().with_orderbook_client(Arc::new(orderbook.clone())),
    )?;

    let posted = sdk
        .post_limit_order(sample_limit_parameters(), &signer, None)
        .await?;
    let state = orderbook.state();
    let sent_order = state
        .sent_orders
        .first()
        .expect("example limit order must be sent");

    let report = json!({
        "surface": "cow-sdk::TradingSdk::post_limit_order",
        "mode": "simulated-transport",
        "result": {
            "orderId": posted.order_id.as_str(),
            "signatureLength": posted.signature.len(),
            "signingScheme": format!("{:?}", posted.signing_scheme)
        },
        "submission": {
            "quoteId": sent_order.quote_id,
            "sellAmount": sent_order.sell_amount,
            "buyAmount": sent_order.buy_amount,
            "uploadedAppDataCount": state.uploads.len()
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
