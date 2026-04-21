use std::{env, error::Error, io};

use serde_json::json;

use cow_sdk::SupportedChainId;
use cow_sdk_subgraph::{SubgraphApi, SubgraphConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = required_env("THE_GRAPH_API_KEY")?;
    let chain_id = optional_supported_chain_id("COW_SUBGRAPH_CHAIN_ID")?;
    let api = SubgraphApi::builder().chain(chain_id).api_key(api_key).build();

    let totals = api.get_totals().await?;

    let report = json!({
        "surface": "cow-sdk-subgraph",
        "mode": "live",
        "apiName": api.api_name(),
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
