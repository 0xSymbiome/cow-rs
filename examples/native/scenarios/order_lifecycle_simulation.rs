use std::{error::Error, sync::Arc};

use serde_json::json;

use cow_sdk::prelude::{SupportedChainId, TradingSdk};
use cow_sdk::trading::{OrderTraderParameters, PartialTraderParameters, TradingSdkOptions};

use cow_sdk_examples_native::support::{
    MockOrderbook, MockSigner, sample_open_order, sample_order_uid, sample_owner,
    sample_quote_response,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let orderbook = MockOrderbook::new(SupportedChainId::Sepolia, sample_quote_response());
    orderbook.push_order(sample_open_order());
    let signer = MockSigner::default();
    let sdk = TradingSdk::new(
        PartialTraderParameters::new()
            .with_chain_id(SupportedChainId::Sepolia)
            .with_app_code("cow-rs-order-lifecycle".to_owned())
            .with_owner(sample_owner()),
        TradingSdkOptions::new().with_orderbook_client(Arc::new(orderbook.clone())),
    )?;

    let params = OrderTraderParameters::new(sample_order_uid());

    let order = sdk.get_order(&params).await?;
    let cancelled = sdk.off_chain_cancel_order(&params, &signer).await?;
    let state = orderbook.state();

    let report = json!({
        "surface": "cow-sdk::TradingSdk::order_lifecycle",
        "mode": "simulated-transport",
        "order": {
            "uid": order.uid.as_str(),
            "owner": order.owner.as_str(),
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
