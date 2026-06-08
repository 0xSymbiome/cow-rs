//! Single-order lookup and off-chain cancellation.
//!
//! Looks up an order by uid (`Trading::order`) and cancels it off-chain
//! (`Trading::off_chain_cancel_order`) through a transport-mocked orderbook and
//! signer, inspecting the signed cancellation the SDK records. Off-chain
//! cancellation is a signed API call, not an on-chain transaction.

use std::{error::Error, sync::Arc};

use serde_json::json;

use cow_sdk::core::SupportedChainId;
use cow_sdk::trading::{OrderTraderParameters, Trading};

use cow_sdk::testing::{MockOrderbook, MockSigner};
use cow_sdk_examples_native::support::{
    sample_open_order, sample_order_uid, sample_owner, sample_quote_response,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Transport-mocked orderbook seeded with one open order to look up.
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

    // Both calls key off the order uid.
    let params = OrderTraderParameters::new(sample_order_uid());

    // Fetch the order, then cancel it off-chain — a signed API call, not a transaction.
    let order = trading.order(&params).await?;
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
