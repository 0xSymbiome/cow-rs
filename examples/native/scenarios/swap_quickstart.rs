//! Smallest deterministic end-to-end swap.
//!
//! Construct a ready-state trading client, then quote, sign, and post a swap in
//! one call through the fluent swap builder, against a transport-mocked
//! orderbook. No network and no private key, so it runs the same way on every
//! machine — the shortest path from the facade to a posted order.

use std::error::Error;

use cow_sdk::core::{Amount, SupportedChainId};
use cow_sdk::trading::Trading;

use cow_sdk::testing::{MockOrderbook, MockSigner};
use cow_sdk_examples_native::support::{COW, OWNER, WETH, sample_quote_response};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Deterministic, transport-mocked orderbook and signer.
    let orderbook = MockOrderbook::builder(SupportedChainId::Sepolia)
        .quote(sample_quote_response())
        .build();
    let signer = MockSigner::builder().address(OWNER).build();

    // Construct a ready-state trading client with the mock orderbook injected.
    // `orderbook(...)` takes the client by value — no `Arc` at the call site.
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("cow-rs-quickstart")
        .orderbook(orderbook)
        .build()?;

    // Sell 0.1 WETH for COW. The sell and buy tokens have named setters, so they
    // cannot be transposed. The owner defaults to the signer's address; set it
    // explicitly with `.owner(...)` for quote-only or delegated flows.
    let posted = trading
        .swap()
        .sell_token(WETH)
        .buy_token(COW)
        .sell_amount(Amount::parse_units("0.1", 18)?)
        .slippage_bps(50)
        .execute(&signer)
        .await?;

    println!("posted order: {}", posted.order_id);
    Ok(())
}
