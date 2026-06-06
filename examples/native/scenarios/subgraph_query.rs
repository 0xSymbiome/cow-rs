//! Read-only subgraph access: the canonical typed helpers and the explicit
//! `SubgraphQueryRequest` escape hatch, both against a local mock transport.
//!
//! Subgraph access deliberately uses `cow-sdk-subgraph` directly rather than the
//! root `cow-sdk` facade, which stays trading-first.

use std::{collections::BTreeMap, error::Error};

use serde::Deserialize;
use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_partial_json, method, path},
};

use cow_sdk::prelude::SupportedChainId;
use cow_sdk_subgraph::{ExternalHostPolicy, SubgraphApi, SubgraphQueryRequest};

/// A caller-owned response shape, deserialized straight out of `run_query`.
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

    // Canonical typed helpers (one mount per operation; the operation name in the
    // request body selects the matching response).
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_partial_json(json!({ "operationName": "Totals" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "totals": [{
                    "tokens": "1234",
                    "orders": "5678",
                    "traders": "90",
                    "settlements": "12",
                    "volumeUsd": "345678.90",
                    "volumeEth": "123.45",
                    "feesUsd": "678.90",
                    "feesEth": "0.45"
                }]
            }
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_partial_json(
            json!({ "operationName": "LastDaysVolume" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": { "dailyTotals": [{ "timestamp": "1710000000", "volumeUsd": "12345.67" }] }
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_partial_json(
            json!({ "operationName": "LastHoursVolume" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": { "hourlyTotals": [{ "timestamp": "1710003600", "volumeUsd": "456.78" }] }
        })))
        .mount(&server)
        .await;

    // Custom GraphQL through the `run_query` escape hatch, for documents the typed
    // helpers do not cover.
    let document = "query TokensByVolume($limit: Int!) {\n  tokens(first: $limit, orderBy: totalVolumeUsd, orderDirection: desc) {\n    address\n    symbol\n    totalVolumeUsd\n    priceUsd\n  }\n}";
    let custom_request = SubgraphQueryRequest::new(document)
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

    let subgraph = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("review-key")
        .with_external_host_policy(ExternalHostPolicy::Test)
        .base_urls(base_urls)
        .build()?;

    // Canonical typed helpers.
    let totals = subgraph.totals().await?;
    let last_days = subgraph.last_days_volume(7).await?;
    let last_hours = subgraph.last_hours_volume(24).await?;

    // Custom GraphQL deserialized into a caller-owned type.
    let custom: TokensByVolumeResponse = subgraph.run_query(custom_request).await?;

    let report = json!({
        "surface": "cow-sdk-subgraph",
        "mode": "simulated-transport",
        "apiName": subgraph.api_name(),
        "canonicalHelpers": {
            "totals": {
                "tokens": totals.tokens,
                "orders": totals.orders,
                "volumeUsd": totals.volume_usd
            },
            "lastDaysVolume": {
                "rows": last_days.daily_totals.len(),
                "first": last_days.daily_totals.first().map(|row| json!({
                    "timestamp": row.timestamp,
                    "volumeUsd": row.volume_usd
                }))
            },
            "lastHoursVolume": {
                "rows": last_hours.hourly_totals.len(),
                "first": last_hours.hourly_totals.first().map(|row| json!({
                    "timestamp": row.timestamp,
                    "volumeUsd": row.volume_usd
                }))
            }
        },
        "customQuery": {
            "operationName": "TokensByVolume",
            "tokens": custom.tokens.iter().map(|token| json!({
                "address": token.address,
                "symbol": token.symbol,
                "totalVolumeUsd": token.total_volume_usd,
                "priceUsd": token.price_usd
            })).collect::<Vec<_>>()
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
