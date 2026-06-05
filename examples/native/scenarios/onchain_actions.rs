//! On-chain order actions: pre-sign and on-chain cancellation.
//!
//! Builds a pre-sign transaction (`get_pre_sign_transaction`) and on-chain
//! cancellation call data (`onchain_cancellation_transaction`), then dispatches
//! an on-chain cancel (`Trading::on_chain_cancel_order`) for both a regular and
//! an EthFlow order, against a transport-mocked orderbook and signer. These are
//! the smart-contract paths, distinct from the off-chain signed cancellation.

use std::{error::Error, sync::Arc};

use serde_json::json;

use cow_sdk::core::HexData;
use cow_sdk::prelude::{SupportedChainId, Trading};
use cow_sdk::trading::{
    OrderTraderParameters, get_pre_sign_transaction,
    onchain_cancellation_transaction,
};

use cow_sdk::testing::{MockOrderbook, MockSigner};
use cow_sdk_examples_native::support::{
    sample_open_order, sample_order_uid, sample_owner, sample_quote_response,
    sample_trader_parameters, text_preview,
};

fn call_data_prefix(data: &HexData) -> String {
    text_preview(&data.to_hex_string(), 10).to_owned()
}

fn sample_ethflow_order() -> cow_sdk::orderbook::Order {
    let mut order = sample_open_order();
    order.ethflow_data = Some(cow_sdk::orderbook::EthflowData::new(order.valid_to));
    order
}

fn trading_sdk(orderbook: MockOrderbook) -> Trading {
    let trader = sample_trader_parameters();
    let mut builder = Trading::builder()
        .chain_id(trader.chain_id)
        .app_code(trader.app_code);
    if let Some(env) = trader.env {
        builder = builder.env(env);
    }

    builder
        .orderbook_client(Arc::new(orderbook))
        .build()
        .expect("example trading sdk construction should succeed")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let chain_id = SupportedChainId::Sepolia;
    let preview_signer = MockSigner::builder().address(sample_owner()).build();
    let order_uid = sample_order_uid();
    let params = OrderTraderParameters::new(order_uid).with_chain_id(chain_id);

    // Build call data only (no dispatch): a pre-sign transaction, plus cancellation
    // call data for a regular order and an EthFlow order — these take different routes.
    let pre_sign = get_pre_sign_transaction(&preview_signer, chain_id, &order_uid, None).await?;
    let regular_preview =
        onchain_cancellation_transaction(&preview_signer, chain_id, &sample_open_order(), None)
            .await?;
    let ethflow_preview =
        onchain_cancellation_transaction(&preview_signer, chain_id, &sample_ethflow_order(), None)
            .await?;

    // Dispatch a real on-chain cancel for a regular order; the SDK routes it through
    // the settlement contract and the signer records the sent transaction.
    let regular_orderbook = MockOrderbook::builder(chain_id)
        .quote(sample_quote_response())
        .build();
    regular_orderbook.push_order(sample_open_order());
    let regular_signer = MockSigner::builder().address(sample_owner()).build();
    let regular_sdk = trading_sdk(regular_orderbook);
    let regular_hash = regular_sdk
        .on_chain_cancel_order(&params, &regular_signer)
        .await?;
    let regular_sent = regular_signer
        .recorded()
        .sent_transactions
        .last()
        .cloned()
        .expect("regular cancellation should send a transaction");

    // Same for an EthFlow order — the SDK routes this through the eth-flow contract.
    let ethflow_orderbook = MockOrderbook::builder(chain_id)
        .quote(sample_quote_response())
        .build();
    ethflow_orderbook.push_order(sample_ethflow_order());
    let ethflow_signer = MockSigner::builder().address(sample_owner()).build();
    let ethflow_sdk = trading_sdk(ethflow_orderbook);
    let ethflow_hash = ethflow_sdk
        .on_chain_cancel_order(&params, &ethflow_signer)
        .await?;
    let ethflow_sent = ethflow_signer
        .recorded()
        .sent_transactions
        .last()
        .cloned()
        .expect("ethflow cancellation should send a transaction");

    let report = json!({
        "surface": "cow-sdk::trading::get_pre_sign_transaction + cow-sdk::Trading::on_chain_cancel_order",
        "mode": "simulated-transport",
        "preSignTransaction": {
            "orderUid": order_uid.to_hex_string(),
            "contract": pre_sign.to.as_ref().map(|address| address.to_hex_string()),
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
                "contract": regular_preview.to.as_ref().map(|address| address.to_hex_string()),
                "callDataPrefix": call_data_prefix(
                    regular_preview
                        .data
                        .as_ref()
                        .expect("regular preview should include call data"),
                )
            },
            "ethFlowOrder": {
                "route": "eth-flow",
                "contract": ethflow_preview.to.as_ref().map(|address| address.to_hex_string()),
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
                "txHash": regular_hash.to_hex_string(),
                "contract": regular_sent.to.as_ref().map(|address| address.to_hex_string()),
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
                "txHash": ethflow_hash.to_hex_string(),
                "contract": ethflow_sent.to.as_ref().map(|address| address.to_hex_string()),
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
