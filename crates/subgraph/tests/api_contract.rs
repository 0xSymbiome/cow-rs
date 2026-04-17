use cow_sdk_core::{DEFAULT_HTTP_TIMEOUT, HttpClientPolicy, SupportedChainId};
use cow_sdk_subgraph::{
    DEFAULT_SUBGRAPH_USER_AGENT, DailyTotal, HourlyTotal, LAST_DAYS_VOLUME_QUERY,
    LAST_HOURS_VOLUME_QUERY, LastDaysVolumeResponse, LastHoursVolumeResponse, SubgraphApi,
    SubgraphApiBaseUrls, SubgraphConfig, SubgraphError, SubgraphGraphQlError,
    SubgraphGraphQlErrorLocation, SubgraphQueryRequest, SubgraphRequestErrorContext,
    SubgraphTransportPolicy, TOTALS_QUERY, Total,
};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::net::TcpListener;
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
            "https://gateway.thegraph.com/api/<redacted>/subgraphs/id/8mdwJG7YCSwqfxUbhCypZvoubeZcFVpCHb4zmHhvuKTD"
                .to_owned()
        )
    );
    assert_eq!(
        prod_config
            .get(&SupportedChainId::GnosisChain)
            .and_then(Clone::clone),
        Some(
            "https://gateway.thegraph.com/api/<redacted>/subgraphs/id/HTQcP2gLuAy235CMNE8ApN4cbzpLVjjNxtCAUfpzRubq"
                .to_owned()
        )
    );
    assert_eq!(
        prod_config
            .get(&SupportedChainId::ArbitrumOne)
            .and_then(Clone::clone),
        Some(
            "https://gateway.thegraph.com/api/<redacted>/subgraphs/id/CQ8g2uJCjdAkUSNkVbd9oqqRP2GALKu1jJCD3fyY5tdc"
                .to_owned()
        )
    );
    assert_eq!(
        prod_config.get(&SupportedChainId::Base).and_then(Clone::clone),
        Some(
            "https://gateway.thegraph.com/api/<redacted>/subgraphs/id/EYfBtJDj2thuBCVhdpYDpzfsWzDg3qzpEsitqMouU4Rg"
                .to_owned()
        )
    );
    assert_eq!(
        prod_config.get(&SupportedChainId::Sepolia).and_then(Clone::clone),
        Some(
            "https://gateway.thegraph.com/api/<redacted>/subgraphs/id/31isonmztVX9ejBneP6SaVDQwEtyKCGBb3RTafB9Uf2y"
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

#[test]
fn default_transport_policy_is_explicit_and_reviewable() {
    let api = SubgraphApi::new("FakeApiKey");

    assert_eq!(api.client_policy().timeout(), Some(DEFAULT_HTTP_TIMEOUT));
    assert_eq!(
        api.client_policy().user_agent(),
        DEFAULT_SUBGRAPH_USER_AGENT
    );
}

#[test]
fn debug_output_keeps_subgraph_contract_visible_without_printing_prod_urls() {
    let api = SubgraphApi::new("FakeApiKey");
    let debug = format!("{api:?}");

    assert!(debug.contains("SubgraphApi"));
    assert!(debug.contains("supported_prod_chains"));
    assert!(!debug.contains("FakeApiKey"));
    assert!(!debug.contains("gateway.thegraph.com"));
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

    assert_graphql_request(&request, TOTALS_QUERY, Some("Totals"), None);
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
        Some("LastDaysVolume"),
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
        Some("LastHoursVolume"),
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
async fn run_query_supports_variableized_custom_queries() {
    let server = MockServer::start().await;
    let api = api_with_override(&server);
    let query = "query TokensByVolume($limit: Int!) {\n  tokens(first: $limit, orderBy: totalVolumeUsd, orderDirection: desc) {\n    address\n    symbol\n    totalVolumeUsd\n    priceUsd\n  }\n}";
    let request = SubgraphQueryRequest::new(query)
        .with_variables(json!({ "limit": 5 }))
        .with_operation_name("TokensByVolume");

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

    let response: TokensByVolumeResponse = api.run_query(request).await.unwrap();
    let request = only_request(&server).await;

    assert_graphql_request(
        &request,
        query,
        Some("TokensByVolume"),
        Some(json!({ "limit": 5 })),
    );
    assert_eq!(response.tokens.len(), 1);
    assert_eq!(response.tokens[0].symbol, "WXDAI");
}

#[tokio::test]
async fn run_query_supports_explicit_operation_name_for_multi_operation_documents() {
    let server = MockServer::start().await;
    let api = api_with_override(&server);
    let document = "query TokensByVolume {\n  tokens(first: 1) {\n    symbol\n  }\n}\n\nquery TotalsForAudit {\n  totals {\n    orders\n  }\n}";
    let request = SubgraphQueryRequest::new(document).with_operation_name("TokensByVolume");

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "tokens": [
                    {
                        "symbol": "WXDAI"
                    }
                ]
            }
        })))
        .mount(&server)
        .await;

    let response: TokensByVolumeResponse = api.run_query(request).await.unwrap();
    let request = only_request(&server).await;

    assert_graphql_request(&request, document, Some("TokensByVolume"), None);
    assert_eq!(response.tokens.len(), 1);
    assert_eq!(response.tokens[0].symbol, "WXDAI");
}

#[tokio::test]
async fn multi_operation_document_without_operation_name_surfaces_typed_graphql_context() {
    let server = MockServer::start().await;
    let api = api_with_override(&server);
    let document = "query TokensByVolume {\n  tokens(first: 1) {\n    symbol\n  }\n}\n\nquery TotalsForAudit {\n  totals {\n    orders\n  }\n}";

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "errors": [
                {
                    "message": "Must provide operation name if query contains multiple operations."
                }
            ]
        })))
        .mount(&server)
        .await;

    let error = api
        .run_query::<Value, _>(document)
        .await
        .expect_err("multi-operation documents without operationName must fail explicitly");
    let request = only_request(&server).await;

    assert_graphql_request(&request, document, None, None);
    match error {
        SubgraphError::GraphQl { context, errors } => {
            assert_eq!(
                *context,
                SubgraphRequestErrorContext {
                    chain_id: u64::from(SupportedChainId::Mainnet),
                    api: server.uri(),
                    document: document.to_owned(),
                    operation_name: None,
                    variables: None,
                }
            );
            assert_eq!(
                errors,
                vec![SubgraphGraphQlError {
                    message: "Must provide operation name if query contains multiple operations."
                        .to_owned(),
                    locations: vec![],
                }]
            );
        }
        other => panic!("expected GraphQl error, got {other:?}"),
    }
}

#[tokio::test]
async fn run_query_accepts_anonymous_documents_without_operation_name() {
    let server = MockServer::start().await;
    let api = api_with_override(&server);
    let query = "{\n  totals {\n    tokens\n    orders\n    traders\n    settlements\n  }\n}";

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "totals": [
                    {
                        "tokens": "192",
                        "orders": "365210",
                        "traders": "50731",
                        "settlements": "160092"
                    }
                ]
            }
        })))
        .mount(&server)
        .await;

    let response: cow_sdk_subgraph::TotalsResponse = api.run_query(query).await.unwrap();
    let request = only_request(&server).await;

    assert_graphql_request(&request, query, None, None);
    assert_eq!(response.totals.len(), 1);
    assert_eq!(response.totals[0].orders, "365210");
}

#[tokio::test]
async fn run_query_with_config_honors_chain_override_for_generic_queries() {
    let server = MockServer::start().await;
    let base_urls: SubgraphApiBaseUrls = [
        (SupportedChainId::Mainnet, None),
        (SupportedChainId::GnosisChain, Some(server.uri())),
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
    let api = SubgraphApi::with_config(
        "FakeApiKey",
        SubgraphConfig {
            chain_id: SupportedChainId::Mainnet,
            base_urls: Some(base_urls),
        },
    );
    let query = "query TotalsForAudit { totals { orders } }";

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "totals": [
                    {
                        "orders": "365210"
                    }
                ]
            }
        })))
        .mount(&server)
        .await;

    let response: Value = api
        .run_query_with_config(
            SubgraphQueryRequest::new(query).with_operation_name("TotalsForAudit"),
            cow_sdk_subgraph::SubgraphConfigOverride {
                chain_id: Some(SupportedChainId::GnosisChain),
                base_urls: None,
            },
        )
        .await
        .expect("chain override should drive generic-query transport resolution");
    let request = only_request(&server).await;

    assert_graphql_request(&request, query, Some("TotalsForAudit"), None);
    assert_eq!(response["totals"][0]["orders"], "365210");
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

    assert_graphql_request(&request, TOTALS_QUERY, Some("Totals"), None);
}

#[tokio::test]
async fn transport_policy_override_rebuilds_client_with_custom_user_agent() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .and(header("user-agent", "custom-subgraph-client/9.9.9"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "totals": [
                    {
                        "tokens": "1",
                        "orders": "2",
                        "traders": "3",
                        "settlements": "4"
                    }
                ]
            }
        })))
        .mount(&server)
        .await;

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
    let transport_policy = SubgraphTransportPolicy::default().with_client_policy(
        HttpClientPolicy::new("custom-subgraph-client/9.9.9")
            .expect("custom user-agent must be valid")
            .without_timeout(),
    );
    let api = SubgraphApi::with_config_and_transport_policy(
        "FakeApiKey",
        SubgraphConfig {
            chain_id: SupportedChainId::Mainnet,
            base_urls: Some(base_urls),
        },
        transport_policy,
    );

    let totals = api.get_totals().await.expect("custom policy should work");

    assert_eq!(totals.tokens, "1");
    assert_eq!(api.client_policy().timeout(), None);
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

    let error = api
        .run_query::<Value, _>(SubgraphQueryRequest::new(query).with_operation_name("InvalidQuery"))
        .await
        .unwrap_err();
    let request = only_request(&server).await;

    assert_graphql_request(&request, query, Some("InvalidQuery"), None);
    match error {
        SubgraphError::GraphQl { context, errors } => {
            assert_eq!(
                *context,
                SubgraphRequestErrorContext {
                    chain_id: u64::from(SupportedChainId::Mainnet),
                    api: server.uri(),
                    document: query.to_owned(),
                    operation_name: Some("InvalidQuery".to_owned()),
                    variables: None,
                }
            );
            assert_eq!(
                errors,
                vec![SubgraphGraphQlError {
                    message: "Type `Query` has no field `invalidQuery`".to_owned(),
                    locations: vec![SubgraphGraphQlErrorLocation { line: 2, column: 9 }],
                }]
            );
        }
        other => panic!("expected GraphQl error, got {other:?}"),
    }
}

#[tokio::test]
async fn graphql_error_preserves_variables_in_typed_context() {
    let server = MockServer::start().await;
    let api = api_with_override(&server);
    let query =
        "query TokensByVolume($limit: Int!) {\n  tokens(first: $limit) {\n    symbol\n  }\n}";
    let request = SubgraphQueryRequest::new(query)
        .with_variables(json!({ "limit": 5 }))
        .with_operation_name("TokensByVolume");

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "errors": [
                {
                    "message": "Field `tokens` is unavailable for the requested arguments."
                }
            ]
        })))
        .mount(&server)
        .await;

    let error = api
        .run_query::<Value, _>(request)
        .await
        .expect_err("GraphQL failures should preserve request variables");
    let captured_request = only_request(&server).await;

    assert_graphql_request(
        &captured_request,
        query,
        Some("TokensByVolume"),
        Some(json!({ "limit": 5 })),
    );
    match error {
        SubgraphError::GraphQl { context, errors } => {
            assert_eq!(
                *context,
                SubgraphRequestErrorContext {
                    chain_id: u64::from(SupportedChainId::Mainnet),
                    api: server.uri(),
                    document: query.to_owned(),
                    operation_name: Some("TokensByVolume".to_owned()),
                    variables: Some(json!({ "limit": 5 })),
                }
            );
            assert_eq!(
                errors,
                vec![SubgraphGraphQlError {
                    message: "Field `tokens` is unavailable for the requested arguments."
                        .to_owned(),
                    locations: vec![],
                }]
            );
        }
        other => panic!("expected GraphQl error, got {other:?}"),
    }
}

#[tokio::test]
async fn malformed_success_response_surfaces_serialization_error() {
    let server = MockServer::start().await;
    let api = api_with_override(&server);

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not-json"))
        .mount(&server)
        .await;

    let error = api
        .get_totals()
        .await
        .expect_err("invalid json should fail");

    match error {
        SubgraphError::Serialization {
            context,
            body,
            details,
        } => {
            assert_eq!(
                *context,
                SubgraphRequestErrorContext {
                    chain_id: u64::from(SupportedChainId::Mainnet),
                    api: server.uri(),
                    document: TOTALS_QUERY.to_owned(),
                    operation_name: Some("Totals".to_owned()),
                    variables: None,
                }
            );
            assert_eq!(body, "not-json");
            assert!(!details.is_empty());
        }
        other => panic!("expected serialization error, got {other:?}"),
    }
}

#[tokio::test]
async fn non_success_status_surfaces_http_status_error() {
    let server = MockServer::start().await;
    let api = api_with_override(&server);
    let query = "query TokensByVolume { tokens(first: 1) { symbol } }";

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(500).set_body_string("upstream exploded"))
        .mount(&server)
        .await;

    let error = api
        .run_query::<Value, _>(
            SubgraphQueryRequest::new(query).with_operation_name("TokensByVolume"),
        )
        .await
        .expect_err("http failure should surface typed status context");

    match error {
        SubgraphError::HttpStatus {
            context,
            status,
            body,
        } => {
            assert_eq!(
                *context,
                SubgraphRequestErrorContext {
                    chain_id: u64::from(SupportedChainId::Mainnet),
                    api: server.uri(),
                    document: query.to_owned(),
                    operation_name: Some("TokensByVolume".to_owned()),
                    variables: None,
                }
            );
            assert_eq!(status, 500);
            assert_eq!(body, "upstream exploded");
        }
        other => panic!("expected HttpStatus error, got {other:?}"),
    }
}

#[tokio::test]
async fn missing_data_surfaces_typed_missing_data_error_for_generic_queries() {
    let server = MockServer::start().await;
    let api = api_with_override(&server);
    let query = "query TokensByVolume($limit: Int!) { tokens(first: $limit) { symbol } }";
    let request = SubgraphQueryRequest::new(query)
        .with_variables(json!({ "limit": 5 }))
        .with_operation_name("TokensByVolume");

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": null })))
        .mount(&server)
        .await;

    let error = api
        .run_query::<Value, _>(request)
        .await
        .expect_err("missing data should fail with typed context");

    match error {
        SubgraphError::MissingData { context } => {
            assert_eq!(
                *context,
                SubgraphRequestErrorContext {
                    chain_id: u64::from(SupportedChainId::Mainnet),
                    api: server.uri(),
                    document: query.to_owned(),
                    operation_name: Some("TokensByVolume".to_owned()),
                    variables: Some(json!({ "limit": 5 })),
                }
            );
        }
        other => panic!("expected MissingData error, got {other:?}"),
    }
}

#[tokio::test]
async fn transport_failures_surface_typed_context() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("ephemeral port must be available");
    let endpoint_origin = format!(
        "http://127.0.0.1:{}",
        listener
            .local_addr()
            .expect("bound listener must expose a local address")
            .port()
    );
    let endpoint = format!("{endpoint_origin}/private/path?token=secret");
    drop(listener);

    let base_urls: SubgraphApiBaseUrls = [
        (SupportedChainId::Mainnet, Some(endpoint.clone())),
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
    let api = SubgraphApi::with_config(
        "FakeApiKey",
        SubgraphConfig {
            chain_id: SupportedChainId::Mainnet,
            base_urls: Some(base_urls),
        },
    );
    let query = "query TokensByVolume { tokens(first: 1) { symbol } }";

    let error = api
        .run_query::<Value, _>(
            SubgraphQueryRequest::new(query).with_operation_name("TokensByVolume"),
        )
        .await
        .expect_err("connection failure should surface typed transport context");

    match error {
        SubgraphError::Transport { context, details } => {
            assert_eq!(
                *context,
                SubgraphRequestErrorContext {
                    chain_id: u64::from(SupportedChainId::Mainnet),
                    api: endpoint_origin,
                    document: query.to_owned(),
                    operation_name: Some("TokensByVolume".to_owned()),
                    variables: None,
                }
            );
            assert!(!details.is_empty());
        }
        other => panic!("expected Transport error, got {other:?}"),
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
    operation_name: Option<&str>,
    variables: Option<Value>,
) {
    let body: Value = serde_json::from_slice(&request.body).unwrap();
    let mut expected = Map::new();
    expected.insert("query".to_owned(), Value::String(query.to_owned()));

    if let Some(variables) = variables {
        expected.insert("variables".to_owned(), variables);
    }

    if let Some(operation_name) = operation_name {
        expected.insert(
            "operationName".to_owned(),
            Value::String(operation_name.to_owned()),
        );
    }

    assert_eq!(body, Value::Object(expected));
}

#[derive(Debug, Deserialize)]
struct TokensByVolumeResponse {
    tokens: Vec<TokenByVolume>,
}

#[derive(Debug, Deserialize)]
struct TokenByVolume {
    symbol: String,
}

#[tokio::test]
async fn get_totals_with_cancellation_returns_cancelled_when_token_is_fired_before_send() {
    let api = SubgraphApi::new("FakeApiKey");
    let token = cow_sdk_core::CancellationToken::new();
    token.cancel();

    let error = api
        .get_totals_with_cancellation(&token)
        .await
        .expect_err("pre-cancelled token must produce a Cancelled error");
    assert!(matches!(error, SubgraphError::Cancelled));
}

#[tokio::test]
async fn get_totals_with_cancellation_aborts_an_in_flight_request() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "data": { "totals": [] } }))
                .set_delay(std::time::Duration::from_secs(30)),
        )
        .mount(&server)
        .await;

    let api = api_with_override(&server);
    let token = cow_sdk_core::CancellationToken::new();
    let token_for_task = token.clone();

    let started = std::time::Instant::now();
    let task = tokio::spawn(async move { api.get_totals_with_cancellation(&token_for_task).await });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    token.cancel();

    let result = task.await.expect("cancellation task should not panic");
    let elapsed = started.elapsed();

    assert!(matches!(result, Err(SubgraphError::Cancelled)));
    assert!(
        elapsed < std::time::Duration::from_secs(5),
        "cancellation must drop the in-flight future within the request deadline; elapsed = {elapsed:?}"
    );
}

#[tokio::test]
async fn shared_client_fans_queries_across_multiple_subgraph_instances() {
    let first = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "totals": [
                    {
                        "tokens": "100",
                        "orders": "200",
                        "traders": "50",
                        "settlements": "150",
                        "volumeUsd": "1000",
                        "volumeEth": "2000",
                        "feesUsd": "10",
                        "feesEth": "20"
                    }
                ]
            }
        })))
        .expect(1)
        .mount(&first)
        .await;

    let second = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "totals": [
                    {
                        "tokens": "300",
                        "orders": "400",
                        "traders": "75",
                        "settlements": "250",
                        "volumeUsd": "3000",
                        "volumeEth": "4000",
                        "feesUsd": "30",
                        "feesEth": "40"
                    }
                ]
            }
        })))
        .expect(1)
        .mount(&second)
        .await;

    let shared = reqwest::Client::builder()
        .user_agent(DEFAULT_SUBGRAPH_USER_AGENT)
        .build()
        .expect("reqwest client must build for the shared-client regression test");

    let first_base_urls: SubgraphApiBaseUrls =
        std::iter::once((SupportedChainId::Mainnet, Some(first.uri()))).collect();
    let first_api = SubgraphApi::from_shared_client_with_config(
        shared.clone(),
        "FakeApiKey",
        SubgraphConfig {
            chain_id: SupportedChainId::Mainnet,
            base_urls: Some(first_base_urls),
        },
    );

    let second_base_urls: SubgraphApiBaseUrls =
        std::iter::once((SupportedChainId::GnosisChain, Some(second.uri()))).collect();
    let second_api = SubgraphApi::from_shared_client_with_config(
        shared,
        "FakeApiKey",
        SubgraphConfig {
            chain_id: SupportedChainId::GnosisChain,
            base_urls: Some(second_base_urls),
        },
    );

    let first_totals = first_api
        .get_totals()
        .await
        .expect("first shared-client query must succeed");
    let second_totals = second_api
        .get_totals()
        .await
        .expect("second shared-client query must succeed");

    assert_eq!(first_totals.tokens, "100");
    assert_eq!(second_totals.tokens, "300");
}
