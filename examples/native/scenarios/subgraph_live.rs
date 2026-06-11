//! Opt-in live subgraph query.
//!
//! Calls `SubgraphApi::totals` against the real CoW subgraph, requiring a
//! `THE_GRAPH_API_KEY` and an optional chain override from the environment. Like
//! `orderbook_live`, it contacts a live service and is excluded from the
//! deterministic runner.

use std::error::Error;

use serde_json::json;

use cow_sdk::subgraph::SubgraphApi;

use cow_sdk_examples_native::support::{optional_supported_chain_id, required_env};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // The Graph API key is required for this live call; the chain defaults to mainnet.
    let api_key = required_env("THE_GRAPH_API_KEY")?;
    let chain_id = optional_supported_chain_id("COW_SUBGRAPH_CHAIN_ID")?;

    // Build the read-only subgraph client through the cow-sdk `subgraph` feature.
    let subgraph = SubgraphApi::builder()
        .chain(chain_id)
        .api_key(api_key)
        .build()?;

    // The one live call: protocol-wide totals.
    let totals = subgraph.totals().await?;

    let report = json!({
        "surface": "cow_sdk::subgraph",
        "mode": "live",
        "apiName": subgraph.api_name(),
        "chainId": u64::from(chain_id),
        "totals": {
            "tokens": totals.tokens,
            "orders": totals.orders,
            "traders": totals.traders,
            "settlements": totals.settlements,
            "volumeUsd": totals.volume_usd,
            "feesUsd": totals.fees_usd
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
