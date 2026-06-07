//! Smallest deterministic end-to-end swap.
//!
//! Construct a ready-state trading client, then quote, sign, and post a swap in one call
//! against a transport-mocked orderbook. No network and no private key, so it
//! runs the same way on every machine — the shortest path from the facade to a
//! posted order.

use std::{error::Error, sync::Arc};

use cow_sdk::core::OrderKind;
use cow_sdk::prelude::{Amount, SupportedChainId, TradeParameters, Trading};

use cow_sdk::testing::{MockOrderbook, MockSigner};
use cow_sdk_examples_native::support::{
    sample_buy_token, sample_owner, sample_quote_response, sample_sell_token,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Deterministic, transport-mocked orderbook and signer.
    let orderbook = MockOrderbook::builder(SupportedChainId::Sepolia)
        .quote(sample_quote_response())
        .build();
    let signer = MockSigner::builder().address(sample_owner()).build();

    // Construct a ready-state trading client with the mock orderbook injected. A concrete
    // `Arc<MockOrderbook>` coerces into the `Arc<dyn OrderbookClient>` the
    // option expects — no explicit cast needed.
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("cow-rs-quickstart")
        .orderbook_client(Arc::new(orderbook))
        .build()?;

    // Sell 0.1 WETH for COW. The owner is set explicitly here; with a real
    // signer it defaults to the signer's address.
    let params = TradeParameters::new(
        OrderKind::Sell,
        sample_sell_token(),
        sample_buy_token(),
        Amount::parse_units("0.1", 18)?,
    )
    .with_owner(sample_owner())
    .with_slippage_bps(50);

    // One call quotes, signs, and posts.
    let posted = trading.post_swap_order(params, &signer, None).await?;

    println!("posted order: {}", posted.order_id.to_hex_string());
    Ok(())
}
