use std::{env, error::Error, io};

use serde_json::json;

use cow_sdk::{ApiContext, CowEnv, OrderBookApi, SupportedChainId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let env = optional_cow_env("COW_SMOKE_ORDERBOOK_ENV")?;
    let chain_id = optional_supported_chain_id("COW_SMOKE_ORDERBOOK_CHAIN_ID")?;
    let api_key = optional_env("COW_SMOKE_ORDERBOOK_API_KEY");
    let base_url_override = optional_env("COW_SMOKE_ORDERBOOK_BASE_URL");

    let context = ApiContext {
        chain_id,
        env,
        base_urls: None,
        api_key,
    };
    let resolved_base_url = base_url_override
        .clone()
        .unwrap_or(context.resolved_base_url()?);
    let api = if let Some(base_url) = base_url_override {
        OrderBookApi::new_with_base_url(context, base_url)
    } else {
        OrderBookApi::new(context)
    };

    let version = api.get_version().await?;
    let report = json!({
        "surface": "cow-sdk-orderbook",
        "mode": "live",
        "env": env.as_str(),
        "chainId": u64::from(chain_id),
        "partnerApi": api.context().api_key.is_some(),
        "baseUrl": resolved_base_url,
        "version": version,
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn optional_env(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}

fn optional_cow_env(name: &str) -> Result<CowEnv, Box<dyn Error>> {
    let Some(raw_value) = optional_env(name) else {
        return Ok(CowEnv::Prod);
    };

    match raw_value.to_ascii_lowercase().as_str() {
        "prod" | "production" => Ok(CowEnv::Prod),
        "staging" | "barn" => Ok(CowEnv::Staging),
        other => Err(io::Error::other(format!(
            "{name} must be one of prod or staging. Received {other}."
        ))
        .into()),
    }
}

fn optional_supported_chain_id(name: &str) -> Result<SupportedChainId, Box<dyn Error>> {
    let Some(raw_value) = optional_env(name) else {
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
