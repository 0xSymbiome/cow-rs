//! On-chain order actions: pre-sign and on-chain cancellation.
//!
//! Builds a pre-sign transaction (`pre_sign_transaction`) and on-chain
//! cancellation call data (`onchain_cancellation_transaction`), then dispatches
//! an on-chain cancel (`Trading::onchain_cancel_order`) for both a regular and
//! an `EthFlow` order, against a transport-mocked orderbook and signer. These are
//! the smart-contract paths, distinct from the off-chain signed cancellation.

#![allow(
    clippy::redundant_closure_for_method_calls,
    clippy::too_many_lines,
    reason = "example scenario: a linear end-to-end narrative whose explicit `|value| value.to_hex_string()` closures read better for a learner than fully-qualified method references"
)]

use std::error::Error;

use serde_json::json;

use cow_sdk::core::SupportedChainId;
use cow_sdk::trading::{
    OrderTraderParams, Trading, onchain_cancellation_transaction, pre_sign_transaction,
};

use cow_sdk::testing::{MockOrderbook, MockSigner};
use cow_sdk_examples_native::support::{
    OWNER, call_data_prefix, sample_open_order, sample_order_uid, sample_quote_response,
    sample_trader_parameters,
};

fn sample_ethflow_order() -> cow_sdk::orderbook::Order {
    let mut order = sample_open_order();
    order.ethflow_data = Some(cow_sdk::orderbook::EthflowData::new(order.valid_to));
    order
}

fn trading_client(orderbook: MockOrderbook) -> Trading {
    let trader = sample_trader_parameters();
    Trading::builder()
        .chain_id(trader.chain_id)
        .app_code(trader.app_code)
        .orderbook(orderbook)
        .build()
        .expect("example trading client construction should succeed")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let chain_id = SupportedChainId::Sepolia;
    let preview_signer = MockSigner::builder().address(OWNER).build();
    let order_uid = sample_order_uid();
    let params = OrderTraderParams::new(order_uid).with_chain_id(chain_id);

    // Build call data only (no dispatch): a pre-sign transaction, plus cancellation
    // call data for a regular order and an EthFlow order — these take different routes.
    let pre_sign = pre_sign_transaction(&preview_signer, chain_id, &order_uid, None).await?;
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
    let regular_signer = MockSigner::builder().address(OWNER).build();
    let regular_trading = trading_client(regular_orderbook);
    let regular_hash = regular_trading
        .onchain_cancel_order(&params, &regular_signer)
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
    let ethflow_signer = MockSigner::builder().address(OWNER).build();
    let ethflow_trading = trading_client(ethflow_orderbook);
    let ethflow_hash = ethflow_trading
        .onchain_cancel_order(&params, &ethflow_signer)
        .await?;
    let ethflow_sent = ethflow_signer
        .recorded()
        .sent_transactions
        .last()
        .cloned()
        .expect("ethflow cancellation should send a transaction");

    // `pre_sign_transaction` returns a concrete `PreparedTransaction`: every
    // field is unconditionally set, so the report reads them directly.
    let report = json!({
        "surface": "cow_sdk::trading::pre_sign_transaction + cow_sdk::trading::Trading::onchain_cancel_order",
        "mode": "simulated-transport",
        "preSignTransaction": {
            "orderUid": order_uid.to_hex_string(),
            "contract": pre_sign.to.to_hex_string(),
            "value": pre_sign.value.to_string(),
            "gasLimit": pre_sign.gas_limit.to_string(),
            "callDataPrefix": call_data_prefix(&pre_sign.data)
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
