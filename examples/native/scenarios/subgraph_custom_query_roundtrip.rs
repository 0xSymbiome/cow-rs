use std::{collections::BTreeMap, error::Error};

use serde::Deserialize;
use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_partial_json, method, path},
};

use cow_sdk::prelude::SupportedChainId;
use cow_sdk_subgraph::{ExternalHostPolicy, SubgraphApi, SubgraphQueryRequest};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokensByVolumeResponse {
    tokens: Vec<TokenByVolume>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenByVolume {
    address: String,
    symbol: String,
    total_volume_usd: String,
    price_usd: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    let base_urls = BTreeMap::from([(SupportedChainId::Mainnet, Some(server.uri()))]);
    let document = "query TokensByVolume($limit: Int!) {\n  tokens(first: $limit, orderBy: totalVolumeUsd, orderDirection: desc) {\n    address\n    symbol\n    totalVolumeUsd\n    priceUsd\n  }\n}";
    let request = SubgraphQueryRequest::new(document)
        .with_variables(json!({ "limit": 2 }))
        .with_operation_name("TokensByVolume");

    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_partial_json(json!({
            "operationName": "TokensByVolume",
            "variables": { "limit": 2 }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "tokens": [
                    {
                        "address": "0xe91d153e0b41518a2ce8dd3d7944fa863463a97d",
                        "symbol": "WXDAI",
                        "totalVolumeUsd": "32889034.621839712648167717",
                        "priceUsd": "1"
                    },
                    {
                        "address": "0x9c58bacc331c9aa871afd802db6379a98e80cedb",
                        "symbol": "GNO",
                        "totalVolumeUsd": "27440021.002913573190214812",
                        "priceUsd": "304.12"
                    }
                ]
            }
        })))
        .mount(&server)
        .await;

    let api = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("review-key")
        .with_external_host_policy(ExternalHostPolicy::Test)
        .base_urls(base_urls)
        .build()?;

    let response: TokensByVolumeResponse = api.run_query(request).await?;

    let report = json!({
        "surface": "cow-sdk-subgraph",
        "mode": "simulated-transport",
        "queryContract": {
            "documentType": "custom",
            "operationName": "TokensByVolume",
            "variables": {
                "limit": 2
            }
        },
        "tokens": response.tokens.iter().map(|token| {
            json!({
                "address": token.address,
                "symbol": token.symbol,
                "totalVolumeUsd": token.total_volume_usd,
                "priceUsd": token.price_usd
            })
        }).collect::<Vec<_>>()
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
