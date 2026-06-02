use std::{error::Error, sync::Arc};

use serde_json::json;

use cow_sdk::core::Amount;
use cow_sdk::prelude::{SupportedChainId, Trading};
use cow_sdk::trading::{
    AllowanceParameters, ApprovalParameters, OrderTraderParameters,
};

use cow_sdk_examples_native::support::{
    MockOrderbook, MockProvider, MockSigner, sample_owner, sample_quote_response,
    sample_sell_token, sample_trade_parameters,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let orderbook = MockOrderbook::new(SupportedChainId::Sepolia, sample_quote_response());
    let signer = MockSigner::default();
    let mut provider = MockProvider::default();
    provider.signer = Some(signer.clone());

    let sdk = Trading::builder()
        .with_chain_id(SupportedChainId::Sepolia)
        .with_app_code("cow-rs-native-examples")
        .with_orderbook_client(Arc::new(orderbook.clone()))
        .build()?;

    let quote = sdk
        .get_quote_results(sample_trade_parameters(), &signer, None)
        .await?;
    let post_result = sdk
        .post_swap_order(sample_trade_parameters(), &signer, None)
        .await?;
    let allowance = sdk
        .get_cow_protocol_allowance(
            &provider,
            &AllowanceParameters::new(sample_sell_token(), sample_owner()),
        )
        .await?;
    let approval_tx_hash = sdk
        .approve_cow_protocol(
            &signer,
            &ApprovalParameters::new(
                sample_sell_token(),
                Amount::from_units(1, 18)
                    .expect("example approval amount must remain valid"),
            ),
        )
        .await?;
    let cancelled = sdk
        .off_chain_cancel_order(
            &OrderTraderParameters::new(post_result.order_id.clone()),
            &signer,
        )
        .await?;

    let orderbook_state = orderbook.state();
    let signer_state = signer.state();
    let provider_state = provider.state();

    let report = json!({
        "surface": "cow-sdk::Trading",
        "mode": "simulated-transport",
        "quote": {
            "suggestedSlippageBps": quote.suggested_slippage_bps,
            "appDataHex": quote.app_data_info.app_data_keccak256.to_hex_string()
        },
        "post": {
            "orderId": post_result.order_id.to_hex_string(),
            "signatureLength": post_result.signature.len(),
            "uploadedAppDataCount": orderbook_state.uploads.len(),
            "sentOrderCount": orderbook_state.sent_orders.len()
        },
        "allowanceAndApproval": {
            "allowance": allowance,
            "approvalTxHash": approval_tx_hash,
            "approvalContractRead": provider_state.last_contract_call.as_ref().map(|call| {
                json!({
                    "address": call.address.to_hex_string(),
                    "method": call.method
                })
            })
        },
        "cancellation": {
            "dispatched": cancelled,
            "signedCancellationCount": orderbook_state.cancellations.len()
        },
        "signer": {
            "sentTransactionCount": signer_state.sent_transactions.len()
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
