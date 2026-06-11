//! Signed limit-order submission, plus the signer-less pre-sign variant.
//!
//! Builds and posts a limit order through `Trading::post_limit_order` against a
//! transport-mocked orderbook and signer, then inspects the recorded submission.
//! The limit path takes an explicit price rather than a fetched quote. A second
//! section posts the same parameters through `Trading::post_limit_order_presign`
//! — the smart-account path that needs no signer at all.

use std::{error::Error, sync::Arc};

use serde_json::json;

use cow_sdk::core::SupportedChainId;
use cow_sdk::trading::Trading;

use cow_sdk::testing::{MockOrderbook, MockSigner};
use cow_sdk_examples_native::support::{OWNER, sample_limit_parameters, sample_quote_response};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Transport-mocked orderbook and signer; the signer address becomes the owner.
    let orderbook = MockOrderbook::builder(SupportedChainId::Sepolia)
        .quote(sample_quote_response())
        .build();
    let signer = MockSigner::builder().address(OWNER).build();
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("cow-rs-limit-order")
        .orderbook_client(Arc::new(orderbook.clone()))
        .build()?;

    // Post a limit order: the price comes from the parameters, not a fetched quote.
    let posted = trading
        .post_limit_order(sample_limit_parameters(), &signer, None)
        .await?;

    // Pre-sign variant: no signer participates, so the explicit owner carried by
    // the parameters identifies the account. The order is posted under the
    // `presign` scheme with an empty signature and only becomes fillable once
    // the owner flips the on-chain pre-signature flag (`setPreSignature`) — the
    // smart-account / Safe placement path.
    let presigned = trading
        .post_limit_order_presign(sample_limit_parameters(), None)
        .await?;

    // Inspect what the orderbook recorded for the submissions.
    let state = orderbook.recorded();
    let sent_order = state
        .sent_orders
        .first()
        .expect("example limit order must be sent");

    let report = json!({
        "surface": "cow_sdk::trading::Trading::{post_limit_order, post_limit_order_presign}",
        "mode": "simulated-transport",
        "result": {
            "orderId": posted.order_id.to_hex_string(),
            "signatureLength": posted.signature.len(),
            "signingScheme": posted.signing_scheme
        },
        "presign": {
            "orderId": presigned.order_id.to_hex_string(),
            "signingScheme": presigned.signing_scheme,
            "signatureLength": presigned.signature.len()
        },
        "submission": {
            "quoteId": sent_order.quote_id,
            "sellAmount": sent_order.sell_amount,
            "buyAmount": sent_order.buy_amount,
            "uploadedAppDataCount": state.uploads.len()
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
