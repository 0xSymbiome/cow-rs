use std::{collections::BTreeMap, error::Error};

use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_partial_json, method, path},
};

use cow_sdk::SupportedChainId;
use cow_sdk_subgraph::{SubgraphApi, SubgraphConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    let base_urls = BTreeMap::from([(SupportedChainId::Mainnet, Some(server.uri()))]);

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
            "data": {
                "dailyTotals": [{
                    "timestamp": "1710000000",
                    "volumeUsd": "12345.67"
                }]
            }
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_partial_json(
            json!({ "operationName": "LastHoursVolume" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "hourlyTotals": [{
                    "timestamp": "1710003600",
                    "volumeUsd": "456.78"
                }]
            }
        })))
        .mount(&server)
        .await;

    let api = SubgraphApi::with_config(
        "review-key",
        SubgraphConfig {
            chain_id: SupportedChainId::Mainnet,
            base_urls: Some(base_urls),
        },
    );

    let totals = api.get_totals().await?;
    let last_days = api.get_last_days_volume(7).await?;
    let last_hours = api.get_last_hours_volume(24).await?;

    let report = json!({
        "surface": "cow-sdk-subgraph",
        "mode": "simulated-transport",
        "queryContract": {
            "documentType": "canonical-helper",
            "operations": ["Totals", "LastDaysVolume", "LastHoursVolume"]
        },
        "apiName": api.api_name(),
        "totals": {
            "tokens": totals.tokens,
            "orders": totals.orders,
            "volumeUsd": totals.volume_usd
        },
        "lastDaysVolume": {
            "rows": last_days.daily_totals.len(),
            "first": last_days.daily_totals.first().map(|row| {
                json!({
                    "timestamp": row.timestamp,
                    "volumeUsd": row.volume_usd
                })
            })
        },
        "lastHoursVolume": {
            "rows": last_hours.hourly_totals.len(),
            "first": last_hours.hourly_totals.first().map(|row| {
                json!({
                    "timestamp": row.timestamp,
                    "volumeUsd": row.volume_usd
                })
            })
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
