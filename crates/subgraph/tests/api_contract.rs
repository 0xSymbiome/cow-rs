use cow_sdk_core::{
    DEFAULT_HTTP_TIMEOUT, HttpClientPolicy, REDACTED_PLACEHOLDER, SupportedChainId,
    TransportErrorClass,
};
use cow_sdk_subgraph::{
    DailyTotal, ExternalHostPolicy, HourlyTotal, LAST_DAYS_VOLUME_QUERY, LAST_HOURS_VOLUME_QUERY,
    LastDaysVolumeResponse, LastHoursVolumeResponse, SubgraphApi, SubgraphApiBaseUrls,
    SubgraphError, SubgraphGraphQlError, SubgraphGraphQlErrorLocation, SubgraphQueryRequest,
    SubgraphRequestErrorContext, TOTALS_QUERY, Total,
};
use cow_sdk_transport_policy::{DEFAULT_SUBGRAPH_USER_AGENT, TransportPolicy};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use wiremock::{
    Mock, MockServer, Request, ResponseTemplate,
    matchers::{header, method, path},
};

#[tokio::test]
async fn prod_url_map_matches_pinned_supported_and_unsupported_chains() {
    let api = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("FakeApiKey")
        .build()
        .expect("default subgraph client must build");
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
    let api = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("FakeApiKey")
        .build()
        .expect("default subgraph client must build");

    assert_eq!(api.client_policy().timeout(), Some(DEFAULT_HTTP_TIMEOUT));
    assert_eq!(
        api.client_policy().user_agent(),
        DEFAULT_SUBGRAPH_USER_AGENT
    );
}

#[test]
fn debug_output_keeps_subgraph_contract_visible_without_printing_prod_urls() {
    let api = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("FakeApiKey")
        .build()
        .expect("default subgraph client must build");
    let debug = format!("{api:?}");

    assert!(debug.contains("SubgraphApi"));
    assert!(debug.contains("supported_prod_chains"));
    assert!(!debug.contains("FakeApiKey"));
    assert!(!debug.contains("gateway.thegraph.com"));
}

#[test]
fn config_debug_and_serialize_redact_custom_base_url_credentials() {
    let base_urls: SubgraphApiBaseUrls = [
        (
            SupportedChainId::Mainnet,
            Some("https://user:pass@example.test/path?apiKey=secret-token".to_owned()),
        ),
        (SupportedChainId::GnosisChain, None),
    ]
    .into_iter()
    .collect();
    let config = cow_sdk_subgraph::SubgraphConfig::new(SupportedChainId::Mainnet, Some(base_urls));

    let debug = format!("{config:#?}");
    let json = serde_json::to_value(&config).expect("subgraph config serializes");

    assert!(debug.contains(REDACTED_PLACEHOLDER));
    assert_eq!(json["baseUrls"]["1"], REDACTED_PLACEHOLDER);
    assert_eq!(json["baseUrls"]["100"], serde_json::Value::Null);
    for rendered in [debug, json.to_string()] {
        assert!(!rendered.contains("user:pass"));
        assert!(!rendered.contains("apiKey=secret-token"));
        assert!(!rendered.contains("example.test"));
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

    assert_graphql_request(&request, TOTALS_QUERY, Some("Totals"), None);
    assert_eq!(
        totals,
        Total::new("192", "365210", "50731", "160092")
            .with_volume_usd("49548634.23978489392550883815112596")
            .with_volume_eth("20349080.82753326160179174564685693")
            .with_fees_usd("1495.18088540037791409373835505834")
            .with_fees_eth("632.7328748466552906975758491191759")
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
        LastDaysVolumeResponse::new(vec![
            DailyTotal::new(1_651_104_000).with_volume_usd("32085.1639220805155999650325844739"),
            DailyTotal::new(1_651_017_600).with_volume_usd("34693.62007717297749801092930059675"),
        ])
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
        LastHoursVolumeResponse::new(vec![
            HourlyTotal::new(1_651_186_800).with_volume_usd("190.9404913756501392195019404899438"),
            HourlyTotal::new(1_651_183_200).with_volume_usd("529.9946238000561779423929757743504"),
        ])
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
                SubgraphRequestErrorContext::new(
                    u64::from(SupportedChainId::Mainnet),
                    server.uri(),
                    document,
                    None,
                    None,
                )
            );
            assert_eq!(
                errors,
                vec![SubgraphGraphQlError::new(
                    "Must provide operation name if query contains multiple operations.",
                    vec![],
                )]
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
    let api = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("FakeApiKey")
        .with_external_host_policy(ExternalHostPolicy::Test)
        .base_urls(base_urls)
        .build()
        .expect("subgraph test client with loopback override must build");
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
            cow_sdk_subgraph::SubgraphConfigOverride::new(
                Some(SupportedChainId::GnosisChain),
                None,
            ),
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
    let api = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("FakeApiKey")
        .with_external_host_policy(ExternalHostPolicy::AllowAny)
        .base_urls(custom_urls)
        .build()
        .expect("subgraph test client with custom overrides must build");

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
    let transport_policy = TransportPolicy::default_subgraph().with_client_policy(
        HttpClientPolicy::new("custom-subgraph-client/9.9.9")
            .expect("custom user-agent must be valid")
            .without_timeout(),
    );
    let api = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("FakeApiKey")
        .with_external_host_policy(ExternalHostPolicy::Test)
        .base_urls(base_urls)
        .transport_policy(transport_policy)
        .build()
        .expect("subgraph test client with loopback override must build");

    let totals = api.get_totals().await.expect("custom policy should work");

    assert_eq!(totals.tokens, "1");
    assert_eq!(api.client_policy().timeout(), None);
}

#[tokio::test]
async fn unsupported_network_rejects_before_transport() {
    let api = SubgraphApi::builder()
        .chain(SupportedChainId::Polygon)
        .api_key("FakeApiKey")
        .build()
        .expect("default subgraph client must build");

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
                SubgraphRequestErrorContext::new(
                    u64::from(SupportedChainId::Mainnet),
                    server.uri(),
                    query,
                    Some("InvalidQuery".to_owned()),
                    None,
                )
            );
            assert_eq!(
                errors,
                vec![SubgraphGraphQlError::new(
                    "Type `Query` has no field `invalidQuery`",
                    vec![SubgraphGraphQlErrorLocation::new(2, 9)],
                )]
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
                SubgraphRequestErrorContext::new(
                    u64::from(SupportedChainId::Mainnet),
                    server.uri(),
                    query,
                    Some("TokensByVolume".to_owned()),
                    Some(json!({ "limit": 5 })),
                )
            );
            assert_eq!(
                errors,
                vec![SubgraphGraphQlError::new(
                    "Field `tokens` is unavailable for the requested arguments.",
                    vec![],
                )]
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
                SubgraphRequestErrorContext::new(
                    u64::from(SupportedChainId::Mainnet),
                    server.uri(),
                    TOTALS_QUERY,
                    Some("Totals".to_owned()),
                    None,
                )
            );
            assert_eq!(body.as_inner(), "not-json");
            assert!(!details.as_inner().is_empty());
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
                SubgraphRequestErrorContext::new(
                    u64::from(SupportedChainId::Mainnet),
                    server.uri(),
                    query,
                    Some("TokensByVolume".to_owned()),
                    None,
                )
            );
            assert_eq!(status, 500);
            assert_eq!(body.as_inner(), "upstream exploded");
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
                SubgraphRequestErrorContext::new(
                    u64::from(SupportedChainId::Mainnet),
                    server.uri(),
                    query,
                    Some("TokensByVolume".to_owned()),
                    Some(json!({ "limit": 5 })),
                )
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
    let api = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("FakeApiKey")
        .with_external_host_policy(ExternalHostPolicy::Test)
        .base_urls(base_urls)
        .build()
        .expect("subgraph test client with loopback override must build");
    let query = "query TokensByVolume { tokens(first: 1) { symbol } }";

    let error = api
        .run_query::<Value, _>(
            SubgraphQueryRequest::new(query).with_operation_name("TokensByVolume"),
        )
        .await
        .expect_err("connection failure should surface typed transport context");

    match error {
        SubgraphError::Transport {
            context,
            class,
            details,
        } => {
            assert_eq!(
                *context,
                SubgraphRequestErrorContext::new(
                    u64::from(SupportedChainId::Mainnet),
                    endpoint_origin,
                    query,
                    Some("TokensByVolume".to_owned()),
                    None,
                )
            );
            assert_eq!(class, TransportErrorClass::Connect);
            assert!(!details.as_inner().is_empty());
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

    SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("FakeApiKey")
        .with_external_host_policy(ExternalHostPolicy::Test)
        .base_urls(base_urls)
        .build()
        .expect("subgraph test client with loopback override must build")
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
async fn get_totals_returns_cancelled_when_combinator_token_fires_before_send() {
    use cow_sdk_core::Cancellable;

    let api = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("FakeApiKey")
        .build()
        .expect("default subgraph client must build");
    let token = cow_sdk_core::CancellationToken::new();
    token.cancel();

    let error = api
        .get_totals()
        .cancel_with(&token)
        .await
        .expect_err("pre-cancelled token must produce a Cancelled error");
    assert!(matches!(error, SubgraphError::Cancelled));
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn get_totals_combinator_aborts_an_in_flight_request() {
    use cow_sdk_core::Cancellable;

    struct DropSpy(Arc<AtomicBool>);

    impl Drop for DropSpy {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

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
    let dropped = Arc::new(AtomicBool::new(false));
    let spy = DropSpy(Arc::clone(&dropped));

    let started = std::time::Instant::now();
    let task = tokio::spawn(async move {
        let _spy = spy;
        api.get_totals().cancel_with(&token_for_task).await
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    token.cancel();

    let result = task.await.expect("cancellation task should not panic");
    let elapsed = started.elapsed();

    assert!(matches!(result, Err(SubgraphError::Cancelled)));
    assert!(
        elapsed < std::time::Duration::from_secs(5),
        "cancellation must drop the in-flight future within the request deadline; elapsed = {elapsed:?}"
    );
    assert!(
        dropped.load(Ordering::SeqCst),
        "the inner request future must be dropped when the cancellation token fires"
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
    let first_api = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("FakeApiKey")
        .with_external_host_policy(ExternalHostPolicy::Test)
        .base_urls(first_base_urls)
        .client(shared.clone())
        .build()
        .expect("first subgraph client with loopback override must build");

    let second_base_urls: SubgraphApiBaseUrls =
        std::iter::once((SupportedChainId::GnosisChain, Some(second.uri()))).collect();
    let second_api = SubgraphApi::builder()
        .chain(SupportedChainId::GnosisChain)
        .api_key("FakeApiKey")
        .with_external_host_policy(ExternalHostPolicy::Test)
        .base_urls(second_base_urls)
        .client(shared)
        .build()
        .expect("second subgraph client with loopback override must build");

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

mod recording_transport {
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use async_trait::async_trait;
    use cow_sdk_core::{HttpTransport, SupportedChainId, TransportError};
    use cow_sdk_subgraph::{
        ExternalHostPolicy, SubgraphApi, SubgraphApiBaseUrls, SubgraphError, SubgraphQueryRequest,
    };
    use cow_sdk_transport_policy::{RetryPolicy, TransportPolicy};
    use serde_json::{Value, json};

    #[derive(Debug, Clone)]
    struct RecordedRequest {
        method: &'static str,
        url: String,
        body: String,
    }

    #[derive(Debug, Clone)]
    enum Canned {
        Ok(String),
        HttpStatus {
            status: u16,
            headers: Vec<(String, String)>,
            body: String,
        },
    }

    #[derive(Debug)]
    struct RecordingTransport {
        calls: Mutex<Vec<RecordedRequest>>,
        responses: Mutex<VecDeque<Canned>>,
    }

    impl RecordingTransport {
        fn new(responses: impl IntoIterator<Item = Canned>) -> Arc<Self> {
            Arc::new(Self {
                calls: Mutex::new(Vec::new()),
                responses: Mutex::new(responses.into_iter().collect()),
            })
        }

        fn observed(&self) -> Vec<RecordedRequest> {
            self.calls
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone()
        }

        fn record(&self, request: RecordedRequest) -> Canned {
            self.calls
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(request);
            self.responses
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .pop_front()
                .expect("recording transport must have a canned response for every call")
        }
    }

    #[async_trait]
    impl HttpTransport for RecordingTransport {
        async fn get(
            &self,
            path: &str,
            _headers: &[(String, String)],
            _timeout: Option<Duration>,
        ) -> Result<String, TransportError> {
            let canned = self.record(RecordedRequest {
                method: "GET",
                url: path.to_owned(),
                body: String::new(),
            });
            transport_result(canned)
        }

        async fn post(
            &self,
            path: &str,
            body: &str,
            _headers: &[(String, String)],
            _timeout: Option<Duration>,
        ) -> Result<String, TransportError> {
            let canned = self.record(RecordedRequest {
                method: "POST",
                url: path.to_owned(),
                body: body.to_owned(),
            });
            transport_result(canned)
        }

        async fn put(
            &self,
            path: &str,
            body: &str,
            _headers: &[(String, String)],
            _timeout: Option<Duration>,
        ) -> Result<String, TransportError> {
            let canned = self.record(RecordedRequest {
                method: "PUT",
                url: path.to_owned(),
                body: body.to_owned(),
            });
            transport_result(canned)
        }

        async fn delete(
            &self,
            path: &str,
            body: &str,
            _headers: &[(String, String)],
            _timeout: Option<Duration>,
        ) -> Result<String, TransportError> {
            let canned = self.record(RecordedRequest {
                method: "DELETE",
                url: path.to_owned(),
                body: body.to_owned(),
            });
            transport_result(canned)
        }
    }

    fn transport_result(canned: Canned) -> Result<String, TransportError> {
        match canned {
            Canned::Ok(body) => Ok(body),
            Canned::HttpStatus {
                status,
                headers,
                body,
            } => Err(TransportError::HttpStatus {
                status,
                headers: headers
                    .into_iter()
                    .map(|(name, value)| (name, value.into()))
                    .collect(),
                body: body.into(),
            }),
        }
    }

    const RECORDING_BASE_URL: &str = "https://subgraph-recording.example";

    fn api_with_recorder(recorder: Arc<RecordingTransport>) -> SubgraphApi {
        api_with_recorder_and_policy(
            recorder,
            TransportPolicy::default_subgraph().with_retry(RetryPolicy::no_retry()),
        )
    }

    fn api_with_recorder_and_policy(
        recorder: Arc<RecordingTransport>,
        transport_policy: TransportPolicy,
    ) -> SubgraphApi {
        let base_urls: SubgraphApiBaseUrls = [
            (
                SupportedChainId::Mainnet,
                Some(RECORDING_BASE_URL.to_owned()),
            ),
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
            .api_key("FakeApiKey")
            .with_external_host_policy(ExternalHostPolicy::Allow(vec![
                "subgraph-recording.example".to_owned(),
            ]))
            .base_urls(base_urls)
            .transport_policy(transport_policy)
            .transport(recorder as Arc<dyn HttpTransport + Send + Sync>)
            .build()
            .expect("subgraph client with recording transport override must build")
    }

    #[tokio::test]
    async fn subgraph_run_query_dispatches_through_injected_transport() {
        let recorder = RecordingTransport::new([Canned::Ok(
            json!({
                "data": {
                    "tokens": [
                        { "symbol": "WXDAI" }
                    ]
                }
            })
            .to_string(),
        )]);
        let api = api_with_recorder(recorder.clone());
        let query = "query TokensByVolume { tokens(first: 1) { symbol } }";

        let response: Value = api
            .run_query(SubgraphQueryRequest::new(query).with_operation_name("TokensByVolume"))
            .await
            .expect("the injected transport must deliver the canned response");

        assert_eq!(response["tokens"][0]["symbol"], "WXDAI");
        let calls = recorder.observed();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].method, "POST");
        assert!(
            calls[0].url.starts_with(RECORDING_BASE_URL),
            "dispatched URL must match the injected base URL: {}",
            calls[0].url
        );
        assert!(
            calls[0].body.contains("TokensByVolume"),
            "the POST body must carry the GraphQL envelope: {}",
            calls[0].body
        );
    }

    #[tokio::test]
    async fn subgraph_errors_field_surfaces_as_graphql_error_through_injected_transport() {
        let recorder = RecordingTransport::new([Canned::Ok(
            json!({
                "errors": [
                    { "message": "Type `Query` has no field `tokens`" }
                ],
                "data": null,
            })
            .to_string(),
        )]);
        let api = api_with_recorder(recorder.clone());
        let query = "query TokensByVolume { tokens(first: 1) { symbol } }";

        let error = api
            .run_query::<Value, _>(
                SubgraphQueryRequest::new(query).with_operation_name("TokensByVolume"),
            )
            .await
            .expect_err("GraphQL errors must surface through the typed error channel");

        match error {
            SubgraphError::GraphQl { errors, .. } => {
                assert_eq!(errors.len(), 1);
                assert!(errors[0].message.as_inner().contains("no field `tokens`"));
            }
            other => panic!("expected GraphQl error, got {other:?}"),
        }
        let calls = recorder.observed();
        assert_eq!(calls.len(), 1);
    }

    #[tokio::test]
    async fn subgraph_missing_data_surfaces_as_missing_data_error_through_injected_transport() {
        let recorder = RecordingTransport::new([Canned::Ok(json!({ "data": null }).to_string())]);
        let api = api_with_recorder(recorder.clone());
        let query = "query TokensByVolume { tokens(first: 1) { symbol } }";

        let error = api
            .run_query::<Value, _>(
                SubgraphQueryRequest::new(query).with_operation_name("TokensByVolume"),
            )
            .await
            .expect_err("missing data must surface as MissingData");

        match error {
            SubgraphError::MissingData { .. } => {}
            other => panic!("expected MissingData error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn subgraph_http_status_error_propagates_through_injected_transport() {
        let recorder = RecordingTransport::new([Canned::HttpStatus {
            status: 502,
            headers: Vec::new(),
            body: "upstream unavailable".to_owned(),
        }]);
        let api = api_with_recorder(recorder.clone());
        let query = "query TokensByVolume { tokens(first: 1) { symbol } }";

        let error = api
            .run_query::<Value, _>(
                SubgraphQueryRequest::new(query).with_operation_name("TokensByVolume"),
            )
            .await
            .expect_err("a 502 must surface through the typed HttpStatus channel");

        match error {
            SubgraphError::HttpStatus { status, body, .. } => {
                assert_eq!(status, 502);
                assert_eq!(body.as_inner(), "upstream unavailable");
            }
            other => panic!("expected HttpStatus error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn subgraph_retries_transient_status_through_injected_transport() {
        let recorder = RecordingTransport::new([
            Canned::HttpStatus {
                status: 503,
                headers: Vec::new(),
                body: "upstream unavailable".to_owned(),
            },
            Canned::Ok(
                json!({
                    "data": {
                        "tokens": [
                            { "symbol": "WXDAI" }
                        ]
                    }
                })
                .to_string(),
            ),
        ]);
        let transport_policy = TransportPolicy::default_subgraph().with_retry(
            RetryPolicy::builder()
                .max_attempts(2)
                .base_delay(Duration::ZERO)
                .max_delay(Duration::ZERO)
                .build(),
        );
        let api = api_with_recorder_and_policy(recorder.clone(), transport_policy);
        let query = "query TokensByVolume { tokens(first: 1) { symbol } }";

        let response: Value = api
            .run_query(SubgraphQueryRequest::new(query).with_operation_name("TokensByVolume"))
            .await
            .expect("a transient status must retry and return the successful response");

        assert_eq!(response["tokens"][0]["symbol"], "WXDAI");
        assert_eq!(recorder.observed().len(), 2);
    }
}
