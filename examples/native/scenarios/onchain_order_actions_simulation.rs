use std::{error::Error, sync::Arc};

use serde_json::json;

use cow_sdk::trading::{get_pre_sign_transaction, onchain_cancellation_transaction};
use cow_sdk::{OrderTraderParameters, PartialTraderParameters, TradingSdk, TradingSdkOptions};

use cow_sdk_examples_native::support::{
    MockOrderbook, MockSigner, sample_open_order, sample_order_uid, sample_owner,
    sample_quote_response, sample_trader_parameters, text_preview,
};

fn call_data_prefix(data: &cow_sdk::HexData) -> &str {
    text_preview(data.as_str(), 10)
}

fn sample_ethflow_order() -> cow_sdk::orderbook::Order {
    let mut order = sample_open_order();
    order.ethflow_data = Some(cow_sdk::orderbook::EthflowData::new(order.valid_to));
    order
}

fn trading_sdk(orderbook: MockOrderbook) -> TradingSdk {
    let trader = sample_trader_parameters();
    let mut partial = PartialTraderParameters::new()
        .with_chain_id(trader.chain_id)
        .with_app_code(trader.app_code)
        .with_owner(sample_owner());
    if let Some(env) = trader.env {
        partial = partial.with_env(env);
    }

    TradingSdk::new(
        partial,
        TradingSdkOptions::new().with_orderbook_client(Arc::new(orderbook)),
    )
    .expect("example trading sdk construction should succeed")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let chain_id = cow_sdk::SupportedChainId::Sepolia;
    let preview_signer = MockSigner::default();
    let order_uid = sample_order_uid();
    let params = OrderTraderParameters::new(order_uid.clone()).with_chain_id(chain_id);

    let pre_sign = get_pre_sign_transaction(&preview_signer, chain_id, &order_uid, None)?;
    let regular_preview =
        onchain_cancellation_transaction(&preview_signer, chain_id, &sample_open_order(), None)?;
    let ethflow_preview =
        onchain_cancellation_transaction(&preview_signer, chain_id, &sample_ethflow_order(), None)?;

    let regular_orderbook = MockOrderbook::new(chain_id, sample_quote_response());
    regular_orderbook.push_order(sample_open_order());
    let regular_signer = MockSigner::default();
    let regular_sdk = trading_sdk(regular_orderbook);
    let regular_hash = regular_sdk
        .on_chain_cancel_order(&params, &regular_signer)
        .await?;
    let regular_sent = regular_signer
        .state()
        .sent_transactions
        .last()
        .cloned()
        .expect("regular cancellation should send a transaction");

    let ethflow_orderbook = MockOrderbook::new(chain_id, sample_quote_response());
    ethflow_orderbook.push_order(sample_ethflow_order());
    let ethflow_signer = MockSigner::default();
    let ethflow_sdk = trading_sdk(ethflow_orderbook);
    let ethflow_hash = ethflow_sdk
        .on_chain_cancel_order(&params, &ethflow_signer)
        .await?;
    let ethflow_sent = ethflow_signer
        .state()
        .sent_transactions
        .last()
        .cloned()
        .expect("ethflow cancellation should send a transaction");

    let report = json!({
        "surface": "cow-sdk::trading::get_pre_sign_transaction + cow-sdk::TradingSdk::on_chain_cancel_order",
        "mode": "simulated-transport",
        "preSignTransaction": {
            "orderUid": order_uid.as_str(),
            "contract": pre_sign.to.as_ref().map(|address| address.as_str()),
            "value": pre_sign.value.as_ref().map(ToString::to_string),
            "gasLimit": pre_sign.gas_limit.as_ref().map(ToString::to_string),
            "callDataPrefix": call_data_prefix(
                pre_sign
                    .data
                    .as_ref()
                    .expect("pre-sign transaction should include call data"),
            )
        },
        "cancellationPreview": {
            "regularOrder": {
                "route": "settlement",
                "contract": regular_preview.to.as_ref().map(|address| address.as_str()),
                "callDataPrefix": call_data_prefix(
                    regular_preview
                        .data
                        .as_ref()
                        .expect("regular preview should include call data"),
                )
            },
            "ethFlowOrder": {
                "route": "eth-flow",
                "contract": ethflow_preview.to.as_ref().map(|address| address.as_str()),
                "callDataPrefix": call_data_prefix(
                    ethflow_preview
                        .data
                        .as_ref()
                        .expect("ethflow preview should include call data"),
                )
            }
        },
        "cancellationDispatch": {
            "regularOrder": {
                "route": "settlement",
                "txHash": regular_hash.as_str(),
                "contract": regular_sent.to.as_ref().map(|address| address.as_str()),
                "gasLimit": regular_sent.gas_limit.as_ref().map(ToString::to_string),
                "callDataPrefix": call_data_prefix(
                    regular_sent
                        .data
                        .as_ref()
                        .expect("regular cancellation should include call data"),
                )
            },
            "ethFlowOrder": {
                "route": "eth-flow",
                "txHash": ethflow_hash.as_str(),
                "contract": ethflow_sent.to.as_ref().map(|address| address.as_str()),
                "gasLimit": ethflow_sent.gas_limit.as_ref().map(ToString::to_string),
                "callDataPrefix": call_data_prefix(
                    ethflow_sent
                        .data
                        .as_ref()
                        .expect("ethflow cancellation should include call data"),
                )
            }
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
