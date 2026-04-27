//! Typed totals query plus an explicit raw-document escape hatch.
//!
//! This example shows two public paths through `cow-sdk-subgraph` against a
//! local `wiremock::MockServer` (no live API key required):
//!
//! 1. The canonical typed query `TOTALS_QUERY` → `get_totals()` →
//!    `Total`, which covers the common "give me one well-known result"
//!    use-case.
//! 2. The explicit `run_query` escape hatch that builds a
//!    `SubgraphQueryRequest` with a custom document, optional variables,
//!    and an explicit `operation_name`, decoded into an arbitrary
//!    `serde_json::Value`. This is the supported path for consumers that
//!    need a query the typed methods do not already cover.
//!
//! Run with:
//!
//! ```text
//! cargo run -p cow-sdk-subgraph --example typed_query_with_escape_hatch
//! ```

use cow_sdk_core::SupportedChainId;
use cow_sdk_subgraph::{
    ExternalHostPolicy, SubgraphApi, SubgraphApiBaseUrls, SubgraphQueryRequest, Total,
};
use serde_json::{Value, json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;

    // Typed TOTALS_QUERY response.
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "totals": [
                    {
                        "tokens": "250",
                        "orders": "365210",
                        "traders": "41820",
                        "settlements": "9152",
                        "volumeUsd": "123456789.0",
                        "volumeEth": "50000.0",
                        "feesUsd": "12345.67",
                        "feesEth": "7.89"
                    }
                ]
            }
        })))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    // Escape-hatch raw-document response.
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "tokens": [
                    {
                        "id": "0x0625afb445c3b6b7b929342a04a22599fd5dbb59",
                        "symbol": "COW",
                        "numberOfTrades": "12345",
                        "totalVolumeUsd": "98765.43"
                    }
                ]
            }
        })))
        .mount(&server)
        .await;

    let api = api_pointed_at(&server);

    // 1. Canonical typed path.
    let totals: Total = api.get_totals().await?;
    println!(
        "typed TOTALS_QUERY: tokens={} orders={} traders={} settlements={}",
        totals.tokens, totals.orders, totals.traders, totals.settlements,
    );

    // 2. Explicit run_query escape hatch with document, variables, and operation_name.
    let document = "query TokensByVolume($limit: Int!) { \
        tokens(first: $limit, orderBy: totalVolumeUsd, orderDirection: desc) { \
            id \
            symbol \
            numberOfTrades \
            totalVolumeUsd \
        } \
    }";
    let request = SubgraphQueryRequest::new(document)
        .with_operation_name("TokensByVolume")
        .with_variables(json!({ "limit": 5 }));
    let escape_hatch: Value = api.run_query(request).await?;

    if let Some(first) = escape_hatch["tokens"].as_array().and_then(|a| a.first()) {
        println!(
            "escape-hatch TokensByVolume: top symbol={} trades={}",
            first["symbol"].as_str().unwrap_or("<unknown>"),
            first["numberOfTrades"].as_str().unwrap_or("0"),
        );
    }

    Ok(())
}

/// Builds a `SubgraphApi` whose Mainnet endpoint points at the local mock
/// server. Every other chain stays at its default public resolution, which is
/// unreached by this example because the API call above is pinned to Mainnet.
fn api_pointed_at(server: &MockServer) -> SubgraphApi {
    let base_urls: SubgraphApiBaseUrls = [
        (SupportedChainId::Mainnet, Some(server.uri())),
        (SupportedChainId::GnosisChain, None),
        (SupportedChainId::ArbitrumOne, None),
        (SupportedChainId::Base, None),
        (SupportedChainId::Sepolia, None),
        (SupportedChainId::Polygon, None),
        (SupportedChainId::Avalanche, None),
        (SupportedChainId::Bnb, None),
        (SupportedChainId::Linea, None),
        (SupportedChainId::Plasma, None),
        (SupportedChainId::Ink, None),
    ]
    .into_iter()
    .collect();

    SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("example-api-key")
        .with_external_host_policy(ExternalHostPolicy::Test)
        .base_urls(base_urls)
        .build()
        .expect("subgraph example client with local mock endpoint must build")
}
