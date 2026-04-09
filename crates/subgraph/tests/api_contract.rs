use cow_sdk_core::SupportedChainId;
use cow_sdk_subgraph::{
    DailyTotal, HourlyTotal, LAST_DAYS_VOLUME_QUERY, LAST_HOURS_VOLUME_QUERY,
    LastDaysVolumeResponse, LastHoursVolumeResponse, SubgraphApi, SubgraphApiBaseUrls,
    SubgraphConfig, SubgraphError, TOTALS_QUERY, Total,
};
use serde::Deserialize;
use serde_json::{Value, json};
use wiremock::{
    Mock, MockServer, Request, ResponseTemplate,
    matchers::{header, method, path},
};

#[tokio::test]
async fn prod_url_map_matches_pinned_supported_and_unsupported_chains() {
    let api = SubgraphApi::new("FakeApiKey");
    let prod_config = api.prod_config();

    assert_eq!(
        prod_config.get(&SupportedChainId::Mainnet).and_then(Clone::clone),
        Some(
            "https://gateway.thegraph.com/api/FakeApiKey/subgraphs/id/8mdwJG7YCSwqfxUbhCypZvoubeZcFVpCHb4zmHhvuKTD"
                .to_owned()
        )
    );
    assert_eq!(
        prod_config
            .get(&SupportedChainId::GnosisChain)
            .and_then(Clone::clone),
        Some(
            "https://gateway.thegraph.com/api/FakeApiKey/subgraphs/id/HTQcP2gLuAy235CMNE8ApN4cbzpLVjjNxtCAUfpzRubq"
                .to_owned()
        )
    );
    assert_eq!(
        prod_config
            .get(&SupportedChainId::ArbitrumOne)
            .and_then(Clone::clone),
        Some(
            "https://gateway.thegraph.com/api/FakeApiKey/subgraphs/id/CQ8g2uJCjdAkUSNkVbd9oqqRP2GALKu1jJCD3fyY5tdc"
                .to_owned()
        )
    );
    assert_eq!(
        prod_config.get(&SupportedChainId::Base).and_then(Clone::clone),
        Some(
            "https://gateway.thegraph.com/api/FakeApiKey/subgraphs/id/EYfBtJDj2thuBCVhdpYDpzfsWzDg3qzpEsitqMouU4Rg"
                .to_owned()
        )
    );
    assert_eq!(
        prod_config.get(&SupportedChainId::Sepolia).and_then(Clone::clone),
        Some(
            "https://gateway.thegraph.com/api/FakeApiKey/subgraphs/id/31isonmztVX9ejBneP6SaVDQwEtyKCGBb3RTafB9Uf2y"
                .to_owned()
        )
    );

    for unsupported_chain in [
        SupportedChainId::Polygon,
        SupportedChainId::Avalanche,
        SupportedChainId::Bnb,
        SupportedChainId::Linea,
        SupportedChainId::Plasma,
        SupportedChainId::Ink,
    ] {
        assert_eq!(
            prod_config.get(&unsupported_chain).and_then(Clone::clone),
            None
        );
    }
}

#[tokio::test]
async fn get_totals_posts_totals_operation_and_returns_first_row() {
    let server = MockServer::start().await;
    let api = api_with_override(&server);

    Mock::given(method("POST"))
        .and(path("/"))
        .and(header("content-type", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "totals": [
                    {
                        "tokens": "192",
                        "orders": "365210",
                        "traders": "50731",
                        "settlements": "160092",
                        "volumeUsd": "49548634.23978489392550883815112596",
                        "volumeEth": "20349080.82753326160179174564685693",
                        "feesUsd": "1495.18088540037791409373835505834",
                        "feesEth": "632.7328748466552906975758491191759"
                    }
                ]
            }
        })))
        .mount(&server)
        .await;

    let totals = api.get_totals().await.unwrap();
    let request = only_request(&server).await;

    assert_graphql_request(&request, TOTALS_QUERY, "Totals", None);
    assert_eq!(
        totals,
        Total {
            tokens: "192".to_owned(),
            orders: "365210".to_owned(),
            traders: "50731".to_owned(),
            settlements: "160092".to_owned(),
            volume_usd: Some("49548634.23978489392550883815112596".to_owned()),
            volume_eth: Some("20349080.82753326160179174564685693".to_owned()),
            fees_usd: Some("1495.18088540037791409373835505834".to_owned()),
            fees_eth: Some("632.7328748466552906975758491191759".to_owned()),
        }
    );
}

#[tokio::test]
async fn get_last_days_volume_posts_variableized_query() {
    let server = MockServer::start().await;
    let api = api_with_override(&server);

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "dailyTotals": [
                    {
                        "timestamp": "1651104000",
                        "volumeUsd": "32085.1639220805155999650325844739"
                    },
                    {
                        "timestamp": "1651017600",
                        "volumeUsd": "34693.62007717297749801092930059675"
                    }
                ]
            }
        })))
        .mount(&server)
        .await;

    let response = api.get_last_days_volume(7).await.unwrap();
    let request = only_request(&server).await;

    assert_graphql_request(
        &request,
        LAST_DAYS_VOLUME_QUERY,
        "LastDaysVolume",
        Some(json!({ "days": 7 })),
    );
    assert_eq!(
        response,
        LastDaysVolumeResponse {
            daily_totals: vec![
                DailyTotal {
                    timestamp: 1_651_104_000,
                    volume_usd: Some("32085.1639220805155999650325844739".to_owned()),
                },
                DailyTotal {
                    timestamp: 1_651_017_600,
                    volume_usd: Some("34693.62007717297749801092930059675".to_owned()),
                },
            ],
        }
    );
}

#[tokio::test]
async fn get_last_hours_volume_posts_variableized_query() {
    let server = MockServer::start().await;
    let api = api_with_override(&server);

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "hourlyTotals": [
                    {
                        "timestamp": "1651186800",
                        "volumeUsd": "190.9404913756501392195019404899438"
                    },
                    {
                        "timestamp": "1651183200",
                        "volumeUsd": "529.9946238000561779423929757743504"
                    }
                ]
            }
        })))
        .mount(&server)
        .await;

    let response = api.get_last_hours_volume(24).await.unwrap();
    let request = only_request(&server).await;

    assert_graphql_request(
        &request,
        LAST_HOURS_VOLUME_QUERY,
        "LastHoursVolume",
        Some(json!({ "hours": 24 })),
    );
    assert_eq!(
        response,
        LastHoursVolumeResponse {
            hourly_totals: vec![
                HourlyTotal {
                    timestamp: 1_651_186_800,
                    volume_usd: Some("190.9404913756501392195019404899438".to_owned()),
                },
                HourlyTotal {
                    timestamp: 1_651_183_200,
                    volume_usd: Some("529.9946238000561779423929757743504".to_owned()),
                },
            ],
        }
    );
}

#[tokio::test]
async fn run_query_supports_custom_queries() {
    let server = MockServer::start().await;
    let api = api_with_override(&server);
    let query = "query TokensByVolume {\n  tokens(first: 5, orderBy: totalVolumeUsd, orderDirection: desc) {\n    address\n    symbol\n    totalVolumeUsd\n    priceUsd\n  }\n}";

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "tokens": [
                    {
                        "address": "0xe91d153e0b41518a2ce8dd3d7944fa863463a97d",
                        "symbol": "WXDAI",
                        "totalVolumeUsd": "32889034.621839712648167717",
                        "priceUsd": "1"
                    }
                ]
            }
        })))
        .mount(&server)
        .await;

    let response: TokensByVolumeResponse = api.run_query(query, None).await.unwrap();
    let request = only_request(&server).await;

    assert_graphql_request(&request, query, "TokensByVolume", None);
    assert_eq!(response.tokens.len(), 1);
    assert_eq!(response.tokens[0].symbol, "WXDAI");
}

#[tokio::test]
async fn run_query_uses_custom_base_url_overrides() {
    let server = MockServer::start().await;
    let custom_urls: SubgraphApiBaseUrls = [
        (SupportedChainId::Mainnet, Some(server.uri())),
        (
            SupportedChainId::GnosisChain,
            Some("https://example.com/xdai".to_owned()),
        ),
        (SupportedChainId::Base, None),
        (SupportedChainId::ArbitrumOne, None),
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
    let api = SubgraphApi::with_config(
        "FakeApiKey",
        SubgraphConfig {
            chain_id: SupportedChainId::Mainnet,
            base_urls: Some(custom_urls),
        },
    );

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "totals": [
                    {
                        "tokens": "192",
                        "orders": "365210",
                        "traders": "50731",
                        "settlements": "160092",
                        "volumeUsd": "49548634.23978489392550883815112596",
                        "volumeEth": "20349080.82753326160179174564685693",
                        "feesUsd": "1495.18088540037791409373835505834",
                        "feesEth": "632.7328748466552906975758491191759"
                    }
                ]
            }
        })))
        .mount(&server)
        .await;

    let _ = api.get_totals().await.unwrap();
    let request = only_request(&server).await;

    assert_graphql_request(&request, TOTALS_QUERY, "Totals", None);
}

#[tokio::test]
async fn unsupported_network_rejects_before_transport() {
    let api = SubgraphApi::with_config(
        "FakeApiKey",
        SubgraphConfig {
            chain_id: SupportedChainId::Polygon,
            base_urls: None,
        },
    );

    let error = api.get_totals().await.unwrap_err();

    assert_eq!(error, SubgraphError::UnsupportedNetwork { chain_id: 137 });
    assert!(
        error
            .to_string()
            .contains("Unsupported Network. The subgraph API is not available in the Network 137")
    );
}

#[tokio::test]
async fn empty_totals_rejects_instead_of_returning_default() {
    let server = MockServer::start().await;
    let api = api_with_override(&server);

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "totals": []
            }
        })))
        .mount(&server)
        .await;

    let error = api.get_totals().await.unwrap_err();

    assert_eq!(error, SubgraphError::NoTotalsFound);
}

#[tokio::test]
async fn invalid_graphql_query_surfaces_typed_context() {
    let server = MockServer::start().await;
    let api = api_with_override(&server);
    let query = "query InvalidQuery {\n  invalidQuery {\n    field1\n    field2\n  }\n}";

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "errors": [
                {
                    "locations": [
                        {
                            "line": 2,
                            "column": 9
                        }
                    ],
                    "message": "Type `Query` has no field `invalidQuery`"
                }
            ]
        })))
        .mount(&server)
        .await;

    let error = api.run_query::<Value>(query, None).await.unwrap_err();
    let request = only_request(&server).await;

    assert_graphql_request(&request, query, "InvalidQuery", None);
    match error {
        SubgraphError::QueryFailed {
            query: failed_query,
            variables,
            api,
            inner_error,
        } => {
            assert_eq!(failed_query, query);
            assert_eq!(variables, "undefined");
            assert_eq!(api, server.uri());
            assert!(inner_error.contains("invalidQuery"));
        }
        other => panic!("expected QueryFailed error, got {other:?}"),
    }
}

fn api_with_override(server: &MockServer) -> SubgraphApi {
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

    SubgraphApi::with_config(
        "FakeApiKey",
        SubgraphConfig {
            chain_id: SupportedChainId::Mainnet,
            base_urls: Some(base_urls),
        },
    )
}

async fn only_request(server: &MockServer) -> Request {
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    requests.into_iter().next().unwrap()
}

fn assert_graphql_request(
    request: &Request,
    query: &str,
    operation_name: &str,
    variables: Option<Value>,
) {
    let body: Value = serde_json::from_slice(&request.body).unwrap();
    let expected = match variables {
        Some(variables) => json!({
            "query": query,
            "variables": variables,
            "operationName": operation_name
        }),
        None => json!({
            "query": query,
            "operationName": operation_name
        }),
    };

    assert_eq!(body, expected);
}

#[derive(Debug, Deserialize)]
struct TokensByVolumeResponse {
    tokens: Vec<TokenByVolume>,
}

#[derive(Debug, Deserialize)]
struct TokenByVolume {
    symbol: String,
}
