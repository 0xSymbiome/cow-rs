use std::{error::Error, sync::Arc};

use serde_json::json;

use cow_sdk::prelude::{SupportedChainId, Trading};
use cow_sdk::trading::OrderTraderParameters;

use cow_sdk::testing::{MockOrderbook, MockSigner};
use cow_sdk_examples_native::support::{
    sample_open_order, sample_order_uid, sample_owner, sample_quote_response,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let orderbook = MockOrderbook::builder(SupportedChainId::Sepolia)
        .quote(sample_quote_response())
        .build();
    orderbook.push_order(sample_open_order());
    let signer = MockSigner::builder().address(sample_owner()).build();
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("cow-rs-order-lifecycle")
        .orderbook_client(Arc::new(orderbook.clone()))
        .build()?;

    let params = OrderTraderParameters::new(sample_order_uid());

    let order = trading.get_order(&params).await?;
    let cancelled = trading.off_chain_cancel_order(&params, &signer).await?;
    let state = orderbook.recorded();

    let report = json!({
        "surface": "cow-sdk::Trading::order_lifecycle",
        "mode": "simulated-transport",
        "order": {
            "uid": order.uid.to_hex_string(),
            "owner": order.owner.to_hex_string(),
            "status": format!("{:?}", order.status),
            "kind": format!("{:?}", order.kind)
        },
        "cancellation": {
            "dispatched": cancelled,
            "signedCancellationCount": state.cancellations.len()
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
