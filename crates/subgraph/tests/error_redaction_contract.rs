use cow_sdk_core::{
    REDACTED_PLACEHOLDER, REDACTED_RESPONSE_BODY_MAX_BYTES, RESPONSE_BODY_TRUNCATION_MARKER,
};
use cow_sdk_subgraph::{SubgraphError, SubgraphQueryRequest};
use serde_json::json;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

mod common;

const RAW_AUTH_TOKEN: &str = "subgraph-bearer-secret";
const RAW_API_KEY: &str = "subgraph-api-key-secret";
const RAW_JWT: &str = "eyJhbGciOiJIUzI1NiJ9.eyJzdWJncmFwaCI6ImNvdyJ9.signature";

#[tokio::test]
async fn http_status_body_is_redacted_at_storage_and_public_representations() {
    let server = MockServer::start().await;
    let api = common::loopback_client(server.uri());
    let body = json!({
        "errors": [
            {
                "message": format!(
                    "upstream echoed Authorization: Bearer {RAW_AUTH_TOKEN}; api_key={RAW_API_KEY}; jwt={RAW_JWT}"
                )
            }
        ]
    })
    .to_string();

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(503).set_body_string(body))
        .mount(&server)
        .await;

    let error = api
        .query::<serde_json::Value, _>(
            SubgraphQueryRequest::new("query Totals { totals { orders } }")
                .with_operation_name("Totals"),
        )
        .await
        .expect_err("HTTP status failures must surface through SubgraphError::HttpStatus");

    let SubgraphError::HttpStatus { body, .. } = &error else {
        panic!("expected HttpStatus error, got {error:?}");
    };
    assert_sanitized_storage(body.as_inner());
    assert_public_representations_are_redacted(&error);
}

#[tokio::test]
async fn serialization_body_is_redacted_at_storage_and_public_representations() {
    let server = MockServer::start().await;
    let api = common::loopback_client(server.uri());
    let body = format!(
        "not-json Authorization: Bearer {RAW_AUTH_TOKEN}; token={RAW_API_KEY}; jwt={RAW_JWT}"
    );

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let error = api
        .query::<serde_json::Value, _>(
            SubgraphQueryRequest::new("query Totals { totals { orders } }")
                .with_operation_name("Totals"),
        )
        .await
        .expect_err("malformed success bodies must surface through SubgraphError::Serialization");

    let SubgraphError::Serialization { body, .. } = &error else {
        panic!("expected Serialization error, got {error:?}");
    };
    assert_sanitized_storage(body.as_inner());
    assert_public_representations_are_redacted(&error);
}

fn assert_sanitized_storage(stored: &str) {
    assert!(stored.contains(REDACTED_PLACEHOLDER));
    assert!(
        stored.len() <= REDACTED_RESPONSE_BODY_MAX_BYTES + RESPONSE_BODY_TRUNCATION_MARKER.len()
    );
    assert_no_raw_credentials(stored);
}

fn assert_public_representations_are_redacted(error: &SubgraphError) {
    let display = error.to_string();
    let compact_debug = format!("{error:?}");
    let pretty_debug = format!("{error:#?}");
    let json = serde_json::to_string(error).expect("SubgraphError must serialize diagnostically");

    for rendered in [display, compact_debug, pretty_debug, json] {
        assert!(rendered.contains(REDACTED_PLACEHOLDER));
        assert_no_raw_credentials(&rendered);
    }
}

fn assert_no_raw_credentials(rendered: &str) {
    for forbidden in [RAW_AUTH_TOKEN, RAW_API_KEY, RAW_JWT] {
        assert!(
            !rendered.contains(forbidden),
            "rendered output leaked {forbidden}: {rendered}"
        );
    }
}
