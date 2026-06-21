//! Receiver redirect: pay swap proceeds to an address other than the owner.
//!
//! A bot signs from its hot wallet — the order owner — but directs the bought
//! tokens to a separate address such as a treasury, a cold wallet, or a
//! contract. Set `receiver` on the trade parameters and the signed order carries
//! it verbatim, so the settlement contract releases the buy token to the
//! receiver instead of the owner. The owner still signs (and the SDK's
//! owner-recovery gate still binds the signature to the owner); only the payout
//! destination moves.
//!
//! When `receiver` is omitted it defaults to the owner (the pay-to-owner
//! sentinel), so this scenario is the explicit redirect case: `receiver` is
//! `ALT_RECEIVER`, distinct from the signing `OWNER`, and the posted wire order
//! carries that redirect target.

use std::error::Error;

use serde_json::json;

use cow_sdk::core::SupportedChainId;
use cow_sdk::trading::Trading;

use cow_sdk::testing::{MockOrderbook, MockSigner};
use cow_sdk_examples_native::support::{
    ALT_RECEIVER, OWNER, sample_limit_parameters, sample_quote_response,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let orderbook = MockOrderbook::builder(SupportedChainId::Sepolia)
        .quote(sample_quote_response())
        .build();
    let signer = MockSigner::builder().address(OWNER).build();
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("cow-rs-native-examples")
        .orderbook(orderbook.clone())
        .build()?;

    // The owner signs, but the bought tokens are redirected to ALT_RECEIVER.
    let parameters = sample_limit_parameters().with_receiver(ALT_RECEIVER);
    let post = trading.post_limit_order(parameters, &signer, None).await?;

    let sent = orderbook.recorded().sent_orders;
    let posted_receiver = sent.first().and_then(|order| order.receiver);

    // The signed wire order carries the redirect receiver, distinct from the owner.
    assert_eq!(
        posted_receiver,
        Some(ALT_RECEIVER),
        "the posted order must carry the redirect receiver",
    );
    assert_ne!(
        posted_receiver,
        Some(OWNER),
        "a receiver redirect must differ from the owner",
    );

    let report = json!({
        "surface": "cow_sdk::trading::Trading::post_limit_order (receiver redirect)",
        "mode": "simulated-transport",
        "owner": OWNER.to_hex_string(),
        "receiver": posted_receiver.map(|receiver| receiver.to_hex_string()),
        "signingScheme": post.signing_scheme,
        "postedOrderCount": sent.len(),
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
