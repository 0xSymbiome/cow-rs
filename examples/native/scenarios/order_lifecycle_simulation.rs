use std::{error::Error, sync::Arc};

use serde_json::json;

use cow_sdk::{
    OrderTraderParameters, PartialTraderParameters, SupportedChainId, TradingSdk, TradingSdkOptions,
};

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
        PartialTraderParameters {
            chain_id: Some(SupportedChainId::Sepolia),
            app_code: Some("cow-rs-order-lifecycle".to_owned()),
            owner: Some(sample_owner()),
            env: None,
            settlement_contract_override: None,
            eth_flow_contract_override: None,
        },
        TradingSdkOptions {
            order_book_api: Some(Arc::new(orderbook.clone())),
        },
    );

    let params = OrderTraderParameters {
        order_uid: sample_order_uid(),
        chain_id: None,
        env: None,
        settlement_contract_override: None,
        eth_flow_contract_override: None,
    };

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
