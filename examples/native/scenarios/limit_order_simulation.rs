use std::{error::Error, sync::Arc};

use serde_json::json;

use cow_sdk::prelude::{SupportedChainId, Trading};

use cow_sdk_examples_native::support::{
    MockOrderbook, MockSigner, sample_limit_parameters, sample_owner, sample_quote_response,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let orderbook = MockOrderbook::new(SupportedChainId::Sepolia, sample_quote_response());
    let signer = MockSigner::default();
    let sdk = Trading::builder()
        .with_chain_id(SupportedChainId::Sepolia)
        .with_app_code("cow-rs-limit-order")
        .with_orderbook_client(Arc::new(orderbook.clone()))
        .build()?;

    let posted = sdk
        .post_limit_order(sample_limit_parameters(), &signer, None)
        .await?;
    let state = orderbook.state();
    let sent_order = state
        .sent_orders
        .first()
        .expect("example limit order must be sent");

    let report = json!({
        "surface": "cow-sdk::Trading::post_limit_order",
        "mode": "simulated-transport",
        "result": {
            "orderId": posted.order_id.to_hex_string(),
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
