//! Opt-in live subgraph query.
//!
//! Calls `SubgraphApi::get_totals` against the real CoW subgraph, requiring a
//! `THE_GRAPH_API_KEY` and an optional chain override from the environment. Like
//! `orderbook_live`, it contacts a live service and is excluded from the
//! deterministic runner.

use std::{env, error::Error, io};

use serde_json::json;

use cow_sdk::prelude::SupportedChainId;
use cow_sdk_subgraph::SubgraphApi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // The Graph API key is required for this live call; the chain defaults to mainnet.
    let api_key = required_env("THE_GRAPH_API_KEY")?;
    let chain_id = optional_supported_chain_id("COW_SUBGRAPH_CHAIN_ID")?;

    // Build the read-only subgraph client (used directly, not through the facade).
    let subgraph = SubgraphApi::builder()
        .chain(chain_id)
        .api_key(api_key)
        .build()?;

    // The one live call: protocol-wide totals.
    let totals = subgraph.get_totals().await?;

    let report = json!({
        "surface": "cow-sdk-subgraph",
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

fn required_env(name: &str) -> Result<String, Box<dyn Error>> {
    env::var(name).map_err(|_| {
        io::Error::other(format!(
            "{name} is required for this live example. Configure it explicitly before running."
        ))
        .into()
    })
}

fn optional_supported_chain_id(name: &str) -> Result<SupportedChainId, Box<dyn Error>> {
    let Some(raw_value) = env::var(name).ok() else {
        return Ok(SupportedChainId::Mainnet);
    };
    let chain_id: u64 = raw_value.parse()?;
    SupportedChainId::try_from(chain_id).map_err(|error| {
        io::Error::other(format!(
            "{name} must be a supported chain id. Received {chain_id}: {error}"
        ))
        .into()
    })
}
