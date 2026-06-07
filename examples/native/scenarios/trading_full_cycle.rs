//! The full high-level `Trading` cycle.
//!
//! Walks the complete facade surface against transport-mocked doubles: fetch a
//! quote (`quote_results`), post a swap (`post_swap_order`), read the
//! protocol allowance (`cow_protocol_allowance`), send an approval
//! (`approve_cow_protocol`), and cancel off-chain (`off_chain_cancel_order`).
//! The only scenario that also exercises `MockProvider`, for the allowance and
//! approval reads.

use std::{error::Error, sync::Arc};

use serde_json::json;

use cow_sdk::prelude::{Amount, SupportedChainId, Trading};
use cow_sdk::trading::{
    AllowanceParameters, ApprovalParameters, OrderTraderParameters,
};

use cow_sdk::testing::{MockOrderbook, MockProvider, MockSigner};
use cow_sdk_examples_native::support::{
    sample_owner, sample_quote_response, sample_sell_token, sample_trade_parameters,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Three doubles for the full cycle: the orderbook (quotes + posts), the signer,
    // and a provider for the allowance/approval reads. The provider shares the signer.
    let orderbook = MockOrderbook::builder(SupportedChainId::Sepolia)
        .quote(sample_quote_response())
        .build();
    let signer = MockSigner::builder().address(sample_owner()).build();
    let provider = MockProvider::builder().signer(signer.clone()).build();

    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("cow-rs-native-examples")
        .orderbook_client(Arc::new(orderbook.clone()))
        .build()?;

    // 1. Quote — signed quote results carry the merged app data.
    let quote = trading
        .quote_results(sample_trade_parameters(), &signer, None)
        .await?;

    // 2. Post the swap order.
    let post_result = trading
        .post_swap_order(sample_trade_parameters(), &signer, None)
        .await?;

    // 3. Read the protocol allowance for the sell token (through the provider).
    let allowance = trading
        .cow_protocol_allowance(
            &provider,
            &AllowanceParameters::new(sample_sell_token(), sample_owner()),
        )
        .await?;

    // 4. Approve the protocol to spend the sell token (sends a transaction).
    let approval_tx_hash = trading
        .approve_cow_protocol(
            &signer,
            &ApprovalParameters::new(
                sample_sell_token(),
                Amount::from_units(1, 18)
                    .expect("example approval amount must remain valid"),
            ),
        )
        .await?;

    // 5. Cancel the posted order off-chain.
    let cancelled = trading
        .off_chain_cancel_order(
            &OrderTraderParameters::new(post_result.order_id),
            &signer,
        )
        .await?;

    let orderbook_state = orderbook.recorded();
    let signer_state = signer.recorded();
    let provider_state = provider.recorded();

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
            "approvalContractRead": provider_state.contract_reads.last().map(|call| {
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
