use std::{error::Error, sync::Arc};

use serde_json::json;

use cow_sdk::core::{Amount, Provider};
use cow_sdk::prelude::{SupportedChainId, TradingSdk};
use cow_sdk::trading::{
    AllowanceParameters, ApprovalParameters, OrderTraderParameters, TradingSdkOptions,
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
    provider.set_signer(signer.clone());

    let sdk = TradingSdk::builder()
        .with_chain_id(SupportedChainId::Sepolia)
        .with_app_code("cow-rs-native-examples")
        .with_owner(sample_owner())
        .with_options(TradingSdkOptions::new().with_orderbook_client(Arc::new(
            orderbook.clone(),
        )))
        .build_ready()?;

    let quote = sdk
        .get_quote_results(sample_trade_parameters(), &signer, None)
        .await?;
    let post_result = sdk
        .post_swap_order(sample_trade_parameters(), &signer, None)
        .await?;
    let allowance = sdk.get_cow_protocol_allowance(
        &provider,
        &AllowanceParameters::new(sample_sell_token(), sample_owner()),
    )?;
    let approval_tx_hash = sdk.approve_cow_protocol(
        &signer,
        &ApprovalParameters::new(
            sample_sell_token(),
            Amount::new("1000000000000000000").expect("example approval amount must remain valid"),
        ),
    )?;
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
        "surface": "cow-sdk::TradingSdk",
        "mode": "simulated-transport",
        "quote": {
            "suggestedSlippageBps": quote.suggested_slippage_bps,
            "appDataHex": quote.app_data_info.app_data_keccak256.as_str()
        },
        "post": {
            "orderId": post_result.order_id.as_str(),
            "signatureLength": post_result.signature.len(),
            "uploadedAppDataCount": orderbook_state.uploads.len(),
            "sentOrderCount": orderbook_state.sent_orders.len()
        },
        "allowanceAndApproval": {
            "allowance": allowance,
            "approvalTxHash": approval_tx_hash,
            "approvalContractRead": provider_state.last_contract_call.as_ref().map(|call| {
                json!({
                    "address": call.address.as_str(),
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
